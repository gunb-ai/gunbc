set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14615 @ a226684d762 -- falsifier drop: DERIVED population equals the 20 body identities.
# Arms: 3 witness claims by name; mutation A = drop the roster half of the fold (join must FAIL,
# others hold); mutation B = break the identity shape (names claim must FAIL); restores byte-exact.
OID=a226684d7628acc961144a8beb66dc3071b0c91f  # gh pr view 14615 --json headRefOid
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w615
git fetch -q origin +refs/pull/14615/head:refs/verifier/14615 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14615)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14615) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w615 $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w615
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
DROP=dag/gunbc/rung_drop/falsifier_rehomed_rows_unexecuted.dag
WIT=src/v2/test/claim/falsifier_rehomed_rows_unexecuted_witness.dag
cp $DROP /tmp/615_drop.orig
[ "$(sha $DROP)" = "$(git cat-file blob HEAD:$DROP | sha256sum | cut -d' ' -f1)" ] || { echo NOT_HEAD_BLOB; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14615 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14615/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12) built $(date -u +%H:%M:%S)Z inside head worktree"
J=falsifier_rehomed_drop_population_is_the_identity_join_of_both_frontiers
N=falsifier_rehomed_drop_population_names_entry_and_function
R=falsifier_rehomed_drop_population_excludes_unlisted_RED_control_holds
run() { # tag fns...
  local tag=$1; shift
  echo "-------- [$tag] $(date -u +%H:%M:%S)Z fns=$*"
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$1" > /tmp/c615_$tag.out 2>&1; local rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT EXCLUDED"; return 0; }
  echo "  rc=$rc"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c615_$tag.out | sed 's/^/    V| /' | cut -c1-150
  echo "  counts: PASS=$(grep -ac '^PASS ' /tmp/c615_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/c615_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c615_$tag.out)"
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c615_$tag.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
  return 0
}
echo "=== A. clean: all 3 by name ==="; run clean "$J,$N,$R"
echo "=== B. MUTATION: drop the roster half of the derived fold ==="
grep -c 'concat(falsifier_rehomed_bin_wet_rows, falsifier_rehomed_roster_rows)' $DROP | xargs echo "  anchor (expect 1):"
sed -i 's/concat(falsifier_rehomed_bin_wet_rows, falsifier_rehomed_roster_rows)/falsifier_rehomed_bin_wet_rows/' $DROP
sha256sum $DROP | sed 's/^/  mutated /'
run mut_noroster "$J,$N,$R"
git cat-file blob HEAD:$DROP > $DROP
cmp -s /tmp/615_drop.orig $DROP && echo "  RESTORED_BYTE_IDENTICAL drop" || echo "  !!RESTORE_FAILED"
echo "=== C. MUTATION: break the identity shape (entry omitted, '::' kept) ==="
grep -c 'concat(r.entry, concat("::", r.function))' $DROP | xargs echo "  anchor (expect 1):"
sed -i 's/concat(r.entry, concat("::", r.function))/concat("::", r.function)/' $DROP || { echo SED_FAILED; exit 9; }
run mut_shape "$J,$N,$R"
git cat-file blob HEAD:$DROP > $DROP
cmp -s /tmp/615_drop.orig $DROP && echo "  RESTORED_BYTE_IDENTICAL drop" || echo "  !!RESTORE_FAILED"
echo "=== D. final re-run after restores ==="; run final "$J,$N,$R"
cmp -s /tmp/615_drop.orig $DROP && echo FINAL_MATCH_drop || echo FINAL_MISMATCH_drop
echo DONE_615_A
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
