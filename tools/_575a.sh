set -u
cd /root/workspace/repo-root 2>/dev/null || true
HEADSHA=6bec9c50fe5393d3691095b3643c98d4b2a89225
# Why a second dispatch: attempt 1's RED-A plant was a NO-OP. `git update-index --remove <path>` is
# documented to do nothing while the file still exists on disk, so the index never changed
# (MID_LISTED=1) and that arm was vacuous -- 3 PASS proved nothing about the plant. This run uses
# `--force-remove`, which removes the entry regardless, and asserts the removal took effect BEFORE
# running the claim, so a silent no-op cannot be reported as a green or as a red.
WT=/tmp/wt575b
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
git fetch -q origin $HEADSHA 2>&1 | tail -1
git cat-file -e $HEADSHA^{commit} || { echo "HEAD_COMMIT_UNAVAILABLE_ON_RUNNER"; exit 9; }
git worktree add -q --detach $WT $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
[ "$(git -C $WT rev-parse HEAD)" = "$HEADSHA" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }

REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
HREX=55763b1ed1675c2cffd05dbc308dc3992d61628dbdb75d4d383f5308a2ffaa15
[ "$(sha256sum $WT/$REX | cut -d' ' -f1)" = "$HREX" ] || { echo "SUBJECT_DRIFT"; exit 9; }
echo "SUBJECT_OK outside_discovery_roots_census_real_execution.dag ${HREX:0:16}"

export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
cd $WT
REXFNS=$(grep "^test fn" $REX | sed 's/^test fn //; s/() -> Bool {//' | paste -sd, -)
grep "^test fn" $REX | sed 's/^test fn //; s/() -> Bool {//' | sort > /tmp/exp.txt

run() { # tag
  timeout 1200 $CB --wet --source-root dag --source-root src/v2 --source-root src/v1 \
    --entry "$REX" --functions "$REXFNS" > /tmp/r_$1.log 2>&1
  rc=$?
  sed 's/\x1b\[[0-9;]*m//g' /tmp/r_$1.log > /tmp/r_${1}_c.log
  echo "--- [$1] rc=$rc bytes=$(wc -c < /tmp/r_${1}_c.log)"
  echo "  counts: PASS=$(grep -acE '^PASS ' /tmp/r_${1}_c.log) FAIL=$(grep -acE '^FAIL ' /tmp/r_${1}_c.log) UNOBSERVED=$(grep -acE '^UNOBSERVED ' /tmp/r_${1}_c.log)"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/r_${1}_c.log | sed 's/^\([A-Za-z]*\) \([^ ]*\).*/\2/' | sort > /tmp/got_$1.txt
  echo "  printed_total=$(wc -l < /tmp/got_$1.txt) expected=$(wc -l < /tmp/exp.txt)"
  echo "  NEVER-PRINTED:"; comm -23 /tmp/exp.txt /tmp/got_$1.txt | sed 's/^/    NP /'
  echo "  EXTRA:"; comm -13 /tmp/exp.txt /tmp/got_$1.txt | sed 's/^/    EX /'
  echo "  verdicts:"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/r_${1}_c.log | sed 's/^/    /' | cut -c1-165
  echo "  census/claim output lines naming the subject:"; grep -aE "OUTSIDE-DISCOVERY-ROOTS src/v1/tests/claim" /tmp/r_${1}_c.log | sed 's/^/    /' | cut -c1-175
  echo "  panic=$(grep -ac panicked /tmp/r_${1}_c.log)"
}

DROP=src/v1/tests/claim/type_declaration_occurrence_control_test.dag
echo "=== CONTROL (index intact)"
echo "  pre: ls_files=$(git ls-files -- $DROP | wc -l) glob=$(git ls-files '*.dag' | grep -c "^$DROP$") on_disk=$([ -f $DROP ] && echo yes)"
run control

echo "=== RED-A: FORCE-remove one claim-bearing path from the INDEX (the listing's real input)"
git update-index --force-remove -- $DROP
echo "  mid: ls_files=$(git ls-files -- $DROP | wc -l) glob=$(git ls-files '*.dag' | grep -c "^$DROP$") on_disk=$([ -f $DROP ] && echo yes)"
if [ "$(git ls-files '*.dag' | grep -c "^$DROP$")" != "0" ]; then
  echo "  PLANT_DID_NOT_LAND - refusing to report this arm either way"; git update-index --add -- $DROP; exit 8
fi
echo "  PLANT_LANDED (the index no longer tracks it; the file is still on disk, which is the point)"
run redA
echo "  restoring the index entry"
git update-index --add -- $DROP
echo "  post: ls_files=$(git ls-files -- $DROP | wc -l) glob=$(git ls-files '*.dag' | grep -c "^$DROP$") on_disk=$([ -f $DROP ] && echo yes)"
[ "$(git ls-files -- $DROP | wc -l)" = "1" ] && echo "  INDEX_RESTORED" || echo "  INDEX_NOT_RESTORED"

echo "=== CONTROL again after restore (a restore is re-run, not trusted)"
run control_after

cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
rm -rf $WT
echo DONE_14575A
