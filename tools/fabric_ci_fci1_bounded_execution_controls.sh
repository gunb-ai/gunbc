#!/usr/bin/env bash
set -euo pipefail

# FCI-1 bounded-driver one-axis controls. This starts no Work command and touches no reservation.
# SCAFFOLD - dissolve-on: bash-emit (#5828 / ROADMAP 6-shell-slice0 / shell-to-intent Phase 2)
# realizes this runner through orchestration emit or typed host_effect_apply, without a
# medium-as-string concat scaffold. That capability -- .dag-to-bash emission for a foreign
# executor -- is what replaces a hand-shell carrier; modeled lifecycle actuation alone would
# sequence the gate and still leave this transport hand-authored.
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1 || {
  echo 'ControlRefused: derived repo_root is not a git checkout' >&2; exit 2;
}
source "$repo_root/tools/fabric_ci_fci1_bounded_execution_context.env"
[[ ${EUID} -eq 0 ]] || { echo 'ControlRefused: must run as root on srv3' >&2; exit 2; }
[[ $(hostname -s) == srv3 ]] || { echo 'ControlRefused: host is not srv3' >&2; exit 2; }
gunbc_bin="$repo_root/target/release/gunbc"
[[ -x $gunbc_bin ]] || { echo 'ControlRefused: exact-tree gunbc absent' >&2; exit 2; }

canonical_root=/var/lib/gunbc/fabric/allocation
scratch=$(mktemp -d /tmp/fci1-bounded-controls.XXXXXX)
trap 'rm -rf -- "$scratch"' EXIT

store_digest() {
  if [[ -d $canonical_root ]]; then
    {
      find "$canonical_root" -xdev -printf '%P|%y|%m|%u|%g|%s\n'
      find "$canonical_root" -xdev -type f -exec sha256sum {} +
    } | sort | sha256sum
  else
    printf 'absent\n'
  fi
}

assert_unit_collected() {
  local unit=$1 observed
  observed=$(systemctl show "$unit" --property=LoadState --property=ActiveState 2>&1 || true)
  [[ $observed == *'LoadState=not-found'* && $observed == *'ActiveState=inactive'* ]] || {
    echo "ControlRefused: unit not collected: unit=$unit observed=$observed" >&2; exit 1;
  }
}

run_pure_entry=("$gunbc_bin" run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/instruments/fabric_control_plane_live_probe.dag \
  --function fci1_assert_checkpoint_token --arg token=none)
bounded_pure_entry=(/bin/bash -c '
  cgroup=$(sed -n "s/^0:://p" /proc/self/cgroup)
  [[ -n $cgroup && $(<"/sys/fs/cgroup$cgroup/memory.max") == "$1" ]] || exit 85
  [[ $(<"/sys/fs/cgroup$cgroup/memory.high") == "$2" ]] || exit 85
  shift 2
  exec "$@"
' fci1-bounded-control "$FCI1_MEMORY_MAX_BYTES" "$FCI1_MEMORY_HIGH_BYTES" "${run_pure_entry[@]}")

before_store=$(store_digest)
before_runtime=$(find /run -maxdepth 1 -name 'fci1-*' -printf '%f\n' | sort)
if pgrep -fa '/opt/fabric-cells/.*/(runner|work)' >"$scratch/process-before"; then
  echo 'ControlRefused: fabric-cell Work process existed before controls' >&2; exit 1
fi

# Parent route: change only the stated axis from the positive row.
fci1_run_bounded_driver "$repo_root" "${bounded_pure_entry[@]}"
assert_unit_collected "$FCI1_DRIVER_UNIT"
if fci1_run_bounded_driver /tmp "${run_pure_entry[@]}" >"$scratch/driver-wrong-cwd" 2>&1; then
  echo 'ControlRefused: parent wrong-WorkingDirectory row succeeded' >&2; exit 1
fi
grep -Eq 'workspace root|process_workspace_root' "$scratch/driver-wrong-cwd" || {
  echo 'ControlRefused: parent wrong-cwd row did not name the workspace-root refusal' >&2; exit 1;
}
assert_unit_collected "$FCI1_DRIVER_UNIT"
if fci1_run_unbounded_memory_control fci1-driver-unbounded-control.service "$repo_root" \
  "${run_pure_entry[@]}" >"$scratch/driver-unbounded" 2>&1; then
  echo 'ControlRefused: parent unbounded-MemoryMax row succeeded' >&2; exit 1
fi
grep -q 'HostBudgetUnreadable' "$scratch/driver-unbounded" || {
  echo 'ControlRefused: parent unbounded row did not name HostBudgetUnreadable' >&2; exit 1;
}
assert_unit_collected fci1-driver-unbounded-control.service

# Independently rooted submitter route: the same three rows, with a distinct unit identity.
fci1_run_bounded_submitter fci1-submitter-positive-control.service "$repo_root" "${bounded_pure_entry[@]}"
assert_unit_collected fci1-submitter-positive-control.service
if fci1_run_bounded_submitter fci1-submitter-wrong-cwd-control.service /tmp \
  "${run_pure_entry[@]}" >"$scratch/submitter-wrong-cwd" 2>&1; then
  echo 'ControlRefused: submitter wrong-WorkingDirectory row succeeded' >&2; exit 1
fi
grep -Eq 'workspace root|process_workspace_root' "$scratch/submitter-wrong-cwd" || {
  echo 'ControlRefused: submitter wrong-cwd row did not name the workspace-root refusal' >&2; exit 1;
}
assert_unit_collected fci1-submitter-wrong-cwd-control.service
if fci1_run_unbounded_memory_control fci1-submitter-unbounded-control.service "$repo_root" \
  "${run_pure_entry[@]}" >"$scratch/submitter-unbounded" 2>&1; then
  echo 'ControlRefused: submitter unbounded-MemoryMax row succeeded' >&2; exit 1
fi
grep -q 'HostBudgetUnreadable' "$scratch/submitter-unbounded" || {
  echo 'ControlRefused: submitter unbounded row did not name HostBudgetUnreadable' >&2; exit 1;
}
assert_unit_collected fci1-submitter-unbounded-control.service

exact_status=0
fci1_run_bounded_driver "$repo_root" /bin/sh -c 'exit 86' || exact_status=$?
[[ $exact_status == 86 ]] || {
  echo "ControlRefused: exact inner status 86 emerged as $exact_status" >&2; exit 1;
}
assert_unit_collected "$FCI1_DRIVER_UNIT"

after_store=$(store_digest)
after_runtime=$(find /run -maxdepth 1 -name 'fci1-*' -printf '%f\n' | sort)
[[ $after_store == "$before_store" ]] || {
  echo 'ControlRefused: canonical allocation store changed' >&2; exit 1;
}
[[ $after_runtime == "$before_runtime" ]] || {
  echo 'ControlRefused: disposable FCI-1 root population changed' >&2; exit 1;
}
if pgrep -fa '/opt/fabric-cells/.*/(runner|work)' >"$scratch/process-after"; then
  echo 'ControlRefused: fabric-cell Work process exists after controls' >&2; exit 1
fi

printf 'FCI1BoundedExecutionControlsAccepted|systemd=%s|memory_max=%s|memory_high=%s\n' \
  "$(systemctl --version | sed -n '1p')" "$FCI1_MEMORY_MAX_BYTES" "$FCI1_MEMORY_HIGH_BYTES"
