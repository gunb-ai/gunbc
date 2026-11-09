#!/bin/bash
# scratch (NOT for commit): final verification B -- runner set + generated artifact gate.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgtB
if [ ! -x /tmp/tgtB/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheB ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch > /tmp/tgtB-build.log 2>&1
  echo "BUILD_RC=$?"; tail -3 /tmp/tgtB-build.log
fi
mkdir -p /tmp/zlogB
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtB/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogB/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogB/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogB/$b.log)"
  grep '^FAIL' "/tmp/zlogB/$b.log" | cut -c1-200
}
run_one dag/test/claim/runner/runner_slot_allocation_witness.dag
run_one dag/test/claim/runner/runner_slot_retirement_witness.dag
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
run_one dag/test/claim/runner/runner_capacity_realize_witness.dag
run_one dag/test/claim/runner/runner_lifecycle_witness.dag
/tmp/tgtB/release/claim_batch --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/srv3/gunbc-ghrunner.sudoers > /tmp/zlogB/gate3.log 2>&1
echo "GATE_SRV3 rc=$?"; tail -6 /tmp/zlogB/gate3.log | cut -c1-200
/tmp/tgtB/release/claim_batch --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/srv4/gunbc-ghrunner.sudoers > /tmp/zlogB/gate4.log 2>&1
echo "GATE_SRV4 rc=$?"; tail -6 /tmp/zlogB/gate4.log | cut -c1-200
echo "VB DONE"
