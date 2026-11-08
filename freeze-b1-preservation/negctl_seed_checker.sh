#!/usr/bin/env bash
# NEGATIVE CONTROLS for prerun_checks_freeze2_seed.sh (sha de333026...).
# Each case builds a MUTANT seed commit in a private throwaway clone and requires the checker to (a) exit
# non-zero AND (b) print a FAIL line naming the EXPECTED reason - so a FAIL for an unrelated reason does not
# count as a passing control, and an always-failing checker cannot pass either (case P proves it passes on the
# genuine seed). Local git only: never builds or runs gunbc, claim_batch or claim_executor.
set -u
SRC=/var/tmp/freeze2-cut-15695/repo
CHECKER=/tmp/packet/prerun_checks_freeze2_seed.sh
DIFF=/tmp/packet/SEED_PROBE_SET_rebase_candidate.diff
BASE=57ce2e787aa5227fc1840961c607981f4abdde86
SEED=c07e2b8b666e0d2c529a4b0c44475f28db6cc129
MAIN=7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9
RTP=src/v2/test/claim/namespace_xl0/return_tail_position.dag
W=$(mktemp -d /var/tmp/seednegctl.XXXXXX); chmod 700 "$W"
OUT="$W/NEGCTL_seed_checker.txt"; : > "$OUT"
pass=0; bad=0
say(){ printf '%s\n' "$*" | tee -a "$OUT"; }

newcommit(){ # newcommit <clone> <parent-sha> ; edits already applied in worktree -> prints commit sha
  local c=$1 p=$2 t
  ( cd "$c" && git add -A >/dev/null 2>&1 && t=$(git write-tree) && git commit-tree "$t" -p "$p" )
}

run_case(){ # run_case <name> <expected-rc: nonzero|zero> <expected-fail-substring|-> <seed> [diff]
  local name=$1 wantrc=$2 wantfail=$3 seed=$4 diff=${5:-$DIFF} d out rc
  d="$W/run-$name"; mkdir -p "$d"
  out=$(TMPDIR="$d" REPO_DIR="$SRC" bash "$CHECKER" "$seed" "$BASE" "$diff" "$d/work" 2>&1); rc=$?
  local ok=1 why='' esc
  esc=$(printf '%s' "$wantfail" | sed 's/[][\.*^$*/]/\\&/g')
  if [ "$wantrc" = nonzero ] && [ "$rc" = 0 ]; then ok=0; why='exited 0 (should have refused)'; fi
  if [ "$wantrc" = zero ]; then
    if [ "$rc" != 0 ]; then ok=0; why="exited $rc (should have passed)"; fi
    printf '%s' "$out" | grep -q '^STATUS: PASS' || { ok=0; why="${why:+$why; }no STATUS: PASS"; }
    if printf '%s' "$out" | grep -q '^FAIL '; then ok=0; why="${why:+$why; }a FAIL line in the positive case:"; fi
  fi
  if [ "$wantrc" = nonzero ] && [ "$wantfail" != - ]; then
    # the named reason must appear ON A FAIL LINE, not merely somewhere in the log
    printf '%s' "$out" | grep -qE "^FAIL .*$esc" || { ok=0; why="${why:+$why; }no FAIL line naming: $wantfail"; }
  fi
  if [ "$ok" = 1 ]; then pass=$((pass+1)); say "OK   $name (rc=$rc)"; else bad=$((bad+1)); say "BAD  $name (rc=$rc): $why"; fi
  printf '%s\n' "$out" > "$W/log-$name.txt"
  rm -rf "$d/work" "$d/clone" 2>/dev/null
}

say "== negative controls for prerun_checks_freeze2_seed.sh  $(date -u +%FT%TZ)"
say "checker $(sha256sum "$CHECKER" | cut -d' ' -f1)"
say "diff    $(sha256sum "$DIFF" | cut -d' ' -f1)"
say ""

# P. the genuine seed must PASS (guards against an always-failing checker)
run_case P-zero PASS - "$SEED" -

# 1. the base passed off as the seed
run_case M1-base-as-seed nonzero "seed tree" "$BASE" -

# 2. probe literal changed (ABSENT -> GONE): tree, digest and verbatim-line checks must all bite
C="$W/m2"; git clone --no-hardlinks -q -s "$SRC" "$C" && git -C "$C" checkout -q "$SEED"
sed -i 's/=="ABSENT"/=="GONE"/' "$C/$RTP"
M2=$(newcommit "$C" "$BASE"); say "M2 mutant commit (literal changed) = $M2"
run_case M2-literal-changed nonzero "probe block sha256" "$M2" -

# 3. one probe test removed: 21 -> 20
C="$W/m3"; git clone --no-hardlinks -q -s "$SRC" "$C" && git -C "$C" checkout -q "$SEED"
python3 - "$C/$RTP" <<'PY'
import re,sys
p=sys.argv[1]; s=open(p).read()
i=s.index('test fn rtp_ctl_spelling_is_absent')
j=s.index('test fn ', i+10) if 'test fn ' in s[i+10:] else len(s)
open(p,'w').write(s[:i]+s[j:])
PY
M3=$(newcommit "$C" "$BASE"); say "M3 mutant commit (probe removed) = $M3  count=$(grep -c '^test fn ' "$C/$RTP")"
run_case M3-probe-removed nonzero "test fn" "$M3" -

# 4. the correct tree hanging off TWO parents (a merge masquerading as a seed)
T=$(git -C "$SRC" rev-parse "$SEED^{tree}")
M4=$(git -C "$SRC" commit-tree "$T" -p "$BASE" -p "$MAIN")
say "M4 mutant commit (2 parents, correct tree $T) = $M4"
run_case M4-two-parents nonzero "parents" "$M4" -

# 5. a DIFFERENT diff artifact of the same shape (one byte of probe text changed in the diff itself)
sed 's/=="RESOLVED"/=="RESOLVE_D"/' "$DIFF" > "$W/mutated.diff"
run_case M5-diff-drifted nonzero "diff sha256" "$SEED" "$W/mutated.diff"

# 6. a seed with NO probe block at all (base + an unrelated edit to the same file)
C="$W/m6"; git clone --no-hardlinks -q -s "$SRC" "$C" && git -C "$C" checkout -q "$BASE"
printf '\n// freeze2 unrelated edit, no probes here\n' >> "$C/$RTP"
M6=$(newcommit "$C" "$BASE"); say "M6 mutant commit (no probe block) = $M6"
run_case M6-no-probe-block nonzero "probe block sha256 NONE" "$M6" -

# 7. the right content hanging off the WRONG parent (main instead of the base)
M7=$(git -C "$SRC" commit-tree "$T" -p "$MAIN")
say "M7 mutant commit (parent = main, not the base) = $M7"
run_case M7-wrong-parent nonzero "seed parent" "$M7" -

say ""
say "NEGCTL RESULT: OK=$pass BAD=$bad  (8 cases: 1 positive + 7 mutations)"
say "RECEIPT: $OUT"
[ "$bad" = 0 ] || exit 1
exit 0
