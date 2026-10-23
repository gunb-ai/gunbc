#!/usr/bin/env bash
# #14893 @ab3279b680 E2: rfm-rung-census via the REAL route: gunbc test //gunbc/instruments:rfm-rung-census
# (target_invocation_host registry: NativeClaimProgram entry dag/gunbc/instruments/rfm_rung_census.dag).
# Test verb takes ONLY the target (D's --source-root flags were a clap usage error -> 189B).
# head: retired row AtCeiling -> census_below_ceiling must NOT include it; base: BelowCeiling -> it must.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=ab3279b68044af1af59311cb6e1f0307f88a0d92
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893e2 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893e2
echo 51539607552 > /sys/fs/cgroup/ver893e2/memory.max
RB=$(cat /sys/fs/cgroup/ver893e2/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893e2 memory.max=$RB (read-back, bounded)"
exec 11>/sys/fs/cgroup/ver893e2/cgroup.procs; echo $BASHPID >&11
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893e2 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893e2)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$HEAD_SHA"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893e2
T=/tmp/w893e2_h; B=/tmp/w893e2_b
for d in $T $B; do git worktree prune >/dev/null 2>&1; rm -rf $d; done
git worktree add -q --force --detach $T $HEAD_SHA || { echo WT_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
ROW=dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag
grep -q AtCeiling $T/$ROW || { echo HEAD_ROW_UNEXPECTED; exit 9; }
cp $REPO/tools/stage893/a_refusal_prints_and_the_line_continues.base.dag $B/$ROW || exit 9
grep -q BelowCeiling $B/$ROW || { echo RESTORE_NOOP; exit 9; }
echo TREES_OK
(cd $T && cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GUNBC=$CARGO_TARGET_DIR/release/gunbc
[ -x $GUNBC ] || { echo NO_GUNBC; exit 9; }
cp $GUNBC /tmp/gunbc893e2; rm -f $GUNBC
echo "GUNBC_SHA=$(sha /tmp/gunbc893e2) (one binary, both trees)"
TARGET='//gunbc/instruments:rfm-rung-census'
census() { # tag tree
  local tag=$1 tree=$2
  local out=/tmp/w893e2_$tag.out
  cd $tree || exit 9
  timeout 1500 /tmp/gunbc893e2 test "$TARGET" > $out 2>&1; local rc=$?
  echo "  [$tag] rc=$rc BYTES=$(wc -c < $out) $(date -u +%H:%M:%S)Z"
  grep -aE 'census_read|census_below|below_ceiling|case |roster|held|REFUS|native-claim' $out | head -24 | sed 's/^/    C| /' | cut -c1-190
  cd $REPO || exit 9
}
echo "=== census HEAD $(date -u +%H:%M:%S)Z"; census e2_head $T
echo "=== census BASE $(date -u +%H:%M:%S)Z"; census e2_base $B
echo DONE_14893E2
