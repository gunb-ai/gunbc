set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14652 hermetic re-run on the AUTHOR's exact roots (dag + src/v2 only). Prior hermetic arm added
# --source-root src/v1 -> census claims read false (author's stated caveat). Wet 4/4 already GOOD.
HEADSHA=c3c84d6d62537d03feb20326e4440a86cf599cfd
F=dag/test/claim/record_escapes_declared_optional_return_witness.dag
ALL4="w_an_equality_between_a_coproduct_and_a_record_is_refused,w_an_equality_between_unrelated_records_is_refused,w_the_census_refuses_a_blatant_mismatch,w_the_one_axis_green_sources_are_admitted"
echo "START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w652h2
git fetch -q origin +refs/pull/14652/head:refs/verifier/14652b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14652b)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14652b) STOP"; exit 9; }
git worktree add -q --detach /tmp/w652h2 $HEADSHA || { echo WT_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w652h2 || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WRONG_COMMIT; exit 9; }
( CARGO_TARGET_DIR=/tmp/w652h2/.t652 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CB=/tmp/w652h2/.t652/release/claim_batch; [ -x $CB ] || { echo NO_CB; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12)"
for n in w_an_equality_between_a_coproduct_and_a_record_is_refused w_an_equality_between_unrelated_records_is_refused w_the_census_refuses_a_blatant_mismatch w_the_one_axis_green_sources_are_admitted; do
  grep -q "^test fn $n$" $F || echo "  !! NAME_NOT_IN_FILE $n"
done
echo "=== hermetic, author roots (dag + src/v2 only) ==="
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $F --functions "$ALL4" > /tmp/c652_auth_h.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652_auth_h.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652_auth_h.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c652_auth_h.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c652_auth_h.out | sed 's/^/    V| /' | cut -c1-150
echo "=== wet, author roots (re-confirm on this worktree) ==="
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $F --functions "$ALL4" > /tmp/c652_auth_w.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c652_auth_w.out) FAIL=$(grep -ac '^FAIL ' /tmp/c652_auth_w.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c652_auth_w.out | sed 's/^/    V| /' | cut -c1-150
echo DONE_652_B
cd /root/workspace/repo-root
git worktree remove --force /tmp/w652h2 >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
