#!/usr/bin/env bash
set -u
export CARGO_TARGET_DIR=/root/gunbc-target
cargo build -p v1-compiler --bin claim_batch --release > /root/build-cb.log 2>&1
echo "BUILD_RC=$?"; tail -1 /root/build-cb.log
BIN=/root/gunbc-target/release/claim_batch
echo "--- start $(date -u +%H:%M:%S)"
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_replacement_readback_witness.dag --functions retirement_is_withheld_while_the_replacement_has_never_run_a_job,retirement_is_authorized_once_every_replacement_has_registered_and_completed_a_job,a_kept_slot_the_host_has_never_reported_live_holds_the_retirement_too,one_proven_replacement_of_two_does_not_authorize_the_drain,a_collected_run_that_concluded_failure_is_not_a_completed_job,a_completed_job_on_another_slot_proves_nothing_for_this_one,a_replacement_that_never_appeared_in_the_roster_is_not_registered,a_replacement_registered_offline_is_not_a_slot_that_needs_provisioning,an_unreadable_roster_read_keeps_the_retiree_running_and_says_so,a_partial_roster_read_is_never_read_as_not_registered,a_hand_started_slot_is_refused_even_when_every_replacement_is_proven,the_refusal_names_the_unit_and_says_it_is_not_this_plans_to_take,a_still_desired_unit_is_never_drained_even_when_a_retiree_list_claims_it,a_drain_candidate_nothing_declares_is_refused,a_pure_shrink_needs_no_replacement_readback,an_unresolvable_target_population_withholds_rather_than_reading_empty,a_fabric_member_is_authorized_without_a_replacement_readback,the_hand_notice_says_the_retirement_is_now_safe_once_the_replacements_are_proven,the_hand_notice_names_the_replacement_and_the_reason_while_it_is_unproven,every_withheld_cause_names_the_replacement_it_is_waiting_on,the_authority_is_per_unit_and_not_per_host,the_observe_pass_notice_never_announces_a_retirement_it_did_not_read,the_drain_leg_drains_no_declared_retiree_before_the_kept_slots_are_proven,the_drain_leg_drains_the_retirees_once_the_kept_slots_have_run_a_job,the_drain_leg_holds_while_a_slot_the_host_is_bringing_up_has_never_run_a_job,the_drain_leg_waits_when_only_one_kept_slot_has_run_a_job,the_hand_capacity_route_reports_the_unit_and_never_drains_it,the_wire_names_the_slots_that_cover_a_retirement > /root/rb.log 2>&1 &
P1=$!
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_slot_actuation_witness.dag --functions a_masked_desired_slot_plans_unmask_enable_start,an_active_slot_is_a_noop,activation_refuses_an_uninstalled_unit_and_a_failed_readback,a_slot_running_a_job_is_masked_and_not_stopped,an_idle_slot_is_masked_then_stopped,never_mask_a_running_unit_leaves_a_busy_slot_untouched_and_stops_an_idle_one_before_masking,mask_while_running_is_the_unchanged_default,a_drained_slot_is_a_noop,an_unread_worker_refuses_and_partial_states_are_not_drained,the_job_process_is_attributed_to_its_own_unit,a_reading_is_joined_by_id_and_a_missing_block_is_unread,every_actuation_argv_crosses_the_ssh_edge,the_cpu_dropin_waits_for_the_slot_population,an_unknown_systemd_word_refuses_and_is_never_planned,a_static_or_linked_unit_refuses_activation,a_deactivating_slot_waits_for_its_stop_and_is_not_a_running_job,a_missing_unit_is_not_installed_not_masked,an_empty_crashlooping_slot_reads_idle_and_drains,a_nonzero_or_unread_task_count_keeps_the_blind_listing_refused,an_attributed_worker_is_a_job_whatever_the_count,an_unattributed_listing_refuses_and_does_not_stop,a_fully_read_local_observation_is_settled_and_carries_the_grounded_main_process,a_zero_main_pid_is_no_main_process_and_an_unread_one_never_settles,a_pid_without_its_grounding_is_ungrounded_not_an_identity,an_unobserved_worker_or_unit_leg_never_settles_the_local_observation,a_withheld_retirement_renders_the_standing_it_carries,a_silent_gate_is_undecided_rather_than_refused_and_never_settles,a_hand_capacity_notice_reports_the_refusal_and_settles > /root/sa.log 2>&1 &
P2=$!
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_host_file_converge_witness.dag --functions observe_on_srv1_lists_open_transition_retirees,the_population_is_desired_units_then_loaded_extras,the_desired_population_is_the_two_files_and_the_dropin_is_the_rendered_one > /root/hc.log 2>&1 &
P3=$!
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_replacement_readback_control.dag --functions positive_control_a_claim_that_is_false_comes_back_red > /root/ctl.log 2>&1 &
P4=$!
wait $P1; echo "RB_RC=$?"
wait $P2; echo "SA_RC=$?"
wait $P3; echo "HC_RC=$?"
wait $P4; echo "CTL_RC=$?"
for tag in rb sa hc ctl; do
  echo "== $tag"
  grep -c '^PASS' /root/$tag.log | sed "s/^/$(echo $tag | tr 'a-z' 'A-Z')_PASS=/"
  grep -c '^FAIL' /root/$tag.log | sed "s/^/$(echo $tag | tr 'a-z' 'A-Z')_FAIL=/"
  grep '^FAIL' /root/$tag.log | cut -c1-1000 | head -20
done
echo "ENDALL $(date -u +%H:%M:%S)"
