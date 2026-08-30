#!/usr/bin/env bash
set -euo pipefail
# SCAFFOLD — dissolve-on: modeled lifecycle actuation sufficient to sequence a wet gate from .dag.
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export FABRIC_CI_GUNBC_BIN=${FABRIC_CI_GUNBC_BIN:-"$repo_root/target/release/gunbc"}
export FABRIC_CI_SOURCE_ROOT=$repo_root FABRIC_CI_ENTRY="$repo_root/dag/gunbc/instruments/fabric_ci_evidence.dag"
export FABRIC_CI_LOG FABRIC_CI_VALUE_ROOT
FABRIC_CI_LOG=$(mktemp); FABRIC_CI_VALUE_ROOT=$(mktemp -d); snapshot=$(mktemp)
cp "$FABRIC_CI_ENTRY" "$snapshot"
cleanup() { cp "$snapshot" "$FABRIC_CI_ENTRY"; rm -f "$snapshot" "$FABRIC_CI_LOG"; rm -rf "$FABRIC_CI_VALUE_ROOT"; }
trap cleanup EXIT
source "$repo_root/tools/fabric_ci_evidence_driver.sh"
fabric_ci_driver_init
[[ -x $FABRIC_CI_GUNBC_BIN ]] || exit 2
sha256sum "$FABRIC_CI_ENTRY"; cmp -s "$snapshot" "$FABRIC_CI_ENTRY"
fabric_ci_run_assertion fabric_ci_calibration_assertion_success
fabric_ci_observe_status 2 "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_bool_must_refuse
fabric_ci_observe_status 2 "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_raw_coproduct_must_not_cross
alpha=$(fabric_ci_capture_transport fabric_ci_calibration_transport_alpha alpha)
beta=$(fabric_ci_capture_transport fabric_ci_calibration_transport_beta beta)
fabric_ci_run_assertion fabric_ci_calibration_assert_transport --arg wire="$alpha"
fabric_ci_run_assertion fabric_ci_calibration_assert_beta_transport --arg wire="$beta"
if fabric_ci_run_assertion fabric_ci_calibration_assert_transport --arg wire="$beta"; then exit 1; fi
# A perfectly framed value in the interpreter diagnostic cannot substitute for the absent file.
if fabric_ci_capture_transport fabric_ci_calibration_diagnostic_forgery diagnostic-only >/dev/null; then exit 1; fi
last=${alpha: -1}; [[ $last == 0 ]] && replacement=1 || replacement=0
altered=${alpha::-1}$replacement
if fabric_ci_run_assertion fabric_ci_calibration_assert_transport --arg wire="$altered"; then exit 1; fi
if fabric_ci_run_assertion fabric_ci_calibration_collapsed_projection_must_red; then exit 1; fi
# The real transport consumes Write.success and therefore refuses a write failure.
if fabric_ci_run_assertion fabric_ci_calibration_transport_alpha --arg path=/proc/fci-evidence-write-refusal; then exit 1; fi
# This deliberately defective entry ignores Write.success. Status zero cannot replace its absent file.
if fabric_ci_capture_transport fabric_ci_calibration_transport_ignores_write_failure ignored >/dev/null; then exit 1; fi
sed -i '/FabricCiCalibrationBeta { payload: payload } =>/d' "$FABRIC_CI_ENTRY"
if "$FABRIC_CI_GUNBC_BIN" run --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_assertion_success 2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr; then exit 1; fi
cp "$snapshot" "$FABRIC_CI_ENTRY"; cmp -s "$snapshot" "$FABRIC_CI_ENTRY"
fabric_ci_run_assertion fabric_ci_calibration_assertion_success
echo 'FCI-EVIDENCE-0 calibration held'
