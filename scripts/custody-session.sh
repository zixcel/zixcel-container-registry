# Sourced by owner-local tools; this file contains no secret or shell evaluation.
task_owner=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if [[ -z ${DBUS_SESSION_BUS_ADDRESS:-} ]]; then
  task_session="$task_owner/.local/custody-session.address"
  for task_path in "$task_owner/.local" "$task_session"; do
    [[ ! -L $task_path && -O $task_path ]] || { echo 'No owned OS custody session.' >&2; return 2; }
    task_mode=$(stat -c %a -- "$task_path")
    (( (8#$task_mode & 077) == 0 )) || return 2
  done
  [[ -f $task_session && $(stat -c %s -- "$task_session") -le 4096 ]] || return 2
  IFS= read -r DBUS_SESSION_BUS_ADDRESS < "$task_session"
fi
[[ $DBUS_SESSION_BUS_ADDRESS =~ ^unix:path=(/[^,\;[:space:]]+)(,guid=[a-f0-9]{32})?$ ]] || return 2
task_socket=${BASH_REMATCH[1]}
[[ -S $task_socket && -O $task_socket ]] || { echo 'OS custody session expired; unlock it first.' >&2; return 2; }
export DBUS_SESSION_BUS_ADDRESS
unset task_owner task_session task_path task_mode task_socket
