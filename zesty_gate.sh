#!/bin/bash
# scratch (NOT for commit): compile the instrument module, then run the fci1 witness that drives it.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
if [ ! -x /tmp/tgt/release/claim_batch ] || [ ! -x /tmp/tgt/release/gunbc ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc || { echo "BUILD FAILED"; exit 1; }
fi
rm -rf /tmp/probeout3
/tmp/tgt/release/gunbc compile --source-root dag --source-root src/v2 \
  --entry dag/gunbc/instruments/fabric_control_plane_live_probe.dag --target dag \
  --output-dir /tmp/probeout3 > /tmp/compile3.log 2>&1
echo "COMPILE_RC=$?"
grep -vE "advisory\[|phase=" /tmp/compile3.log | head -40
f=dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
/tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /tmp/fci1_now.log 2>&1
echo "WITNESS_RC=$?"
echo "RESULT fci1 PASS=$(grep -c '^PASS' /tmp/fci1_now.log) FAIL=$(grep -c '^FAIL' /tmp/fci1_now.log)"
grep '^FAIL' /tmp/fci1_now.log | cut -c1-200
echo "GATE DONE"
