set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14671 enrol the 18 ledger_row_coherence claims (one batch). Manager scope: 18/18 PASS wet at
# head; the 18 ENROLLED identities match the file's test fns EXACTLY, both directions. Statics
# (local, run pre-dispatch): comm -3 of file-test-fns vs row-functions == empty (18/18 both ways);
# all 18 rows module_path test.claim.ledger_row_coherence + ExpectedToHold.
HEADSHA=8b6fa692557687874a554955092b279ed7a1698d   # gh pr view 14671 --json headRefOid (head moved 2b789ef->8b6fa69 by main-merge; PR-file delta main..head = +108 = 18x6 rows)
LRC=dag/test/claim/ledger_row_coherence.dag
TERM=src/v2/workflow/local_repo_wet_terminal.dag
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w671h
git fetch -q origin +refs/pull/14671/head:refs/verifier/14671 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14671)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14671) STOP"; exit 9; }
git worktree add -q --detach /tmp/w671h $HEADSHA || { echo WT_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w671h || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WRONG_COMMIT; exit 9; }
( CARGO_TARGET_DIR=/tmp/w671h/.t671 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CB=/tmp/w671h/.t671/release/claim_batch; [ -x $CB ] || { echo NO_CB; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $LRC | sed 's/^test fn //' | sort | paste -sd,)
echo "file_fns=$(echo $FNS | tr ',' '\n' | wc -l)"
echo "=== A. wet run, 18 by name ==="
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $LRC --functions "$FNS" > /tmp/c671_wet.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c671_wet.out) FAIL=$(grep -ac '^FAIL ' /tmp/c671_wet.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c671_wet.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/c671_wet.out | sed 's/^/    V| /' | cut -c1-160
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c671_wet.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
echo "=== B. identity join BOTH directions (rows vs file) ==="
grep -aoE '^test fn [a-zA-Z0-9_]+' $LRC | sed 's/^test fn //' | sort > /tmp/c671_fns.txt
grep -A4 'module_path: "test.claim.ledger_row_coherence"' $TERM | grep -aoE 'function: "[a-zA-Z0-9_]+"' | sed 's/function: "//;s/"//' | grep -v '^main' | sort -u > /tmp/c671_rows.txt
echo "  file=$(wc -l < /tmp/c671_fns.txt) rows=$(wc -l < /tmp/c671_rows.txt)"
echo "  ONLY-IN-FILE:"; comm -23 /tmp/c671_fns.txt /tmp/c671_rows.txt | sed 's/^/    F| /'
echo "  ONLY-IN-ROWS:"; comm -13 /tmp/c671_fns.txt /tmp/c671_rows.txt | sed 's/^/    R| /'
echo "  DUPLICATE-CHECK (each name appears once):"; sort /tmp/c671_rows.txt | uniq -d | sed "s/^/    DUP| /"; echo "  dup_total=$(sort /tmp/c671_rows.txt | uniq -d | wc -l)"
echo "  expectations: ExpectedToHold=$(grep -B1 -A4 'module_path: "test.claim.ledger_row_coherence"' $TERM | grep -ac ExpectedToHold) of 18 rows"
echo DONE_671_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w671h >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
