#!/bin/bash
# scratch (NOT for commit): fci1 witness verdict after the import-list repair.
cd "$(dirname "$0")" || exit 1
export CARGO_TARGET_DIR=/session-home/target-zesty
mkdir -p /session-home/logs
f=dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
/session-home/target-zesty/release/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > /session-home/logs/fci1_v2.log 2>&1
echo "RC=$?"
echo "PASS=$(grep -c '^PASS' /session-home/logs/fci1_v2.log) FAIL=$(grep -c '^FAIL' /session-home/logs/fci1_v2.log)"
grep '^FAIL' /session-home/logs/fci1_v2.log | cut -c1-160
head -3 /session-home/logs/fci1_v2.log | cut -c1-250
