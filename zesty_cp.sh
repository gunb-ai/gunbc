#!/bin/bash
# scratch (NOT for commit): control-plane witness detail at pushed head.
cd "$(dirname "$0")" || exit 1
echo "SYNCED_HEAD=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/tmp/tgtF
[ -x /tmp/tgtF/release/claim_batch ] || { RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheF ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/f-build.log 2>&1; echo "BUILD_RC=$?"; }
mkdir -p /tmp/zlogF
f=dag/test/claim/fabric/fabric_control_plane_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
echo "FN_COUNT=$(echo $fns | tr ',' '\n' | wc -l)"
/tmp/tgtF/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /tmp/zlogF/cp.log 2>&1
echo "CP_RC=$?"
echo "CP_PASS=$(grep -c '^PASS' /tmp/zlogF/cp.log) CP_FAIL=$(grep -c '^FAIL' /tmp/zlogF/cp.log)"
echo "--- non-PASS lines (first 25) ---"
grep -v '^PASS' /tmp/zlogF/cp.log | head -25 | cut -c1-300
echo "--- last 6 lines ---"
tail -6 /tmp/zlogF/cp.log | cut -c1-300
echo "CP DONE"
