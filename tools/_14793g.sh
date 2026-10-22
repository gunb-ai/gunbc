#!/usr/bin/env bash
# #14793 G: strict "same binary" base arm -- build at HEAD (expect 65dfcbbc0807...), then run
# that exact binary over the base tree (839708c925, Symbol field) with the head witness overlaid.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEAD=92d35c4f0db7dd19bf3181fc9e507ee67e434a93
BASE=839708c925ba8a93f17621aa9a64ba7b5b57d0c5
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver793g 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver793g
echo 51539607552 > /sys/fs/cgroup/ver793g/memory.max
RB=$(cat /sys/fs/cgroup/ver793g/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver793g memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t793g
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w793gh /tmp/w793gb
git fetch -q origin +refs/pull/14793/head:refs/verifier/f793g || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f793g)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w793gh $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w793gh || exit 9
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t793g/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) (head run had 65dfcbbc0807...)"
WI=dag/test/claim/observation_unrepresentable_reason_witness.dag
NW="an_empty_literal_reason_is_refused,a_nonempty_str_reason_is_accepted_by_the_retyped_field"
cd /tmp/w793gh || exit 9
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WI --functions "$NW" > /tmp/g793_head.out 2>&1
echo "[G head] rc=$? PASS=$(grep -ac '^PASS ' /tmp/g793_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/g793_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/g793_head.out)"
git worktree add -q --detach /tmp/w793gb $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w793gb || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
grep -a "ObservationUnrepresentable { reason: Symbol }" src/v2/workflow/floor_terminal_ledger.dag | sed 's/^/  BASE_FIELD| /' | head -1
git checkout $HEAD -- $WI
git show $HEAD:$WI > /tmp/witG.expect
[ "$(sha $WI)" = "$(sha /tmp/witG.expect)" ] && echo "  OVERLAY_MATCHES_HEAD_BLOB=YES" || { echo "OVERLAY_MISMATCH -- STOP"; exit 9; }
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WI --functions "$NW" > /tmp/g793_base.out 2>&1
echo "[G base SAME-BINARY] rc=$? PASS=$(grep -ac '^PASS ' /tmp/g793_base.out) FAIL=$(grep -ac '^FAIL ' /tmp/g793_base.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/g793_base.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/g793_base.out | sed 's/^/  V| /' | cut -c1-220
echo DONE_793G $(date -u +%H:%M:%S)Z
