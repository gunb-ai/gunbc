#!/usr/bin/env bash
# #14692 @ 779fd98addff — run the exit-composition unit tests (batch_exit_code) at the head.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=779fd98addfff08aed3de2e269a14b393437c7a5
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver692t 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver692t
echo 51539607552 > /sys/fs/cgroup/ver692t/memory.max
RB=$(cat /sys/fs/cgroup/ver692t/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver692t memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver692t/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14692/head:refs/verifier/692t || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/692t)" = "$OID" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree remove --force /tmp/w692t 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w692t; git worktree add --detach /tmp/w692t $OID 2>&1 | tail -1 | sed 's/^/  WT| /'
cd /tmp/w692t || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t692t
timeout 2400 cargo test --release -p v1-compiler --bin claim_batch batch_exit_code 2>&1 | tail -6 | sed 's/^/  T| /'
echo "DONE_692T $(date -u +%H:%M:%S)Z"
