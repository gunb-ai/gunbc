set -u
cd /root/workspace/repo-root 2>/dev/null || true
HEADSHA=8f7af1271081476ec2c2019db4d2d448c481ea52
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
HEAD_CEN_SHA=726e9c1462a9830a1dc139c84f77679f7ddf1e3c996a5b7839c1086ac2e35a45
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575n
git fetch -q origin refs/pull/14575/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575n $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt575n
sha(){ sha256sum "$1" | cut -d' ' -f1; }
[ "$(sha $WT/$CEN)" = "$HEAD_CEN_SHA" ] && echo "SUBJECT_OK census ${HEAD_CEN_SHA:0:16}" || { echo "SUBJECT_DRIFT census"; exit 9; }
[ "$(sha tools/stage/census_p3b.dag | cut -c1-16)" = "f18053234a825ada" ] || echo "plant p3b $(sha tools/stage/census_p3b.dag | cut -c1-16)"
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575n 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575n
echo 51539607552 > /sys/fs/cgroup/ver575n/memory.max
RB=$(cat /sys/fs/cgroup/ver575n/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575n memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575n/cgroup.procs; echo $BASHPID >&11
cd $WT
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575d/release/claim_batch
echo "CB_SHA=$(sha $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z"
# claim names from head witness file
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l)"
echo "--- [p3b_wit] expect FAIL=the_receipt_credits_the_note_only_at_the_grain_it_holds only ---"
cp -f /root/workspace/repo-root/tools/stage/census_p3b.dag $CEN
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/p3b.out 2>&1; echo "[p3b_wit] rc=$? bytes=$(wc -c < /tmp/p3b.out) census=$(sha $CEN | cut -c1-16) $(date -u +%H:%M:%S)Z"
grep -aE '^FAIL ' /tmp/p3b.out | sed 's/^/  F| /'
grep -aE 'counts: |printed_total' /tmp/p3b.out | sed 's/^/  /'
cp -f /root/workspace/repo-root/tools/stage/census_new_head.dag $CEN
[ "$(sha $CEN)" = "$HEAD_CEN_SHA" ] && echo "RESTORED_BYTE_IDENTICAL ${HEAD_CEN_SHA:0:16}" || echo "RESTORE_MISMATCH"
echo DONE_14575D3 $(date -u +%H:%M:%S)Z
