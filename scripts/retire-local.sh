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
  echo 'Select an accepted zix 0.2+ dispatcher using ZIX_BIN; no implicit installation.' >&2
  exit 2
fi
command -v jq >/dev/null
"$zix_executable" describe docker --format json >/dev/null
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
"$zix_executable" docker repositories delete apply --namespace "$1" \
  --plan-file "$operation_directory/plan.json" --approval-sha256 "$plan_sha256" \
  --journal-directory "$operation_directory/journal" --username "$docker_username" \
  > "$operation_directory/result.json"
jq -e '.result.state == "complete" and .result.verified_absent == true' "$operation_directory/result.json" >/dev/null
printf 'Deletion accepted; private receipt: %s; token discarded on exit.\n' "$operation_directory/result.json"
