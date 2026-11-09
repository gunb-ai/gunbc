#!/bin/bash
# scratch (NOT for commit): which control-plane rows are missing from the batch.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/tmp/tgtF
f=dag/test/claim/fabric/fabric_control_plane_witness.dag
mkdir -p /tmp/zlogF
grep '^PASS' /tmp/zlogF/cp.log | sed 's/^PASS *//' | cut -d' ' -f1 | sort > /tmp/zlogF/passed.txt
grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | sort > /tmp/zlogF/all.txt
echo "=== MISSING (not reported PASS) ==="
comm -23 /tmp/zlogF/all.txt /tmp/zlogF/passed.txt
echo "=== non-phase lines, last 40 ==="
grep -vE '^\[|^\s*$|^(PASS|FAIL)' /tmp/zlogF/cp.log | tail -40 | cut -c1-300
echo "=== individual re-run of missing ==="
for fn in $(comm -23 /tmp/zlogF/all.txt /tmp/zlogF/passed.txt); do
  /tmp/tgtF/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fn" > /tmp/zlogF/one_$fn.log 2>&1
  echo "ONE $fn rc=$? PASS=$(grep -c '^PASS' /tmp/zlogF/one_$fn.log) FAIL=$(grep -c '^FAIL' /tmp/zlogF/one_$fn.log)"
  grep -vE '^\[|^\s*$' /tmp/zlogF/one_$fn.log | grep -v '^claim_batch: resolving' | head -6 | cut -c1-300
done
echo "CP2 DONE"
