set +e
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
mk(){ echo "[$(date +%H:%M:%S)] $*"; }
R=/root/workspace/probe-root
rm -rf ; mkdir -p ; cd  || { echo NO_DIR; exit 0; }
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
E=dag/test/claim/serving_caller_backlog_witness.dag
CSV=w_persist_before_offer_and_duplicate_persist_is_one_waiting_record,w_waiting_survives_ready_supply_drop_and_controller_restart_without_nudge,w_stale_replica_generation_cannot_commit_and_completion_is_not_rerun,w_probe_success_does_not_reset_request_failures_past_the_bound,w_cancellation_is_terminal,w_modeled_rows_are_durable_and_the_serving_route_is_not_and_has_no_router_queue,w_caller_backlog_bases_name_live_declarations,w_encoded_waiting_roundtrips_recovery_history,w_bound_deficit_decode_refuses_a_negative_count,w_tab_in_work_id_roundtrips_through_json,w_tab_in_reservation_roundtrips_through_json,w_note_recovery_unknown_work_is_refused,w_duplicate_records_refuse_persist_and_do_not_revive,w_an_arbitrary_module_cannot_rebind_a_live_predecessor,w_foreign_module_cannot_publish_a_derived_phase,w_foreign_module_cannot_publish_an_initial_record,w_typed_waiting_persist_is_the_admitted_publication_path,w_seat_lease_stale_generation_cannot_commit,w_work_key_is_stable_across_process_attempts_and_ordinals_differ,w_rebind_after_cancel_refuses_and_never_revives,w_commit_under_the_previous_reservation_after_a_rebind_refuses,w_old_holder_refuses_against_the_current_grant_read_from_the_chain,w_an_unread_grant_or_clock_cannot_authorize_completion,w_completion_time_is_observed_not_copied_from_acquisition,w_a_released_or_lapsed_grant_is_not_current,w_harness_caller_turn_refuses_empty_node_root_and_negative_ordinal,w_a_rebind_naming_a_stale_predecessor_refuses_with_the_held_reservation,w_a_seat_is_retired_only_once_acquired_and_no_longer_current,w_the_worker_entry_refuses_a_malformed_belt_turn_before_offering_work,w_a_durable_turn_placed_on_the_operator_endpoint_is_refused_before_work_is_offered,w_a_rebind_naming_the_right_reservation_in_the_wrong_partition_refuses,w_an_unplaced_record_is_claimed_only_by_a_rebind_expecting_no_seat,w_a_commit_against_an_unplaced_record_refuses_at_the_fence
probe(){
  SHA=$1
  mk "PROBE_SHA=$SHA fetch-begin"
  timeout 600 git fetch -q --depth=1 origin $SHA || { echo "FETCH_FAIL $SHA"; return; }
  git checkout -q FETCH_HEAD
  echo "TREE=$(git rev-parse HEAD) EXPECT=$SHA"
  [ "$(git rev-parse HEAD)" = "$SHA" ] || { echo "HEAD_MISMATCH $SHA"; return; }
  cargo build -p v1-compiler --release --features text_lookup_work_counter,interp_test_witness,test_hooks --bin claim_batch 2>&1 | tail -2
  mk "BUILD_RC=$? SHA=$SHA"
  BIN=target/release/claim_batch
  [ -x "$BIN" ] || BIN=$CARGO_TARGET_DIR/release/claim_batch
  timeout 900 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$CSV" > /tmp/probe_${SHA:0:8}.log 2>&1
  echo "SHA=${SHA:0:12} RC=$? PASS=$(grep -acE '^PASS ' /tmp/probe_${SHA:0:8}.log) FAIL=$(grep -acE '^FAIL ' /tmp/probe_${SHA:0:8}.log)"
  grep -aE 'type mismatch|expected .* got|refus|resolve' /tmp/probe_${SHA:0:8}.log | head -3 | cut -c1-220
  mk "PROBE_DONE $SHA"
}
probe 11d6c58f7525a9d100e3c5b76c81fbfeed378444
probe 0b8f72199889847de1ca883bb0071bd72c018dcd
probe 2d79743e2498bda7a95e5b6253ca2e7fd73aa6a7
probe 242ac98b5feff52f11fe56a7732cffb9897f1813
probe 4f4c3e30cafe34a0e98b6eb0b0abcc380d2699b1
mk ALL_DONE_F8
