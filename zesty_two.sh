#!/bin/bash
# scratch (NOT for commit): verify the two repaired witness files.
cd "$(dirname "$0")" || exit 1
CB=/session-home/target-zesty/release/claim_batch
[ -x "$CB" ] || { echo "NO BINARY"; exit 1; }
mkdir -p /session-home/logs
run_one() {
  f="$1"; b=$(basename "$f" .dag)
  fns=$(grep -oP '^test fn \K[A-Za-z0-9_]+' "$f" | paste -sd,)
  [ -z "$fns" ] && { echo "$b SKIP-no-fns"; return; }
  "$CB" --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > "/session-home/logs/$b.log" 2>&1
  echo "RESULT $b exit=$? PASS=$(grep -c '^PASS' /session-home/logs/$b.log) FAIL=$(grep -c '^FAIL' /session-home/logs/$b.log)"
  grep '^FAIL' "/session-home/logs/$b.log" | cut -c1-260 | sed 's/^/  RED /'
  grep -iE "closure diagnostics|error:|not found in scope|does not inhabit" "/session-home/logs/$b.log" | head -12 | sed 's/^/  DIAG /'
}
run_one dag/test/claim/runner/runner_capacity_realize_witness.dag
run_one dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag
echo "TWO DONE"
