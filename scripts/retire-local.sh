#!/usr/bin/env bash
set -euo pipefail
set +x
umask 077
if [[ $# != 2 || ! -t 0 ]]; then
  echo 'usage: bash scripts/retire-local.sh NAMESPACE REPOSITORY,REPOSITORY (interactive terminal)' >&2
  exit 2
fi
zix_executable="${ZIX_BIN:-$(command -v zix || true)}"
if [[ ! -x "$zix_executable" ]]; then
  echo 'Select an accepted zix 0.2.1+ dispatcher using ZIX_BIN; no implicit installation.' >&2
  exit 2
fi
command -v jq >/dev/null
if ! "$zix_executable" describe docker --format json | jq -e '
  .package == "zixcel-container-registry" and
  any(.commands[]; .id == "docker.verify-absent" and .effect == "read")
' >/dev/null; then
  echo 'Install the accepted provider with verify-absent capability before supplying credentials.' >&2
  exit 2
fi
operation_directory="$(mktemp -d "${TMPDIR:-/tmp}/zix-docker-retirement.XXXXXXXX")"
chmod 700 "$operation_directory"
install -d -m 700 "$operation_directory/journal"
trap 'unset ZIX_DOCKER_HUB_TOKEN' EXIT
trap 'printf "Stopped. Inspect private receipts in %s; do not replay writes.\n" "$operation_directory" >&2' ERR
read -rp 'Docker Hub login username: ' docker_username
read -rsp 'Existing Docker Hub PAT (not saved): ' ZIX_DOCKER_HUB_TOKEN
printf '\n' >&2
export ZIX_DOCKER_HUB_TOKEN
"$zix_executable" docker repositories delete plan --namespace "$1" \
  --repositories "$2" --username "$docker_username" > "$operation_directory/response.json"
jq -e '.result.plan.schema == "zix-local-operation/v1" and (.result.sha256 | test("^[a-f0-9]{64}$"))' "$operation_directory/response.json" >/dev/null
jq '.result.plan' "$operation_directory/response.json" > "$operation_directory/plan.json"
plan_sha256="$(jq -er '.result.sha256' "$operation_directory/response.json")"
printf 'Private deletion plan: %s\n' "$operation_directory/plan.json" >&2
jq '.plan | {namespace,expires_at,repositories}' "$operation_directory/plan.json"
printf 'Reviewed plan SHA256 (copy only after checking the targets): %s\n' "$plan_sha256" >&2
"$zix_executable" docker repositories delete apply --namespace "$1" \
  --plan-file "$operation_directory/plan.json" --approval-sha256 "$plan_sha256" \
  --journal-directory "$operation_directory/journal" --username "$docker_username" \
  > "$operation_directory/result.json"
state="$(jq -er '.result.state' "$operation_directory/result.json")"
if [[ "$state" == verification_pending ]]; then
  printf 'Deletion acknowledged; waiting for read-only confirmation. No DELETE will be repeated.\n' >&2
  verified=false
  for attempt in 1 2 3 4 5 6; do
    sleep 2
    observation="$operation_directory/verification-$attempt.json"
    "$zix_executable" docker repositories verify-absent --namespace "$1" \
      --repositories "$2" --username "$docker_username" > "$observation"
    if jq -e --slurpfile plan "$operation_directory/plan.json" '
      .result as $r | ($plan[0].plan.repositories | map(.repository.name) | sort) as $wanted |
      $r.namespace == $plan[0].plan.namespace and $r.all_observed_absent == true and
      ($r.present | length) == 0 and ($r.observed_absent | sort) == $wanted
    ' "$observation" >/dev/null; then
      jq -n --slurpfile acknowledged "$operation_directory/result.json" --slurpfile observed "$observation" \
        '{state:"complete",acknowledged:$acknowledged[0].result,read_only_verification:$observed[0].result}' \
        > "$operation_directory/confirmed.json"
      verified=true
      break
    fi
  done
  if [[ "$verified" != true ]]; then
    printf 'Deletion acknowledged; verification pending. Do not resend writes. Receipt directory: %s\n' "$operation_directory" >&2
    exit 2
  fi
elif [[ "$state" != complete ]]; then
  printf 'Unexpected result; inspect private receipt without replay.\n' >&2
  exit 2
fi
printf 'Deletion accepted and readback verified; private receipt directory: %s; token discarded on exit.\n' "$operation_directory"
