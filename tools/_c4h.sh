set -u
cd /root/workspace/repo-root 2>/dev/null || true
# C4-H for jolly-boar-495: PR 14696 branch fix/c4h-host-budget-typed-refusal head f4cc1b8e2aa9
# Run A: claim_batch inside a REAL bounded subtree (memory.max read-back != max) -> rc=0, PASS,
#        no HostBudgetUnreadable text.
# Run B: inside a subtree with memory.max == max, BUILDBUDDY_RUN_ID unset -> rc=2,
#        'claim_batch: UNOBSERVED cause=HostBudgetUnreadable', no panic, no 137.
A=1747200000
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wc4h
git fetch -q origin +refs/pull/14696/head:refs/verifier/14696 2>&1 | tail -1
HEADSHA=$(git rev-parse refs/verifier/14696) || { echo "FETCH_FAILED"; exit 9; }
echo "HEADSHA=$HEADSHA (gh headRefOid=23d9eadc3e004809505ea959dfa129d3d4f9d8bf, moved from f4cc1b8e2aa9 requested)"
[ "$HEADSHA" = "23d9eadc3e004809505ea959dfa129d3d4f9d8bf" ] || { echo "HEAD_MOVED_from_f4cc1b8e2aa9 -- report and continue? STOP"; exit 9; }
git worktree add -q --detach /tmp/wc4h $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wc4h
sha() { sha256sum "$1" | cut -d' ' -f1; }
cd $WT || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tc4h CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tc4h/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
mkdir -p $WT/target/release && cp -f $CB $WT/target/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA256=$(sha $CB) built $(date -u +%H:%M:%S)Z"
CMD="--claim-run --source-root dag --source-root src/v2 --entry dag/test/claim/react_jsx_emit.dag --functions witness_safe_page_emits_ok"

echo "=== RUN A (bounded subtree, BUILDBUDDY_RUN_ID=${BUILDBUDDY_RUN_ID:-<unset>}) ==="
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verA 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verA
echo 17179869184 > /sys/fs/cgroup/verA/memory.max
RBA=$(cat /sys/fs/cgroup/verA/memory.max)
{ [ "$RBA" != "max" ] && [ -n "$RBA" ]; } || { echo "A_BOUND_READBACK_FAIL got=$RBA"; exit 9; }
echo "  A cgroup=/sys/fs/cgroup/verA memory.max(read-back)=$RBA"
exec 11>>/sys/fs/cgroup/verA/cgroup.procs; echo $BASHPID >&11
cd $WT && timeout 900 $CB $CMD > /tmp/c4h_A.out 2>&1
RC_A=$?
echo "  rc=$RC_A $(date -u +%H:%M:%S)Z"
sed 's/^/    A| /' /tmp/c4h_A.out | grep -aE '^(PASS|FAIL|UNOBSERVED)|HostBudget|panic|OUTPUT-POLICY|REFUSAL' | cut -c1-190 | head -10
echo "  A counters: PASS=$(grep -ac '^PASS ' /tmp/c4h_A.out) FAIL=$(grep -ac '^FAIL ' /tmp/c4h_A.out) HostBudget=$(grep -ac 'HostBudgetUnreadable' /tmp/c4h_A.out) panic=$(grep -ac 'panicked' /tmp/c4h_A.out)"

echo "=== RUN B (unbounded subtree memory.max=max, BUILDBUDDY_RUN_ID UNSET) ==="
rm -rf /sys/fs/cgroup/verB 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verB
echo max > /sys/fs/cgroup/verB/memory.max
RBB=$(cat /sys/fs/cgroup/verB/memory.max)
[ "$RBB" = "max" ] || { echo "B_UNBOUND_SETUP_FAIL got=$RBB"; exit 9; }
echo "  B cgroup=/sys/fs/cgroup/verB memory.max(read-back)=$RBB BUILDBUDDY_RUN_ID=[unset]"
exec 12>>/sys/fs/cgroup/verB/cgroup.procs; echo $BASHPID >&12
cd $WT && env -u BUILDBUDDY_RUN_ID timeout 900 $CB $CMD > /tmp/c4h_B.out 2>&1
RC_B=$?
echo "  B cgroup now: $(cat /proc/self/cgroup)"
echo "  rc=$RC_B $(date -u +%H:%M:%S)Z"
sed 's/^/    B| /' /tmp/c4h_B.out | grep -aE 'UNOBSERVED|HostBudget|panic|OUTPUT-POLICY|REFUSAL' | cut -c1-190 | head -10
echo "  B counters: PASS=$(grep -ac '^PASS ' /tmp/c4h_B.out) FAIL=$(grep -ac '^FAIL ' /tmp/c4h_B.out) UNOBSERVED_HBU=$(grep -ac 'UNOBSERVED cause=HostBudgetUnreadable' /tmp/c4h_B.out) panic=$(grep -ac 'panicked' /tmp/c4h_B.out)"
echo "  B full output (capped 25L):"; sed 's/^/    B| /' /tmp/c4h_B.out | head -25 | cut -c1-200
echo "RUN_A_RC=$RC_A RUN_B_RC=$RC_B CB_SHA256=$(sha $CB) HEAD=$HEADSHA A_MEMMAX=$RBA B_MEMMAX=$RBB"
echo DONE_C4H
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
