#!/usr/bin/env bash
# Outcome-contract fixture for the frame's single-terminal-line rule (review MAJOR-3).
#
# The collector reads ONE OUTCOME line and its rc. This extracts the frame's own terminal machinery verbatim -- the
# _EMITTED guard, emit_outcome, the EXIT trap, the three signal traps, the PIPE trap and die2 -- and drives it
# through the five ways a run can end. Per case it asserts the NUMBER of OUTCOME lines (exactly one where one is
# owed, never two) and the rc it carries, so a duplicated or missing terminal line cannot survive review.
#
# Exit codes: 0 all controls pass, 1 a control failed, 2 an input is missing or the extraction found nothing.
set -uo pipefail
SELF_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)   # inputs resolve next to me
FRAME=${FRAME:-$SELF_DIR/packet-evidence-freeze2.sh}
[ -r "$FRAME" ] || { echo "ABORT (exit 2): frame not readable: $FRAME"; exit 2; }
[ -s "$FRAME" ] || { echo "ABORT (exit 2): frame is empty: $FRAME"; exit 2; }

W=$(mktemp -d "${TMPDIR:-/var/tmp}/outcome-fixture.XXXXXXXX"); chmod 700 "$W"
trap 'chmod -R u+rwX "$W" 2>/dev/null; rm -rf "$W"' EXIT

first(){ grep -n -m1 "$1" "$FRAME" | cut -d: -f1; }
A=$(first '_EMITTED=0'); B=$(first 'die2() {')
[ -n "$A" ] && [ -n "$B" ] || { echo "ABORT (exit 2): the terminal machinery is not findable in the frame"; exit 2; }
[ "$A" -lt "$B" ] || { echo "ABORT (exit 2): machinery anchors out of order"; exit 2; }
# HOW THE STUB SCRIPT IS ASSEMBLED, and in what order -- because the first version of this put the window first and the
# constants after it, and every case then died on `GROUP_KILL: unbound variable` at window line 113: the frame's terminal
# machinery USES those constants at top level, so they have to be defined before it runs.  Nothing here is retyped: the
# constants come from the extractor (which refuses if a constant used by the copied code was not copied), and every
# function comes from the frame by definition, required to occur exactly once.
python3 "$SELF_DIR/build/extract_helpers.py" "$FRAME" --caps > "$W/caps.out" 2>"$W/caps.err" \
  || { echo "ABORT (exit 2): the extractor refused to copy the frame's constants"; cat "$W/caps.err"; exit 2; }
grep -v '^extracted ' "$W/caps.out" > "$W/machinery.sh"
add_def(){ # add_def <name> -- append the frame's definition of <name>, verbatim
  local n a b
  n=$(grep -cE "^$1\(\) \{" "$FRAME")
  if [ "$n" != 1 ]; then echo "ABORT (exit 2): $1 is defined $n times in the frame (want exactly 1)"; exit 2; fi
  a=$(grep -nE "^$1\(\) \{" "$FRAME" | cut -d: -f1)
  b=$(awk -v s="$a" 'NR>=s && /^\}/ {print NR; exit}' "$FRAME")
  if [ -z "$b" ]; then echo "ABORT (exit 2): no closing brace found for $1 at frame L$a"; exit 2; fi
  sed -n "${a},${b}p" "$FRAME" >> "$W/machinery.sh"
}
add_def precheck_group_kill
echo 'precheck_group_kill' >> "$W/machinery.sh"     # the frame calls this once at start; the stub owes it the same call
sed -n "${A},${B}p" "$FRAME" >> "$W/machinery.sh"
# The signal cases need the frame's OWN handlers and its OWN install lines, appended verbatim by definition -- a
# retyped handler would be a fixture testing the fixture instead of the frame.
for f in child_pids alive_pid pg_of tree_alive reap_phase on_signal on_pipe; do
  grep -qE "^$f\(\) \{" "$W/machinery.sh" || add_def "$f"
done
for ins in 'trap on_exit EXIT' 'trap on_pipe PIPE' "trap 'on_signal TERM 143' TERM" "trap 'on_signal INT  130' INT" \
           "trap 'on_signal HUP  129' HUP"; do
  grep -qxF "$ins" "$W/machinery.sh" || grep -hxF "$ins" "$FRAME" >> "$W/machinery.sh"
done
echo "extracted the frame's terminal machinery verbatim: frame L$A..L$B ($(wc -l < "$W/machinery.sh") lines)"

pass=0; bad=0
# A fixture that runs FEWER cases than it claims is not a smaller suite -- it is a control that stopped running. The
# negctl fixture once printed OK=12 BAD=0 with seven cases never executed, so the declared count is checked here too.
EXPECTED_CASES=8
say(){ printf '%s\n' "$*"; }
ok(){ pass=$((pass+1)); say "OK   $*"; }
no(){ bad=$((bad+1)); say "BAD  $*"; }

drive(){ # drive <label> <expected-outcome-prefix|NONE> <want-rc> <body...>
  local label=$1 want=$2 wantrc=$3; shift 3
  { echo 'set -uo pipefail'; echo 'OUTCOME_WHY=""'
    cat "$W/machinery.sh"
    printf '%s\n' "$@"
  } > "$W/case.sh"
  local out rc
  out=$(bash "$W/case.sh" 2>&1); rc=$?
  local n; n=$(printf '%s' "$out" | grep -c '^OUTCOME:' || true)
  if [ "$want" = NONE ]; then
    if [ "$n" = 0 ]; then ok "$label: no OUTCOME line, exactly as the collector rule expects (rc=$rc)"; else no "$label: printed $n OUTCOME line(s) where none was owed"; fi
    return
  fi
  if [ "$n" != 1 ]; then no "$label: printed $n OUTCOME lines, wanted exactly 1"; return; fi
  local line; line=$(printf '%s' "$out" | grep -m1 '^OUTCOME:')
  case "$line" in
    "$want"*) : ;;
    *) no "$label: OUTCOME line is [$line], wanted [$want*]"; return ;;
  esac
  [ "$rc" = "$wantrc" ] && ok "$label: one line [$line] with rc=$rc" || no "$label: rc=$rc wanted $wantrc (line [$line])"
}

drive "normal completion"        "OUTCOME: PASS rc=0"            0   'OUTCOME_WHY="21 OK / 0 FAIL"; exit 0'
drive "a claim failed"           "OUTCOME: FAIL rc=1"            1   'OUTCOME_WHY="3 claims refused"; exit 1'
drive "die2 refusal"             "OUTCOME: NO_OBSERVATION rc=2"  2   'die2 "RECON_REFUSE: tree mismatch"'
drive "SIGTERM"                  "OUTCOME: NO_OBSERVATION rc=143" 143 'kill -TERM $$; sleep 2'
drive "SIGINT"                   "OUTCOME: NO_OBSERVATION rc=130" 130 'kill -INT $$; sleep 2'

# the wall kill: SIGKILL cannot be trapped, so NO line is printed -- the collector rule, stated in the packet, is
# what makes that case attributable. This asserts the frame does not pretend otherwise by printing something.
{ echo 'set -uo pipefail'; echo 'OUTCOME_WHY=""'; cat "$W/machinery.sh"; echo 'sleep 30 & wait'
} > "$W/kill.sh"
timeout -s KILL 1 bash "$W/kill.sh" > "$W/kill.out" 2>&1; krc=$?
n=$(grep -c '^OUTCOME:' "$W/kill.out" || true)
if [ "$krc" = 137 ] && [ "$n" = 0 ]; then ok "wall kill (SIGKILL): rc=137 and no OUTCOME line -- NO_OBSERVATION by the collector rule, not a silent PASS"
else no "wall kill: rc=$krc with $n OUTCOME line(s); expected 137 and 0"; fi

# a die2 INSIDE a bounded phase must still leave the parent's OUTCOME line attributable to the phase: bcall, not
# the subshell, records the reason. Exercised through the frame's real bcall/bounded pair.
BOUNDED_A=$(first 'bounded() {'); BOUNDED_B=$(first 'T_START=$(date +%s)')
# THE PHASE MACHINERY IS TAKEN BY DEFINITION, NOT BY LINE RANGE. This fixture used to cut bounded()/bcall() out
# of the frame between two line numbers. That is the same trap that broke the run frame itself and the gates
# fixture: a line range is a derived fact, and after a revision that lengthens one function the range silently
# becomes a different program -- here it produced a script that exited 0 and printed PASS. extract_helpers.py
# copies the named definitions character for character and REFUSES if a name is absent, which is the assertion
# this case actually needs.
python3 "$SELF_DIR/build/extract_helpers.py" "$FRAME" --require die2 --require pg_of --require alive_pid \
  --require group_members --require bounded --require bcall > "$W/phasefn.sh" \
  || { echo "ABORT (exit 2): the frame does not define the phase machinery this fixture tests"; exit 2; }
if [ -n "$BOUNDED_A" ] && [ -n "$BOUNDED_B" ]; then
  { echo 'set -uo pipefail'; echo 'OUTCOME_WHY=""'
    cat "$W/machinery.sh"
    cat "$W/phasefn.sh"
    echo 'refuser() { die2 "the phase said no"; }'
    echo 'bcall myphase 30 5 refuser'
    echo 'exit 0'
  } > "$W/bounded.sh"
  out=$(bash "$W/bounded.sh" 2>&1); rc=$?
  line=$(printf '%s' "$out" | grep -m1 '^OUTCOME:')
  n=$(printf '%s' "$out" | grep -c '^OUTCOME:' || true)
  if [ "$n" = 1 ] && [ "$rc" = 2 ] && printf '%s' "$line" | grep -q 'myphase'; then
    ok "a refusal inside a bounded phase stays attributed: [$line] (exactly one line, rc=2)"
  else no "bounded-phase refusal: $n line(s), rc=$rc, line [$line]"; fi
  # and the cap-expiry path: a bounded phase that outruns its cap must print TIMEOUT_KILL and be NO OBSERVATION.
  { echo 'set -uo pipefail'; echo 'OUTCOME_WHY=""'
    cat "$W/machinery.sh"
    cat "$W/phasefn.sh"
    echo 'slow() { sleep 30; }'
    echo 'bcall slow 1 1 slow'
    echo 'exit 0'
  } > "$W/capped.sh"
  out=$(bash "$W/capped.sh" 2>&1); rc=$?
  n=$(printf '%s' "$out" | grep -c '^OUTCOME:' || true)
  line=$(printf '%s' "$out" | grep -m1 '^OUTCOME:')
  if [ "$n" = 1 ] && [ "$rc" = 2 ] && printf '%s' "$out" | grep -q 'TIMEOUT_KILL:' && printf '%s' "$line" | grep -q 'NO_OBSERVATION'; then
    ok "a bounded phase that outruns its cap: TIMEOUT_KILL printed, one NO_OBSERVATION line, rc=2 (never a FAIL)"
  else no "cap-expiry path: $n line(s), rc=$rc, TIMEOUT_KILL=$(printf '%s' "$out" | grep -c 'TIMEOUT_KILL:') , line [$line]"; fi
else
  no "bounded()/bcall not findable in the frame, so bounded-phase attribution is untested"
fi

say ""; # THE CASE LEDGER: what ran must equal what is declared.
[ "$((pass+bad))" = "$EXPECTED_CASES" ] || { bad=$((bad+1)); say "BAD  CASES: ran $((pass+bad)) of $EXPECTED_CASES declared cases -- a shortfall is a control failure, not a smaller suite"; }
say "CASES: declared $EXPECTED_CASES, ran $((pass+bad))"
say "OUTCOME FIXTURE: OK=$pass BAD=$bad"
[ "$bad" = 0 ] || exit 1
