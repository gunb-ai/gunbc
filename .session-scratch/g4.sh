set +e
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
mk(){ echo "[$(date +%H:%M:%S)] $*"; }
PIN=11d6c58f7525a9d100e3c5b76c81fbfeed378444
R=/root/workspace/main-root
mk "START host=$(hostname)"
rm -rf $R; mkdir -p $R
git init -q $R && cd $R || { echo NO_DIR; exit 0; }
git remote add origin https://github.com/gunb-ai/gunbc.git
mk "BEGIN fetch"
timeout 600 git fetch --depth=1 origin $PIN 2>&1 | tail -1
mk "FETCH_RC=$?"
git checkout -q FETCH_HEAD
[ "$(git rev-parse HEAD)" = "$PIN" ] || { echo HEAD_IS=$(git rev-parse HEAD); exit 0; }
mk "MAIN_OK=$(git rev-parse HEAD)"
cargo build -p v1-compiler --release --features text_lookup_work_counter,interp_test_witness,test_hooks --bin claim_batch 2>&1 | tail -3
mk "BUILD_RC=$?"
BIN=target/release/claim_batch
[ -x "$BIN" ] || BIN=$CARGO_TARGET_DIR/release/claim_batch
ls -la "$BIN" || { echo NO_BIN; exit 0; }
E=dag/test/claim/ledger_row_coherence.dag
echo "===== LEDGER PER-CLAIM 18 x --function (400s cap) ====="
echo "----- CLAIM a_fully_rendered_ledger_is_coherent -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_fully_rendered_ledger_is_coherent > /tmp/one_a_fully_rendered_ledger_is_coherent.log 2>&1
echo "RC=$? LOGNAME=one_a_fully_rendered_ledger_is_coherent"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_fully_rendered_ledger_is_coherent.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_fully_rendered_ledger_is_coherent.log | head -3 | cut -c1-200
echo "----- CLAIM a_withheld_row_is_named_as_missing -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_withheld_row_is_named_as_missing > /tmp/one_a_withheld_row_is_named_as_missing.log 2>&1
echo "RC=$? LOGNAME=one_a_withheld_row_is_named_as_missing"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_withheld_row_is_named_as_missing.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_withheld_row_is_named_as_missing.log | head -3 | cut -c1-200
echo "----- CLAIM the_rung_drop_ledger_is_covered_without_a_projection_index -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function the_rung_drop_ledger_is_covered_without_a_projection_index > /tmp/one_the_rung_drop_ledger_is_covered_without_a_projection_index.log 2>&1
echo "RC=$? LOGNAME=one_the_rung_drop_ledger_is_covered_without_a_projection_index"
grep -aE '^(PASS|FAIL) ' /tmp/one_the_rung_drop_ledger_is_covered_without_a_projection_index.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_the_rung_drop_ledger_is_covered_without_a_projection_index.log | head -3 | cut -c1-200
echo "----- CLAIM a_non_ledger_artifact_cannot_report_coherent -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_non_ledger_artifact_cannot_report_coherent > /tmp/one_a_non_ledger_artifact_cannot_report_coherent.log 2>&1
echo "RC=$? LOGNAME=one_a_non_ledger_artifact_cannot_report_coherent"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_non_ledger_artifact_cannot_report_coherent.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_non_ledger_artifact_cannot_report_coherent.log | head -3 | cut -c1-200
echo "----- CLAIM the_gate_diagnostic_names_the_missing_identity -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function the_gate_diagnostic_names_the_missing_identity > /tmp/one_the_gate_diagnostic_names_the_missing_identity.log 2>&1
echo "RC=$? LOGNAME=one_the_gate_diagnostic_names_the_missing_identity"
grep -aE '^(PASS|FAIL) ' /tmp/one_the_gate_diagnostic_names_the_missing_identity.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_the_gate_diagnostic_names_the_missing_identity.log | head -3 | cut -c1-200
echo "----- CLAIM a_difference_that_drops_no_row_is_not_reported_incoherent -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_difference_that_drops_no_row_is_not_reported_incoherent > /tmp/one_a_difference_that_drops_no_row_is_not_reported_incoherent.log 2>&1
echo "RC=$? LOGNAME=one_a_difference_that_drops_no_row_is_not_reported_incoherent"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_difference_that_drops_no_row_is_not_reported_incoherent.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_difference_that_drops_no_row_is_not_reported_incoherent.log | head -3 | cut -c1-200
echo "----- CLAIM a_rendered_declaration_parses_back_to_the_same_identities -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_rendered_declaration_parses_back_to_the_same_identities > /tmp/one_a_rendered_declaration_parses_back_to_the_same_identities.log 2>&1
echo "RC=$? LOGNAME=one_a_rendered_declaration_parses_back_to_the_same_identities"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_rendered_declaration_parses_back_to_the_same_identities.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_rendered_declaration_parses_back_to_the_same_identities.log | head -3 | cut -c1-200
echo "----- CLAIM a_coherent_artifact_declares_no_repair -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_coherent_artifact_declares_no_repair > /tmp/one_a_coherent_artifact_declares_no_repair.log 2>&1
echo "RC=$? LOGNAME=one_a_coherent_artifact_declares_no_repair"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_coherent_artifact_declares_no_repair.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_coherent_artifact_declares_no_repair.log | head -3 | cut -c1-200
echo "----- CLAIM a_declaration_matching_the_inference_agrees -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_declaration_matching_the_inference_agrees > /tmp/one_a_declaration_matching_the_inference_agrees.log 2>&1
echo "RC=$? LOGNAME=one_a_declaration_matching_the_inference_agrees"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_declaration_matching_the_inference_agrees.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_declaration_matching_the_inference_agrees.log | head -3 | cut -c1-200
echo "----- CLAIM a_repair_that_under_declares_is_caught_in_the_inferred_direction -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_repair_that_under_declares_is_caught_in_the_inferred_direction > /tmp/one_a_repair_that_under_declares_is_caught_in_the_inferred_direction.log 2>&1
echo "RC=$? LOGNAME=one_a_repair_that_under_declares_is_caught_in_the_inferred_direction"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_repair_that_under_declares_is_caught_in_the_inferred_direction.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_repair_that_under_declares_is_caught_in_the_inferred_direction.log | head -3 | cut -c1-200
echo "----- CLAIM a_declaration_that_over_claims_is_caught_in_the_declared_direction -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_declaration_that_over_claims_is_caught_in_the_declared_direction > /tmp/one_a_declaration_that_over_claims_is_caught_in_the_declared_direction.log 2>&1
echo "RC=$? LOGNAME=one_a_declaration_that_over_claims_is_caught_in_the_declared_direction"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_declaration_that_over_claims_is_caught_in_the_declared_direction.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_declaration_that_over_claims_is_caught_in_the_declared_direction.log | head -3 | cut -c1-200
echo "----- CLAIM a_trailer_missing_its_identity_yields_no_declaration -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_trailer_missing_its_identity_yields_no_declaration > /tmp/one_a_trailer_missing_its_identity_yields_no_declaration.log 2>&1
echo "RC=$? LOGNAME=one_a_trailer_missing_its_identity_yields_no_declaration"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_trailer_missing_its_identity_yields_no_declaration.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_trailer_missing_its_identity_yields_no_declaration.log | head -3 | cut -c1-200
echo "----- CLAIM a_coherent_artifact_still_declares_that_it_was_judged -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_coherent_artifact_still_declares_that_it_was_judged > /tmp/one_a_coherent_artifact_still_declares_that_it_was_judged.log 2>&1
echo "RC=$? LOGNAME=one_a_coherent_artifact_still_declares_that_it_was_judged"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_coherent_artifact_still_declares_that_it_was_judged.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_coherent_artifact_still_declares_that_it_was_judged.log | head -3 | cut -c1-200
echo "----- CLAIM an_absent_declaration_judges_nothing -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function an_absent_declaration_judges_nothing > /tmp/one_an_absent_declaration_judges_nothing.log 2>&1
echo "RC=$? LOGNAME=one_an_absent_declaration_judges_nothing"
grep -aE '^(PASS|FAIL) ' /tmp/one_an_absent_declaration_judges_nothing.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_an_absent_declaration_judges_nothing.log | head -3 | cut -c1-200
echo "----- CLAIM a_repairing_declaration_reports_both_numerator_and_denominator -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function a_repairing_declaration_reports_both_numerator_and_denominator > /tmp/one_a_repairing_declaration_reports_both_numerator_and_denominator.log 2>&1
echo "RC=$? LOGNAME=one_a_repairing_declaration_reports_both_numerator_and_denominator"
grep -aE '^(PASS|FAIL) ' /tmp/one_a_repairing_declaration_reports_both_numerator_and_denominator.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_a_repairing_declaration_reports_both_numerator_and_denominator.log | head -3 | cut -c1-200
echo "----- CLAIM an_unjudgeable_ledger_refuses_rather_than_skipping -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function an_unjudgeable_ledger_refuses_rather_than_skipping > /tmp/one_an_unjudgeable_ledger_refuses_rather_than_skipping.log 2>&1
echo "RC=$? LOGNAME=one_an_unjudgeable_ledger_refuses_rather_than_skipping"
grep -aE '^(PASS|FAIL) ' /tmp/one_an_unjudgeable_ledger_refuses_rather_than_skipping.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_an_unjudgeable_ledger_refuses_rather_than_skipping.log | head -3 | cut -c1-200
echo "----- CLAIM one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one > /tmp/one_one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one.log 2>&1
echo "RC=$? LOGNAME=one_one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one"
grep -aE '^(PASS|FAIL) ' /tmp/one_one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one.log | head -3 | cut -c1-200
echo "----- CLAIM an_unjudgeable_non_ledger_artifact_does_not_stop_the_line -----"
timeout 400 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function an_unjudgeable_non_ledger_artifact_does_not_stop_the_line > /tmp/one_an_unjudgeable_non_ledger_artifact_does_not_stop_the_line.log 2>&1
echo "RC=$? LOGNAME=one_an_unjudgeable_non_ledger_artifact_does_not_stop_the_line"
grep -aE '^(PASS|FAIL) ' /tmp/one_an_unjudgeable_non_ledger_artifact_does_not_stop_the_line.log | cut -c1-150
grep -aE '\[resolve\]|panic|error|Error' /tmp/one_an_unjudgeable_non_ledger_artifact_does_not_stop_the_line.log | head -3 | cut -c1-200
mk "ALL_DONE_F4"
