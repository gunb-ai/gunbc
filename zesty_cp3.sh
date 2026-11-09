#!/bin/bash
# scratch (NOT for commit): control-plane witness diagnosis, self-contained.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgtF
mkdir -p /tmp/zlogF
if [ ! -x /tmp/tgtF/release/claim_batch ]; then
  RUSTC_WRAPPER= SCCACHE_DIR=/tmp/sccacheF ctrl-build --local -- cargo build --release -p v1-compiler --bin claim_batch --bin gunbc > /tmp/f-build.log 2>&1
  echo "BUILD_RC=$?"; tail -2 /tmp/f-build.log
fi
f=dag/test/claim/fabric/fabric_control_plane_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
/tmp/tgtF/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /tmp/zlogF/cp.log 2>&1
echo "BATCH_RC=$? PASS=$(grep -c '^PASS' /tmp/zlogF/cp.log) FAIL=$(grep -c '^FAIL' /tmp/zlogF/cp.log)"
grep '^PASS' /tmp/zlogF/cp.log | sed 's/^PASS *//' | cut -d' ' -f1 | sort > /tmp/zlogF/passed.txt
grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | sort > /tmp/zlogF/all.txt
echo "=== MISSING ==="; comm -23 /tmp/zlogF/all.txt /tmp/zlogF/passed.txt
echo "=== non-phase tail 30 ==="
grep -vE '^\[|^\s*$|^(PASS|FAIL)' /tmp/zlogF/cp.log | tail -30 | cut -c1-300
echo "CP3 DONE"
