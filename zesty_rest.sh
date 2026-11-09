#!/bin/bash
# scratch (NOT for commit): post-rebase re-verification, remaining touched witnesses.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgt
[ -x /tmp/tgt/release/claim_batch ] || RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccache ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch || exit 1
mkdir -p /tmp/zlog
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgt/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlog/$b.log" 2>&1
  rc=$?
  echo "RESULT $b rc=$rc PASS=$(grep -c '^PASS' /tmp/zlog/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlog/$b.log)"
  grep '^FAIL' "/tmp/zlog/$b.log" | cut -c1-260 | sed 's/^/  RED /'
}
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/runner/runner_lifecycle_witness.dag
run_one dag/test/claim/runner/runner_microvm_ci_admission_witness.dag
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
echo "REST DONE"
