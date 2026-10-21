set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14604 SAME-PRODUCT BASE ARM: run the PRE-FIX witness file at the PR's TRUE merge base 482ddb7664c7,
# where the whole tree is byte-identical to head EXCEPT the one claim file (asserted below). Same
# head-built binary as the head arm. This replaces arm-2's ef346f6a checkout, whose product had drifted.
HEADSHA=ba96e6c388434d2a47a1147bb97015dfd4066a2c; MB=482ddb7664c723f18ece3a9583afdc62c877c320
WIT=dag/test/claim/roadmap/roadmap_belt_recapture_wet_witness.dag
echo "HEAD=$HEADSHA MERGE_BASE=$MB START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w604m
git fetch -q origin +refs/pull/14604/head:refs/verifier/14604 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14604)" = "$HEADSHA" ] || { echo "PR_REF_MOVED"; exit 9; }
git cat-file -e $MB^{commit} 2>/dev/null || git fetch -q origin $MB
git cat-file -e $MB^{commit} || { echo MB_MISSING; exit 9; }
# THE VALIDITY GUARD: base..head must differ in NOTHING but the witness file
N=$(git diff --name-only $MB $HEADSHA | grep -v "^$WIT$" | wc -l)
[ "$N" = "0" ] || { echo "MERGE_BASE_NOT_SAME_PRODUCT ($N other files) -- arm would be confounded; STOP"; exit 9; }
echo "same-product guard OK: base..head differs ONLY in $WIT"
git worktree add -q --detach /tmp/w604m $MB || { echo WT_FAILED; exit 9; }
[ "$(git -C /tmp/w604m rev-parse HEAD)" = "$MB" ] || { echo WT_MISMATCH; exit 9; }
cd /tmp/w604m || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14604 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14604/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
echo "CB_SHA=$(sha256sum $CB | cut -d' ' -f1 | cut -c1-12) (head arm was 715e36eb1b15; Rust identical so binary must match)"
cp -f $CB ./claim_batch_run
C1=the_recapture_of_a_committed_attempt_tree_is_minted_by_real_execution
C2=a_clean_worktree_whose_head_is_not_the_recorded_submission_refuses_by_real_execution
C3=a_capture_claiming_this_head_stored_under_another_basename_refuses_by_real_execution
echo "-------- [merge-base] pre-fix witness, pre-fix product, head binary $(date -u +%H:%M:%S)Z"
timeout 1400 ./claim_batch_run --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions $C1,$C2,$C3 > /tmp/mb.out 2> /tmp/mb.err
rc=$?
[ "$rc" = "124" ] && { echo "rc=124 TIMEOUT -- EXCLUDED"; exit 0; }
echo "  rc=$rc stdout=$(wc -c < /tmp/mb.out)B"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/mb.out | sed 's/^/    V| /' | cut -c1-165
echo "  counts: PASS=$(grep -ac '^PASS ' /tmp/mb.out) FAIL=$(grep -ac '^FAIL ' /tmp/mb.out)"
echo DONE_14604_B
cd /root/workspace/repo-root
git worktree remove --force /tmp/w604m >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
