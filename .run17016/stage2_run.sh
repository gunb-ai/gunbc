#!/usr/bin/env bash
# stage2_run v2: VERDICT legs on the TEST route, per eager-koi-371 (the run route refuses Bool
# claims by design; its refusal is kept below as ONE declared fail-closed CONTROL, excluded from
# the claim tally). Shape: ( cd $TREE && "$BIN" test "//test/claim/filesystem_effect_reach_wall/<fn>"
#   --source-root dag --source-root src/v1 --source-root src/v2 ), one invocation per claim, rc from
# the subshell, real outcome lines HELD outcome=Pass / DID-NOT-HOLD. VOID if any claim is refused,
# unobserved, or has no outcome. Population is MEASURED, never remembered (gate below).
set -uo pipefail
HEAD="${1:-}"
EXPECTED_CLAIMS="${2:-34}"
# The leg's own script appends its RC line as its last act (never a parent shell that may exit first).
trap 'echo "STAGE2_LEG_RC=$? APPENDED_BY_SCRIPT_AT=$(date -u +%FT%TZ)"' EXIT
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/stage_common.sh"
export HEAVY_OWNER=eager-koi-371
TREE="$SCRATCH/head-$HEAD"
BIN="$TREE/target/release/gunbc"
WIT="dag/test/claim/filesystem_effect_reach_wall.dag"
LABEL_PREFIX="//test/claim/filesystem_effect_reach_wall"
RUNLOG="$SCRATCH/module_run.log"
: > "$RUNLOG"
FRESH_RC=0
{
  echo "HOST=$(hostname) BOOT=$(boot_id) UTC=$(date -u +%FT%TZ)"
  sha_line "$BIN"; print_cgroup; assert_fresh "$BIN"
} || FRESH_RC=$?
if [ "$FRESH_RC" -ne 0 ]; then echo "VOID_STOP_NOT_FRESH rc=$FRESH_RC (see FRESHNESS_ASSERT_RC line)"; exit 6; fi
INSTR_RC=0; instr_check "$TREE" || INSTR_RC=$?
if [ "$INSTR_RC" -ne 0 ]; then echo "VOID_STOP_INSTRUMENT count_must_be_nonzero"; exit 6; fi
BIN_PROBE_RC=0; binary_probe "$BIN" || BIN_PROBE_RC=$?
if [ "$BIN_PROBE_RC" -ne 0 ]; then echo "VOID_STOP_BINARY_ROUTE"; exit 6; fi

# Population: claims are declared 'test fn <name>() -> Bool' (grammar: src/v2/test/claim/lexical_scope.dag).
ENUM_CMD='grep -nE "^test fn " '"$WIT"
DERIVED_COUNT=$(grep -cE '^test fn ' "$TREE/$WIT"); DERIVED_RC=$?
echo "ENUM_CMD=$ENUM_CMD DERIVED_COUNT=$DERIVED_COUNT ENUM_RC=$DERIVED_RC EXPECTED_CLAIMS=$EXPECTED_CLAIMS"
if [ "$DERIVED_COUNT" -ne "$EXPECTED_CLAIMS" ]; then echo "VOID_STOP_EXPECTED_NE_DERIVED derived=$DERIVED_COUNT expected=$EXPECTED_CLAIMS (memory-gate class failure: expected must be MEASURED)"; exit 6; fi
CLAIM_NAMES=$(grep -oE '^test fn [a-z0-9_]+' "$TREE/$WIT" | sed 's/^test fn //')
DECLARED_COUNT=$(printf '%s\n' "$CLAIM_NAMES" | grep -c .)
echo "CLAIM_NAMES_BEGIN"; printf '%s\n' "$CLAIM_NAMES"; echo "CLAIM_NAMES_END"
echo "DECLARED_COUNT=$DECLARED_COUNT"
if [ "$DECLARED_COUNT" -ne "$EXPECTED_CLAIMS" ] || [ "$DECLARED_COUNT" -eq 0 ]; then echo "VOID_STOP_CLAIM_COUNT_UNEXPECTED expected=$EXPECTED_CLAIMS got=$DECLARED_COUNT (empty population is VOID, not green)"; exit 6; fi
SAMPLE_LABEL="$LABEL_PREFIX:$(printf '%s\n' "$CLAIM_NAMES" | head -1)"
echo "SAMPLE_LABEL_VERBATIM=$SAMPLE_LABEL"
# NEGATIVE TEST of the population gate, printed IN the receipt: a deliberately wrong expected
# count must VOID. A guard nobody has watched fail is decoration.
( EXPECTED_CLAIMS=$((EXPECTED_CLAIMS + 66)); DECLARED_COUNT=$DECLARED_COUNT; \
  if [ "$DECLARED_COUNT" -ne "$EXPECTED_CLAIMS" ]; then echo "NEGATIVE_TEST gate fired as required on wrong expected=$EXPECTED_CLAIMS (declared=$DECLARED_COUNT)"; exit 6; fi )
echo "NEGATIVE_TEST_RC=$? (6 expected: the gate rejects a remembered number)"

# CONTROL (declared, excluded from the tally): the run route refuses Bool claims by design; this
# proves a Bool cannot be laundered into a green through the run route. rc 2 + refusal text on disk.
CTRL_CLAIM=$(printf '%s\n' "$CLAIM_NAMES" | head -1)
CTRL_OUT_FILE="$SCRATCH/control_run_route_bool.out"
( cd "$TREE" && timeout 300 "$BIN" run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$WIT" --function "$CTRL_CLAIM" ) > "$CTRL_OUT_FILE" 2>&1
echo "CONTROL run-route Bool claim $CTRL_CLAIM RC=$? (excluded from tally; expect 2 + ProcessExit refusal)" | tee -a "$RUNLOG"
grep -a "not ProcessExit\|refused" "$CTRL_OUT_FILE" | head -2 >> "$RUNLOG"

SELECTED=0; HELD=0; N_DIDNOTHOLD=0; N_REFUSED=0; N_PANIC=0; TOTAL_SELECTED=0; START=$(date +%s)
for c in $CLAIM_NAMES; do
  if [ $(( $(date +%s) - START )) -gt 21000 ]; then echo "COMPLETENESS=INCOMPLETE_PARTIAL elapsed_s=$(( $(date +%s) - START ))"; exit 6; fi
  OUT_FILE="$SCRATCH/claim_$c.out"
  LABEL="$LABEL_PREFIX:$c"
  echo "LABEL=$LABEL"
  ( cd "$TREE" && timeout 900 "$BIN" test "$LABEL" --source-root dag --source-root src/v1 --source-root src/v2 ) > "$OUT_FILE" 2>&1
  RC=$?
  # Standard claim-leg cross-check (Manager B): the runner cannot distinguish a mistyped label from
  # an honestly empty module (both print rc 2 + "0 claim(s)"), so the OUTSIDE declared count from the
  # SAME grep that produced the names decides, not the runner. selected < declared => VOID whatever rc.
  SELECTED_ONE=$(grep -aoE "[0-9]+ claim\(s\)" "$OUT_FILE" | head -1 | grep -oE "^[0-9]+")
  SELECTED_ONE=${SELECTED_ONE:-0}
  if [ "$SELECTED_ONE" -eq 1 ] && [ "$RC" -eq 0 ]; then V=OK; else V=VOID; fi
  echo "LABEL=$LABEL DECLARED=1 SELECTED=$SELECTED_ONE VERDICT=$V" | tee -a "$RUNLOG"
  if [ "$V" = VOID ]; then echo "VOID_STOP_LABEL_UNDERSELECTED label=$LABEL selected=$SELECTED_ONE rc=$RC (rc 2 + 0 claim(s) is not a verdict)"; exit 6; fi
  TOTAL_SELECTED=$((TOTAL_SELECTED + SELECTED_ONE))
  echo "TEST $c rc=$RC out_bytes=$(wc -c < "$OUT_FILE")" | tee -a "$RUNLOG"
  grep -aE "HELD|DID-NOT-HOLD|outcome=|refus|error|panic" "$OUT_FILE" | head -3 >> "$RUNLOG"; echo "----" >> "$RUNLOG"
  if [ "$RC" -eq 101 ] || grep -aq "panicked" "$OUT_FILE"; then N_PANIC=$((N_PANIC+1)); fi
  if grep -aq "DID-NOT-HOLD" "$OUT_FILE"; then N_DIDNOTHOLD=$((N_DIDNOTHOLD+1)); fi
  if grep -aqE "refus" "$OUT_FILE" && ! grep -aqE "HELD" "$OUT_FILE"; then N_REFUSED=$((N_REFUSED+1)); fi
  if grep -aqE "HELD outcome=Pass" "$OUT_FILE" && [ "$RC" -eq 0 ]; then SELECTED=$((SELECTED+1)); HELD=$((HELD+1)); fi
done
echo "TOTAL_SELECTED=$TOTAL_SELECTED (sum of runner-reported per-label selected; must equal DECLARED=$DECLARED_COUNT)"
echo "SELECTED=$SELECTED HELD=$HELD DECLARED=$DECLARED_COUNT N_DIDNOTHOLD=$N_DIDNOTHOLD N_REFUSED=$N_REFUSED N_PANIC=$N_PANIC WALL_S=$(( $(date +%s) - START ))"
if [ "$N_PANIC" -gt 0 ]; then echo "VOID_STOP_PANIC (rc 101 is never a refusal)"; exit 6; fi
if [ "$N_REFUSED" -gt 0 ]; then echo "VOID_STOP_CLAIM_REFUSED unobserved claims are not greens"; exit 6; fi
if [ "$SELECTED" -ne "$DECLARED_COUNT" ]; then echo "VOID_STOP_SELECTED_NE_DECLARED real_outcomes=$SELECTED declared=$DECLARED_COUNT"; exit 6; fi
if [ "$N_DIDNOTHOLD" -ne 0 ]; then echo "LEG_RED_DID_NOT_HOLD=$N_DIDNOTHOLD (a claim that did not hold is a red, not a void)"; exit 5; fi
echo "STAGE2_VERDICT ALL_CLAIMS_HELD selected=$SELECTED declared=$DECLARED_COUNT"
