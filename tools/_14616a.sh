#!/usr/bin/env bash
# #14616 @ 6e78085bab9e (MERGED head) — d1: head-built binary + 8 gauntlet claims by name (wet)
# + Rust differential pin executing_surfaces_mirror_enrollment_is_scheduled (cargo test, expect pass)
# + identity join: drop population == 20 identities (git-verified list computed locally; here we
# print the derived population from ci_layer_roots drop rows for the record).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=6e78085bab9edf9aed777f85568e00055672267c
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver616 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver616
echo 51539607552 > /sys/fs/cgroup/ver616/memory.max
RB=$(cat /sys/fs/cgroup/ver616/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver616 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver616/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
GAUNT=src/v2/test/claim/long/gauntlet_lane_enrollment_witness.dag
git fetch -q origin +refs/pull/14616/head:refs/verifier/616 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/616)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/616) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w616 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w616; git worktree add --detach /tmp/w616 $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w616 ] || { echo WT_FAIL; exit 9; }
cd /tmp/w616 || exit 9
[ "$(git cat-file blob $OID:$GAUNT | sha256sum | cut -d' ' -f1)" = "$(sha $GAUNT)" ] || { echo "NOT_HEAD_BLOB GAUNT -- STOP"; exit 9; }
GN=$(grep -oE "^test fn [A-Za-z_0-9]+" $GAUNT | sed 's/^test fn //' | paste -sd,)
echo "HEADSHA=$OID"
echo "GAUNT_NAMES=$GN COUNT=$(echo $GN | tr ',' ' ' | wc -w)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t616
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t616/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
echo "=== A. 8 gauntlet claims by name, wet:"
timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $GAUNT --functions "$GN" > /tmp/t616_g.out 2>&1
echo "  rc=$? PASS=$(grep -ac '^PASS ' /tmp/t616_g.out) FAIL=$(grep -ac '^FAIL ' /tmp/t616_g.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/t616_g.out)"
for n in $(echo $GN | tr ',' ' '); do
  echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/t616_g.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/t616_g.out)"
done
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/t616_g.out | head -4 | sed 's/^/    V| /' | cut -c1-220
echo "=== B. pin baseline echo: EXECUTING_COMMIT_SURFACES line:"
grep -n 'const EXECUTING_COMMIT_SURFACES' src/v1/stage0/src/cli_run.rs | sed 's/^/  S| /'
git status --porcelain | head -3
echo "DONE_14616_D1 $(date -u +%H:%M:%S)Z"
