#!/usr/bin/env bash
set -u
export CARGO_TARGET_DIR=/root/gunbc-target
cargo build -p v1-compiler --bin claim_batch --release > /root/build-cb.log 2>&1
echo "BUILD_RC=$?"; tail -1 /root/build-cb.log
BIN=/root/gunbc-target/release/claim_batch
echo "--- start $(date -u +%H:%M:%S)"
echo "--- rb start $(date -u +%H:%M:%S)"
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_replacement_readback_witness.dag --functions retirement_is_withheld_while_the_replacement_has_never_run_a_job,retirement_is_authorized_once_every_replacement_has_registered_and_completed_a_job,a_kept_slot_the_host_has_never_reported_live_holds_the_retirement_too,one_proven_replacement_of_two_does_not_authorize_the_drain,a_collected_run_that_concluded_failure_is_not_a_completed_job,a_completed_job_on_another_slot_proves_nothing_for_this_one,a_replacement_that_never_appeared_in_the_roster_is_not_registered,a_replacement_registered_offline_is_not_a_slot_that_needs_provisioning,an_unreadable_roster_read_keeps_the_retiree_running_and_says_so,a_partial_roster_read_is_never_read_as_not_registered,a_hand_started_slot_is_refused_even_when_every_replacement_is_proven,the_refusal_names_the_unit_and_says_it_is_not_this_plans_to_take,a_still_desired_unit_is_never_drained_even_when_a_retiree_list_claims_it,a_drain_candidate_nothing_declares_is_refused,a_pure_shrink_needs_no_replacement_readback,an_unresolvable_target_population_withholds_rather_than_reading_empty,a_fabric_member_is_authorized_without_a_replacement_readback,the_hand_notice_says_the_retirement_is_now_safe_once_the_replacements_are_proven,the_hand_notice_names_the_replacement_and_the_reason_while_it_is_unproven,every_withheld_cause_names_the_replacement_it_is_waiting_on,the_authority_is_per_unit_and_not_per_host,the_observe_pass_notice_never_announces_a_retirement_it_did_not_read,the_drain_leg_drains_no_declared_retiree_before_the_kept_slots_are_proven,the_drain_leg_drains_the_retirees_once_the_kept_slots_have_run_a_job,the_drain_leg_holds_while_a_slot_the_host_is_bringing_up_has_never_run_a_job,the_drain_leg_waits_when_only_one_kept_slot_has_run_a_job,the_hand_capacity_route_reports_the_unit_and_never_drains_it,the_wire_names_the_slots_that_cover_a_retirement > /root/rb.log 2>&1
echo "RB_RC=$?"
grep -c '^PASS' /root/rb.log | sed 's/^/RB_PASS=/'
grep -c '^FAIL' /root/rb.log | sed 's/^/RB_FAIL=/'
grep '^FAIL' /root/rb.log | cut -c1-1000 | head -20
tail -12 /root/rb.log
echo "--- ctl start $(date -u +%H:%M:%S)"
$BIN --source-root dag --source-root src/v2 --entry dag/test/claim/runner/runner_replacement_readback_control.dag --functions positive_control_a_claim_that_is_false_comes_back_red > /root/ctl.log 2>&1
echo "CTL_RC=$?"
grep -c '^PASS' /root/ctl.log | sed 's/^/CTL_PASS=/'
grep -c '^FAIL' /root/ctl.log | sed 's/^/CTL_FAIL=/'
grep '^FAIL' /root/ctl.log | cut -c1-1000 | head -20
tail -12 /root/ctl.log
echo "ENDALL $(date -u +%H:%M:%S)"
