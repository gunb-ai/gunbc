#!/usr/bin/env bash
# Runner sudoers activation-grant witness dispatch (ctrl-build --remote, one script).
#
# ONE claim_batch process, ONE entry per dispatch: a cold resolve of a claim entry costs 15-20 minutes on
# the runner and the runner caps a script at 45, so entries are not batched (measured: 3 entries did not
# finish inside the window). claim_batch REQUIRES an explicit --functions list per entry, so each list is
# taken from the entry's own `test fn` declarations -- no test name is hand-copied.
#
# The script must NOT end on a pipe (the runner reads the child's stdout); the last command is a plain echo.
set -uo pipefail
cd "$(dirname "$0")/.."
LOG=/tmp/runner_sudoers_activation_witness.out
: > "$LOG"

cargo build --release -p v1-compiler --bin claim_batch --bin gunbc >>"$LOG" 2>&1
BUILD_RC=$?
echo "BUILD_RC=$BUILD_RC" >>"$LOG"
if [ "$BUILD_RC" != "0" ]; then
  echo "=== BUILD FAILED: no witnesses ran ===" >>"$LOG"
  tail -60 "$LOG"
  exit 1
fi

# Entries come from the command line, one per dispatch: a cold resolve of a claim entry costs on the
# order of 15-20 minutes on the runner, and the runner caps a script at 45, so batching entries in one
# process does not fit the window. No entry is run without its own --functions list.
if [ "$#" -gt 0 ]; then
  ENTRIES=("$@")
else
  ENTRIES=(dag/test/claim/runner/runner_service_activation_witness.dag)
fi


ARGS=(./target/release/claim_batch --source-root dag --source-root src/v1 --source-root src/v2)
TOTAL=0
for e in "${ENTRIES[@]}"; do
  FNS=$(grep -oP '^test fn \K\w+' "$e" | paste -sd, -)
  N=$(grep -c '^test fn' "$e")
  TOTAL=$((TOTAL + N))
  ARGS+=(--entry "$e" --functions "$FNS")
done

echo "=== claim_batch: ${#ENTRIES[@]} entries, $TOTAL tests ===" >>"$LOG"
START=$(date +%s)
timeout 2400 "${ARGS[@]}" 2>&1 | tee -a "$LOG" || true
echo "CLAIM_RC=$? ELAPSED=$(( $(date +%s) - START ))s" >>"$LOG"

SUM=/tmp/runner_sudoers_activation_summary.txt
: > "$SUM"
echo "TESTS_REQUESTED=$TOTAL" >>"$SUM"
echo "CLAIM_LINE: $(grep '^CLAIM_RC' "$LOG")" >>"$SUM"
echo "PASS_COUNT=$(grep -c '^PASS ' "$LOG")" >>"$SUM"
echo "SKIP_COUNT=$(grep -c '^SKIP ' "$LOG")" >>"$SUM"
echo "FAIL_COUNT=$(grep -c '^FAIL ' "$LOG")" >>"$SUM"
grep '^FAIL ' "$LOG" | head -30 >>"$SUM" 2>/dev/null
grep -E 'claim_batch:|error:|panic|does not declare|unknown argument|MAIN_WET_RC' "$LOG" | head -40 >>"$SUM"
cat "$SUM"
echo "DISPATCH DONE"
