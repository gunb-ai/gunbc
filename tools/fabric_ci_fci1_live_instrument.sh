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
subject="$repo_root/dag/gunbc/fabric/fabric_required_build_cell.dag"

cleanup() {
  systemctl stop fci1-positive-submitter.service fci1-dependent-submitter.service >/dev/null 2>&1 || true
  rm -f "$FABRIC_CI_LOG"
  rm -rf "$FABRIC_CI_VALUE_ROOT"
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
fabric_ci_run_assertion fci1_assert_frozen_work_key
fabric_ci_run_assertion fci1_assert_host_preconditions
fabric_ci_run_assertion fci1_assert_clean_cell_prestate
fabric_ci_run_assertion fci1_assert_cell_admitted
cell_before=$(fabric_ci_capture_transport fci1_live_cell_observation cell-before)

capture_submitter_transport() {
  local unit=$1 function_name=$2 coordinate=$3 root=$4
  shift 4
  local value_path="$FABRIC_CI_VALUE_ROOT/$coordinate.wire"
  rm -f -- "$value_path"
  systemd-run --quiet --wait --collect --unit="$unit" "$@" \
    "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
    --entry "$FABRIC_CI_ENTRY" --function "$function_name" --arg root="$root" --arg path="$value_path"
  [[ -f $value_path && ! -L $value_path ]] || return 1
  local value
  value=$(<"$value_path")
  [[ $value =~ ^FCIE0X[0-9A-F]+$ ]] || return 1
  printf '%s' "$value"
}

positive_before=$(capture_submitter_transport fci1-positive-submitter.service fci1_live_reserve_and_observe_available positive-before "$canonical_root")
submitter_state=$(systemctl show fci1-positive-submitter.service --property=ActiveState --value 2>&1 || true)
[[ -z $submitter_state || $submitter_state == inactive || $submitter_state == failed ]] || {
  echo "InstrumentRefused: submitting control process still active: $submitter_state" >&2
  exit 1
}
fabric_ci_run_assertion fci1_assert_lifetime_held --arg root="$canonical_root" --arg before_wire="$positive_before"

generation_before=$(fabric_ci_capture_transport fci1_live_binding_sample_wire generation-before --arg root="$canonical_root")
fabric_ci_run_assertion fci1_assert_replace_held --arg root="$canonical_root"
fabric_ci_run_assertion fci1_assert_generation_changed --arg root="$canonical_root" --arg before_wire="$generation_before"

fabric_ci_run_assertion fci1_live_release --arg root="$canonical_root"
fabric_ci_run_assertion fci1_assert_cell_unchanged --arg before_wire="$cell_before"

dependent_before=$(capture_submitter_transport fci1-dependent-submitter.service fci1_live_reserve_and_observe_available dependent-before "$mutation_root" --property=RuntimeDirectory="$mutation_runtime")
[[ ! -e $mutation_root ]] || { echo 'ExpectedOutcomeNotObserved: submitter runtime directory removal' >&2; exit 1; }
fabric_ci_run_assertion fci1_assert_lifetime_dependent --arg root="$mutation_root" --arg before_wire="$dependent_before"
fabric_ci_run_assertion fci1_assert_cell_unchanged --arg before_wire="$cell_before"

echo 'FCI1LiveAccepted'
