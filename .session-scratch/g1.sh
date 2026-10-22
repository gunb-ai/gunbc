set +e
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
mk(){ echo "[$(date +%H:%M:%S)] $*"; }
PIN=f6a9994bf12e9bb7da45d7ccbbe9923718377237
R=/root/workspace/s1-root
mk "START host=$(hostname)"
rm -rf $R; mkdir -p $R
git init -q $R && cd $R || { echo NO_DIR; exit 0; }
git remote add origin https://github.com/gunb-ai/gunbc.git
mk "BEGIN fetch"
timeout 600 git fetch --depth=1 origin $PIN 2>&1 | tail -1
mk "FETCH_RC=$?"
git checkout -q FETCH_HEAD
[ "$(git rev-parse HEAD)" = "$PIN" ] || { echo HEAD_IS=$(git rev-parse HEAD); exit 0; }
mk "PIN_OK du=$(du -sh . | cut -f1)"
echo 'aW1wb3J0IHN5cwpDPSJzcmMvdjIvd29ya2Zsb3cvY2xhaW1fY292ZXJhZ2VfY2Vuc3VzX3MxX2NhcnJpZXJzLmRhZyIKUz0ic3JjL3YyL3dvcmtmbG93L2NsYWltX2NvdmVyYWdlX2NlbnN1c19zMS5kYWciCkw9InNyYy92Mi93b3JrZmxvdy9jbGFpbV9jb3ZlcmFnZV9jZW5zdXNfczFfbGl2ZS5kYWciCk1VVFMgPSB7CiAjIEE6IHJldmVydCB0aGUgZnVuY3Rpb24tZ3JhaW4gYWRhcHRlciAodGhlIHBhcmVudCdzIGZpbmRpbmcgKGEpKQogIkEiOiAoQywgJyAgICBjb3Zlcl9maWxlX2ZuX3JvdyhjYXJyaWVyOiBjYXJyaWVyLCBwYXRoOiB3aXRuZXNzLmVudHJ5LCBmdW5jdGlvbjogd2l0bmVzcy5mdW5jdGlvbiknLAogICAgICAgICAgJyAgICBjb3Zlcl9maWxlX3JvdyhjYXJyaWVyOiBjYXJyaWVyLCBwYXRoOiB3aXRuZXNzLmVudHJ5KScpLAogIyBCOiB0aGUgZmluZGluZ3MgaWdub3JlIGxhYmVsIGRlZmVjdHMgKHRoZSBwYXJlbnQncyBmaW5kaW5nIChiKSwgZmlyc3QgaGFsZikKICJCIjogKFMsICcgICAgICAgICsgY291bnQobGFiZWxfZGVmZWN0cyknLCAnICAgICAgICArIDAnKSwKICMgQzogY3JlZGl0IHRoZSBmaXJzdCBvZiBtYW55ICh0aGUgcGFyZW50J3MgZmluZGluZyAoYiksIHNlY29uZCBoYWxmKQogIkMiOiAoTCwgJyAgICBDb25zIHsgaGVhZDogXywgdGFpbDogXyB9ID0+IFtdJywKICAgICAgICAgICcgICAgQ29ucyB7IGhlYWQ6IGhlYWQsIHRhaWw6IF8gfSA9PiBwcm9kdWNlcl9uYW1lZF9pZGVudGl0aWVzKHByb2R1Y2VyOiBoZWFkLnByb2R1Y2VyLCBpbmRleDogaW5kZXgpJyksCn0KbGFiZWw9c3lzLmFyZ3ZbMV0KcGF0aCxvbGQsbmV3PU1VVFNbbGFiZWxdCnM9b3BlbihwYXRoKS5yZWFkKCkKbj1zLmNvdW50KG9sZCkKaWYgbiE9MToKICAgIHByaW50KCJNVVRBVElPTl9OT1RfQVBQTElFRCAlcyBjb3VudD0lZCIgJSAobGFiZWwsbikpOyBzeXMuZXhpdCgzKQpvcGVuKHBhdGgsInciKS53cml0ZShzLnJlcGxhY2Uob2xkLG5ldykpOyBwcmludCgiTVVUQVRFRCAlcyIgJSBsYWJlbCkK' | base64 -d > /tmp/mutD1.py
mkdir -p $CARGO_TARGET_DIR
cargo build --bin claim_batch 2>&1 | tail -2
mk "BUILD_RC=$?"
BIN=target/debug/claim_batch
[ -x "$BIN" ] || BIN=$CARGO_TARGET_DIR/debug/claim_batch
ls -la "$BIN" || { echo NO_BIN; exit 0; }
C=src/v2/workflow/claim_coverage_census_s1_carriers.dag
S=src/v2/workflow/claim_coverage_census_s1.dag
L=src/v2/workflow/claim_coverage_census_s1_live.dag
W=src/v2/test/claim/claim_coverage_census_s1_witness.dag
mkdir -p /tmp/orig
for f in $C $S $L; do cp $f /tmp/orig/$f; sha256sum $f | awk '{print $1}' > /tmp/orig/$f.sha; done
WIT="a_fixture_whose_claims_are_all_covered_holds_and_exits_zero,removing_a_covered_claim_from_its_drop_makes_the_census_name_it,a_floor_expected_red_row_never_covers_a_claim,the_three_prefix_fill_debt_rows_are_stale_enrolled_by_name,the_prefix_row_is_stale_while_its_renamed_successor_is_grounded,a_fully_resolving_carrier_prints_no_stale_row_and_holds,a_claim_the_universe_holds_is_never_stale_enrolled_when_rows_are_filtered,a_drop_member_that_is_neither_path_nor_identity_is_an_ungrounded_refusal,each_grain_is_grounded_by_the_population_it_joins_and_never_by_spelling,an_identity_whose_module_name_contains_dag_is_grounded_not_refused,an_untyped_drop_member_grounds_in_either_population_and_neither_by_spelling,an_exact_file_member_selecting_no_discovered_claim_is_refused,a_file_grain_cover_grants_exactly_that_files_discovered_identities,a_frontier_pattern_grounding_in_no_file_is_refused_and_one_grounding_covers,a_producer_naming_an_undiscovered_claim_executes_nothing,a_producer_exposing_no_claim_set_leaves_the_universe_uncovered_and_is_a_finding,an_unread_input_exits_two_and_reports_no_claim_population,an_unbound_required_job_label_refuses_rather_than_printing_beside_held,an_ambiguous_required_job_label_refuses_and_names_the_label,the_same_census_holds_when_no_required_job_label_is_defective,a_witness_entry_naming_one_function_becomes_a_file_and_function_member,a_witness_entry_with_no_function_declares_the_whole_file_and_nothing_more,admitting_one_function_grounds_that_claim_and_leaves_its_sibling_uncovered,a_unique_binding_credits_the_identity_its_producer_names,a_label_with_two_bindings_credits_nothing_but_a_unique_one_does,a_binding_whose_entry_the_index_does_not_hold_credits_nothing,the_real_fill_debt_carrier_yields_its_named_active_member_as_an_identity_cover_row,a_real_rung_drop_member_arrives_under_its_own_drop_carrier,a_live_function_resolves_and_names_nothing_for_a_producer_that_names_nothing,a_definite_finding_beside_an_unread_input_exits_one_not_two,an_unread_input_with_no_finding_exits_two_and_names_the_cause,an_s1_finding_alone_exits_one,two_findings_keep_both_reasons_and_stay_at_one,two_held_standings_exit_zero,the_exit_code_is_derived_from_the_composed_standing"
RUN(){
  $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $W --functions $2 > $1 2>&1
  echo "RC=$? PASS=$(grep -acE '^PASS ' $1) FAIL=$(grep -acE '^FAIL ' $1)"
  grep -aE '^FAIL ' $1
}
echo "===== GREEN WITNESS 35 ====="
RUN /tmp/v.log "$WIT"
mk "GREEN done"
echo "===== RED A widen-grain-back (expect FAIL exactly: naming_one + admitting_one) ====="
python3 /tmp/mutD1.py A
RUN /tmp/wa.log "$WIT"
cp /tmp/orig/$C $C; echo "FINAL_C=$(sha256sum $C | awk '{print $1}') EXPECT_C=$(cat /tmp/orig/$C.sha)"
mk "RED A done"
echo "===== RED B findings-ignore-defects (expect FAIL exactly: unbound + ambiguous) ====="
python3 /tmp/mutD1.py B
RUN /tmp/wb.log "$WIT"
cp /tmp/orig/$S $S; echo "FINAL_S=$(sha256sum $S | awk '{print $1}') EXPECT_S=$(cat /tmp/orig/$S.sha)"
mk "RED B done"
echo "===== RED C credit-first-of-many (expect FAIL exactly: two_bindings) ====="
python3 /tmp/mutD1.py C
RUN /tmp/wc.log "$WIT"
cp /tmp/orig/$L $L; echo "FINAL_L=$(sha256sum $L | awk '{print $1}') EXPECT_L=$(cat /tmp/orig/$L.sha)"
mk "RED C done"
echo "===== BACK ====="
RUN /tmp/bk.log "$WIT"
mk "ALL_DONE_F1"
