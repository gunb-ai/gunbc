#!/bin/bash
# scratch (NOT for commit): fabric set + capacity_plan diagnostics + sudoers gate, at pushed head.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtC
RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheC ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/tgtC-build.log 2>&1
echo "BUILD_RC=$?"; tail -3 /tmp/tgtC-build.log
mkdir -p /tmp/zlogC
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtC/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogC/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogC/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogC/$b.log)"
  grep -A14 '^FAIL' "/tmp/zlogC/$b.log" | head -60 | cut -c1-400
}
run_one dag/test/claim/runner/runner_capacity_plan_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
run_one dag/test/claim/fabric/fabric_control_plane_witness.dag
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
run_one dag/test/claim/integration_concern_witness.dag
for h in srv3 srv4; do
  /tmp/tgtC/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/$h/gunbc-ghrunner.sudoers > /tmp/zlogC/gate_$h.log 2>&1
  echo "GATE_$h rc=$?"; tail -8 "/tmp/zlogC/gate_$h.log" | cut -c1-300
done
echo "VC DONE"
