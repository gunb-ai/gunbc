#!/usr/bin/env bash
# NEGATIVE CONTROLS for prerun_checks_freeze2_seed_v2.sh -- proves the checker is load-bearing.
#
# A pre-run checker that only ever sees the good artifact proves nothing. Each case below hands the checker a
# seed that is WRONG in exactly one way and requires: the exact exit code, an 'OUTCOME: FAIL' line, and a FAIL
# line that NAMES the broken identity. The positive case requires rc 0 AND 'OUTCOME: PASS' AND that no FAIL
# line was printed at all.
#
# MUTATION MECHANISM, and why it changed twice:
#  v1 made mutant commits in the main repo but left them UNREACHABLE, so the checker refused for the wrong
#     reason (an unreachable object), which would have looked like a working control.
#  v2 wrote the mutated TREE inside a throwaway clone; the tree object therefore did not exist in REPO_DIR, so
#     `commit-tree` died, printed nothing, and the checker was handed an empty seed -> exit 2. Again: refused
#     for the wrong reason.
#  This version mutates in REPO_DIR's own working tree (checkout --detach parent, edit, add, write-tree,
#     commit-tree, branch -f to keep it reachable), and AFTER EVERY MUTATION IT REQUIRES that the file actually
#     differs from the parent's version. A mutation that changes nothing is reported as 'SETUP' and counted BAD
#     -- it is never allowed to read as a passing control. Several of the sed patterns used here target text
#     that a wrong path assumption could easily have made absent; without this guard that failure mode is
#     invisible.
#
# Every case names what it is meant to detect. Read the case names against the checker's checks.
set -u
SELF_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)   # inputs resolve next to me, never at a build-time absolute path
CHECKER=${CHECKER:-$SELF_DIR/prerun_checks_freeze2_seed_v2.sh}
EXPECT=${EXPECT_FILE:-$SELF_DIR/prerun_expectations_freeze2_v2.env}
DIGEST=${DIGEST_TOOL:-$SELF_DIR/probe_block_digest.py}
SRC=${REPO_DIR:-/var/tmp/freeze2-cut-15695/repo}
FRAME=${FRAME:-$SELF_DIR/packet-evidence-freeze2.sh}
# (defined here, not lower: run_case passes it to the checker at its first use, and under set -u a later
# definition makes every case before it die as "FRAME: unbound variable" -- 18 identical empty-log failures)
SEED=4a9efe081d8046969eb4e9ef06d219e27ce31113
BASE=57ce2e787aa5227fc1840961c607981f4abdde86
MAIN=7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9
SEEDBR=freeze2-seed-v2
RTP=src/v2/test/claim/namespace_xl0/return_tail_position.dag
W=$(mktemp -d "${TMPDIR:-/var/tmp}/seednegctl-v2.XXXXXX"); chmod 700 "$W"
OUT="$W/NEGCTL_seed_checker_v2.txt"; : > "$OUT"
BRANCHES=(); pass=0; bad=0; RAN=''
fpos=0; fmerit=0; finput=0; fpin=0; kpass=0; kfail=0; kabort=0   # class tallies, counted from the cases that ran
# THE CONTROL MUST NOT DEPEND ON THE HOST'S GIT IDENTITY. mkmutant builds commits with commit-tree, which needs an
# author and committer; with no user.name/user.email configured it fails, and because a mutant that cannot be built
# short-circuits `&& run_case`, SEVEN cases (M3, M4, M6-M9 and others) simply never ran while the receipt still
# printed OK=12 BAD=0 rc=0. The identity is now supplied here, so the fixture is self-contained, AND every case is
# declared below and checked against what actually ran, so a skip can never again read as a clean receipt.
export GIT_AUTHOR_NAME=negctl GIT_AUTHOR_EMAIL=negctl@invalid \
       GIT_COMMITTER_NAME=negctl GIT_COMMITTER_EMAIL=negctl@invalid
EXPECTED_CASES='P-genuine M1-base-as-seed M3-literal-changed M4-probe-removed M5-two-parents M6-no-probe-block
M7-wrong-parent M8-b2-regression M9-extra-test-added M10-diff-drifted M11-artifact-absent M12-artifact-emptied
M13-expectations-absent M14-candidate-unresolvable M15-accepted-substituted S9-pin-drift S9-pin-short
S9-pin-nonhex S9-path-gone S9-frame-absent'
EXPECTED_TOTAL=$(printf '%s\n' $EXPECTED_CASES | wc -w)
say(){ printf '%s\n' "$*" | tee -a "$OUT"; }
restore(){ git -C "$SRC" checkout -f -q "$SEEDBR" 2>/dev/null || git -C "$SRC" checkout -f -q "$SEED" 2>/dev/null || true; }
cleanup(){ restore; for b in "${BRANCHES[@]:-}"; do [ -n "$b" ] && git -C "$SRC" branch -D "$b" >/dev/null 2>&1; done; }
trap cleanup EXIT
trap 'cleanup; exit 2' INT TERM

mkmutant(){ # mkmutant <name> <parent> <path> <shell mutation on "$F">  -> prints sha; keeps it reachable; LOUD on no-op
  local name=$1 parent=$2 path=$3 cmd=$4 tid sha d
  restore
  git -C "$SRC" checkout -q --detach "$parent" 2>>"$OUT" || { say "SETUP $name: checkout $parent failed"; return 1; }
  F="$SRC/$path"; export F
  eval "$cmd" >>"$OUT" 2>&1 || { say "SETUP $name: mutation command failed"; return 1; }   # BOTH streams to the log:
  # a mutation helper that prints would otherwise land in mkmutant's stdout, which the caller captures as the commit id
  [ -f "$SRC/$path" ] || { say "SETUP $name: $path does not exist after the mutation"; return 1; }
  git -C "$SRC" add -- "$path" >>"$OUT" 2>&1 || { say "SETUP $name: git add failed (path present in the tree?)"; return 1; }
  # ANTI-NO-OP GUARD: the mutated file MUST differ from the parent's version.
  if git -C "$SRC" diff --quiet "$parent" -- "$path"; then
    say "SETUP $name: NO-OP mutation -- $path is byte-identical to $parent; the control would assert nothing"
    return 1
  fi
  tid=$(git -C "$SRC" write-tree 2>>"$OUT") || { say "SETUP $name: write-tree failed"; return 1; }
  sha=$(git -C "$SRC" commit-tree "$tid" -p "$parent" -m "negctl mutant $name" 2>>"$OUT") || { say "SETUP $name: commit-tree failed"; return 1; }
  git -C "$SRC" cat-file -e "$sha" 2>/dev/null || { say "SETUP $name: mutant commit is not in $SRC"; return 1; }
  git -C "$SRC" branch -f "negctl/$name" "$sha" >/dev/null 2>&1 || { say "SETUP $name: could not create a ref for the mutant"; return 1; }
  BRANCHES+=("negctl/$name")
  printf '%s' "$sha"
}

run_case(){ # run_case <name> <want-rc> <want-outcome> <want-fail-substring|-> <seed> [expect-file]
  local name=$1 wantrc=$2 wantout=$3 wantfail=$4 seed=$5 exp=${6:-$EXPECT} d out rc ok=1 why=''
  [ "$exp" = - ] && exp=$EXPECT   # '-' means 'the registered expectations file', not a file called '-'
  if [ -z "$seed" ]; then say "SETUP $name: empty seed argument (a mutation failed upstream)"; bad=$((bad+1)); return; fi
  d="$W/run-$name"; mkdir -p "$d"; chmod 700 "$d"
  out=$(EXPECT_FILE="$exp" DIGEST_TOOL="$DIGEST" REPO_DIR="$SRC" FRAME_SCRIPT="${MUT_FRAME:-$FRAME}" timeout 900 bash "$CHECKER" "$d/work" "$seed" 2>&1); rc=$?
  printf '%s\n' "$out" > "$W/log-$name.txt"
  [ "$rc" = "$wantrc" ] || { ok=0; why="rc=$rc want $wantrc"; }
  printf '%s' "$out" | grep -q "^OUTCOME: $wantout " || { ok=0; why="${why:+$why; }no OUTCOME: $wantout"; }
  if [ "$wantout" = PASS ] && printf '%s' "$out" | grep -q '^FAIL '; then ok=0; why="${why:+$why; }a FAIL line was printed in a PASS case"; fi
  if [ "$wantfail" != - ]; then
    local esc line; esc=$(printf '%s' "$wantfail" | sed 's/[][\\.*^$*/]/\\&/g')
    # a merit case names its failure on a FAIL line; an input-class case must name the MISSING INPUT on its
    # OUTCOME: ABORT line. Checking a FAIL line for both is what let M11/M12 pass while aborting on the wrong
    # input, so the line each case is allowed to satisfy is chosen by its own declared outcome.
    line='^FAIL '; [ "$wantout" = ABORT ] && line='^OUTCOME: ABORT '
    printf '%s' "$out" | grep -qE "$line.*$esc" || { ok=0; why="${why:+$why; }no $line naming: $wantfail"; }
  fi
  # Class tallies are COUNTED from the run. The v4 receipt line typed "the four S9 frame-pin cross-checks" after a
  # fifth had been added, and a stale class figure in a receipt is the same error the reviewers called out for the
  # packet header -- so the sentence is now arithmetic over the cases that actually ran.
  case "$name" in P-*) fam=positive;; S9-*) fam=pin;; 'M11'*|'M12'*|'M13'*|'M14'*) fam=input;; M*) fam=merit;; esac
  case "$fam" in positive) fpos=$((fpos+1));; merit) fmerit=$((fmerit+1));; input) finput=$((finput+1));; pin) fpin=$((fpin+1));; esac
  case "$wantout" in PASS) kpass=$((kpass+1));; FAIL) kfail=$((kfail+1));; ABORT) kabort=$((kabort+1));; esac
  if [ "$ok" = 1 ]; then pass=$((pass+1)); say "OK   $name (rc=$rc)"; else bad=$((bad+1)); say "BAD  $name (rc=$rc): $why"; fi
  RAN="$RAN $name"
  rm -rf "$d/work"
}

mcase(){ # mcase <name> <want-rc> <want-outcome> <want-fail> <mkmutant args...>
  # A mutant that cannot be built must FAIL the case, not skip it. `M=$(mkmutant ...) && run_case ...` short-circuits
  # and leaves both counters untouched, which is precisely how seven cases went unrun behind a green receipt.
  local name=$1 wr=$2 wo=$3 wf=$4; shift 4
  local M
  if ! M=$(mkmutant "$@"); then
    bad=$((bad+1)); RAN="$RAN $name"
    say "BAD  $name: the mutant could not be built, so the control never ran (counted, never skipped)"
    return
  fi
  [ -n "$M" ] || { bad=$((bad+1)); RAN="$RAN $name"; say "BAD  $name: mkmutant printed no commit id"; return; }
  run_case "$name" "$wr" "$wo" "$wf" "$M" -
}

say "== negative controls for $CHECKER  $(date -u +%FT%TZ)"
say "checker  $(sha256sum "$CHECKER" | cut -d' ' -f1)"
say "expect   $(sha256sum "$EXPECT"  | cut -d' ' -f1)"
say "repo     $SRC   seed $SEED"
say ""

say "-- 1. the real thing must pass, and the case must be capable of noticing that it did"
run_case P-genuine 0 PASS - "$SEED" -

say "-- 2. the BASE handed in as the seed (identities must be checked, not assumed)"
run_case M1-base-as-seed 1 FAIL "seed tree" "$BASE" -

say "-- 3. a probe literal changed inside the seed: the block digest must catch it"
mcase M3-literal-changed 1 FAIL "probe block sha256" m3 "freeze2-seed-v2" "$RTP" "sed -i 's/== \"ABSENT\"/== \"GONE\"/' \"\$F\""

say "-- 4. one probe test removed: the 21-test count must catch it"
# The four probe witnesses are single-line declarations (measured at seed file lines 362,364,366,368), so the
# deletion is exact and cannot swallow a neighbour.
mcase M4-probe-removed 1 FAIL "test fn" m4 "freeze2-seed-v2" "$RTP" "python3 $SELF_DIR/mut_m4.py \"\$F\""

say "-- 5. two parents (an accidental merge commit instead of a single-parent seed)"
P1=$(git -C "$SRC" commit-tree "$(git -C "$SRC" rev-parse 'freeze2-seed-v2^{tree}')" -p "$BASE" -p "$MAIN" -m "two-parent mutant" 2>>"$OUT")
if [ -n "$P1" ] && git -C "$SRC" cat-file -e "$P1" 2>/dev/null; then
  git -C "$SRC" branch -f negctl/m5 "$P1" >/dev/null && BRANCHES+=("negctl/m5")
  run_case M5-two-parents 1 FAIL "parents" "$P1" -
else
  bad=$((bad+1)); RAN="$RAN M5-two-parents"; say "BAD  M5-two-parents: could not build the two-parent commit, so the control never ran (counted, never skipped)"
fi

say "-- 6. the seed commit swapped for one whose tree lacks the probe block entirely"
# The block is removed AT THE SEED. An earlier version built it from the BASE, which never had the block, so the
# mutation was a no-op -- which is exactly what the anti-no-op guard exists to catch.
mcase M6-no-probe-block 1 FAIL "probe block sha256" m6 "freeze2-seed-v2" "$RTP" "python3 $SELF_DIR/mut_m6.py \"\$F\""

say "-- 7. the seed parented on the MAIN component instead of the base"
# ANY change will do: this case is about PARENT IDENTITY only. (An earlier version reused the ABSENT literal,
# which does not exist at the main component -- the anti-no-op guard caught it, and the control silently tested
# nothing until then.)
mcase M7-wrong-parent 1 FAIL "seed parent" m7 "$MAIN" "$RTP" "printf '# negctl parent-identity marker\\n' >> \"\$F\""

say "-- 8. the B2 REGRESSION: the non-canonical lexeme arms put back. This is the control for the very defect
--    that forced the re-freeze, so the checker must refuse it."
mcase M8-b2-regression 1 FAIL "B2 regression" m8 "freeze2-seed-v2" "$RTP" "sed -i -e 's/^    Present { value: s } =>/    optional_present(value: s) =>/' -e 's/^    Absent =>/    optional_absent() =>/' \"\$F\""

say "-- 9. the ADDED-LINE COUNT must be exactly 2/2: add a third added line"
mcase M9-extra-test-added 1 FAIL "21" m9 "freeze2-seed-v2" "$RTP" "python3 $SELF_DIR/mut_m9.py \"\$F\""

say "-- 10-12. input class: a missing/empty artifact must exit 2 (an ABORT), never 1 and never 0, while a"
say "--       DRIFTED artifact stays a merit failure (rc=1, OUTCOME: FAIL). The distinction is the point of"
say "--       these cases, so they must differ in exactly ONE input: the other three artifacts are staged as"
say "--       real files with absolute paths, otherwise the checker aborts on a missing input and M11/M12 would"
say "--       'pass' without ever testing the input they name."
mkdir -p "$W/stage"; chmod 700 "$W/stage"
cp "$EXPECT" "$W/stage/expect.base"
for _a in PROBE_DIFF ACCEPTED_DIFF RTP_ARTIFACT CE_ARTIFACT; do
  _v=$(grep "^$_a=" "$EXPECT" | head -1 | cut -d= -f2-)
  _abs="$W/stage/$(basename "$_v")"
  [ -f "$_abs" ] || cp "$SELF_DIR/$_v" "$_abs"
  sed -i "s|^$_a=.*|$_a=$_abs|" "$W/stage/expect.base"
done
sed 's/== "ABSENT"/== "ABSENT-DRIFT"/' "$SELF_DIR/SEED_PROBE_SET_rebase_candidate_v2.diff" > "$W/drifted.diff"
E="$W/expect-drift.env"; sed "s|^PROBE_DIFF=.*|PROBE_DIFF=$W/drifted.diff|" "$W/stage/expect.base" > "$E"
run_case M10-diff-drifted 1 FAIL "diff sha256" "$SEED" "$E"
E2="$W/expect-nodiff.env"; sed "s|^PROBE_DIFF=.*|PROBE_DIFF=/nonexistent/absent.diff|" "$W/stage/expect.base" > "$E2"
run_case M11-artifact-absent 2 ABORT "PROBE_DIFF missing" "$SEED" "$E2"
: > "$W/empty.diff"
E3="$W/expect-empty.env"; sed "s|^PROBE_DIFF=.*|PROBE_DIFF=$W/empty.diff|" "$W/stage/expect.base" > "$E3"
run_case M12-artifact-emptied 2 ABORT "is empty" "$SEED" "$E3"
E4="$W/expect-noexpect.env"
run_case M13-expectations-absent 2 ABORT - "$SEED" /nonexistent/expect.env

run_case M14-candidate-unresolvable 1 FAIL "not a resolvable commit" 0000000000000000000000000000000000000000 -

restore
say ""
# M15: the ACCEPTED artifact substituted. This is the defect the digest comparison was added for: the step used to
# PRINT the accepted digest and compare it against nothing, so pointing ACCEPTED_DIFF at any other probe set passed
# unnoticed. The substitute here is the SAME well-formed drifted diff M10 uses for the candidate: it parses and it
# digests, so the only thing that can catch it is the registered comparison -- file sha first, then block digest.
# (An earlier version of this case substituted the CANDIDATE diff itself, and the checker aborted with rc=2 instead
# of failing with rc=1; that difference in class was not explained by anything in the checker I could point at, so
# the case was rebuilt on a substitute whose path through the checker is known rather than tuned to whatever rc it
# happened to produce.)
sed "s|^ACCEPTED_DIFF=.*|ACCEPTED_DIFF=$W/drifted.diff|" "$W/stage/expect.base" > "$W/expect-accsub.env"
run_case M15-accepted-substituted 1 FAIL "accepted artifact" "$SEED" "$W/expect-accsub.env"

say "-- S9: the run frame's blob pins are cross-checked against git at the candidate"
[ -f "$FRAME" ] || { say "SETUP S9: no frame at $FRAME"; bad=$((bad+1)); }
mkframe(){ # mkframe <name> <sed-expr>  -> path to a mutated copy, refusing a no-op mutation
  local n=$1 e=$2
  # bash expands all words of one `local` before assigning any of them, so $n cannot be used in the same
  # statement that declares it: under set -u that is "n: unbound variable", and it cost four S9 cases.
  local f="$W/frame-$n.sh"
  sed "$e" "$FRAME" > "$f" || { say "SETUP frame-$n: sed failed"; return 1; }
  cmp -s "$f" "$FRAME" && { say "SETUP frame-$n: the mutation changed NOTHING (pattern did not match)"; return 1; }
  printf '%s' "$f"
}
GOODPIN=f2009028bba066c829d555492b7a7d644083e86f   # measured with git ls-tree at the seed
if F=$(mkframe pin-drift "s|$GOODPIN|f200902855004b7385897cae205f046636c56d25|"); then
  MUT_FRAME="$F" run_case S9-pin-drift 1 FAIL "the pin is not a measurement" "$SEED" -; MUT_FRAME=
else bad=$((bad+1)); fi
if F=$(mkframe pin-short "s|ee1c679e7ef1e2923edadfe0a6f0c63cf881b019|ee1c679e|"); then
  MUT_FRAME="$F" run_case S9-pin-short 1 FAIL "not 40" "$SEED" -; MUT_FRAME=
else bad=$((bad+1)); fi
if F=$(mkframe pin-nonhex "s|$GOODPIN|zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz|"); then
  MUT_FRAME="$F" run_case S9-pin-nonhex 1 FAIL "non-hex blob pin" "$SEED" -; MUT_FRAME=
else bad=$((bad+1)); fi
if F=$(mkframe pin-path-gone "s|src/v2/test/claim/self_host/closure_emission.dag|src/v2/test/claim/self_host/not_there.dag|"); then
  MUT_FRAME="$F" run_case S9-path-gone 1 FAIL "does not exist at the candidate" "$SEED" -; MUT_FRAME=
else bad=$((bad+1)); fi
MUT_FRAME="$W/no-such-frame.sh" run_case S9-frame-absent 2 ABORT - "$SEED" -; MUT_FRAME=

# THE CASE LEDGER. A receipt that says OK=12 BAD=0 is only meaningful if 12 is the whole set: seven of the nineteen
# cases once went unrun behind exactly that line, because a mutant that failed to build short-circuited its case and
# touched neither counter. Every case is declared in EXPECTED_CASES; what actually ran is recorded in RAN; the two
# must agree in both directions, and no case may run twice.
missing=''; for c in $EXPECTED_CASES; do case " $(printf '%s ' $RAN) " in *" $c "*) ;; *) missing="$missing $c";; esac; done
# both sides normalised to a single space-separated line: comparing against the RAW EXPECTED_CASES string made a
# name that sits at the end of a continuation line look "undeclared" (a space never follows it), which reported six
# perfectly declared cases as extras. The comparison must be over tokens, not over substrings of a multi-line string.
DECL=$(printf '%s ' $EXPECTED_CASES)
extra='';   for c in $RAN;         do case " $DECL " in *" $c "*) ;; *) extra="$extra $c";; esac; done
dupes=$(printf '%s\n' $RAN | sort | uniq -d | tr '\n' ' ')
n_ran=$(printf '%s\n' $RAN | wc -w)
[ -n "$missing" ] && { bad=$((bad+1)); say "BAD  CASES-NOT-RUN: declared but never executed:$missing"; }
[ -n "$extra" ]   && { bad=$((bad+1)); say "BAD  CASES-UNDECLARED: ran something not in the declared set:$extra"; }
[ -n "$dupes" ]   && { bad=$((bad+1)); say "BAD  CASES-DUPLICATED: $dupes"; }
say "CASES: declared $EXPECTED_TOTAL, ran $n_ran (a shortfall is a control failure, not a smaller suite)"
_cls="$fpos positive + $fmerit merit-or-drift refusals + $finput input-class aborts + $fpin frame-pin cross-checks"
_outc="$kpass PASS, $kfail FAIL, $kabort ABORT"
[ "$((fpos+fmerit+finput+fpin))" = "$EXPECTED_TOTAL" ] || { say "BAD  class tally ($fpos+$fmerit+$finput+$fpin) does not cover the declared $EXPECTED_TOTAL cases"; bad=$((bad+1)); }
[ "$((kpass+kfail+kabort))" = "$EXPECTED_TOTAL" ] || { say "BAD  outcome tally ($kpass+$kfail+$kabort) does not cover the declared $EXPECTED_TOTAL cases"; bad=$((bad+1)); }
say "NEGCTL RESULT: OK=$pass BAD=$bad  (total $((pass+bad)) = $_cls; by declared outcome: $_outc. The class and outcome figures are COUNTED from the cases that ran, never typed, and each must cover the declared total; the total is likewise computed against the declared case list, which is how four silently-skipped S9 cases and seven skipped mutants were caught.)"
say "RECEIPT: $OUT"
[ "$bad" = 0 ] || exit 1
