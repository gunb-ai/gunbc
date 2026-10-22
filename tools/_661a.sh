set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14661 row 9: enrol ml_the_key_law_names_both_occurrences_holds. Body: claim PASSES under
# claim_batch --wet --source-root dag --source-root src/v2 (author's exact route); enrolled
# ExpectedToHold on local_repo_wet_schedule. Manager scope: PASS on enrolled route at head;
# enrolment row resolves by identity; mutation to the claim's SUBJECT (map_introduction.dag's
# key law) turns it red. Statics (local): claim at src/v2/test/claim/body_lowering/map_literal.dag;
# subject fn map_introduction_duplicate_key in src/v2/std/map_introduction.dag; anchor
# 'second_index: acc.index' count=1; row name hits=2 in local_repo_wet_terminal.dag.
HEADSHA=78a3d151e9b536df0cc2610242bb796375a11df0   # gh pr view 14661 --json headRefOid
F=src/v2/test/claim/body_lowering/map_literal.dag
SUBJ=src/v2/std/map_introduction.dag
TERM=src/v2/workflow/local_repo_wet_terminal.dag
CLAIM=ml_the_key_law_names_both_occurrences_holds
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w661h
git fetch -q origin +refs/pull/14661/head:refs/verifier/14661 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14661)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14661) STOP"; exit 9; }
git worktree add -q --detach /tmp/w661h $HEADSHA || { echo WT_H_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w661h || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "=== build claim_batch at HEAD ==="
( CARGO_TARGET_DIR=/tmp/w661h/.t661h cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CB=/tmp/w661h/.t661h/release/claim_batch; [ -x $CB ] || { echo NO_CB; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12)"
echo "=== A. enrolled route: author's exact command ==="
timeout 900 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $F --functions $CLAIM > /tmp/c661_author.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c661_author.out) FAIL=$(grep -ac '^FAIL ' /tmp/c661_author.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c661_author.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c661_author.out | sed 's/^/    V| /' | cut -c1-150
grep -a 'witness\]' /tmp/c661_author.out | head -1 | sed 's/^/    W| /' | cut -c1-200
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c661_author.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== B. enrolment row resolves by identity ==="
grep -n -B2 -A6 "$CLAIM" $TERM | head -40 | sed 's/^/    ROW| /' | cut -c1-170
echo "  (expect a WetScheduledClaim row: entry v2.test.claim.body_lowering.map_literal, function $CLAIM, status ExpectedToHold)"
grep -c 'WetScheduledClaim {' $TERM | sed 's/^/  TERM| WetScheduledClaim_rows_total=/'
echo "=== C. MUTATION on the subject (map_introduction.dag): second_index off-by-one ==="
cp $SUBJ /tmp/c661_s.orig; HB=$(sha $SUBJ)
A=$(grep -c 'second_index: acc.index }$' $SUBJ)
echo "  ANCHOR=$A (expect 1)"
[ "$A" = "1" ] || { echo "  ANCHOR_BAD skip-mutation"; echo DONE_661_A; exit 0; }
python3 - <<'PY'
p="src/v2/std/map_introduction.dag"
s=open(p).read()
old="second_index: acc.index }"
assert s.count(old)==1, "anchor %d" % s.count(old)
open(p,'w').write(s.replace(old,"second_index: acc.index + 1 }"))
print("MUTATED anchor=1")
PY
cmp -s $SUBJ /tmp/c661_s.orig && echo "  !! MUTATION_NOOP" || echo "  mutated sha=$(sha $SUBJ|cut -c1-12)"
timeout 900 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $F --functions $CLAIM > /tmp/c661_mut.out 2>&1; rc=$?
echo "  MUT rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c661_mut.out) FAIL=$(grep -ac '^FAIL ' /tmp/c661_mut.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c661_mut.out | sed 's/^/    V| /' | cut -c1-150
cp /tmp/c661_s.orig $SUBJ
cmp -s $SUBJ /tmp/c661_s.orig && [ "$(sha $SUBJ)" = "$HB" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO"
timeout 900 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $F --functions $CLAIM > /tmp/c661_post.out 2>&1; rc=$?
echo "  POST-restore rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c661_post.out) FAIL=$(grep -ac '^FAIL ' /tmp/c661_post.out)"
git status --porcelain | sed 's/^/    PORCELAIN| /'
echo DONE_661_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w661h >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
