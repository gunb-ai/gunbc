set -uo pipefail
REPO=/root/workspace/repo-root
HEADSHA=ff85b62ae57f5f6bc2aeef903c463eb04bd56d04
WIT=dag/test/claim/instrument_dispatch_workflow_witness.dag
cd $REPO
git fetch --force origin $HEADSHA 2>&1 | tail -1
git worktree add -f /tmp/wt613i $HEADSHA 2>&1 | tail -1
cd /tmp/wt613i
git rev-parse HEAD | grep -q "^$HEADSHA" || { echo "PR_REF_MOVED/GUARD FAIL"; exit 9; }
export CARGO_TARGET_DIR=$REPO/.t613i
CG=/sys/fs/cgroup/ver613i; mkdir -p $CG
if [ -f $CG/cgroup.subtree_control ]; then echo +memory > $CG/cgroup.subtree_control 2>/dev/null || true; fi
echo 51539607552 > $CG/memory.max
RB=$(cat $CG/memory.max); [ "$RB" != "max" ] || { echo "WRAPPER NOT BOUNDED"; exit 9; }
echo "CGROUP_WRAPPER=$CG memory.max=$RB (read-back, bounded)"
exec 12>>$CG/cgroup.procs 2>/dev/null && echo $BASHPID >&12 2>/dev/null || echo "NOTE: fd join unavailable; runner ExternallyBounded (BUILDBUDDY_RUN_ID=$BUILDBUDDY_RUN_ID)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=$REPO/.t613i/release/claim_batch
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha256sum $CB|cut -c1-12) $(date -u +%H:%M:%SZ)"
FNS=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | awk '{print $3}' | paste -sd, -)
echo "CLAIM_NAMES=$FNS"; echo "CLAIM_COUNT=$(echo $FNS | tr ',' '\n' | wc -l)"
timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$FNS" > /tmp/c613_head.out 2>&1; echo "[head] rc=$?"
grep -aE '^(PASS|FAIL|UNOBS) ' /tmp/c613_head.out | sed 's/^/  | /'
grep -ac '^PASS ' /tmp/c613_head.out | sed 's/^/PASS_COUNT=/'; grep -ac '^FAIL ' /tmp/c613_head.out | sed 's/^/FAIL_COUNT=/'
echo DONE_613I $(date -u +%H:%M:%SZ)
