#!/usr/bin/env bash
# #14886 B: the two long spot-check claims, ONE head-built claim_batch, bound cgroup, parallel.
# commit_witness_claim_roster_holds (was FAIL->fixed in-PR; verify PASS at head) and
# roster_std_change_dag_zero_suspects_within_ratchet (ProcessExit claim; author PASS 223s resolve).
# Manager asked: --timeout 2h semantics -> outer timeout 5400 (90m per spot); note runner cap in receipt.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver886b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver886b
echo 51539607552 > /sys/fs/cgroup/ver886b/memory.max
RB=$(cat /sys/fs/cgroup/ver886b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver886b memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14886/head:refs/verifier/f14886b || { echo FETCH_FAIL; exit 9; }
FULL=$(git rev-parse refs/verifier/f14886b)
[ "${FULL:0:12}" = "a87a952f4492" ] || { echo "PR_REF_MOVED got=$FULL -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$FULL"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
T=/tmp/w886b_h
git worktree prune >/dev/null 2>&1; rm -rf $T
git worktree add -q --force --detach $T "$FULL" || { echo WT_FAIL; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t886b
if [ -x /root/workspace/repo-root/.t886b/release/claim_batch ]; then echo CACHED; else
  (cd $T && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1)
fi
CB=/root/workspace/repo-root/.t886b/release/claim_batch
[ -x $CB ] || { echo NO_BINARY; exit 9; }
echo "CB_SHA=$(sha $CB)"
spot() {
  local tag=$1 entry=$2 fn=$3
  local out=/tmp/w886b_s_$tag.out
  ( cd $T && exec 11>/sys/fs/cgroup/ver886b/cgroup.procs && echo $BASHPID >&11 && \
    timeout 5400 $CB --claim-run --source-root dag --source-root src/v2 --entry $entry --functions "$fn" > $out 2>&1 )
  local rc=$?
  echo "  [spot $tag] rc=$rc PASS=$(grep -ac "^PASS $fn" $out) FAIL=$(grep -ac "^FAIL $fn" $out) PANIC=$(grep -ac "^PANICKED" $out) BYTES=$(wc -c < $out) $(date -u +%H:%M:%S)Z"
  grep -aE "^(PASS|FAIL|PANICKED) $fn" $out | head -2 | sed 's/^/    S| /' | cut -c1-220
  grep -aiE 'not held|runtime error|refus|resolve' $out | head -4 | sed 's/^/    E| /' | cut -c1-190
}
echo "== SPOTS start $(date -u +%H:%M:%S)Z"
spot cwcr dag/test/claim/long/commit_witness_claim_roster_witness.dag commit_witness_claim_roster_holds &
P1=$!
spot rsc  src/v2/test/claim/complexity/accumulator_copy_roster_gate_std.dag roster_std_change_dag_zero_suspects_within_ratchet &
P2=$!
wait $P1; wait $P2
echo DONE_14886B
