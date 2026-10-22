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
echo "===== LEDGER BATCH 18 (3000s cap) ====="
timeout 3000 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "a_fully_rendered_ledger_is_coherent,a_withheld_row_is_named_as_missing,the_rung_drop_ledger_is_covered_without_a_projection_index,a_non_ledger_artifact_cannot_report_coherent,the_gate_diagnostic_names_the_missing_identity,a_difference_that_drops_no_row_is_not_reported_incoherent,a_rendered_declaration_parses_back_to_the_same_identities,a_coherent_artifact_declares_no_repair,a_declaration_matching_the_inference_agrees,a_repair_that_under_declares_is_caught_in_the_inferred_direction,a_declaration_that_over_claims_is_caught_in_the_declared_direction,a_trailer_missing_its_identity_yields_no_declaration,a_coherent_artifact_still_declares_that_it_was_judged,an_absent_declaration_judges_nothing,a_repairing_declaration_reports_both_numerator_and_denominator,an_unjudgeable_ledger_refuses_rather_than_skipping,one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one,an_unjudgeable_non_ledger_artifact_does_not_stop_the_line" > /tmp/led_batch.log 2>&1
echo "BATCH_RC=$?"
grep -acE '^PASS ' /tmp/led_batch.log | sed 's/^/PASSCOUNT=/'
grep -acE '^FAIL ' /tmp/led_batch.log | sed 's/^/FAILCOUNT=/'
grep -aE '^(PASS|FAIL) ' /tmp/led_batch.log | cut -c1-150
echo "--- diagnostics ---"
grep -aE '\[resolve\]|panic|error\[|Error:' /tmp/led_batch.log | head -8 | cut -c1-220
mk "ALL_DONE_F5"
