set -u
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
B=target/release/claim_batch
git rev-parse --short HEAD
F=the_recapture_of_a_committed_attempt_tree_is_minted_by_real_execution,a_clean_worktree_whose_head_is_not_the_recorded_submission_refuses_by_real_execution,a_capture_claiming_this_head_stored_under_another_basename_refuses_by_real_execution,a_standalone_repository_at_the_attempt_worktree_is_refused_by_the_git_trust_wall,a_commit_step_settled_at_another_head_leaves_the_capture_owed_not_absent,no_commit_step_record_leaves_a_misaddressed_capture_owed_not_absent
echo BELT_WET; timeout 1500 $B --wet --source-root dag --source-root src/v2 --entry dag/test/claim/roadmap/roadmap_belt_recapture_wet_witness.dag --functions $F 2>&1 | grep -aE "^(PASS|FAIL|UNOBS)" | cut -c1-200; echo rc=${PIPESTATUS[0]}
echo MUTATION_RENAME_ALL
sed -i 's/a_capture_claiming_this_head_stored_under_another_basename_refuses_by_real_execution/a_capture_claiming_this_head_RENAMED/g' src/v2/workflow/local_repo_wet_terminal.dag
git status --short | grep -v run2 | head
for f in each_enrolled_wet_red_is_a_scheduled_claim_the_schedule_still_expects_to_hold the_dry_lane_hold_still_wins_and_the_wet_enrolment_is_load_bearing; do timeout 600 $B --source-root dag --source-root src/v2 --entry dag/test/claim/floor/floor_expected_red_coherence_witness.dag --functions $f 2>&1 | grep -aE "^(PASS|FAIL|UNOBS)" | cut -c1-200; done
git checkout -- src/v2/workflow/local_repo_wet_terminal.dag
echo ENDRUN
