#!/usr/bin/env bash
# #14893 D: rfm-rung-census execution, head vs base (RFM row BelowCeiling restored).
# The census case live_roster_every_read_row_is_admitted holds iff every read roster row is admitted.
# Head: retired row AtCeiling -> admitted -> 6/6 cases held. Base: row BelowCeiling -> refusal prints
# (that is the RFM identity!) -> the live-roster case must NOT hold. Discover the invocation shape:
# try gunbc test <target>, then claim_batch --native-claim-report; usage lines printed on empty.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=794a10d4ae612da1794d2e9949ba36faf50b1002
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893d 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893d
echo 51539607552 > /sys/fs/cgroup/ver893d/memory.max
RB=$(cat /sys/fs/cgroup/ver893d/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893d memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893d || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893d)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893
T=/tmp/w893d_h; B=/tmp/w893d_b
git worktree prune >/dev/null 2>&1 || true; rm -rf $T $B
git worktree add -q --force --detach $T $HEAD_SHA || { echo WT_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
cp $REPO/tools/stage893/a_refusal_prints_and_the_line_continues.base.dag $B/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || exit 9
grep -q BelowCeiling $B/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || { echo RESTORE_NOOP; exit 9; }
echo BASE_RESTORE_OK
(cd $T && cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
GUNBC=$CARGO_TARGET_DIR/release/gunbc; CB=$CARGO_TARGET_DIR/release/claim_batch
echo "GUNBC_SHA=$(sha $GUNBC) CB_SHA=$(sha $CB)"
TARGET='//gunbc/instruments:rfm-rung-census'
census() { # tag tree
  local tag=$1 tree=$2
  local out=/tmp/w893d_$tag.out
  cd $tree || exit 9
  timeout 1500 $GUNBC test "$TARGET" --source-root dag --source-root src/v2 > $out 2>&1; local rc=$?
  local bytes=$(wc -c < $out)
  if [ $bytes -lt 50 ]; then
    echo "  [$tag gunbc-test] rc=$rc BYTES=$bytes -> retry without source-root flags"
    timeout 1500 $GUNBC test "$TARGET" > $out 2>&1; rc=$?; bytes=$(wc -c < $out)
  fi
  if [ $bytes -lt 50 ]; then
    echo "  [$tag gunbc-test] rc=$rc BYTES=$bytes -> try claim_batch native-claim-report"
    timeout 1500 $CB --native-claim-report --target "$TARGET" --source-root dag --source-root src/v2 > $out 2>&1; rc=$?; bytes=$(wc -c < $out)
  fi
  echo "  [$tag] rc=$rc BYTES=$bytes $(date -u +%H:%M:%S)Z"
  grep -aE 'case |roster |refus|held' $out | head -14 | sed 's/^/    C| /' | cut -c1-190
  if [ $bytes -lt 120 ]; then echo "    ---- usage dump ----"; (timeout 60 $GUNBC --help 2>&1 | head -15; timeout 60 $CB --help 2>&1 | head -20) | sed 's/^/    U| /' | cut -c1-190; fi
  cd $REPO || exit 9
}
echo "=== census head: $(date -u +%H:%M:%S)Z"; census d_head $T
echo "=== census base: $(date -u +%H:%M:%S)Z"; census d_base $B
echo DONE_14893D
