#!/bin/bash
# scratch (NOT for commit): D1 fabric set + sudoers gate, at pushed head.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtD1
RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheD1 ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/d1-build.log 2>&1
echo "BUILD_RC=$?"; tail -2 /tmp/d1-build.log
mkdir -p /tmp/zlogD1
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtD1/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogD1/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogD1/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogD1/$b.log)"
  grep -A6 '^FAIL' "/tmp/zlogD1/$b.log" | head -40 | cut -c1-380
}
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
run_one dag/test/claim/fabric/fabric_control_plane_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
run_one dag/test/claim/integration_concern_witness.dag
for h in srv3 srv4; do
  /tmp/tgtD1/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/$h/gunbc-ghrunner.sudoers > /tmp/zlogD1/gate_$h.log 2>&1
  echo "GATE_$h rc=$?"; tail -6 "/tmp/zlogD1/gate_$h.log" | cut -c1-260
done
echo "D1 DONE"
