#!/usr/bin/env bash
set -u
export CARGO_TARGET_DIR=/root/gunbc-target
cargo build -p v1-compiler --bin claim_batch --release > /root/build-cb.log 2>&1
echo "BUILD_RC=$?"; tail -1 /root/build-cb.log
BIN=/root/gunbc-target/release/claim_batch
echo "--- start $(date -u +%H:%M:%S)"
echo "--- sa start $(date -u +%H:%M:%S)"
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_slot_actuation_witness.dag --functions a_masked_desired_slot_plans_unmask_enable_start,an_active_slot_is_a_noop,activation_refuses_an_uninstalled_unit_and_a_failed_readback,a_slot_running_a_job_is_masked_and_not_stopped,an_idle_slot_is_masked_then_stopped,never_mask_a_running_unit_leaves_a_busy_slot_untouched_and_stops_an_idle_one_before_masking,mask_while_running_is_the_unchanged_default,a_drained_slot_is_a_noop,an_unread_worker_refuses_and_partial_states_are_not_drained,the_job_process_is_attributed_to_its_own_unit,a_reading_is_joined_by_id_and_a_missing_block_is_unread,every_actuation_argv_crosses_the_ssh_edge,the_cpu_dropin_waits_for_the_slot_population,an_unknown_systemd_word_refuses_and_is_never_planned,a_static_or_linked_unit_refuses_activation,a_deactivating_slot_waits_for_its_stop_and_is_not_a_running_job,a_missing_unit_is_not_installed_not_masked,an_empty_crashlooping_slot_reads_idle_and_drains,a_nonzero_or_unread_task_count_keeps_the_blind_listing_refused,an_attributed_worker_is_a_job_whatever_the_count,an_unattributed_listing_refuses_and_does_not_stop,a_fully_read_local_observation_is_settled_and_carries_the_grounded_main_process,a_zero_main_pid_is_no_main_process_and_an_unread_one_never_settles,a_pid_without_its_grounding_is_ungrounded_not_an_identity,an_unobserved_worker_or_unit_leg_never_settles_the_local_observation,a_withheld_retirement_renders_the_standing_it_carries,a_silent_gate_is_undecided_rather_than_refused_and_never_settles,a_hand_capacity_notice_reports_the_refusal_and_settles > /root/sa.log 2>&1
echo "SA_RC=$?"
grep -c '^PASS' /root/sa.log | sed 's/^/SA_PASS=/'
grep -c '^FAIL' /root/sa.log | sed 's/^/SA_FAIL=/'
grep '^FAIL' /root/sa.log | cut -c1-1000 | head -20
tail -12 /root/sa.log
echo "--- hc start $(date -u +%H:%M:%S)"
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_host_file_converge_witness.dag --functions observe_on_srv1_lists_open_transition_retirees,the_population_is_desired_units_then_loaded_extras,the_desired_population_is_the_two_files_and_the_dropin_is_the_rendered_one > /root/hc.log 2>&1
echo "HC_RC=$?"
grep -c '^PASS' /root/hc.log | sed 's/^/HC_PASS=/'
grep -c '^FAIL' /root/hc.log | sed 's/^/HC_FAIL=/'
grep '^FAIL' /root/hc.log | cut -c1-1000 | head -20
tail -12 /root/hc.log
echo "ENDALL $(date -u +%H:%M:%S)"
