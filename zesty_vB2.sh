#!/bin/bash
# scratch (NOT for commit): verification B at pushed head; full diagnostics.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtB2
if [ ! -x /tmp/tgtB2/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheB ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch > /tmp/tgtB2-build.log 2>&1
  echo "BUILD_RC=$?"; tail -3 /tmp/tgtB2-build.log
fi
mkdir -p /tmp/zlogB2
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtB2/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogB2/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogB2/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogB2/$b.log)"
  grep '^FAIL' "/tmp/zlogB2/$b.log" | head -4 | cut -c1-1500
}
run_one dag/test/claim/runner/runner_slot_allocation_witness.dag
run_one dag/test/claim/runner/runner_slot_retirement_witness.dag
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
run_one dag/test/claim/runner/runner_capacity_realize_witness.dag
run_one dag/test/claim/runner/runner_lifecycle_witness.dag
/tmp/tgtB2/release/claim_batch --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/srv3/gunbc-ghrunner.sudoers > /tmp/zlogB2/gate3.log 2>&1
echo "GATE_SRV3 rc=$?"; tail -4 /tmp/zlogB2/gate3.log | cut -c1-300
/tmp/tgtB2/release/claim_batch --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/srv4/gunbc-ghrunner.sudoers > /tmp/zlogB2/gate4.log 2>&1
echo "GATE_SRV4 rc=$?"; tail -4 /tmp/zlogB2/gate4.log | cut -c1-300
echo "VB2 DONE"
