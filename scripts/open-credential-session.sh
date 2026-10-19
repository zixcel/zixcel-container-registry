#!/usr/bin/env bash
set +x
set -euo pipefail
umask 077
# Use the installed OS provider; no vault format or key derivation is implemented here.
task_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
[[ -t 0 && $# == 0 ]] || { echo 'Run interactively without arguments.' >&2; exit 2; }
if [[ ${ZIX_DOCKER_OS_SESSION:-} != 1 ]]; then
  if pgrep -u "$(id -u)" -f '(^|/)gnome-keyring-daemon([[:space:]]|$)' >/dev/null; then
    echo 'An OS keyring provider already exists. Use its owning desktop/session; do not open a competing provider.' >&2
    exit 2
  fi
  task_dbus_runtime=$(mktemp -d)
  trap 'rm -rf -- "$task_dbus_runtime"' EXIT
  env -u GNOME_KEYRING_CONTROL XDG_RUNTIME_DIR="$task_dbus_runtime" dbus-run-session -- env ZIX_DOCKER_OS_SESSION=1 bash "$0"
  exit
fi
task_runtime=$(mktemp -d)
task_daemon='' task_password='' task_endpoint='' task_address=''
cleanup() {
  unset task_password
  if [[ -n $task_endpoint && -f $task_endpoint && ! -L $task_endpoint ]] &&
    [[ $(cat -- "$task_endpoint") == "$task_address" ]]; then rm -- "$task_endpoint"; fi
  if [[ -n $task_daemon ]]; then kill "$task_daemon" 2>/dev/null || true; wait "$task_daemon" 2>/dev/null || true; fi
  rm -rf -- "$task_runtime"
}
trap cleanup EXIT
read -r -s -p 'OS login-keyring password (existing password, or choose a non-empty password for a new OS keyring): ' task_password
printf '\n'
[[ -n $task_password ]] || exit 2
gnome-keyring-daemon --foreground --unlock --components=secrets \
  --control-directory="$task_runtime/control" \
  < <(printf '%s' "$task_password") >"$task_runtime/provider.log" 2>&1 &
task_daemon=$!
unset task_password
gdbus wait --session --timeout 10 org.freedesktop.secrets
task_locked=$(gdbus call --session --dest org.freedesktop.secrets \
  --object-path /org/freedesktop/secrets/collection/login \
  --method org.freedesktop.DBus.Properties.Get org.freedesktop.Secret.Collection Locked)
[[ $task_locked == '(<false>,)' ]] || { echo 'OS keyring not unlocked; no credentials imported.' >&2; exit 2; }
task_alias=$(gdbus call --session --dest org.freedesktop.secrets \
  --object-path /org/freedesktop/secrets --method org.freedesktop.Secret.Service.ReadAlias default)
if [[ $task_alias == "(objectpath '/',)" ]]; then
  gdbus call --session --dest org.freedesktop.secrets --object-path /org/freedesktop/secrets \
    --method org.freedesktop.Secret.Service.SetAlias default /org/freedesktop/secrets/collection/login >/dev/null
fi
cd "$task_root"
[[ ! -L .local ]] || exit 2
mkdir -p -m 700 .local
[[ -O .local && $(stat -c %a .local) == 700 ]] || exit 2
task_address=$DBUS_SESSION_BUS_ADDRESS
# Noclobber refuses a prior session marker; never silently switches an owner.
(set -o noclobber; printf '%s\n' "$task_address" > .local/custody-session.address)
task_endpoint="$task_root/.local/custody-session.address"
echo 'OS keyring session ready. Existing encrypted OS files are retained; no API credential imported.'
echo 'Enroll Docker PAT once with scripts/credential-local.sh; unlock never approves a remote write.'
echo 'Keep this shell open while working. exit stops this session provider; no service or CI credential is installed.'
bash --noprofile --norc -i
