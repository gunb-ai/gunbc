#!/bin/bash
# scratch (NOT for commit): verification A at pushed head; full diagnostics.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtA2
if [ ! -x /tmp/tgtA2/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheA ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch > /tmp/tgtA2-build.log 2>&1
  echo "BUILD_RC=$?"; tail -3 /tmp/tgtA2-build.log
fi
mkdir -p /tmp/zlogA2
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtA2/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogA2/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogA2/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogA2/$b.log)"
  grep '^FAIL' "/tmp/zlogA2/$b.log" | head -4 | cut -c1-1500
}
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
run_one dag/test/claim/fabric/fabric_control_plane_witness.dag
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
run_one dag/test/claim/integration_concern_witness.dag
echo "VA2 DONE"
