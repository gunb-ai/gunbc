set -u
cd /root/workspace/repo-root 2>/dev/null || true
HEADSHA=cba9dc575f7a8f9b2dc9d377f0127117f74b473a
PAIR=src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575r
git fetch -q origin refs/pull/14575/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575r $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575r 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575r
echo 51539607552 > /sys/fs/cgroup/ver575r/memory.max
RB=$(cat /sys/fs/cgroup/ver575r/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575r memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575r/cgroup.procs; echo $BASHPID >&11
cd /tmp/wt575r
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575d/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
echo "=== THE OBSERVING CLAIM: pairing claim by name (author-stated 3432s; expect PASS=1 rc=0: exit-1 netted by the new row) ==="
timeout 3000 $CB --claim-run --source-root dag --source-root src/v2 --entry $PAIR --functions "the_module_real_path_pairing_row_grounds_against_the_real_discovery" > /tmp/p575r.out 2>&1; echo "[pairing-head] rc=$?"
grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/p575r.out | sed 's/^/  | /'
tail -6 /tmp/p575r.out | cut -c1-220 | sed 's/^/  T| /'
echo DONE_14575D5 $(date -u +%H:%M:%SZ)
