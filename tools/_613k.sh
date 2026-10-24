set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @b7854d98d76 census FALSIFICATION arm: mutate ONLY the edited row's standing
# (current -> AtCeiling while declared BelowCeiling = the census's own built-in red fixture shape).
# Expect live_roster_every_read_row_is_admitted to flip held -> observed=812/813 refused: proves the
# edited row is IN the read roster AND its admission is load-bearing (aggregate is not vacuous).
OID=b7854d98d76584c65b04df7ce1a971b1ab7322fc
ROWF=dag/gunbc/recurring_failure_mode/workflow_expressions_assembled_from_string_fragments.dag
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613k
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613g 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613g 2>/dev/null)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613g 2>/dev/null) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613k $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613k
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
cp $ROWF /tmp/613k_row.orig
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613k CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t14613k/release/gunbc
[ -x $GB ] || { echo "NO gunbc -- nothing below is a result"; exit 9; }
echo "GB_SHA=$(sha $GB)"
run_census () { # <tag>
  ( cd $WT && timeout 2400 $GB test //gunbc/instruments:rfm-rung-census ) > /tmp/c613k_$1.out 2>&1; local rc=$?
  local HELD REF; HELD=$(grep -ac ' held ' /tmp/c613k_$1.out); REF=$(grep -acE '^FAIL ' /tmp/c613k_$1.out)
  echo "  ROW $1 rc=$rc held-lines=$HELD FAIL-lines=$REF"
  grep -aE 'live_roster_every_read_row_is_admitted' /tmp/c613k_$1.out | sed 's/^/  V| /' | cut -c1-190
  grep -aE '^FAIL |^UNOBSERVED ' /tmp/c613k_$1.out | head -8 | sed 's/^/  F| /' | cut -c1-190
  [ "$rc" = "0" ] && [ "$((HELD+REF))" = "0" ] && echo "  ROW $1 !! rc=0 zero-verdict VOID"
  return 0
}
echo "-------- K1 census with edited row intact (baseline this dispatch) $(date -u +%H:%M:%S)Z"
run_census base
echo "-------- K2 falsification: edited row's current := AtCeiling (declared BelowCeiling) $(date -u +%H:%M:%S)Z"
grep -c 'current: FoundOnRung { rung: MechanicallyPreventable }, ceiling: StructurallyImpossible, blocker: ClimbableButUnbuilt, trigger_capability: "every workflow model' $ROWF | xargs echo "  anchor unique-in-file (expect 1):"
sed -i 's/current: FoundOnRung { rung: MechanicallyPreventable }, ceiling: StructurallyImpossible, blocker: ClimbableButUnbuilt, trigger_capability: "every workflow model/current: FoundOnRung { rung: StructurallyImpossible }, ceiling: StructurallyImpossible, blocker: ClimbableButUnbuilt, trigger_capability: "every workflow model/' $ROWF
grep -c 'current: FoundOnRung { rung: StructurallyImpossible }, ceiling: StructurallyImpossible' $ROWF | xargs echo "  mutation applied (expect 1):"
run_census mut
git cat-file blob HEAD:$ROWF > $ROWF; cmp -s /tmp/613k_row.orig $ROWF && echo "  RESTORED_BYTE_IDENTICAL row" || echo "  !!RESTORE_FAILED"
echo "-------- K3 census after restore $(date -u +%H:%M:%S)Z"
run_census final
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_613_K
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
