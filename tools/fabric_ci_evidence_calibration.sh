#!/usr/bin/env bash
set -euo pipefail

# Executable FCI-EVIDENCE-0 calibration. Source mutation uses an explicit verified byte snapshot;
# restore never consults git, because remote runners reconstruct the tested tree as a patch.

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export FABRIC_CI_GUNBC_BIN=${FABRIC_CI_GUNBC_BIN:-"$repo_root/target/release/gunbc"}
export FABRIC_CI_SOURCE_ROOT=$repo_root
export FABRIC_CI_ENTRY="$repo_root/dag/gunbc/instruments/fabric_ci_evidence.dag"
export FABRIC_CI_LOG
FABRIC_CI_LOG=$(mktemp)
snapshot=$(mktemp)
cp "$FABRIC_CI_ENTRY" "$snapshot"

cleanup() {
  cp "$snapshot" "$FABRIC_CI_ENTRY"
  rm -f "$snapshot" "$FABRIC_CI_LOG"
}
trap cleanup EXIT

# shellcheck source=tools/fabric_ci_evidence_driver.sh
source "$repo_root/tools/fabric_ci_evidence_driver.sh"
fabric_ci_driver_init

if [[ ! -x $FABRIC_CI_GUNBC_BIN ]]; then
  echo 'FCI-EVIDENCE-0 refused: exact-tree ./target/release/gunbc absent' >&2
  exit 2
fi

echo 'subject-hash-before:'
sha256sum "$FABRIC_CI_ENTRY"
cmp -s "$snapshot" "$FABRIC_CI_ENTRY"

fabric_ci_run_assertion fabric_ci_calibration_assertion_success
fabric_ci_observe_status 2 "$FABRIC_CI_GUNBC_BIN" run \
  --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
  --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_bool_must_refuse
fabric_ci_observe_status 2 "$FABRIC_CI_GUNBC_BIN" run \
  --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
  --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_raw_coproduct_must_not_cross

wire=$(fabric_ci_capture_transport fabric_ci_calibration_transport)
fabric_ci_run_assertion fabric_ci_calibration_assert_transport --arg wire="$wire"

last=${wire: -1}
if [[ $last == 0 ]]; then replacement=1; else replacement=0; fi
altered=${wire::-1}$replacement
if fabric_ci_run_assertion fabric_ci_calibration_assert_transport --arg wire="$altered"; then
  echo 'FCI-EVIDENCE-0 falsifier failed: altered transport byte was accepted' >&2
  exit 1
fi

if fabric_ci_run_assertion fabric_ci_calibration_collapsed_projection_must_red; then
  echo 'FCI-EVIDENCE-0 falsifier failed: collapsed projection was not red' >&2
  exit 1
fi

if fabric_ci_capture_transport fabric_ci_calibration_transport_converted_to_exit >/dev/null; then
  echo 'FCI-EVIDENCE-0 falsifier failed: ProcessExit was accepted as value transport' >&2
  exit 1
fi

# Deleting one arm must be a compile-time failure, not a fallback rendering.
sed -i '/FabricCiCalibrationBeta { payload: payload } =>/d' "$FABRIC_CI_ENTRY"
echo 'subject-hash-after-mutation:'
sha256sum "$FABRIC_CI_ENTRY"
if "$FABRIC_CI_GUNBC_BIN" run \
    --source-root "$repo_root/dag" --source-root "$repo_root/src/v2" \
    --entry "$FABRIC_CI_ENTRY" --function fabric_ci_calibration_assertion_success \
    2>&1 | tee -a "$FABRIC_CI_LOG" /dev/stderr; then
  echo 'FCI-EVIDENCE-0 falsifier failed: missing projection arm compiled' >&2
  exit 1
fi

cp "$snapshot" "$FABRIC_CI_ENTRY"
echo 'subject-hash-after-restore:'
sha256sum "$FABRIC_CI_ENTRY"
cmp -s "$snapshot" "$FABRIC_CI_ENTRY"
echo 'subject-hash-before-final-rerun:'
sha256sum "$FABRIC_CI_ENTRY"
fabric_ci_run_assertion fabric_ci_calibration_assertion_success

echo 'FCI-EVIDENCE-0 calibration held'
