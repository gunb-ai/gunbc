# freeze-2 v6 -- one revision covering N1, N2 and the 4 minors (keen-otter msg_53860227, sleek-owl msg_a8326df9, parent msg_9a51b1d7)
Bounds UNCHANGED: RT 4200, driver 4500, 32/28 GiB, 4096 tasks, 3200%. Caps and PLANNED_TOTAL 4018 unchanged.

## N1 BLOCKING -- no cap kills anything (CONFIRMED in the bytes, before touching them)
caprun L224-226 and bounded L279-281 do:
    local pg=${GROUP_KILL-}
    if [ -z "$pg" ]; then if set -m 2>/dev/null; then pg=1; set +m ...; else pg=0; fi; fi
    [ "$pg" = 1 ] && { set -m 2>/dev/null || pg=0; set +m 2>/dev/null || true; }
so job control is switched OFF immediately BEFORE `{ ... } &`, the child is NOT a group leader, `kill -- -PID`
answers ESRCH and reaches nothing, there is no fallback, and `wait` blocks until the child ends by itself.
The pre-work probe (L178-182) launches WITH job control ON, so it reports GROUP_KILL=1 while every real launch
could not be group-killed -- which is why every v5 receipt prints a false `(group-kill=1)` / TIMEOUT_KILL.
FIX (frame): (a) leave job control ON across the launch when the host supports it; (b) read the launched child's
pgid from /proc/$pid/stat and report THAT per launch (never inherit the probe's answer); (c) after TERM->KILL,
run a post-kill liveness scan of the whole group (group_members by pgid) and emit TIMEOUT_KILL killed=yes ONLY
when the scan finds nothing alive; otherwise emit KILL_FAILED with the surviving pids and exit 125 (which bcall
turns into NO_OBSERVATION, never a kill receipt). Stated limit: a phase that deliberately `setsid`s a descendant
leaves the group and is invisible to this scan; the outer backstop is the keeper's `<J>.scope` with
RuntimeMaxSec=driver 4500, which is why RT 4200 < driver 4500 stays.
FIX (controls, executable, raw evidence not receipt labels): hermetic cases
  o1 overrun-child+descendant: stub spawns a child AND a grandchild, sleeps past a shortened phase cap; assert
     wall <= cap+grace+slack, BOTH recorded pids dead after the phase (kill -0 / /proc state), TIMEOUT_KILL
     killed=yes present exactly once, [cap] line reports the per-launch measured group-kill=1.
  o2 INVERSION naive v5 launch (the `set +m` before launch, applied to the EXTRACTED function only): the same
     case must FAIL -- the phase runs its full length and the recorded pids are still alive. This is the
     "today's set +m variant must FAIL that case" inversion.
  o3 INVERSION kill made to fail (fixed code, target deliberately wrong): must print KILL_FAILED with survivors
     and exit 125, and must NOT print any TIMEOUT_KILL/killed=yes line.

## N2 MAJOR -- all-PASS text with a non-zero executor status reads as PASS
run() checks only rc_in_contract (0/1/2 pass), and the terminal decision is
    [ "$MISSING" = 0 ] || die2 ... ; [ "$FAILS" = 0 ] || exit 1 ; ... exit 0
so claim_batch exiting 1 (cost-accounting refusal) or 2 (no observation) with 34 PASS lines ends 'OUTCOME: PASS rc=0'.
FIX: accumulate every entry's exit status; require ALL ZERO on the path to exit 0, else
die2 naming each entry and its rc. Controls: 34 PASS + rc 1 -> NO_OBSERVATION rc=2 naming the rc; 34 PASS + rc 2 -> same.

## MINORS
 m1 rc=137 status record: bounded() ran `{ "$fn" "$@"; echo "$?" > "$sf"; } &` and die2's `exit 2` skips the echo
    -> empty file -> reported 137 + "died from a signal" while the real code is 2. FIX: write the status from an
    EXIT trap inside the subshell. INVERSIONS: (a) a phase SIGKILLed (untrappable) still gives 137 +
    STATUS_UNRECORDED; (b) the naive (untrapped, v5) variant reproduces 137 for a deliberate refusal.
 m2 'OUTCOME: PASS rc=0 unspecified': derive the reason from measured counters (MISSING=0 FAILS=0, executor rcs
    all 0, the expected-claim count computed from the rosters), and print no reason when there is none.
 m3 header: label the v17 roster lines (rtp blob 514c1d32 = 17 fns at the v17 pin 495da688, NOT this seed) as
    superseded, and repair the broken 'down to#' comment.
 m4 seed-checker negctl: document AND set its own git identity, create/require the freeze2-seed-v2 branch it
    depends on, and print WHY a mutant failed to build (capture the clone/apply output instead of swallowing it).

## Re-run everything against the new bytes, retaining RAW evidence
all existing hermetic cases + the new ones above; gates/argv/outcome/recon/accounting/order controls; auditor +
its negctl; seed prerun + its 20-mutant negctl (now self-contained); v4/v5 whole-frame inversions (altered patch,
omitted patch, reordered, wrong tree, dirty checkout, missing completion line, no-verdict-line witness, deadline).
Then re-pin every fixture's frame sha, re-measure every receipt, re-freeze the packet as v6, post, live-verify.
