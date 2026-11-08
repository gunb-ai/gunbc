#!/usr/bin/env bash
# cap_kill_fixture_v1.sh <frame> <frame-sha> -- the EXECUTABLE control for N1 (caps must actually kill) and for the
# rc=137 status fix (minor m1). Primitive level, deliberately.
#
# WHY PRIMITIVE LEVEL WHEN THE HERMETIC CONTROL RUNS THE WHOLE FRAME. The whole frame can only be made to overrun a
# cap by waiting for a real one (CAP_BUILD 454 s + grace 15 s), and it cannot be shown failing at all without
# editing the frozen bytes. So the failure modes are reproduced here, on an EXTRACTED COPY of the frame's own
# functions -- the frame itself is never written to, and its sha is asserted before and after. What this file
# proves is about the mechanism; what the hermetic overrun case proves is that the frozen frame applies that
# mechanism at its real caps. Both receipts are published, and each says which of the two it is.
#
# EVIDENCE STANDARD (keen-otter msg_53860227): raw wall time, raw exit status, and PROCESS LIVENESS ON RECORDED
# PIDS. A receipt label is never the assertion -- a receipt that lies about a kill is the defect under test.
#
# WHY RECORDED PIDS AND NOT `pkill -f <pattern>`: during development this fixture's cleanup pattern matched the
# command line of the very process that had launched it (which merely contained the pattern text as source) and
# sent it SIGTERM. Pattern cleanup can therefore kill an unrelated job; recorded pids cannot. The body of each
# phase writes its own pid and the pid of the descendant it spawns into $W/rec.<case>, and liveness is asked about
# exactly those pids.
set -uo pipefail
FRAME=${1:-}; WANT_SHA=${2:-}
[ -n "$FRAME" ] && [ -f "$FRAME" ] || { echo "usage: $0 <frame> <frame-sha>"; exit 2; }
frame_sha() { sha256sum -- "$FRAME" | cut -d' ' -f1; }
have_sha() { [ "$(frame_sha)" = "$WANT_SHA" ]; }
have_sha || { echo "REFUSED: frame sha is not the pinned $WANT_SHA"; exit 2; }
START_SHA=$(frame_sha)

W=$(mktemp -d "${TMPDIR:-/tmp}/capkill-XXXXXX")
cleanup() { # kill ONLY the pids this fixture recorded, then remove the work dir
  local f p
  for f in "$W"/rec.*; do
    [ -f "$f" ] || continue
    while read -r p; do case "$p" in ''|*[!0-9]*) continue ;; esac; kill -9 "$p" 2>/dev/null || true; done < "$f"
  done
  rm -rf "$W"
}
trap cleanup EXIT

# ---------------------------------------------------------------- the frame's OWN functions, verbatim
python3 "$(dirname "$0")/extract_helpers.py" "$FRAME" --caps \
  --require die2 --require pg_of --require alive_pid --require group_members --require tree_alive \
  --require precheck_group_kill \
  > "$W/extracted.sh" || exit 2
{ printf 'OUTCOME=NOTSET; OUTCOME_WHY=""\nemit_outcome() { :; }\n'   # die2 sets these; the fixture reads rc instead
  cat "$W/extracted.sh"; } > "$W/fixed.sh"
[ -s "$W/fixed.sh" ] || { echo "REFUSED: extraction produced nothing"; exit 2; }

# The two NAIVE variants are re-created from the fixed text by NAMED substitutions, so each one differs from the
# frozen frame in exactly the way it is claimed to, and nothing else.
python3 - "$W/extracted.sh" "$W/naive.sh" <<'PY'
import sys
src = open(sys.argv[1]).read()
new_launch = 'if [ "$(pg_of "$pid")" = "$pid" ]; then pg=1; tgt="-$pid"; else pg=0; tgt="$pid"; fi'
new_pg = 'local pg=${GROUP_KILL-1}\n  if [ "$pg" = 1 ]; then set -m 2>/dev/null || pg=0; else set +m 2>/dev/null || true; fi'
v5_pg = '''local pg=${GROUP_KILL-}
  if [ -z "$pg" ]; then if set -m 2>/dev/null; then pg=1; set +m 2>/dev/null || true; else pg=0; fi; fi
  [ "$pg" = 1 ] && { set -m 2>/dev/null || pg=0; set +m 2>/dev/null || true; }'''
for pat, want, what in ((new_launch, 2, 'per-launch pgid measurement'), (new_pg, 2, 'job-control block')):
    if src.count(pat) != want:
        sys.exit("REFUSED: expected %d %s in the extracted text, found %d" % (want, what, src.count(pat)))
out = src.replace(new_pg, v5_pg).replace(
    new_launch, 'tgt="-$pid"   # v5 ASSUMED the child was a group leader; it never measured one')
traps = out.count('{ trap \'echo "$?" > "$sf"\' EXIT; "$@"; } & pid=$!') + \
        out.count('{ trap \'echo "$?" > "$sf"\' EXIT; "$fn" "$@"; } & pid=$!')
if traps != 2: sys.exit("REFUSED: expected 2 trapped launches, found %d" % traps)
out = out.replace('{ trap \'echo "$?" > "$sf"\' EXIT; "$@"; } & pid=$!', '{ "$@"; echo "$?" > "$sf"; } & pid=$!')
out = out.replace('{ trap \'echo "$?" > "$sf"\' EXIT; "$fn" "$@"; } & pid=$!', '{ "$fn" "$@"; echo "$?" > "$sf"; } & pid=$!')
open(sys.argv[2], 'w').write("# NAIVE (v5) launch: job control off before the launch, group target assumed, status\n"
                             "# written after the call. Derived from the frozen text by the named substitutions above.\n" + out)
print("naive variant written")
PY
[ -f "$W/naive.sh" ] || exit 2
# v5-EXACT: the naive launch AND the way v5 ended its ladder -- an unconditional `wait`. A group kill that reaches
# nothing then lets `wait` block until the child finishes by itself, the scan afterwards sees an empty group, and
# the receipt claims 'killed=yes' for a kill that never happened. That laundering is what review measured.
python3 - "$W/naive.sh" "$W/naive2.sh" <<'PYV5'
import sys
src = open(sys.argv[1]).read()
ESC = ("  local direct_alive=0 k2=0\n"
       "  if [ \"$overrun\" = 1 ] && alive_pid \"$pid\"; then\n"
       "    kill -KILL \"$pid\" 2>/dev/null || true\n"
       "    while alive_pid \"$pid\" && [ \"$k2\" -lt \"$grace\" ]; do sleep 1; k2=$((k2+1)); done\n"
       "  fi\n"
       "  if alive_pid \"$pid\"; then direct_alive=1; else wait \"$pid\" 2>/dev/null || true; fi")
V5 = ("  local direct_alive=0\n"
      "  wait \"$pid\" 2>/dev/null || true")
n = src.count(ESC)
if n != 2: sys.exit("REFUSED: expected the escalation block in BOTH extracted ladders, found %d" % n)
open(sys.argv[2], 'w').write("# v5-EXACT shape: no escalation, an unconditional wait before the scan.\n" + src.replace(ESC, V5))
print("v5-exact variant written (both ladders: no escalation, unconditional wait)")
PYV5
[ -f "$W/naive2.sh" ] || exit 2
# BAD KILL: everything fixed except the signal target, which points at a group that does not exist. This is the
# inversion that answers "a kill failure must not be able to emit a successful kill receipt".
python3 - "$W/extracted.sh" "$W/badkill.sh" <<'PY'
import sys
src = open(sys.argv[1]).read()
if src.count('tgt="-$pid"') != 2: sys.exit("REFUSED: expected 2 group-target assignments")
open(sys.argv[2], 'w').write("# BAD-KILL variant: the target is a group that does not exist, so nothing is reached.\n"
                             + src.replace('tgt="-$pid"', 'tgt="-$((pid+100000))"'))
print("bad-kill variant written")
PY
[ -f "$W/badkill.sh" ] || exit 2
# WRAPPER-LADDER variant, derived from the extracted text by a named substitution in its own file, so the
# substitution itself is auditable and cannot be mistaken for anything else.
python3 "$(dirname "$0")/mk_wrapper_ladder_variant.py" "$W/extracted.sh" "$W/wrapperladder.sh" || exit 2
[ -f "$W/wrapperladder.sh" ] || exit 2
# ARM EVERY VARIANT THE WAY THE FRAME ARMS ITSELF. The extracted text carries GROUP_KILL=0 -- that is the frame's
# initializer -- and only the precheck probe raises it to 1 before any phase launches. The probe used to be a bare
# top-level block, which extraction cannot carry, so every case here silently ran in the degraded direct-child
# mode the real run never uses. It is a function now, so the frame's OWN arming runs in each variant.
for v in fixed naive naive2 badkill wrapperladder; do
  [ -f "$W/$v.sh" ] && printf '\nprecheck_group_kill >/dev/null 2>&1 || true\n' >> "$W/$v.sh"
done
for v in fixed naive naive2 badkill wrapperladder; do
  grep -aq '^precheck_group_kill ' "$W/$v.sh" || { echo "REFUSED: variant $v is not armed with the frame's own precheck"; exit 2; }
done

OK=0; BAD=0
chk() { if [ "$1" = 0 ]; then OK=$((OK+1)); echo "OK   $2"; else BAD=$((BAD+1)); echo "BAD  $2"; fi; }
rec_alive() { # rec_alive <case> -> success if any RECORDED pid of that case is still running (a zombie is not)
  local f=$W/rec.$1 p st
  [ -f "$f" ] || return 1
  while read -r p; do
    case "$p" in ''|*[!0-9]*) continue ;; esac
    st=$(awk '{print $3}' "/proc/$p/stat" 2>/dev/null) || continue
    case "$st" in Z|X) continue ;; *) return 0 ;; esac
  done < "$f"
  return 1
}
rec_kill() { local f=$W/rec.$1 p; [ -f "$f" ] || return 0
  while read -r p; do case "$p" in ''|*[!0-9]*) continue ;; esac; kill -9 "$p" 2>/dev/null || true; done < "$f"; }
body() { # body <case> -- the phase command: it records its own pid and the pid of the descendant it spawns, both
         # inside the phase's process group, then stays alive well past the cap.
  printf 'echo $$ > %s/rec.%s; sleep 40 & echo $! >> %s/rec.%s; wait' "$W" "$1" "$W" "$1"; }
# body_ignterm <case> -- the bounded command IGNORES SIGTERM (the descendant it forks inherits that), while the
# wrapper subshell caprun launched around it does not. That is the measured shape: the wrapper dies to the
# group TERM, and everything that ignored TERM is still in the same process group.
body_ignterm() {
  printf 'trap "" TERM; echo $$ > %s/rec.%s; sleep 40 & echo $! >> %s/rec.%s; wait' "$W" "$1" "$W" "$1"; }
run_variant_ig() { # run_variant_ig <label> <variant-file> <cap> <grace> <case>
  local label=$1 file=$2 cap=$3 grace=$4 case=$5 t0 rc
  t0=$(date +%s)
  bash -c "set -uo pipefail; source '$file'; caprun x $cap $grace bash -c '$(body_ignterm "$case")'" > "$W/out.$label" 2>&1; rc=$?
  LAST_EL=$(( $(date +%s) - t0 )); LAST_RC=$rc; LAST_OUT=$(cat "$W/out.$label")
  echo "  [$label] rc=$rc elapsed=${LAST_EL}s"; sed 's/^/      | /' "$W/out.$label" | head -6
}
run_variant() { # run_variant <label> <variant-file> <cap> <grace> <case>
  local label=$1 file=$2 cap=$3 grace=$4 case=$5 t0 rc
  t0=$(date +%s)
  # output goes to a FILE, never through a command substitution: a substitution waits for every writer of the pipe,
  # and a surviving descendant holds it open -- which made this fixture report a 40 s wall for a run the frame
  # itself finished in 8 s. Wall times here are the wall times of the variant, not of the fixture's pipe.
  bash -c "set -uo pipefail; source '$file'; caprun x $cap $grace bash -c '$(body "$case")'" > "$W/out.$label" 2>&1; rc=$?
  LAST_EL=$(( $(date +%s) - t0 )); LAST_RC=$rc; LAST_OUT=$(cat "$W/out.$label")
  echo "  [$label] rc=$rc elapsed=${LAST_EL}s"; sed 's/^/      | /' "$W/out.$label" | head -6
}

# ---- P1: the FROZEN launch shape kills a child AND the descendant it spawned, inside cap+grace ----------------
echo "== P1 fixed launch: an over-cap child with a spawned descendant must die within cap+grace"
run_variant fixed "$W/fixed.sh" 4 2 p1
[ "$LAST_RC" = 124 ] && chk 0 "P1: exit 124 (cut off, not completed)" || { BAD=$((BAD+1)); echo "BAD  P1: rc=$LAST_RC want 124"; }
rec_alive p1 && { BAD=$((BAD+1)); echo "BAD  P1: recorded pids still running after a receipt that claimed killed=yes"; } \
              || chk 0 "P1: LIVENESS on the recorded pids -- neither the child nor its descendant survives the kill"
[ "$LAST_EL" -le 12 ] && chk 0 "P1: wall ${LAST_EL}s, inside cap 4 + grace 2 + the direct-pid escalation, not the child's 40 s" \
                      || { BAD=$((BAD+1)); echo "BAD  P1: wall ${LAST_EL}s exceeds cap+grace"; }
printf '%s' "$LAST_OUT" | grep -aq 'TIMEOUT_KILL.*killed=yes' && chk 0 "P1: the kill receipt was EARNED by the post-kill liveness scan" \
                                                               || { BAD=$((BAD+1)); echo "BAD  P1: no earned kill receipt"; }
printf '%s' "$LAST_OUT" | grep -aq 'group-kill=1' && chk 0 "P1: the receipt states group-kill=1 MEASURED on this child, and the liveness result corroborates it" \
                                                   || { BAD=$((BAD+1)); echo "BAD  P1: no measured group-kill figure"; }

# ---- P2a (INVERSION): the naive launch shape -- the group kill reaches nothing -------------------------------
echo "== P2a INVERSION the naive (v5) launch: the group kill reaches nothing"
( bash -c "set -uo pipefail; source '$W/naive.sh'; caprun x 4 2 bash -c '$(body p2a)'" ) > "$W/out.p2a" 2>&1 &
p2job=$!
sleep 8
rec_alive p2a && chk 0 "P2a: 8 s past an expired 4 s cap plus 2 s grace, BOTH recorded pids are still running -- the group kill reached nothing" \
             || { BAD=$((BAD+1)); echo "BAD  P2a: nothing alive mid-flight, so the naive launch did kill the group and the inversion proves nothing"; }
wait "$p2job"; p2rc=$?; sed 's/^/      | /' "$W/out.p2a" | head -4
# The naive variant's OWN scan is blind, and that is the finding. With job control off at the launch the child's
# pgid is the launcher's, not the child's -- so `kill -- -PID` reached nothing AND `group_members PID` afterwards
# finds nobody either, because no process carries that pgid. The direct child is then killed by the individual
# escalation, the scan reports an empty group, and the receipt claims 'killed=yes' while the recorded descendants
# keep running. Escalation alone is therefore not enough: the frame has to MEASURE the group per launch, which is
# what the frozen bytes now do. The assertions below expect exactly this signature, and say why.
if [ "$p2rc" = 124 ]; then chk 0 "P2a: the naive variant returns 124 -- the direct child died, but the group scan could not see the strays (no process carries that pgid)"; else BAD=$((BAD+1)); echo "BAD  P2a: rc=$p2rc, want 124 for the signature described above"; fi
if grep -aq 'KILL_FAILED' "$W/out.p2a"; then BAD=$((BAD+1)); echo "BAD  P2a: the naive variant reported KILL_FAILED, so its scan is not blind as described"; else chk 0 "P2a: no KILL_FAILED in the naive variant -- a scan built on an unmeasured pgid cannot detect its own blindness"; fi
if grep -aq 'TIMEOUT_KILL.*killed=yes' "$W/out.p2a"; then chk 0 "P2a: the naive receipt STILL claims killed=yes while the recorded descendants run on -- the false receipt, reproduced; only the per-launch measurement removes it"; else BAD=$((BAD+1)); echo "BAD  P2a: no false claim, so the inversion no longer reproduces the reported defect"; fi
rec_alive p2a && chk 0 "P2a: after the run the recorded descendants are STILL alive -- the difference from P1 is a live process table, not a receipt label" \
                  || { BAD=$((BAD+1)); echo "BAD  P2a: no strays, so the launch shape made no difference"; }
rec_kill p2a; sleep 1

# ---- P2b (INVERSION): v5-EXACT -- a failed kill laundered into 'killed=yes' ---------------------------------
echo "== P2b INVERSION v5-exact: the unconditional wait launders a kill that reached nothing into killed=yes"
p3start=$(date +%s)
( bash -c "set -uo pipefail; source '$W/naive2.sh'; caprun x 4 2 bash -c '$(body p2b)'" ) > "$W/out.p2b" 2>&1 &
p2bjob=$!
sleep 8
rec_alive p2b && chk 0 "P2b: mid-flight both recorded pids are alive 4 s past the cap -- nothing had been killed" \
               || { BAD=$((BAD+1)); echo "BAD  P2b: nothing alive mid-flight; the defect did not reproduce"; }
wait "$p2bjob"; p2bel=$(( $(date +%s) - p3start )); sed 's/^/      | /' "$W/out.p2b" | head -3
[ "$p2bel" -ge 30 ] && chk 0 "P2b: the phase ran the child's FULL length (measured ${p2bel}s) -- the cap bounded nothing" \
                    || { BAD=$((BAD+1)); echo "BAD  P2b: wall ${p2bel}s, expected ~40 s"; }
grep -aq 'TIMEOUT_KILL.*killed=yes' "$W/out.p2b" && chk 0 "P2b: the v5-exact receipt claims 'killed=yes' for a kill that reached nothing -- the laundering review reported, reproduced from the frozen text; the mid-flight liveness above is the proof" \
                                                || { BAD=$((BAD+1)); echo "BAD  P2b: no false claim, so the reported defect is not reproduced"; }
rec_kill p2b; sleep 1

# ---- P3 (INVERSION): a kill made to FAIL must not emit a kill receipt ---------------------------------------
echo "== P3 INVERSION a failed kill must give KILL_FAILED naming survivors, never TIMEOUT_KILL"
run_variant badkill "$W/badkill.sh" 4 2 p3
[ "$LAST_RC" = 125 ] && [ "$LAST_EL" -le 12 ] && chk 0 "P3: exit 125 reached in ${LAST_EL}s (cap 4 + grace 2 + escalation), not after the child's 40 s" \
                   || { BAD=$((BAD+1)); echo "BAD  P3: rc=$LAST_RC elapsed=${LAST_EL}s, want 125 quickly"; }
printf '%s' "$LAST_OUT" | grep -aq 'KILL_FAILED' && chk 0 "P3: KILL_FAILED is reported and names the surviving pids" \
                                                || { BAD=$((BAD+1)); echo "BAD  P3: no KILL_FAILED line"; }
printf '%s' "$LAST_OUT" | grep -aq 'TIMEOUT_KILL' && { BAD=$((BAD+1)); echo "BAD  P3: a kill receipt was emitted for a kill that failed"; } \
                                                || chk 0 "P3: NO kill receipt for a failed kill"
rec_alive p3 && chk 0 "P3: the survivors the frame named really are running -- the KILL_FAILED line is not decorative" \
                || { BAD=$((BAD+1)); echo "BAD  P3: nothing left running, so the case did not exercise a failed kill"; }
rec_kill p3; sleep 1
rec_alive p3 && { BAD=$((BAD+1)); echo "BAD  P3: the fixture's own cleanup left a recorded pid running"; } \
              || chk 0 "P3: recorded-pid cleanup left nothing running"

# ---- P4: a deliberate refusal inside a bounded phase records its own code (the rc=137 fix, m1) ---------------
echo "== P4 a deliberate refusal inside a bounded phase records rc 2, not 137"
p4=$(bash -c "set -uo pipefail; source '$W/fixed.sh'; refuse_phase() { die2 'the gate refused: memory.max disagrees'; }; bounded x 10 2 refuse_phase; echo \$?" 2>&1 | tail -1)
[ "$p4" = 2 ] && chk 0 "P4: bounded returned 2 for a deliberate die2 refusal (so bcall can name the phase instead of a signal)" \
              || { BAD=$((BAD+1)); echo "BAD  P4: got $p4 want 2"; }
p4n=$(bash -c "set -uo pipefail; source '$W/naive.sh'; refuse_phase() { die2 'the gate refused: memory.max disagrees'; }; bounded x 10 2 refuse_phase; echo \$?" 2>&1 | tail -1)
[ "$p4n" = 137 ] && chk 0 "P4 INVERSION: the naive untrapped variant reproduces 137 for the same refusal -- the 'died from a signal' report was a masking, confirmed" \
                 || { BAD=$((BAD+1)); echo "BAD  P4 inversion: naive got $p4n want 137"; }

# ---- P5: a real SIGKILL must still read as 137 ---------------------------------------------------------------
echo "== P5 a phase killed by a signal that cannot be trapped must still report 137"
p5=$(bash -c "set -uo pipefail; source '$W/fixed.sh'; sig_phase() { kill -9 \"\$BASHPID\"; }; bounded x 10 2 sig_phase; echo \$?" 2>&1 | tail -1)
[ "$p5" = 137 ] && chk 0 "P5: SIGKILL of the phase subshell reports 137 (STATUS_UNRECORDED), never success" \
                || { BAD=$((BAD+1)); echo "BAD  P5: got $p5 want 137"; }

# ---- P7: a bounded command that IGNORES SIGTERM must still be killed, by the group, inside cap+grace ------
echo "== P7 a TERM-ignoring command and its descendant: the ladder must escalate to the group KILL"
run_variant_ig tree "$W/fixed.sh" 4 2 p7
[ "$LAST_RC" = 124 ] && chk 0 "P7: exit 124 -- the cap fired and the tree was cleared" \
                      || { BAD=$((BAD+1)); echo "BAD  P7: rc=$LAST_RC want 124"; }
[ "$LAST_EL" -le 12 ] && chk 0 "P7: wall ${LAST_EL}s, inside cap 4 plus three rungs of grace 2: the TERM-ignoring command did not run its 40 s" \
                       || { BAD=$((BAD+1)); echo "BAD  P7: wall ${LAST_EL}s, the ladder did not escalate promptly"; }
printf '%s' "$LAST_OUT" | grep -aq 'TIMEOUT_KILL.*killed=yes' \
  && chk 0 "P7: TIMEOUT_KILL with killed=yes, earned by the post-kill scan of the group" \
  || { BAD=$((BAD+1)); echo "BAD  P7: no earned kill receipt"; }
rec_alive p7 && { BAD=$((BAD+1)); echo "BAD  P7: a recorded pid is STILL RUNNING after a receipt claiming killed=yes"; } \
              || chk 0 "P7: LIVENESS on the recorded pids -- the TERM-ignoring command and its descendant are dead"

# ---- P8 (INVERSION): the wrapper-ladder form must leave that survivor running ------------------------------
echo "== P8 INVERSION a ladder that measures the launched pid, not the tree, fails this case"
run_variant_ig wrapper "$W/wrapperladder.sh" 4 2 p8
[ "$LAST_RC" = 125 ] && chk 0 "P8: exit 125, a FAILED kill -- what the frozen frame reported when measured end to end" \
                      || { BAD=$((BAD+1)); echo "BAD  P8: rc=$LAST_RC want 125"; }
printf '%s' "$LAST_OUT" | grep -aq 'KILL_FAILED' && chk 0 "P8: KILL_FAILED reported" \
  || { BAD=$((BAD+1)); echo "BAD  P8: no KILL_FAILED"; }
printf '%s' "$LAST_OUT" | grep -aq 'TIMEOUT_KILL' && { BAD=$((BAD+1)); echo "BAD  P8: the wrapper ladder claimed a successful kill"; } \
  || chk 0 "P8: no kill receipt claimed for a kill that never happened"
rec_alive p8 && chk 0 "P8: the TERM-ignoring pids really are left running by that ladder, so P7 measures the difference" \
                || { BAD=$((BAD+1)); echo "BAD  P8: nothing survived, so the inversion did not reproduce the defect"; }
rec_kill p8; sleep 1
rec_alive p8 && { BAD=$((BAD+1)); echo "BAD  P8: the fixture left a recorded pid running after its own cleanup"; } \
              || chk 0 "P8: recorded-pid cleanup left nothing running"

# ---- P6: a successful phase still returns 0 through the trap -------------------------------------------------
echo "== P6 a successful phase still returns 0"
p6=$(bash -c "set -uo pipefail; source '$W/fixed.sh'; ok_phase() { return 0; }; bounded x 10 2 ok_phase; echo \$?" 2>&1 | tail -1)
[ "$p6" = 0 ] && chk 0 "P6: exit 0 preserved" || { BAD=$((BAD+1)); echo "BAD  P6: got $p6 want 0"; }

[ "$(frame_sha)" = "$START_SHA" ] && chk 0 "the frame's bytes did not move during this control (sha $(frame_sha | cut -c1-16))" \
                                 || { BAD=$((BAD+1)); echo "BAD the frame's sha MOVED during the control"; }
echo "CAPKILL OK=$OK BAD=$BAD  (frame $(frame_sha), variants derived from it by the named substitutions above)"
[ "$BAD" = 0 ] || exit 1
exit 0
