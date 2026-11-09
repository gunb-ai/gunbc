#!/bin/bash
# scratch (NOT for commit): post-rebase verification of every witness file this branch touches.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
if [ ! -x /tmp/tgt/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || {
    echo "BUILD FAILED"; exit 1; }
fi
mkdir -p /tmp/zestylogs
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "$b SKIP-no-fns"; return; }
  /tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" \
    --functions "$fns" > "/tmp/zestylogs/$b.log" 2>&1
  echo "RESULT $b exit=$? PASS=$(grep -c '^PASS' /tmp/zestylogs/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zestylogs/$b.log)"
  grep '^FAIL' "/tmp/zestylogs/$b.log" | sed 's/^/  RED /'
}
run_one dag/test/claim/runner/runner_slot_allocation_witness.dag
run_one dag/test/claim/runner/runner_slot_retirement_witness.dag
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
run_one dag/test/claim/runner/runner_capacity_realize_witness.dag
run_one dag/test/claim/runner/runner_lifecycle_witness.dag
run_one dag/test/claim/runner/runner_microvm_ci_admission_witness.dag
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
echo "FULL DONE"
