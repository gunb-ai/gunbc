set -u
cd /root/workspace/repo-root 2>/dev/null || true
HEADSHA=cba9dc575f7a8f9b2dc9d377f0127117f74b473a
CENW=dag/test/claim/claim_coverage_census_witness.dag
DEBT=src/v2/workflow/floor_pure_producer_share.dag
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575s
git fetch -q origin $HEADSHA 2>&1 | tail -1
[ "$(git rev-parse $HEADSHA)" = "$HEADSHA" ] || { echo "SHA_MISSING"; exit 9; }
git worktree add -q --detach /tmp/wt575s $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575s 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575s
echo 51539607552 > /sys/fs/cgroup/ver575s/memory.max
RB=$(cat /sys/fs/cgroup/ver575s/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575s memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575s/cgroup.procs; echo $BASHPID >&11
cd /tmp/wt575s
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575d/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $CENW | sed 's/^test fn //' | paste -sd, -)
echo "CENW_CLAIMS=$(echo $NAMES | tr ',' '\n' | wc -l)"
echo "=== A. census_witness 6 claims at head (by name) ==="
timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $CENW --functions "$NAMES" > /tmp/c575s_a.out 2>&1; echo "[cenw-head] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c575s_a.out | sed 's/^/  | /'
echo "=== B. MUTATION: fabricate the new row's claim identity; expect the census to observe it ==="
python3 - <<'PY'
p="/tmp/wt575s/src/v2/workflow/floor_pure_producer_share.dag"
s=open(p).read()
a='claim: "the_live_entry_exits_one_while_the_wet_lane_has_no_executor"'
assert s.count(a)==1, ("anchor count", s.count(a))
open(p,"w").write(s.replace(a,'claim: "the_live_entry_exits_one_while_the_wet_lane_has_no_executor_NOPE"')); print("MUT applied")
PY
timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $CENW --functions "$NAMES" > /tmp/c575s_b.out 2>&1; echo "[cenw-mut] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c575s_b.out | sed 's/^/  | /'
echo "=== C. restore ==="
git checkout -- $DEBT
git status --porcelain -- src/v2/workflow/ | head -2; echo "DIRTY_ABOVE (empty = byte-exact)"
timeout 1500 $CB --claim-run --source-root dag --source-root src/v2 --entry $CENW --functions "$NAMES" > /tmp/c575s_c.out 2>&1; echo "[cenw-post] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c575s_c.out | sed 's/^/  | /'
echo DONE_14575D6 $(date -u +%H:%M:%SZ)
