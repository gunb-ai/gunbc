#!/usr/bin/env bash
# #14793 F: corrected BASE arm. Base = merge-base 839708c925 (field reason: Symbol; witness file is
# PR-added, so overlay the head's witness file onto the base tree -- the fixture sources are exactly
# the head's). Same head-built binary as the head run (CB 65dfcbbc0807... deterministic).
# Expect: rc=1, FAIL exactly an_empty_literal_reason_is_refused (empty accepted by Symbol), control PASS.
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
rm -rf /sys/fs/cgroup/ver793f 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver793f
echo 51539607552 > /sys/fs/cgroup/ver793f/memory.max
RB=$(cat /sys/fs/cgroup/ver793f/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver793f memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t793
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w793f
git fetch -q origin +refs/pull/14793/head:refs/verifier/f793f || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f793f)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git cat-file -e $HEAD:dag/test/claim/observation_unrepresentable_reason_witness.dag 2>/dev/null || { echo NO_WIT; exit 9; }
git worktree add -q --detach /tmp/w793f $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w793f || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
grep -a "ObservationUnrepresentable { reason: Symbol }" src/v2/workflow/floor_terminal_ledger.dag | sed 's/^/  BASE_FIELD| /' | head -1
git -C /tmp/w793f checkout $HEAD -- dag/test/claim/observation_unrepresentable_reason_witness.dag
# verify against head blob directly
git show $HEAD:dag/test/claim/observation_unrepresentable_reason_witness.dag > /tmp/wit793_head.expect
[ "$(sha dag/test/claim/observation_unrepresentable_reason_witness.dag)" = "$(sha /tmp/wit793_head.expect)" ] && echo "  OVERLAY_MATCHES_HEAD_BLOB=YES" || { echo "OVERLAY_MISMATCH -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t793/release/claim_batch
[ -x $CB ] || { echo "NO BINARY (build?)"; cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1; }
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB) (expect 65dfcbbc0807... same head-built binary as head run)"
NW="an_empty_literal_reason_is_refused,a_nonempty_str_reason_is_accepted_by_the_retyped_field"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry dag/test/claim/observation_unrepresentable_reason_witness.dag --functions "$NW" > /tmp/f793_baseF.out 2>&1
echo "[baseF] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f793_baseF.out) FAIL=$(grep -ac '^FAIL ' /tmp/f793_baseF.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f793_baseF.out) bytes=$(wc -c < /tmp/f793_baseF.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f793_baseF.out | sed 's/^/  V| /' | cut -c1-220
head -8 /tmp/f793_baseF.out | sed 's/^/  O| /' | cut -c1-200
echo DONE_793F $(date -u +%H:%M:%S)Z
