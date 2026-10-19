#!/usr/bin/env bash
set +x
set -euo pipefail
umask 077
[[ -t 0 && $# == 5 && $1 =~ ^(import|status|lock|unlock)$ ]] || {
  echo 'usage: credential-local.sh import|status|lock|unlock ABSOLUTE_BINARY NAMESPACE USER COLLECTION' >&2; exit 2;
}
[[ $2 == /* && -f $2 && ! -L $2 && -x $2 && -O $2 ]] || exit 2
source "$(dirname -- "${BASH_SOURCE[0]}")/custody-session.sh"
task_pat=''
trap 'unset task_pat' EXIT INT TERM
if [[ $1 == import ]]; then
  read -rsp 'Existing Docker Hub PAT (saved only in OS keyring): ' task_pat
  printf '\n' >&2
  [[ -n $task_pat ]] || exit 2
  printf '%s' "$task_pat" | "$2" credential-import "$3" "$4" "$5"
else
  "$2" "credential-$1" "$3" "$4" "$5"
fi
