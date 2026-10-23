#!/usr/bin/env bash
# #14893 D2: execute the rfm-rung-census REPORT fn directly through the claim route, head vs base,
# ONE head-built binary (the census .dag is identical; the tree differs only in the RFM row file).
# Manager ask: the retired row's new AtCeiling standing must RESOLVE in the census: expect census_below_ceiling
# one lower at head than base (row no longer below-ceiling), live_roster case held, no refusal.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=794a10d4ae612da1794d2e9949ba36faf50b1002
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893d2 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893d2
echo 51539607552 > /sys/fs/cgroup/ver893d2/memory.max
RB=$(cat /sys/fs/cgroup/ver893d2/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893d2 memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893d2 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893d2)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893
T=/tmp/w893d2_h; B=/tmp/w893d2_b
git worktree prune >/dev/null 2>&1 || true; rm -rf $T $B
git worktree add -q --force --detach $T $HEAD_SHA || { echo WT_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
cp $REPO/tools/stage893/a_refusal_prints_and_the_line_continues.base.dag $B/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || exit 9
grep -q BelowCeiling $B/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || { echo RESTORE_NOOP; exit 9; }
grep -q AtCeiling $T/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || { echo HEAD_ROW_UNEXPECTED; exit 9; }
echo TREES_OK
# warm cache: build head bin if missing (deterministic; sha printed)
(cd $T && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1)
CB=$CARGO_TARGET_DIR/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
echo "CB_SHA=$(sha $CB) (single driver, both trees)"
INS=dag/gunbc/instruments/rfm_rung_census.dag
run() { # tag tree
  local tag=$1 tree=$2
  cd $tree || exit 9
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $INS --functions native_claim_report > /tmp/w893d2_$tag.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc BYTES=$(wc -c < /tmp/w893d2_$tag.out) PASS=$(grep -ac '^PASS native_claim_report' /tmp/w893d2_$tag.out) $(date -u +%H:%M:%S)Z"
  grep -aE 'roster |census_read|census_below|case |refus|not held|held' /tmp/w893d2_$tag.out | head -16 | sed 's/^/    C| /' | cut -c1-200
  cd $REPO || exit 9
}
echo "=== census head: $(date -u +%H:%M:%S)Z"; run d2_head $T
echo "=== census base: $(date -u +%H:%M:%S)Z"; run d2_base $B
echo DONE_14893D2
