#!/bin/bash
# scratch (NOT for commit): verify the instrument + the witnesses over the unions I extended.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
if [ ! -x /tmp/tgt/release/claim_batch ] || [ ! -x /tmp/tgt/release/gunbc ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc || { echo "BUILD FAILED"; exit 1; }
fi
rm -rf /tmp/probeout4
/tmp/tgt/release/gunbc compile --source-root dag --source-root src/v2 \
  --entry dag/gunbc/instruments/fabric_control_plane_live_probe.dag --target dag \
  --output-dir /tmp/probeout4 > /tmp/compile4.log 2>&1
echo "COMPILE_RC=$?"
grep -vE "advisory\[|phase=" /tmp/compile4.log | head -30
mkdir -p /tmp/zlog4
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlog4/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlog4/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlog4/$b.log)"
  grep '^FAIL' "/tmp/zlog4/$b.log" | cut -c1-230 | sed 's/^/  RED /'
  grep -vE "^(PASS|FAIL|advisory)" "/tmp/zlog4/$b.log" | head -6 | sed 's/^/  DIAG /'
}
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
run_one dag/test/claim/fabric/fabric_control_plane_witness.dag
run_one dag/test/claim/fabric/fabric_allocation_store_seal_witness.dag
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
echo "FAB DONE"
