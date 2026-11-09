#!/bin/bash
# scratch (NOT for commit): fabric witnesses + gates at pushed head.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtE
RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheE ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/e-build.log 2>&1
echo "BUILD_RC=$?"; tail -2 /tmp/e-build.log
mkdir -p /tmp/zlogE
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "RESULT $b SKIP-no-fns"; return; }
  /tmp/tgtE/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/tmp/zlogE/$b.log" 2>&1
  echo "RESULT $b rc=$? PASS=$(grep -c '^PASS' /tmp/zlogE/$b.log) FAIL=$(grep -c '^FAIL' /tmp/zlogE/$b.log)"
  grep -E 'error:|^FAIL' "/tmp/zlogE/$b.log" | head -6 | cut -c1-420
}
run_one dag/test/claim/fabric/fabric_cell_converge_witness.dag
run_one dag/test/claim/fabric/fabric_cell_observe_witness.dag
run_one dag/test/claim/fabric/fabric_control_plane_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
for h in srv3 srv4; do
  /tmp/tgtE/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one --arg path=provisioning/$h/gunbc-ghrunner.sudoers > /tmp/zlogE/gate_$h.log 2>&1
  echo "GATE_$h rc=$?"; tail -5 "/tmp/zlogE/gate_$h.log" | cut -c1-260
done
echo "E1 DONE"
