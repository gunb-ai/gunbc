#!/usr/bin/env bash
set -euo pipefail

# FCI-1 reservation instrument. It starts no Work command.
# SCAFFOLD — dissolve-on: modeled lifecycle actuation sufficient to sequence a wet gate from .dag.
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export FABRIC_CI_GUNBC_BIN="$repo_root/target/release/gunbc"
export FABRIC_CI_SOURCE_ROOT="$repo_root"
export FABRIC_CI_ENTRY="$repo_root/dag/gunbc/instruments/fabric_control_plane_live_probe.dag"
export FABRIC_CI_LOG FABRIC_CI_VALUE_ROOT
FABRIC_CI_LOG=$(mktemp)
FABRIC_CI_VALUE_ROOT=$(mktemp -d)
source "$repo_root/tools/fabric_ci_evidence_driver.sh"
fabric_ci_driver_init

canonical_root=/var/lib/gunbc/fabric/allocation
mutation_runtime=fci1-submitter-owned-reservation
mutation_root=/run/$mutation_runtime
generation_root=
subject="$repo_root/dag/gunbc/fabric/fabric_required_build_cell.dag"
canonical_reservation_may_be_held=0
generation_reservation_may_be_held=0
allocation_before=
positive_before=

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
  [[ $load_state == loaded || $load_state == not-found ]] || return 1
}

cleanup() {
  local original_status=$?
  local cleanup_status=0
  trap - EXIT
  set +e
  systemctl stop fci1-positive-submitter.service fci1-generation-submitter.service fci1-dependent-submitter.service >/dev/null 2>&1 || true
  if (( canonical_reservation_may_be_held )); then
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_live_release --arg root="$canonical_root" >/dev/null 2>&1
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_assert_allocation_restored \
      --arg root="$canonical_root" --arg before_wire="$allocation_before" --arg held_wire="$positive_before"
    cleanup_status=$?
  fi
  if (( generation_reservation_may_be_held )) && [[ -n $generation_root ]]; then
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
      --entry "$FABRIC_CI_ENTRY" --function fci1_live_release --arg root="$generation_root" >/dev/null 2>&1
    local generation_cleanup_status=$?
    (( cleanup_status != 0 )) || cleanup_status=$generation_cleanup_status
  fi
  [[ -z $generation_root ]] || rm -rf -- "$generation_root"
  if [[ -e $mutation_root || -L $mutation_root ]]; then
    echo 'InstrumentRefused: dependent RuntimeDirectory remains after cleanup' >&2
    cleanup_status=1
  fi
  local cleanup_unit
  for cleanup_unit in fci1-positive-submitter.service fci1-generation-submitter.service fci1-dependent-submitter.service; do
    if ! unit_terminal_observed "$cleanup_unit"; then
      echo "InstrumentRefused: transient unit terminal state unestablished: $cleanup_unit" >&2
      cleanup_status=1
    fi
  done
  rm -f "$FABRIC_CI_LOG"
  rm -rf "$FABRIC_CI_VALUE_ROOT"
  if (( cleanup_status != 0 )); then
    echo 'InstrumentRefused: cleanup terminal postconditions did not all hold' >&2
    exit 1
  fi
  exit "$original_status"
}
trap cleanup EXIT

[[ ${EUID} -eq 0 ]] || { echo 'InstrumentRefused: must run as root on srv3' >&2; exit 2; }
[[ $(hostname -s) == srv3 ]] || { echo 'InstrumentRefused: host is not srv3' >&2; exit 2; }
[[ -x $FABRIC_CI_GUNBC_BIN ]] || { echo 'InstrumentRefused: exact-tree gunbc absent' >&2; exit 2; }

# The merged evidence calibration runs against this exact binary before any srv3 observation or
# reservation effect. It grades only the evidence boundary; FCI-1 cannot self-grade that contract.
FABRIC_CI_GUNBC_BIN="$FABRIC_CI_GUNBC_BIN" bash "$repo_root/tools/fabric_ci_evidence_calibration.sh"

sha256sum "$subject"
fabric_ci_run_assertion fci1_invalid_before_wire_refuses
fabric_ci_run_assertion fci1_assert_slot_absent_admitted
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

capture_submitter_transport() {
  local unit=$1 function_name=$2 coordinate=$3 root=$4
  shift 4
  local value_path="$FABRIC_CI_VALUE_ROOT/$coordinate.wire"
  rm -f -- "$value_path"
  local run_status=0
  if systemd-run --quiet --wait --collect --unit="$unit" "$@" \
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
    --entry "$FABRIC_CI_ENTRY" --function "$function_name" --arg root="$root" --arg path="$value_path"; then
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

canonical_reservation_may_be_held=1
positive_before=$(capture_submitter_transport fci1-positive-submitter.service fci1_live_reserve_and_observe_available positive-before "$canonical_root")
unit_terminal_observed fci1-positive-submitter.service || {
  echo 'InstrumentRefused: submitting control process terminal state unestablished' >&2
  exit 1
}
fabric_ci_run_assertion fci1_assert_lifetime_held --arg root="$canonical_root" --arg before_wire="$positive_before"

fabric_ci_run_assertion fci1_live_release --arg root="$canonical_root"
fabric_ci_run_assertion fci1_assert_allocation_restored --arg root="$canonical_root" --arg before_wire="$allocation_before" --arg held_wire="$positive_before"
canonical_reservation_may_be_held=0
fabric_ci_run_assertion fci1_assert_cell_unchanged --arg before_wire="$cell_before"

# Generation replacement is a mutation falsifier, not part of the canonical lifecycle. Its
# run-unique disposable store uses the production reservation/CAS entries and is removed afterward;
# the canonical root sees only reserve, independent observation, and release.
generation_root=$(mktemp -d /run/fci1-generation-mutation.XXXXXX)
generation_reservation_may_be_held=1
generation_before=$(capture_submitter_transport fci1-generation-submitter.service fci1_live_reserve_and_observe_available generation-before "$generation_root")
fabric_ci_run_assertion fci1_assert_replace_held --arg root="$generation_root"
fabric_ci_run_assertion fci1_assert_generation_changed --arg root="$generation_root" --arg before_wire="$generation_before"
fabric_ci_run_assertion fci1_live_release --arg root="$generation_root"
generation_reservation_may_be_held=0
rm -rf -- "$generation_root"
generation_root=

dependent_before=$(capture_submitter_transport fci1-dependent-submitter.service fci1_live_reserve_and_observe_available dependent-before "$mutation_root" --property=RuntimeDirectory="$mutation_runtime")
[[ ! -e $mutation_root && ! -L $mutation_root ]] || { echo 'ExpectedOutcomeNotObserved: submitter runtime directory removal' >&2; exit 1; }
unit_terminal_observed fci1-dependent-submitter.service || { echo 'InstrumentRefused: dependent submitter terminal state unestablished' >&2; exit 1; }
fabric_ci_run_assertion fci1_assert_lifetime_dependent --arg root="$mutation_root" --arg before_wire="$dependent_before"
fabric_ci_run_assertion fci1_assert_cell_unchanged --arg before_wire="$cell_before"

echo 'FCI1LiveAccepted'
