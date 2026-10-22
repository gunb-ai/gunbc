set -u
cd /root/workspace/repo-root 2>/dev/null || true
HEADSHA=cba9dc575f7a8f9b2dc9d377f0127117f74b473a
PAIR=src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag
WITN=src/v2/test/claim/claim_coverage_census_s1_witness.dag
DEBT=src/v2/workflow/floor_pure_producer_share.dag
ROW='claim: "the_live_entry_exits_one_while_the_wet_lane_has_no_executor"'
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575q
git fetch -q origin refs/pull/14575/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575q $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt575q
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575q 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575q
echo 51539607552 > /sys/fs/cgroup/ver575q/memory.max
RB=$(cat /sys/fs/cgroup/ver575q/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575q memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575q/cgroup.procs; echo $BASHPID >&11
cd $WT
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575d/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
NAMES_W=$(grep -oE '^test fn [A-Za-z_0-9]+' $WITN | sed 's/^test fn //' | paste -sd, -)
echo "=== A. debt-list validators at head (s1_witness 6, by name; the pairing claim is the author's >3400s live route — NOT re-run, recorded as author-stated) ==="
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $WITN --functions "$NAMES_W" > /tmp/p575q_w.out 2>&1; echo "[s1_witness] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/p575q_w.out | sed 's/^/  | /'
echo "=== B. MUTATION: fabricate the enrolled claim identity (row must be caught or that is a finding) ==="
grep -c "$ROW" $DEBT
python3 - <<'PY'
p="/tmp/wt575q/src/v2/workflow/floor_pure_producer_share.dag"
s=open(p).read()
a='claim: "the_live_entry_exits_one_while_the_wet_lane_has_no_executor"'
assert s.count(a)==1, ("anchor count", s.count(a))
open(p,"w").write(s.replace(a,'claim: "the_live_entry_exits_one_while_the_wet_lane_has_no_executor_NOPE"')); print("MUT applied")
PY
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $WITN --functions "$NAMES_W" > /tmp/p575q_wm.out 2>&1; echo "[s1_witness-mut] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/p575q_wm.out | sed 's/^/  | /'
echo "=== C. restore + post ==="
git checkout -- $DEBT
git status --porcelain -- src/v2/workflow/ | head -2; echo "DIRTY_ABOVE (empty = byte-exact)"
timeout 2400 $CB --claim-run --source-root dag --source-root src/v2 --entry $WITN --functions "$NAMES_W" > /tmp/p575q_wp.out 2>&1; echo "[s1_witness-post] rc=$?"; grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/p575q_wp.out | sed 's/^/  | /'
echo DONE_14575D4 $(date -u +%H:%M:%SZ)
