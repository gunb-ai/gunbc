set -euo pipefail
umask 077
[[ $# == 5 ]] || exit 64
executor_helper=$1
executor_unit=$2
executor_lock=$3
executor_polls=$4
executor_systemctl=$5
[[ $executor_helper == /* && $executor_lock == /* && $executor_systemctl == /* ]] || exit 64
[[ $executor_polls =~ ^[1-9][0-9]{0,4}$ ]] || exit 64
[[ ${INVOCATION_ID:-} =~ ^[a-f0-9]{32}$ ]] || exit 64
exec 9< "$executor_lock"
flock -w "$executor_polls" -x 9 || exit 70
executor_decision=$(timeout --kill-after=5s "${executor_polls}s" "$executor_helper" prepare "$INVOCATION_ID")
[[ $executor_decision =~ ^(start|wait|settle|complete|drain):([a-f0-9]{32}|none)$ ]] || exit 65
executor_action=${executor_decision%%:*}
executor_expected=${executor_decision#*:}
executor_observe() {
  local output key value
  output=$(timeout --kill-after=5s 30s "$executor_systemctl" show "$executor_unit" --property=Id --property=LoadState --property=ActiveState --property=MainPID --property=InvocationID --property=Job) || return 1
  local -A fields=()
  while IFS='=' read -r key value; do
    [[ -n $key && ! -v fields[$key] ]] || return 1
    case $key in Id|LoadState|ActiveState|MainPID|InvocationID|Job) fields[$key]=$value ;; *) return 1 ;; esac
  done <<< "$output"
  [[ ${#fields[@]} == 6 ]] || return 1
  [[ ${fields[Id]} == "$executor_unit" && ${fields[LoadState]} == loaded ]] || return 1
  executor_seen=${fields[InvocationID]}
  executor_active=${fields[ActiveState]}
  executor_pid=${fields[MainPID]}
  executor_job=${fields[Job]}
}
if [[ $executor_action == drain ]]; then
  timeout --kill-after=5s "${executor_polls}s" "$executor_systemctl" stop "$executor_unit"
  executor_observe || exit 66
  [[ $executor_active == inactive || $executor_active == failed ]] || exit 67
  [[ $executor_pid == 0 && -z $executor_job ]] || exit 67
  executor_decision=$(timeout --kill-after=5s "${executor_polls}s" "$executor_helper" rearm "$INVOCATION_ID")
  [[ $executor_decision =~ ^start:([a-f0-9]{32}|none)$ ]] || exit 65
  executor_action=start
  executor_expected=${executor_decision#*:}
fi
if [[ $executor_action == start ]]; then
  executor_observe || exit 66
  [[ ${executor_seen:-none} == "$executor_expected" ]] || exit 67
  [[ $executor_active == inactive || $executor_active == failed ]] || exit 67
  [[ $executor_pid == 0 && -z $executor_job ]] || exit 67
  timeout --kill-after=5s 30s "$executor_systemctl" start "$executor_unit"
  executor_observe || exit 66
  [[ $executor_seen =~ ^[a-f0-9]{32}$ && $executor_seen != "$executor_expected" ]] || exit 67
  executor_expected=$executor_seen
  timeout --kill-after=5s "${executor_polls}s" "$executor_helper" bind "$INVOCATION_ID" "$executor_expected"
fi
[[ $executor_expected =~ ^[a-f0-9]{32}$ ]] || exit 65
if [[ $executor_action == start || $executor_action == wait ]]; then
executor_terminated=false
for ((executor_poll=0; executor_poll<executor_polls; executor_poll++)); do
  executor_observe || exit 66
  [[ $executor_seen == "$executor_expected" ]] || exit 67
  if [[ $executor_active == inactive || $executor_active == failed ]]; then
    [[ $executor_pid == 0 && -z $executor_job ]] || exit 67
    executor_observe || exit 66
    [[ $executor_seen == "$executor_expected" && $executor_pid == 0 && -z $executor_job ]] || exit 67
    [[ $executor_active == inactive || $executor_active == failed ]] || exit 67
    executor_terminated=true
    break
  fi
  [[ $executor_active == active || $executor_active == activating || $executor_active == deactivating ]] || exit 67
  sleep 1
done
[[ $executor_terminated == true ]] || exit 68
fi
executor_terminal=$(timeout --kill-after=5s "${executor_polls}s" "$executor_helper" finish "$INVOCATION_ID" "$executor_expected")
[[ $executor_terminal == 'commissioning-fleet-generation-committed' ]] || exit 69
printf '%s\n' "$executor_terminal"