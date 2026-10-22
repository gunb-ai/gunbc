#!/usr/bin/env bash
# #14575 R3: discriminating renderer mutation. R1 (prefix-only swap) left all 6 texts distinct,
# so the distinct-renderings claim CORRECTLY stayed green -- not a weakness, a non-mutation.
# R1b collapses BOTH axes on one arm: IndexUnread/ReadNoAnswer now renders the exact MarkerUnread/
# ReadNoAnswer text. Expect exactly one FAIL: the_six_unread_causes_render_to_six_distinct_receipt_texts,
# siblings 6/6 PASS (same head-built binary a4775d375427).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
HEAD=873fb35e529335ae150dd6e2748ae801d8389eb4
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
CEN=src/v2/workflow/outside_discovery_roots_census.dag
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575r3 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575r3
echo 51539607552 > /sys/fs/cgroup/ver575r3/memory.max
RB=$(cat /sys/fs/cgroup/ver575r3/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575r3 memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575r
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575r3 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f575r3)" = "$HEAD" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t14575r/release/claim_batch
[ -x $CB ] || { echo NO_BINARY_BUILDING; rm -rf /tmp/w575r3b; git worktree add -q --detach /tmp/w575r3b $HEAD && cd /tmp/w575r3b && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2; cd $REPO; }
echo "CB_SHA=$(sha $CB) (expect a4775d375427... if warm)"
CLAIMS="an_unreadable_listing_is_not_an_empty_census,a_listing_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_listing_answer_is_the_index_no_answer_arm,a_marker_read_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_marker_answer_is_the_marker_no_answer_arm,the_six_unread_causes_render_to_six_distinct_receipt_texts,a_refused_exit_code_and_stderr_reach_the_rendered_receipt"
rm -rf /tmp/w575r3; git worktree add -q --detach /tmp/w575r3 $HEAD || { echo WT_FAIL; exit 9; }
[ "$(sha /tmp/w575r3/$CEN)" = "$(sha tools/stage/census873_r1b.dag)" ] && { echo "PLANT_IS_HEAD -- STOP"; exit 9; }
cp tools/stage/census873_r1b.dag /tmp/w575r3/$CEN || exit 9
echo "  [r1b] census sha=$(sha /tmp/w575r3/$CEN) $(date -u +%H:%M:%S)Z"
cd /tmp/w575r3 || exit 9
timeout 1450 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions $CLAIMS > /tmp/r575_r1b.out 2>&1
echo "  [r1b] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r575_r1b.out) FAIL=$(grep -ac '^FAIL ' /tmp/r575_r1b.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r575_r1b.out)"
grep -aE '^FAIL ' /tmp/r575_r1b.out | sed 's/^/    F| /' | cut -c1-200
cd $REPO || exit 9
echo DONE_R3
