#!/usr/bin/env bash
# #14575 R2 @ 873fb35e529: plant arms with AIMED claim sets (warm target dir -> no rebuild).
# P1 -> join claim; P2 -> citation claim; P3b -> trio; R1 -> renderer collision; R2 -> exit_data dropped.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEADSHA=873fb35e529335ae150dd6e2748ae801d8389eb4
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver575r2 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver575r2
echo 51539607552 > /sys/fs/cgroup/ver575r2/memory.max
RB=$(cat /sys/fs/cgroup/ver575r2/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver575r2 memory.max=$RB (read-back, bounded)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14575r CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
sha(){ sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575r2
[ "$(git rev-parse refs/verifier/f575r1 2>/dev/null)" = "$HEADSHA" ] || { git fetch -q origin +refs/pull/14575/head:refs/verifier/f575r1 2>&1 | tail -1; }
[ "$(git rev-parse refs/verifier/f575r1)" = "$HEADSHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575r2 $HEADSHA || { echo WT_FAIL; exit 9; }
cd /tmp/wt575r2 || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WT_NOT_AT_HEAD; exit 9; }
[ "$(git cat-file blob $HEADSHA:$WIT | sha256sum | cut -d' ' -f1)" = "$(sha $WIT)" ] || { echo "NOT_HEAD_BLOB WIT -- STOP"; exit 9; }
HEAD_CEN_SHA=$(sha $CEN); echo "HEAD_CEN_SHA=${HEAD_CEN_SHA}"
cp -f $CEN /tmp/cen575r2_orig
CB=/root/workspace/repo-root/.t14575r/release/claim_batch
[ -x $CB ] || { echo "NO BINARY; building"; cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1; }
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB)"
AIM_P1="the_join_requires_the_fact_to_name_the_path_and_the_module_to_be_emitted"
AIM_P2="the_citation_list_covers_every_authority_the_cause_carries"
AIM_P3B="the_receipt_credits_the_note_only_at_the_grain_it_holds,every_receipt_line_says_the_marker_answer_is_a_pattern_match_not_the_marker_authority,the_census_receipt_never_borrows_the_marker_authoritys_name"
AIM_R="an_unreadable_listing_is_not_an_empty_census,a_listing_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_listing_answer_is_the_index_no_answer_arm,a_marker_read_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_marker_answer_is_the_marker_no_answer_arm,the_six_unread_causes_render_to_six_distinct_receipt_texts,a_refused_exit_code_and_stderr_reach_the_rendered_receipt"
# baseline of the aimed set first (control that plants have something to red)
echo "=== aimed baseline (7 cause claims): $(date -u +%H:%M:%S)Z"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$AIM_R" > /tmp/r2_aimbase.out 2>&1
echo "[aimbase] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r2_aimbase.out) FAIL=$(grep -ac '^FAIL ' /tmp/r2_aimbase.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r2_aimbase.out)"
runaim() { # $1 plant $2 tag $3 names
  cp -f /root/workspace/repo-root/tools/stage/$1 $CEN
  echo "  [$2] census sha=$(sha $CEN | cut -c1-16) $(date -u +%H:%M:%S)Z"
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$3" > /tmp/r2_$2.out 2>&1
  echo "  [$2] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r2_$2.out) FAIL=$(grep -ac '^FAIL ' /tmp/r2_$2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/r2_$2.out)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/r2_$2.out | sed 's/^/    F| /' | cut -c1-220
  cp -f /tmp/cen575r2_orig $CEN
  [ "$(sha $CEN)" = "$HEAD_CEN_SHA" ] && echo "  RESTORE_BYTE_IDENTICAL" || echo "  RESTORE_MISMATCH -- STOP"
}
echo "=== P1 join-conjunct:"; runaim census873_p1.dag p1 "$AIM_P1"
echo "=== P2 citation-list:"; runaim census873_p2.dag p2 "$AIM_P2"
echo "=== P3b receipt attribution:"; runaim census873_p3b.dag p3b "$AIM_P3B"
echo "=== R1 renderer prefix collision:"; runaim census873_r1.dag r1 "$AIM_R"
echo "=== R2 renderer drops exit_code/stderr:"; runaim census873_r2.dag r2 "$AIM_R"
git status --porcelain | head -3
echo DONE_575R2 $(date -u +%H:%M:%S)Z
