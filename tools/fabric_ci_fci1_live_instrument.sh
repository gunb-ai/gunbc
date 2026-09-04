#!/usr/bin/env bash
set -euo pipefail

# FCI-1 reservation instrument. It starts no Work command.
# SCAFFOLD - dissolve-on: bash-emit (#5828 / ROADMAP 6-shell-slice0 / shell-to-intent Phase 2)
# realizes this runner through orchestration emit or typed host_effect_apply, without a
# medium-as-string concat scaffold. That capability -- .dag-to-bash emission for a foreign
# executor -- is what replaces a hand-shell carrier; modeled lifecycle actuation alone would
# sequence the gate and still leave this transport hand-authored.
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1 || { echo 'InstrumentRefused: derived repo_root is not a git checkout' >&2; exit 2; }
cd "$repo_root" || { echo 'InstrumentRefused: cannot enter derived repo_root' >&2; exit 2; }
source "$repo_root/tools/fabric_ci_fci1_bounded_execution_context.env"

# The outer process does discovery only. The complete instrument re-enters under the typed FCI-1
# envelope before the first gunbc call, so calibration, assertions and cleanup all inherit the
# same finite hard bound. The marker selects a phase; admission below comes only from fresh manager
# and cgroup observations made by the inner process.
if [[ ${FCI1_BOUNDED_INNER-0} != 1 ]]; then
  [[ ${EUID} -eq 0 ]] || { echo 'InstrumentRefused: must run as root on srv3' >&2; exit 2; }
  [[ $(hostname -s) == srv3 ]] || { echo 'InstrumentRefused: host is not srv3' >&2; exit 2; }
  [[ -x $repo_root/target/release/gunbc ]] || { echo 'InstrumentRefused: exact-tree gunbc absent' >&2; exit 2; }
  outer_status=0
  fci1_run_bounded_driver "$repo_root" env FCI1_BOUNDED_INNER=1 \
    FCI1_CHECKPOINT="${FCI1_CHECKPOINT-__empty__}" "$repo_root/tools/fabric_ci_fci1_live_instrument.sh" || outer_status=$?
  driver_terminal=$(systemctl show "$FCI1_DRIVER_UNIT" --property=LoadState --property=ActiveState 2>&1 || true)
  [[ $driver_terminal == *'LoadState=not-found'* && $driver_terminal == *'ActiveState=inactive'* ]] || {
    echo "InstrumentRefused: bounded driver was not positively collected: $driver_terminal" >&2
    exit 1
  }
  echo "bounded-driver-terminal-observed: unit=$FCI1_DRIVER_UNIT status=$outer_status LoadState=not-found ActiveState=inactive" >&2
  exit "$outer_status"
fi

export FABRIC_CI_GUNBC_BIN="$repo_root/target/release/gunbc"
export FABRIC_CI_SOURCE_ROOT="$repo_root"
export FABRIC_CI_ENTRY="$repo_root/dag/gunbc/instruments/fabric_control_plane_live_probe.dag"
export FABRIC_CI_LOG FABRIC_CI_VALUE_ROOT
FABRIC_CI_LOG=$(mktemp)
FABRIC_CI_VALUE_ROOT=$(mktemp -d)
receipt_root=
checkpoint=${FCI1_CHECKPOINT-__empty__}
injected_failure_status=86
source "$repo_root/tools/fabric_ci_evidence_driver.sh"
fabric_ci_driver_init

[[ ${EUID} -eq 0 ]] || { echo 'InstrumentRefused: bounded inner is not root' >&2; exit 2; }
[[ $(hostname -s) == srv3 ]] || { echo 'InstrumentRefused: bounded inner host is not srv3' >&2; exit 2; }
[[ -x $FABRIC_CI_GUNBC_BIN ]] || { echo 'InstrumentRefused: exact-tree gunbc absent' >&2; exit 2; }

# Fresh live admission. No value inherited from the outer marker is evidence. Manager properties,
# the process's cgroup membership, and the controller files must independently name one context.
driver_show=$(systemctl show "$FCI1_DRIVER_UNIT" \
  --property=ControlGroup --property=WorkingDirectory --property=MemoryMax --property=MemoryHigh) || {
  echo 'InstrumentRefused: bounded driver properties unreadable' >&2; exit 2;
}
driver_control_group=$(sed -n 's/^ControlGroup=//p' <<<"$driver_show")
driver_working_directory=$(sed -n 's/^WorkingDirectory=//p' <<<"$driver_show")
driver_memory_max=$(sed -n 's/^MemoryMax=//p' <<<"$driver_show")
driver_memory_high=$(sed -n 's/^MemoryHigh=//p' <<<"$driver_show")
process_control_group=$(sed -n 's/^0:://p' /proc/self/cgroup)
[[ -n $driver_control_group && $driver_control_group == "$process_control_group" ]] || {
  echo 'InstrumentRefused: process is not in the observed bounded driver cgroup' >&2; exit 2;
}
[[ $driver_working_directory == "$repo_root" && $(pwd -P) == "$repo_root" ]] || {
  echo 'InstrumentRefused: bounded driver workspace root mismatch' >&2; exit 2;
}
[[ $driver_memory_max == "$FCI1_MEMORY_MAX_BYTES" && $driver_memory_high == "$FCI1_MEMORY_HIGH_BYTES" ]] || {
  echo 'InstrumentRefused: bounded driver manager envelope mismatch' >&2; exit 2;
}
driver_cgroup_path="/sys/fs/cgroup$driver_control_group"
[[ $(<"$driver_cgroup_path/memory.max") == "$FCI1_MEMORY_MAX_BYTES" ]] || {
  echo 'HostBudgetUnreadable: bounded driver has no matching finite cgroup memory.max' >&2; exit 2;
}
[[ $(<"$driver_cgroup_path/memory.high") == "$FCI1_MEMORY_HIGH_BYTES" ]] || {
  echo 'InstrumentRefused: bounded driver has no matching cgroup memory.high' >&2; exit 2;
}
driver_systemd_version=$(systemctl --version | sed -n '1p')

canonical_root=/var/lib/gunbc/fabric/allocation
mutation_runtime=fci1-submitter-owned-reservation
mutation_root=/run/$mutation_runtime
generation_root=
subject="$repo_root/dag/gunbc/fabric/fabric_required_build_cell.dag"
allocation_before=
positive_before=
cell_before=
canonical_authority_path=
generation_authority_path=

receipt_path() {
  printf '%s/%s.txt' "$receipt_root" "$1"
}

checkpoint_if_selected() {
  local phase=$1
  [[ $checkpoint == "$phase" ]] || return 0
  fabric_ci_run_assertion fci1_checkpoint_reached --arg token="$checkpoint" --arg phase="$phase" --arg path="$(receipt_path checkpoint)"
  echo "CheckpointReached: checkpoint=$checkpoint phase=$phase status=$injected_failure_status" | tee -a "$FABRIC_CI_LOG" >&2
  exit "$injected_failure_status"
}

zero_work_receipt() {
  local phase=$1 path
  path=$(receipt_path "zero-work-$phase")
  fabric_ci_run_assertion fci1_write_zero_work_receipt --arg phase="$phase" --arg path="$path"
}

unit_terminal_observed() {
  local unit=$1 output line load_state= active_state=
  if output=$(systemctl show "$unit" --property=LoadState --property=ActiveState 2>&1); then
    :
  else
    return 1
  fi
  [[ -n $output ]] || return 1
  while IFS= read -r line; do
    case $line in
      LoadState=*) load_state=${line#LoadState=} ;;
      ActiveState=*) active_state=${line#ActiveState=} ;;
    esac
  done <<< "$output"
  [[ -n $load_state && -n $active_state ]] || return 1
  [[ $active_state == inactive || $active_state == failed ]] || return 1
  [[ $load_state == not-found ]] || return 1
  echo "unit-terminal-observed: unit=$unit LoadState=$load_state ActiveState=$active_state" >&2
}

cleanup() {
  local original_status=$?
  local cleanup_status=0
  trap - EXIT
  set +e
  systemctl stop fci1-positive-submitter.service fci1-generation-submitter.service fci1-dependent-submitter.service >/dev/null 2>&1 || true
  local cleanup_disposition_path
  if [[ -n $allocation_before ]]; then
    cleanup_disposition_path=$(receipt_path canonical-cleanup)
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_write_canonical_cleanup_receipt \
      --arg root="$canonical_root" --arg before_wire="$allocation_before" \
      --arg authority_path="$canonical_authority_path" --arg path="$cleanup_disposition_path"
    cleanup_status=$?
  fi
  if [[ -n $generation_root ]]; then
    local generation_disposition_path
    generation_disposition_path=$(receipt_path generation-cleanup)
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_write_disposable_cleanup_receipt \
      --arg root="$generation_root" --arg authority_path="$generation_authority_path" --arg path="$generation_disposition_path"
    local generation_cleanup_status=$?
    (( cleanup_status != 0 )) || cleanup_status=$generation_cleanup_status
    if (( generation_cleanup_status == 0 )); then
      rm -rf -- "$generation_root"
    fi
  fi
  if [[ -e $mutation_root || -L $mutation_root ]]; then
    printf 'RuntimeDirectoryObserved|path=%s|standing=present\n' "$mutation_root" >"$(receipt_path runtime-directory)"
    echo 'InstrumentRefused: dependent RuntimeDirectory remains after cleanup' >&2
    cleanup_status=1
  else
    printf 'RuntimeDirectoryObserved|path=%s|standing=absent\n' "$mutation_root" >"$(receipt_path runtime-directory)"
    echo "runtime-directory-observed: path=$mutation_root standing=absent" >&2
  fi
  local cleanup_unit
  for cleanup_unit in fci1-positive-submitter.service fci1-generation-submitter.service fci1-dependent-submitter.service; do
    local unit_receipt
    unit_receipt=$(receipt_path "unit-terminal-${cleanup_unit%.service}")
    if unit_terminal_observed "$cleanup_unit" >"$unit_receipt" 2>&1; then
      :
    else
      systemctl show "$cleanup_unit" --property=LoadState --property=ActiveState >"$unit_receipt" 2>&1 || true
      echo "InstrumentRefused: transient unit terminal state unestablished: $cleanup_unit" >&2
      cleanup_status=1
    fi
  done
  if [[ -n $cell_before ]]; then
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_grade_and_write_cell_receipt \
      --arg before_wire="$cell_before" --arg phase="terminal" --arg path="$(receipt_path cell-terminal)"
    local cell_terminal_status=$?
    (( cleanup_status != 0 )) || cleanup_status=$cell_terminal_status
  fi
  local terminal_zero_work
  terminal_zero_work=$(receipt_path zero-work-terminal)
  "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
    --entry "$FABRIC_CI_ENTRY" --function fci1_write_zero_work_receipt \
    --arg phase="terminal" --arg path="$terminal_zero_work"
  local zero_work_status=$?
  (( cleanup_status != 0 )) || cleanup_status=$zero_work_status
  if [[ ! -f $(receipt_path checkpoint) ]]; then
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_write_checkpoint_terminal_receipt \
      --arg token="$checkpoint" --arg path="$(receipt_path checkpoint)"
    local checkpoint_receipt_status=$?
    (( cleanup_status != 0 )) || cleanup_status=$checkpoint_receipt_status
  fi
  local required_receipt
  local -a required_receipts=(
    bounded-context.txt checkpoint.txt canonical-cleanup.txt cell-terminal.txt zero-work-before-canonical.txt
    zero-work-terminal.txt runtime-directory.txt unit-terminal-fci1-positive-submitter.txt
    unit-terminal-fci1-generation-submitter.txt unit-terminal-fci1-dependent-submitter.txt
  )
  case $checkpoint in
    before-canonical-commit|after-canonical-commit-before-transport|after-held-preserved) ;;
    after-release-before-grading)
      required_receipts+=(canonical-release.txt)
      ;;
    canonical-restored-later)
      required_receipts+=(canonical-release.txt canonical-allocation.txt cell-after-canonical.txt zero-work-after-canonical.txt)
      ;;
    generation-held-one|generation-authority-two-before-commit|generation-held-two)
      required_receipts+=(canonical-release.txt canonical-allocation.txt cell-after-canonical.txt zero-work-after-canonical.txt generation-cleanup.txt)
      ;;
    generation-changed-observed)
      required_receipts+=(canonical-release.txt canonical-allocation.txt cell-after-canonical.txt zero-work-after-canonical.txt generation-cleanup.txt generation-verdict.txt)
      ;;
    generation-free-before-removal)
      required_receipts+=(canonical-release.txt canonical-allocation.txt cell-after-canonical.txt zero-work-after-canonical.txt generation-cleanup.txt generation-verdict.txt)
      ;;
    runtime-directory-terminal)
      required_receipts+=(canonical-release.txt canonical-allocation.txt cell-after-canonical.txt zero-work-after-canonical.txt generation-terminal.txt generation-verdict.txt dependent-lifetime.txt)
      ;;
    none)
      required_receipts+=(canonical-release.txt canonical-allocation.txt cell-after-canonical.txt zero-work-after-canonical.txt generation-terminal.txt generation-verdict.txt dependent-lifetime.txt cell-after-dependent.txt zero-work-after-dependent.txt)
      ;;
  esac
  for required_receipt in "${required_receipts[@]}"; do
    if [[ ! -f $receipt_root/$required_receipt ]]; then
      echo "InstrumentRefused: required receipt missing: checkpoint=$checkpoint file=$required_receipt" >&2
      cleanup_status=1
    fi
  done
  local observed_roster expected_roster
  observed_roster=$(find "$receipt_root" -maxdepth 1 -type f ! -name manifest.txt -printf '%f\n' | sort) || cleanup_status=1
  expected_roster=$(printf '%s\n' "${required_receipts[@]}" | sort) || cleanup_status=1
  if [[ $observed_roster != "$expected_roster" ]]; then
    echo 'InstrumentRefused: receipt bundle member roster mismatch' >&2
    cleanup_status=1
  fi
  local head tree provenance manifest manifest_digest
  head=$(git -C "$repo_root" rev-parse HEAD) || cleanup_status=1
  tree=$(git -C "$repo_root" rev-parse 'HEAD^{tree}') || cleanup_status=1
  provenance=$($FABRIC_CI_GUNBC_BIN --version 2>&1) || cleanup_status=1
  manifest="$receipt_root/manifest.txt"
  {
    printf 'head=%s\ntree=%s\nbinary_provenance=%s\nsystemd_version=%s\ndriver_unit=%s\ncheckpoint=%s\noriginal_exit_status=%s\ncleanup_verdict=%s\n' \
      "$head" "$tree" "$provenance" "$driver_systemd_version" "$FCI1_DRIVER_UNIT" \
      "$checkpoint" "$original_status" "$cleanup_status"
    for required_receipt in "${required_receipts[@]}"; do sha256sum "$receipt_root/$required_receipt"; done
  } >"$manifest" || cleanup_status=1
  manifest_digest=$(sha256sum "$manifest" | awk '{print $1}') || cleanup_status=1
  echo "FCI1EvidenceIdentity: receipt_root=$receipt_root manifest_sha256=$manifest_digest" >&2
  rm -f "$FABRIC_CI_LOG"
  rm -rf "$FABRIC_CI_VALUE_ROOT"
  if (( cleanup_status != 0 )); then
    echo 'InstrumentRefused: cleanup terminal postconditions did not all hold' >&2
    exit 1
  fi
  exit "$original_status"
}
if ! fabric_ci_run_assertion fci1_assert_checkpoint_token --arg token="$checkpoint"; then
  rm -f -- "$FABRIC_CI_LOG"
  rm -rf -- "$FABRIC_CI_VALUE_ROOT"
  exit 2
fi
receipt_root=$(mktemp -d /tmp/fci1-receipt.XXXXXX)
trap cleanup EXIT
{
  printf 'unit=%s\nControlGroup=%s\nWorkingDirectory=%s\nMemoryMax=%s\nMemoryHigh=%s\n' \
    "$FCI1_DRIVER_UNIT" "$driver_control_group" "$driver_working_directory" \
    "$driver_memory_max" "$driver_memory_high"
  printf 'process_cgroup=%s\ncgroup_memory_max=%s\ncgroup_memory_high=%s\nsystemd_version=%s\n' \
    "$process_control_group" "$(<"$driver_cgroup_path/memory.max")" \
    "$(<"$driver_cgroup_path/memory.high")" "$driver_systemd_version"
} >"$(receipt_path bounded-context)"

# The merged evidence calibration runs against this exact binary before any srv3 observation or
# reservation effect. It grades only the evidence boundary; FCI-1 cannot self-grade that contract.
FABRIC_CI_GUNBC_BIN="$FABRIC_CI_GUNBC_BIN" bash "$repo_root/tools/fabric_ci_evidence_calibration.sh"

sha256sum "$subject"
fabric_ci_run_assertion fci1_invalid_before_wire_refuses
fabric_ci_run_assertion fci1_assert_slot_absent_admitted
fabric_ci_run_assertion fci1_assert_unreadable_slot_refused
fabric_ci_run_assertion fci1_assert_exact_absent_to_free_control
fabric_ci_run_assertion fci1_assert_exact_free_to_free_control
fabric_ci_run_assertion fci1_assert_extra_generation_refused_control
fabric_ci_run_assertion fci1_assert_frozen_work_key
fabric_ci_run_assertion fci1_assert_host_preconditions
fabric_ci_run_assertion fci1_assert_clean_cell_prestate
fabric_ci_run_assertion fci1_assert_cell_admitted
cell_before=$(fabric_ci_capture_transport fci1_live_cell_observation cell-before)
allocation_before=$(fabric_ci_capture_transport fci1_live_allocation_prestate allocation-before --arg root="$canonical_root")
fabric_ci_run_assertion fci1_assert_allocation_available --arg root="$canonical_root"
zero_work_receipt before-canonical
checkpoint_if_selected before-canonical-commit

capture_submitter_transport() {
  local unit=$1 function_name=$2 coordinate=$3 root=$4 authority_path=$5
  shift 5
  local value_path="$FABRIC_CI_VALUE_ROOT/$coordinate.wire"
  rm -f -- "$value_path"
  local run_status=0
  local launcher=fci1_run_bounded_submitter
  local runtime_directory=
  if [[ ${1-} == --runtime-directory ]]; then
    runtime_directory=$2
    shift 2
    launcher=fci1_run_bounded_dependent_submitter
  fi
  local -a launch_prefix=("$unit" "$repo_root")
  [[ -z $runtime_directory ]] || launch_prefix+=("$runtime_directory")
  if "$launcher" "${launch_prefix[@]}" "$FABRIC_CI_GUNBC_BIN" run \
    --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
    --entry "$FABRIC_CI_ENTRY" --function "$function_name" --arg root="$root" --arg path="$value_path" \
    --arg cleanup_path="$authority_path" --arg checkpoint="$checkpoint" --arg checkpoint_path="$(receipt_path checkpoint)"; then
    run_status=0
  else
    run_status=$?
  fi
  (( run_status == 0 )) || return "$run_status"
  [[ -f $value_path && ! -L $value_path ]] || return 1
  local value
  value=$(<"$value_path")
  [[ $value =~ ^FCIE0X[0-9A-F]+$ ]] || return 1
  printf '%s' "$value"
}

canonical_authority_path="$FABRIC_CI_VALUE_ROOT/canonical-cleanup-authority.wire"
positive_before=$(capture_submitter_transport fci1-positive-submitter.service fci1_live_reserve_and_observe_available positive-before "$canonical_root" "$canonical_authority_path")
unit_terminal_observed fci1-positive-submitter.service || {
  echo 'InstrumentRefused: submitting control process terminal state unestablished' >&2
  exit 1
}
fabric_ci_run_assertion fci1_assert_lifetime_held --arg root="$canonical_root" --arg before_wire="$positive_before"
checkpoint_if_selected after-held-preserved

release_receipt=$(receipt_path canonical-release)
fabric_ci_run_assertion fci1_live_release_with_receipt --arg root="$canonical_root" --arg path="$release_receipt"
checkpoint_if_selected after-release-before-grading
allocation_receipt=$(receipt_path canonical-allocation)
fabric_ci_run_assertion fci1_grade_and_write_allocation_receipt --arg root="$canonical_root" --arg before_wire="$allocation_before" --arg held_wire="$positive_before" --arg phase="canonical" --arg path="$allocation_receipt"
cell_receipt=$(receipt_path cell-after-canonical)
fabric_ci_run_assertion fci1_grade_and_write_cell_receipt --arg before_wire="$cell_before" --arg phase="after-canonical" --arg path="$cell_receipt"
zero_work_receipt after-canonical
checkpoint_if_selected canonical-restored-later

# Generation replacement is a mutation falsifier, not part of the canonical lifecycle. Its
# run-unique disposable store uses the production reservation/CAS entries and is removed afterward;
# the canonical root sees only reserve, independent observation, and release.
generation_root=$(mktemp -d /run/fci1-generation-mutation.XXXXXX)
generation_authority_path="$FABRIC_CI_VALUE_ROOT/generation-cleanup-authority.wire"
generation_before=$(capture_submitter_transport fci1-generation-submitter.service fci1_live_reserve_and_observe_available generation-before "$generation_root" "$generation_authority_path")
checkpoint_if_selected generation-held-one
fabric_ci_run_assertion fci1_assert_replace_held --arg root="$generation_root" \
  --arg cleanup_path="$generation_authority_path" --arg checkpoint="$checkpoint" \
  --arg checkpoint_path="$(receipt_path checkpoint)"
checkpoint_if_selected generation-held-two
fabric_ci_run_assertion fci1_write_generation_changed_receipt --arg root="$generation_root" --arg before_wire="$generation_before" --arg path="$(receipt_path generation-verdict)"
checkpoint_if_selected generation-changed-observed
fabric_ci_run_assertion fci1_live_release --arg root="$generation_root"
checkpoint_if_selected generation-free-before-removal
generation_terminal_receipt=$(receipt_path generation-terminal)
fabric_ci_run_assertion fci1_write_disposable_cleanup_receipt --arg root="$generation_root" --arg authority_path="$generation_authority_path" --arg path="$generation_terminal_receipt"
rm -rf -- "$generation_root"
generation_root=

dependent_before=$(capture_submitter_transport fci1-dependent-submitter.service fci1_live_reserve_and_observe_available dependent-before "$mutation_root" "$FABRIC_CI_VALUE_ROOT/dependent-cleanup-authority.wire" --runtime-directory "$mutation_runtime")
[[ ! -e $mutation_root && ! -L $mutation_root ]] || { echo 'ExpectedOutcomeNotObserved: submitter runtime directory removal' >&2; exit 1; }
echo "runtime-directory-observed: path=$mutation_root standing=absent" >&2
unit_terminal_observed fci1-dependent-submitter.service || { echo 'InstrumentRefused: dependent submitter terminal state unestablished' >&2; exit 1; }
fabric_ci_run_assertion fci1_write_lifetime_dependent_receipt --arg root="$mutation_root" --arg before_wire="$dependent_before" --arg path="$(receipt_path dependent-lifetime)"
checkpoint_if_selected runtime-directory-terminal
cell_receipt=$(receipt_path cell-after-dependent)
fabric_ci_run_assertion fci1_grade_and_write_cell_receipt --arg before_wire="$cell_before" --arg phase="after-dependent" --arg path="$cell_receipt"
zero_work_receipt after-dependent

echo 'FCI1LiveAccepted'
