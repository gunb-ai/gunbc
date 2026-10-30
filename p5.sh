set -u
echo HEAD=$(git rev-parse HEAD)
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "=== broker_serve WET (all claims incl. 5 clock-read observers)"
./target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/approval_broker_serve_witness.dag --functions approval_broker_table_selects_the_filing_routes_and_no_bearer_route,approval_broker_table_selects_the_store_receipt_route_as_a_post,approval_broker_table_selects_the_device_routes_with_their_methods,approval_broker_table_carries_no_other_route,approval_broker_filing_route_answers_through_the_real_entry,approval_broker_entry_refuses_a_roadmap_path,the_brokers_own_not_found_is_marked_as_having_reached_the_broker,the_two_processes_do_not_produce_the_same_marker_on_one_route,the_loopback_submit_url_names_the_brokers_one_port,the_loopback_status_url_is_the_submit_collection_plus_the_escalation,exactly_one_process_may_write_at_every_authority_value,a_non_writer_submission_is_refused_before_the_store,the_writer_reaches_the_submission_fold_rather_than_the_gate,a_non_writer_decision_is_refused_before_the_store,a_non_writer_may_still_reach_the_pending_read,a_device_route_json_body_parses_as_json_with_no_marker_in_it,a_device_route_json_marker_is_in_the_header_the_reader_looks_in,the_old_body_appended_marker_does_not_parse_as_json,a_json_body_is_served_untouched_with_exactly_one_marker_header,the_field_name_admits_a_token_and_the_wire_carries_it_verbatim,the_field_name_refuses_a_framing_field_and_a_non_token,every_typed_arm_field_name_is_refused_in_any_case --wet --claim-run > /tmp/cb-bserve-wet.log 2>&1
rc=$?
echo "exit=$rc"
grep -E -A4 "^(FAIL|UNOBSERVED)" /tmp/cb-bserve-wet.log | head -40
echo "pass=$(grep -c ^PASS /tmp/cb-bserve-wet.log) fail=$(grep -c ^FAIL /tmp/cb-bserve-wet.log) unobs=$(grep -c ^UNOBSERVED /tmp/cb-bserve-wet.log)"
[ $rc -ne 0 ] && tail -20 /tmp/cb-bserve-wet.log
echo DONE-P5
