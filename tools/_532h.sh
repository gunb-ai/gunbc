set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14532 RE-RUN AT PRINTED HEAD 7371b408d5f (manager routing rule: executed runs at the printed head;
# the 04:33Z verdict was a comment-stripped carry-over and does not meet it). Same arms as _532.sh at
# 581e95295c, but this time EVERYTHING is head worktree: fetch to dedicated ref, worktree --detach at
# the gh-printed OID, build INSIDE it, arms mutate the worktree and restore from `git cat-file blob HEAD:`.
OID=7371b408d5f0d1bc2f91528e4c602df808502299  # gh pr view 14532 --json headRefOid (re-confirmed locally)
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12)"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w532h
git fetch -q origin +refs/pull/14532/head:refs/verifier/14532h 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14532h >/dev/null || { echo FETCH_FAILED; exit 9; }
[ "$(git rev-parse refs/verifier/14532h)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14532h) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w532h $OID || { echo WT_ADD_FAILED; exit 9; }
WT=/tmp/w532h
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
W=dag/test/claim/quarantine_probe_disposition_witness.dag
E=src/v2/workflow/floor_expected_red.dag
ADMIT=dag/gunbc/explicit_witness_admission.dag
DROP=dag/gunbc/rung_drop/transitional_admission_exception.dag
for f in $W $E $ADMIT $DROP; do echo "  head-blob $(sha $f | cut -c1-16) $f"; done
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14532h CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14532h/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) built $(date -u +%H:%M:%S)Z inside the head worktree"
OUT=$WT/.v532h; mkdir -p $OUT
cp $E $OUT/E.orig; cp $W $OUT/W.orig
FNS=$(grep -aoE '^test fn [a-z0-9_]+' $W | sed 's/^test fn //' | sort | paste -sd,)
echo "witness claim_count=$(echo $FNS | tr ',' '\n' | wc -l)"
echo "$FNS" | tr ',' '\n' | sort > $OUT/expect.txt
run() { # tag entry fns
  local tag=$1 entry=$2 fns=$3
  echo "-------- [$tag] $(basename $entry) claims=$(echo $fns | tr ',' '\n' | wc -l) $(date -u +%H:%M:%S)Z"
  rm -f $OUT/r_$tag.log
  timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $entry --functions "$fns" > $OUT/r_$tag.log 2>&1
  local rc=$?
  [ "$rc" = "124" ] && { echo "  rc=124 TIMEOUT -- EXCLUDED, not a result"; return 0; }
  sed 's/\x1b\[[0-9;]*m//g' $OUT/r_$tag.log > $OUT/r_${tag}_c.log
  echo "  claim_batch rc=$rc log_bytes=$(wc -c < $OUT/r_${tag}_c.log)"
  awk '/^PASS /{p++} /^FAIL /{f++} /^UNOBSERVED /{u++} END{printf "    awk: PASS=%d FAIL=%d UNOBS=%d\n", p+0, f+0, u+0}' $OUT/r_${tag}_c.log
  awk '/^FAIL /{print "    FAIL| "$0} /refus|error:|no such|not found|not held/{print "    E|  "$0}' $OUT/r_${tag}_c.log | head -40 | cut -c1-210
  awk '/^PASS /{print "    PASS| "$0}' $OUT/r_${tag}_c.log | head -20 | cut -c1-150
  if grep -qa "hermetic route has no arm" $OUT/r_${tag}_c.log; then
    echo "  !! HERSHEY: hermetic refused a host capability -- re-running wet before reporting:"
    timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $entry --functions "$fns" > $OUT/r_${tag}_wet.log 2>&1
    echo "     wet rc=$?"; awk '/^PASS /{p++} /^FAIL /{f++} END{printf "       wet: PASS=%d FAIL=%d\n", p+0, f+0}' $OUT/r_${tag}_wet.log
    awk '/^FAIL /{print "       wet FAIL| "$0}' $OUT/r_${tag}_wet.log | head -12 | cut -c1-200
  fi
  [ "$rc" = "0" ] && awk '/^PASS /{p++} END{if(p+0==0) print "  !! ZERO_VERDICT rc=0 and no PASS line"}' $OUT/r_${tag}_c.log
  return 0
}
mutE() { python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p,old,new,want=sys.argv[1],sys.argv[2],sys.argv[3],int(sys.argv[4])
s=open(p).read(); n=s.count(old); assert n==want,"anchor count %d want %d"%(n,want)
open(p,"w").write(s.replace(old,new)); print("  MUT ok (%d of %d)"%(n,want))
PY
}
rest() { git cat-file blob HEAD:$2 > $2; cmp -s $OUT/$1.orig $2 && echo "  RESTORED_BYTE_IDENTICAL $1" || echo "  !!RESTORE_FAILED VOID $1"; }

echo "=== 0. clean at printed head: expect 14 PASS / 0 FAIL ==="; run clean $W "$FNS"
echo "=== 1. the two constructor same_arm copies are LIVE RED: both must FAIL ==="
run live_red_dag dag/test/claim/constructor_tag_route_probe_red.dag "same_arm_one_t_two_coproducts_must_refuse"
run live_red_manual src/v2/test/claim/manual/constructor_tag_route_probe_red.dag "same_arm_one_t_two_coproducts_must_refuse"
echo "=== 2. the two map_literal claims the PR de-enrolled must PASS at head ==="
run ml_pair src/v2/test/claim/body_lowering/map_literal.dag "ml_an_elaborated_map_infers_holds,ml_a_same_module_generic_entry_construct_infers_holds"
echo "=== 3. THE TRUE CONTROL: Q2 chunk deleted AND unregistered ==="
mutE $E 'fn floor_expected_red_chunk_constructor_tag_route_probe_red() -> List<String> {
  Cons { head: "test.claim.constructor_tag_route_probe_red.same_arm_one_t_two_coproducts_must_refuse", tail: Cons { head: "v2.test.claim.manual.constructor_tag_route_probe_red.same_arm_one_t_two_coproducts_must_refuse", tail: Empty {} } }
}' 'fn floor_expected_red_chunk_constructor_tag_route_probe_red() -> List<String> {
  []
}' 1 || { echo MUT_FAILED; exit 8; }
mutE $E 'Cons { head: floor_expected_red_chunk_constructor_tag_route_probe_red(), tail: ' 'Cons { head: [], tail: ' 1 || { echo MUT_FAILED; exit 8; }
sha256sum $E | sed 's/^/  mutated /'
run m_chunk_deleted_and_unregistered $W "$FNS"
rest E $E
echo "=== 3b. HALF control: emptied but STILL REGISTERED ==="
mutE $E 'fn floor_expected_red_chunk_constructor_tag_route_probe_red() -> List<String> {
  Cons { head: "test.claim.constructor_tag_route_probe_red.same_arm_one_t_two_coproducts_must_refuse", tail: Cons { head: "v2.test.claim.manual.constructor_tag_route_probe_red.same_arm_one_t_two_coproducts_must_refuse", tail: Empty {} } }
}' 'fn floor_expected_red_chunk_constructor_tag_route_probe_red() -> List<String> {
  []
}' 1 || { echo MUT_FAILED; exit 8; }
run m_chunk_emptied_registered $W "$FNS"
rest E $E
echo "=== 4. the fixture red names the unheld admission: predicate never holds ==="
mutE $W '      if quarantine_probe_disposition_holds(disposition: d) {' '      if false && quarantine_probe_disposition_holds(disposition: d) {' 1 || { echo MUT_FAILED; exit 8; }
run m_predicate_never_holds $W "$FNS"
rest W $W
echo "=== 5. control after all restores ==="; run final $W "$FNS"
for p in "E $E" "W $W"; do set -- $p; cmp -s $OUT/$1.orig $2 && echo FINAL_MATCH_$1 || echo FINAL_MISMATCH_$1; done
echo DONE_532H
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
