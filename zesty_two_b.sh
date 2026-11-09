#!/bin/bash
# scratch (NOT for commit): post-rebase re-verification of the two biggest touched witnesses.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt2
if [ ! -x /tmp/tgt2/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache2 ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || { echo "BUILD FAILED"; exit 1; }
fi
mkdir -p /tmp/zestylogs2
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "$b SKIP-no-fns"; return; }
  /tmp/tgt2/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zestylogs2/$b.log" 2>&1
  echo "RESULT $b exit=$? PASS=$(grep -c '^PASS' /tmp/zestylogs2/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zestylogs2/$b.log)"
  grep '^FAIL' "/tmp/zestylogs2/$b.log" | cut -c1-240 | sed 's/^/  RED /'
  grep -vE "^(PASS|FAIL|advisory)" "/tmp/zestylogs2/$b.log" | head -8 | sed 's/^/  DIAG /'
}
run_one dag/test/claim/runner/runner_slot_allocation_witness.dag
run_one dag/test/claim/runner/runner_slot_retirement_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
echo "TWO_B DONE"
