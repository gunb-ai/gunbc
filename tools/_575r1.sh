#!/usr/bin/env bash
# #14575 R1 @ 873fb35e529: baseline only (wit all-by-name + wet 7). Build head-first. Post-wit at end if time.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEADSHA=873fb35e529335ae150dd6e2748ae801d8389eb4
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575r1 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575r1
echo 51539607552 > /sys/fs/cgroup/ver575r1/memory.max
RB=$(cat /sys/fs/cgroup/ver575r1/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575r1 memory.max=$RB (read-back, bounded)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575r CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
sha(){ sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575r1
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575r1 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/f575r1)" = "$HEADSHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575r1 $HEADSHA || { echo WT_FAIL; exit 9; }
cd /tmp/wt575r1 || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WT_NOT_AT_HEAD; exit 9; }
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
REX=src/v2/test/claim/outside_discovery_roots_census_real_execution.dag
for f in $WIT $REX; do [ "$(git cat-file blob $HEADSHA:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14575r/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd, -)
echo "CLAIM_COUNT=$(echo $NAMES | tr ',' '\n' | wc -l)"
NEW8="an_unreadable_listing_is_not_an_empty_census,a_listing_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_listing_answer_is_the_index_no_answer_arm,a_marker_read_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_marker_answer_is_the_marker_no_answer_arm,the_six_unread_causes_render_to_six_distinct_receipt_texts,a_refused_exit_code_and_stderr_reach_the_rendered_receipt"
echo "=== [ctl_wit] 48 by name: $(date -u +%H:%M:%S)Z"
timeout 1450 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/r1_ctlwit.out 2>&1; echo "[ctl_wit] rc=$? bytes=$(wc -c < /tmp/r1_ctlwit.out) PASS=$(grep -ac '^PASS ' /tmp/r1_ctlwit.out) FAIL=$(grep -ac '^FAIL ' /tmp/r1_ctlwit.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r1_ctlwit.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/r1_ctlwit.out | head -5 | sed 's/^/  F| /' | cut -c1-200
for c in $(echo $NEW8 | tr ',' ' '); do echo "  NEW8 $c: PASS=$(grep -ac "^PASS $c\$" /tmp/r1_ctlwit.out) FAIL=$(grep -ac "^FAIL $c\$" /tmp/r1_ctlwit.out)"; done
echo "=== [ctl_rex] wet 7: $(date -u +%H:%M:%S)Z"
RNAMES=$(grep -oE '^test fn [A-Za-z_0-9]+' $REX | sed 's/^test fn //' | paste -sd, -)
timeout 1450 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $REX --functions "$RNAMES" > /tmp/r1_ctlrex.out 2>&1; echo "[ctl_rex] rc=$? bytes=$(wc -c < /tmp/r1_ctlrex.out) PASS=$(grep -ac '^PASS ' /tmp/r1_ctlrex.out) FAIL=$(grep -ac '^FAIL ' /tmp/r1_ctlrex.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r1_ctlrex.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/r1_ctlrex.out | head -4 | sed 's/^/  F| /' | cut -c1-200
echo DONE_575R1 $(date -u +%H:%M:%S)Z
