#!/bin/bash
# scratch (NOT for commit): D2 runner set, at pushed head.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtD2
RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheD2 ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch > /tmp/d2-build.log 2>&1
echo "BUILD_RC=$?"; tail -2 /tmp/d2-build.log
mkdir -p /tmp/zlogD2
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtD2/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogD2/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogD2/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogD2/$b.log)"
  grep '^FAIL' "/tmp/zlogD2/$b.log" | cut -c1-200
}
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
run_one dag/test/claim/runner/runner_slot_retirement_witness.dag
run_one dag/test/claim/runner/runner_slot_allocation_witness.dag
echo "D2 DONE"
