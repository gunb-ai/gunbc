#!/bin/bash
# scratch (NOT for commit): runner-set witnesses + generated-artifact gates at pushed head.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtF
[ -x /tmp/tgtF/release/claim_batch ] || { RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheF ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/f-build.log 2>&1; echo "BUILD_RC=$?"; }
mkdir -p /tmp/zlogF
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtF/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogF/$b.log" 2>&1
  echo "RESULT $b PASS=$(grep -c '^PASS' /tmp/zlogF/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogF/$b.log)"
  grep -E 'error:' "/tmp/zlogF/$b.log" | head -3 | cut -c1-300
  grep '^FAIL' "/tmp/zlogF/$b.log" | head -3 | cut -c1-170
}
run_one dag/test/claim/runner/runner_slot_allocation_witness.dag
run_one dag/test/claim/runner/runner_slot_retirement_witness.dag
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
run_one dag/test/claim/runner/runner_capacity_realize_witness.dag
run_one dag/test/claim/runner/runner_lifecycle_witness.dag
run_one dag/test/claim/runner/runner_microvm_ci_admission_witness.dag
for h in srv3 srv4; do
  /tmp/tgtF/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/$h/gunbc-ghrunner.sudoers > /tmp/zlogF/gate_$h.log 2>&1
  echo "GATE_$h rc=$?"; tail -3 /tmp/zlogF/gate_$h.log | cut -c1-240
done
echo "RUNNER SET DONE"
