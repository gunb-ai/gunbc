#!/bin/bash
# scratch (NOT for commit): verify the instrument + witnesses over the unions I extended (no compile verb).
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
if [ ! -x /tmp/tgt/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || { echo "BUILD FAILED"; exit 1; }
fi
mkdir -p /tmp/zlog5
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlog5/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlog5/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlog5/$b.log)"
  grep '^FAIL' "/tmp/zlog5/$b.log" | cut -c1-230 | sed 's/^/  RED /'
  grep -vE "^(PASS|FAIL|advisory)" "/tmp/zlog5/$b.log" | head -8 | sed 's/^/  DIAG /'
}
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
run_one dag/test/claim/fabric/fabric_control_plane_witness.dag
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
echo "FAB2 DONE"
