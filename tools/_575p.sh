#!/usr/bin/env bash
# #14575 @ 7df542647be8f36557c1f7712ef8738503900ab8 (moved head; subject census blob
# byte-identical to my 8f7af127 verified head; real_execution file rewritten by the delta).
# Arms: [ctl_wit] 40 hermetic by name; [ctl_rex] wet 7 by name (names re-derived from NEW file);
# P1 join-conjunct plant; P2 citation-list plant (+ wet control under same plant);
# P3b receipt-attribution plant; each with byte-exact restore; post baseline.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEADSHA=7df542647be8f36557c1f7712ef8738503900ab8
[ ${#HEADSHA} -eq 40 ] || { echo "LEN STOP"; exit 9; }
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
HEAD_CEN_SHA=726e9c1462a9830a1dc139c84f77679f7ddf1e3c996a5b7839c1086ac2e35a45
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575p
git fetch -q origin refs/pull/14575/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575p $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt575p
sha(){ sha256sum "$1" | cut -d' ' -f1; }
cd $WT
[ "$(sha $CEN)" = "$HEAD_CEN_SHA" ] && echo "SUBJECT_OK census ${HEAD_CEN_SHA:0:16} (byte-identical to 8f7af127 verified blob)" || { echo "SUBJECT_DRIFT census -- STOP"; exit 9; }
for f in $WIT $REX; do [ "$(git cat-file blob $HEADSHA:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
cp -f $CEN /tmp/cen575p_orig
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575p 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575p
echo 51539607552 > /sys/fs/cgroup/ver575p/memory.max
RB=$(cat /sys/fs/cgroup/ver575p/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575p memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575p/cgroup.procs && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (proceeding)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575d/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l) (40 expected)"
RNAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $REX | sed 's/^test fn //' | paste -sd, -)
echo "REX_COUNT=$(echo $RNAMES | tr ',' '\n' | wc -l) (7 expected): $RNAMES"
echo "=== [ctl_wit] baseline 40: $(date -u +%H:%M:%S)Z"
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/p_ctlwit.out 2>&1; echo "[ctl_wit] rc=$? bytes=$(wc -c < /tmp/p_ctlwit.out)"
grep -aE 'counts: |printed_total' /tmp/p_ctlwit.out | sed 's/^/  /'
echo "PASS=$(grep -ac '^PASS ' /tmp/p_ctlwit.out) FAIL=$(grep -ac '^FAIL ' /tmp/p_ctlwit.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/p_ctlwit.out)"
echo "=== [ctl_rex] wet 7: $(date -u +%H:%M:%S)Z"
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $REX --functions "$RNAMES" > /tmp/p_ctlrex.out 2>&1; echo "[ctl_rex] rc=$? bytes=$(wc -c < /tmp/p_ctlrex.out)"
echo "PASS=$(grep -ac '^PASS ' /tmp/p_ctlrex.out) FAIL=$(grep -ac '^FAIL ' /tmp/p_ctlrex.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/p_ctlrex.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/p_ctlrex.out | sed 's/^/  F| /' | cut -c1-200
plant() { # $1 plantfile $2 tag  -- census subject only
  cp -f /root/workspace/repo-root/tools/stage/$1 $WT/$CEN
  echo "  [$2] census sha=$(sha $WT/$CEN | cut -c1-16) $(date -u +%H:%M:%S)Z"
  timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/p_$2.out 2>&1; echo "  [$2 wit] rc=$?"
  echo "  PASS=$(grep -ac '^PASS ' /tmp/p_$2.out) FAIL=$(grep -ac '^FAIL ' /tmp/p_$2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/p_$2.out)"
  grep -aE '^FAIL ' /tmp/p_$2.out | sed 's/^/    F| /'
  cp -f /tmp/cen575p_orig $WT/$CEN
  [ "$(sha $WT/$CEN)" = "$HEAD_CEN_SHA" ] && echo "  RESTORE_BYTE_IDENTICAL" || echo "  RESTORE_MISMATCH -- STOP"
}
echo "=== P1 join-conjunct plant (expect FAIL exactly the_join_requires_the_fact_to_name_the_path_and_the_module_to_be_emitted):"
plant census_p1.dag p1
echo "=== P2 citation-list plant (expect FAIL exactly the_citation_list_covers_every_authority_the_cause_carries):"
plant census_p2.dag p2
echo "=== P2 wet control under same plant (expect PASS=7; recorded):"
cp -f /root/workspace/repo-root/tools/stage/census_p2.dag $WT/$CEN
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $REX --functions "$RNAMES" > /tmp/p_p2rex.out 2>&1; echo "  [p2_rex] rc=$? PASS=$(grep -ac '^PASS ' /tmp/p_p2rex.out) FAIL=$(grep -ac '^FAIL ' /tmp/p_p2rex.out)"
cp -f /tmp/cen575p_orig $WT/$CEN
[ "$(sha $WT/$CEN)" = "$HEAD_CEN_SHA" ] && echo "  RESTORE_BYTE_IDENTICAL" || { echo "RESTORE_MISMATCH -- STOP"; exit 9; }
echo "=== P3b receipt-attribution plant (expect FAIL exactly the trio):"
plant census_p3b.dag p3b
echo "=== post baseline wit: $(date -u +%H:%M:%S)Z"
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/p_postwit.out 2>&1; echo "[post_wit] rc=$? PASS=$(grep -ac '^PASS ' /tmp/p_postwit.out) FAIL=$(grep -ac '^FAIL ' /tmp/p_postwit.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/p_postwit.out)"
git status --porcelain | head -3
echo DONE_14575P $(date -u +%H:%M:%S)Z
