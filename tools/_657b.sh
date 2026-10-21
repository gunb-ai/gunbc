set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14657b RE-RUN on the ENROLLED route: claim_batch --wet --source-root dag --source-root src/v2
# (local_repo_wet_terminal.dag:187 documents exactly this argv shape; my first run added src/v1
# as a third root -- route deviation, the census module index differs). Cached build reused.
OID=839097ec20a0ebba7788c5d9a1f81d744e54df31
E=dag/test/claim/design_ruling_standing_witness.dag
F=an_unclassified_ruling_row_is_refused_and_a_classified_one_is_not
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver657b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver657b
echo 51539607552 > /sys/fs/cgroup/ver657b/memory.max
RB=$(cat /sys/fs/cgroup/ver657b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver657b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver657b/cgroup.procs; echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w657b
git fetch -q origin +refs/pull/14657/head:refs/verifier/14657b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14657b)" = "$OID" ] || { echo "PR_REF_MOVED want=$OID STOP"; exit 9; }
git worktree add -q --detach /tmp/w657b $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w657b || exit 9
[ "$(git cat-file blob $OID:$E | sha256sum | cut -d' ' -f1)" = "$(sha $E)" ] || { echo "ENTRY_NOT_HEAD_BLOB"; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tbat CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.tbat/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) (reuse; prior 14657 build a06d3676bb5a...)"
echo "=== run on ENROLLED route (dag + src/v2), by name ==="
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $E --functions "$F" > /tmp/c657b2.out 2>&1
echo "  rc=$? PASS=$(grep -ac '^PASS ' /tmp/c657b2.out) FAIL=$(grep -ac '^FAIL ' /tmp/c657b2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c657b2.out) $(date -u +%H:%M:%S)Z"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c657b2.out | sed 's/^/    V| /' | cut -c1-170
echo "=== same claim, 3-root route (reproduce my earlier FAIL; wrong-cause control) ==="
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $E --functions "$F" > /tmp/c657b3.out 2>&1
echo "  rc=$? PASS=$(grep -ac '^PASS ' /tmp/c657b3.out) FAIL=$(grep -ac '^FAIL ' /tmp/c657b3.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c657b3.out) $(date -u +%H:%M:%S)Z"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c657b3.out | sed 's/^/    V| /' | cut -c1-170
echo DONE_657B
cd /root/workspace/repo-root
git worktree remove --force /tmp/w657b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
