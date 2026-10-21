set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14604 @ ba96e6c3884 -- belt recapture claims. PR edits ONE claim file; product identical to base.
# Arms: (1) head: 6 claims by name, wet; (2) base: the 3 original claims by name (expect FAIL);
# (3) sibling control claim, green; (4) MY red-sensitivity mutation: claims 2&3 without the
# settlement step -- if they still PASS, the exact-detail gate is not load-bearing (finding);
# (5) byte-exact restore + one head claim re-run.
HEADSHA=ba96e6c388434d2a47a1147bb97015dfd4066a2c; BASESHA=ef346f6ac57e3a85d326d727ba67e927dba16736  # gh pr view 14604
echo "HEADSHA=$HEADSHA BASE=$BASESHA START $(date -u +%H:%M:%S)Z RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12)"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w604h /tmp/w604b
git fetch -q origin +refs/pull/14604/head:refs/verifier/14604 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14604 >/dev/null || { echo FETCH_FAILED; exit 9; }
[ "$(git rev-parse refs/verifier/14604)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14604)"; exit 9; }
git fetch -q origin $BASESHA 2>&1 | tail -1
git cat-file -e $BASESHA^{commit} || { echo BASE_MISSING; exit 9; }
git worktree add -q --detach /tmp/w604h $HEADSHA || { echo WT_HEAD_FAILED; exit 9; }
git worktree add -q --detach /tmp/w604b $BASESHA || { echo WT_BASE_FAILED; exit 9; }
[ "$(git -C /tmp/w604h rev-parse HEAD)" = "$HEADSHA" ] && [ "$(git -C /tmp/w604b rev-parse HEAD)" = "$BASESHA" ] || { echo WORKTREE_MISMATCH; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=dag/test/claim/roadmap/roadmap_belt_recapture_wet_witness.dag
[ "$(git -C /tmp/w604h cat-file blob HEAD:$WIT | sha256sum | cut -c1-16)" = "e1cc610d1316b190" ] || { echo WITNESS_NOT_HEAD_BLOB; exit 9; }
# product identity: the diff base..head must touch ONLY the witness file
git -C /tmp/w604h diff --name-only $BASESHA $HEADSHA | sed 's/^/  CHANGED /'
[ "$(git -C /tmp/w604h diff --name-only $BASESHA $HEADSHA | grep -vc "^$WIT\$")" = "0" ] || echo "  !! PR touches product files too -- base arm is NOT same-binary valid"
cd /tmp/w604h || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14604 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14604/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) built $(date -u +%H:%M:%S)Z at head"
cp -f $CB /tmp/w604b/claim_batch_head
RUN() { # wt label entry csv
  local wt=$1 label=$2 entry=$3 csv=$4
  echo "-------- [$label] wet $entry :: $(echo $csv | cut -c1-120) $(date -u +%H:%M:%S)Z"
  ( cd $wt && timeout 1400 ./claim_batch_head_run --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $entry --functions $csv > /tmp/c604_$label.out 2> /tmp/c604_$label.err )
  local rc=$?
  if [ "$rc" = "124" ]; then echo "  rc=124 TIMEOUT -- EXCLUDED, not a result"; return 0; fi
  echo "  rc=$rc stdout=$(wc -c < /tmp/c604_$label.out)B stderr=$(wc -c < /tmp/c604_$label.err)B"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c604_$label.out | sed 's/^/    V| /' | cut -c1-165
  echo "  counts: PASS=$(grep -ac '^PASS ' /tmp/c604_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/c604_$label.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c604_$label.out)"
  tail -6 /tmp/c604_$label.err | grep -aE 'error|refused|panic' | sed 's/^/    E| /' | cut -c1-180
  return 0
}
cp -f $CB /tmp/w604h/claim_batch_head_run; cp -f $CB /tmp/w604b/claim_batch_head_run
C1=the_recapture_of_a_committed_attempt_tree_is_minted_by_real_execution
C2=a_clean_worktree_whose_head_is_not_the_recorded_submission_refuses_by_real_execution
C3=a_capture_claiming_this_head_stored_under_another_basename_refuses_by_real_execution
C4=a_standalone_repository_at_the_attempt_worktree_is_refused_by_the_git_trust_wall
C5=a_commit_step_settled_at_another_head_leaves_the_capture_owed_not_absent
C6=no_commit_step_record_leaves_a_misaddressed_capture_owed_not_absent
ALL="$C1,$C2,$C3,$C4,$C5,$C6"
echo "=== 1. HEAD: all 6 claims, wet, by name ==="; RUN /tmp/w604h head "$WIT" "$ALL"
echo "=== 2. BASE (same head-built binary; only the witness file differs): the 3 original claims ==="; RUN /tmp/w604b base "$WIT" "$C1,$C2,$C3"
echo "=== 3. sibling control claim, wet, at head ==="; RUN /tmp/w604h sibling dag/test/claim/roadmap/belt_capture_intent_wet_witness.dag a_kill_after_the_scm_save_does_not_double_capture
echo "=== 4. MY MUTATION: claims 2&3 run WITHOUT the settlement step (expect FAIL; if PASS => the detail gate is not load-bearing) ==="
cp -f $WIT /root/workspace/repo-root/tools/stage/br_head_backup.dag
grep -c 'commit_step_settled(instance: instance) && match' $WIT | xargs echo "  anchor count (expect 2):"
sed -i 's/commit_step_settled(instance: instance) && match/true \&\& match/g' $WIT || { echo SED_FAILED; exit 9; }
[ "$(grep -c 'true && match belt_scm_recapture' $WIT)" = "2" ] || { echo MUTATION_NOT_APPLIED -- aborting to protect the run; cp -f /root/workspace/repo-root/tools/stage/br_head_backup.dag $WIT; exit 9; }
RUN /tmp/w604h mut "$WIT" "$C2,$C3"
echo "=== 5. RESTORE + re-run claim 1 (must be back to PASS) ==="
git -C /tmp/w604h cat-file blob HEAD:$WIT > $WIT 2>/dev/null || cp -f /root/workspace/repo-root/tools/stage/br_head_backup.dag $WIT
[ "$(sha $WIT)" = "$(sha /root/workspace/repo-root/tools/stage/br_head_backup.dag)" ] && echo "  RESTORED_OK" || echo "  RESTORE_MISMATCH -- LEAVING FOR INSPECTION"
RUN /tmp/w604h rest "$WIT" "$C1"
echo DONE_14604_A
cd /root/workspace/repo-root
git worktree remove --force /tmp/w604h >/dev/null 2>&1 || true
git worktree remove --force /tmp/w604b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
