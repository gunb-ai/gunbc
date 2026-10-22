#!/usr/bin/env bash
# #14575 R @ 873fb35e529335ae150dd6e2748ae801d8389eb4 (review 84644: typed CensusUnreadCause).
# Arms: wit all-claims by name (re-derived; expect 48); wet 7 by name; plants P1/P2/P3b (regenerated
# from the new blob) + R1 (renderer prefix collision) + R2 (exit_code/stderr dropped from rendering);
# restore byte-exact; post baseline.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEADSHA=873fb35e529335ae150dd6e2748ae801d8389eb4
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
[ ${#HEADSHA} -eq 40 ] || { echo "LEN STOP"; exit 9; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575r
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575r 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/f575r)" = "$HEADSHA" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f575r) want=$HEADSHA -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575r $HEADSHA || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wt575r
sha(){ sha256sum "$1" | cut -d' ' -f1; }
cd $WT
HEAD_CEN_SHA=$(sha $CEN); echo "HEAD_CEN_SHA=${HEAD_CEN_SHA}"
for f in $WIT $REX; do [ "$(git cat-file blob $HEADSHA:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
cp -f $CEN /tmp/cen575r_orig
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575r 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575r
echo 51539607552 > /sys/fs/cgroup/ver575r/memory.max
RB=$(cat /sys/fs/cgroup/ver575r/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575r memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver575r/cgroup.procs && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (proceeding)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575r CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575r/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l)"
RNAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $REX | sed 's/^test fn //' | paste -sd, -)
echo "REX_COUNT=$(echo $RNAMES | tr ',' '\n' | wc -l)"
NEW8=$(echo "$NAMES" | tr ',' '\n' | grep -aE "unread_arm|no_answer_arm|as_data|six_distinct|reach_the_rendered_receipt" | paste -sd, -)
echo "NEW_CAUSE_CLAIMS: $NEW8"
echo "=== [ctl_wit] baseline all by name: $(date -u +%H:%M:%S)Z"
timeout 1400 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/r_ctlwit.out 2>&1; echo "[ctl_wit] rc=$? bytes=$(wc -c < /tmp/r_ctlwit.out) PASS=$(grep -ac '^PASS ' /tmp/r_ctlwit.out) FAIL=$(grep -ac '^FAIL ' /tmp/r_ctlwit.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r_ctlwit.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/r_ctlwit.out | head -5 | sed 's/^/  F| /' | cut -c1-200
echo "=== [ctl_rex] wet 7: $(date -u +%H:%M:%S)Z"
timeout 1200 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $REX --functions "$RNAMES" > /tmp/r_ctlrex.out 2>&1; echo "[ctl_rex] rc=$? bytes=$(wc -c < /tmp/r_ctlrex.out) PASS=$(grep -ac '^PASS ' /tmp/r_ctlrex.out) FAIL=$(grep -ac '^FAIL ' /tmp/r_ctlrex.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r_ctlrex.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/r_ctlrex.out | head -4 | sed 's/^/  F| /' | cut -c1-200
plant() { # $1 plantfile $2 tag
  cp -f /root/workspace/repo-root/tools/stage/$1 $WT/$CEN
  echo "  [$2] census sha=$(sha $WT/$CEN | cut -c1-16) $(date -u +%H:%M:%S)Z"
  timeout 1400 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/r_$2.out 2>&1
  echo "  [$2 wit] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r_$2.out) FAIL=$(grep -ac '^FAIL ' /tmp/r_$2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r_$2.out)"
  grep -aE '^FAIL ' /tmp/r_$2.out | sed 's/^/    F| /' | cut -c1-220
  cp -f /tmp/cen575r_orig $WT/$CEN
  [ "$(sha $WT/$CEN)" = "$HEAD_CEN_SHA" ] && echo "  RESTORE_BYTE_IDENTICAL" || echo "  RESTORE_MISMATCH -- STOP"
}
echo "=== P1 join-conjunct:"; plant census873_p1.dag p1
echo "=== P2 citation-list:"; plant census873_p2.dag p2
echo "=== P3b receipt attribution:"; plant census873_p3b.dag p3b
echo "=== R1 renderer prefix collision (index prefix -> marker wording):"; plant census873_r1.dag r1
echo "=== R2 renderer drops exit_code/stderr data:"; plant census873_r2.dag r2
echo "=== post baseline wit: $(date -u +%H:%M:%S)Z"
timeout 1400 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/r_postwit.out 2>&1; echo "[post_wit] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r_postwit.out) FAIL=$(grep -ac '^FAIL ' /tmp/r_postwit.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r_postwit.out)"
echo "=== renderer use-site statics (definition + receipts/exit-reason only):"
git grep -n "census_unread_cause_text(" $HEADSHA -- '*.dag' | sed "s|$HEADSHA:||" | sed 's/^/  R| /'
git status --porcelain | head -3
echo DONE_14575R $(date -u +%H:%M:%S)Z
