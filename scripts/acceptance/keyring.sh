#!/usr/bin/env bash
set -euo pipefail
# Synthetic data only, never use the caller's desktop bus or keyring.
task_root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
if [[ ${ZIX_DOCKER_ISOLATED_KEYRING:-} != 1 ]]; then
  command -v dbus-run-session >/dev/null
  command -v gnome-keyring-daemon >/dev/null
  task_private=$(mktemp -d)
  trap 'rm -rf -- "$task_private"' EXIT
  chmod 700 "$task_private"
  mkdir -m 700 "$task_private/data" "$task_private/config" "$task_private/runtime"
  env -u DBUS_SESSION_BUS_ADDRESS -u GNOME_KEYRING_CONTROL \
    ZIX_DOCKER_ISOLATED_KEYRING=1 XDG_DATA_HOME="$task_private/data" \
    XDG_CONFIG_HOME="$task_private/config" XDG_RUNTIME_DIR="$task_private/runtime" \
    dbus-run-session -- bash "$0"
  exit
fi
cd "$task_root"
task_daemon=''
cleanup() {
  if [[ -n $task_daemon ]]; then kill "$task_daemon" 2>/dev/null || true; wait "$task_daemon" 2>/dev/null || true; fi
}
trap cleanup EXIT
start_provider() {
  gnome-keyring-daemon --foreground --unlock --components=secrets \
    --control-directory="$XDG_RUNTIME_DIR/keyring" \
    < <(printf '%s' 'synthetic-keyring-acceptance-password') \
    >"$XDG_RUNTIME_DIR/daemon.log" 2>&1 &
  task_daemon=$!
  for ((task_attempt=0; task_attempt<50; task_attempt++)); do
    if [[ -S $XDG_RUNTIME_DIR/keyring/control ]]; then break; fi
    sleep 0.1
  done
  # This is a public synthetic test password, not a real account credential.
  for ((task_attempt=0; task_attempt<50; task_attempt++)); do
    if gdbus call --session --dest org.freedesktop.secrets \
      --object-path /org/freedesktop/secrets/collection/login \
      --method org.freedesktop.DBus.Properties.Get org.freedesktop.Secret.Collection Locked \
      >"$XDG_RUNTIME_DIR/ready.log" 2>/dev/null; then
      gdbus call --session --dest org.freedesktop.secrets --object-path /org/freedesktop/secrets \
        --method org.freedesktop.Secret.Service.SetAlias default /org/freedesktop/secrets/collection/login >/dev/null
      return
    fi
    sleep 0.1
  done
  cat "$XDG_RUNTIME_DIR/daemon.log" >&2
  return 1
}
run_phase() {
  ZIX_DOCKER_KEYRING_PHASE="$1" cargo test --locked --test os_keyring -- --include-ignored
}
start_provider
run_phase seed
cleanup
task_daemon=''
start_provider
run_phase restart
cleanup
task_daemon=''
DBUS_SESSION_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/absent-bus" run_phase unavailable
echo 'Isolated provider persistence, restart, bound use after lock and unavailable-service acceptance passed.'
