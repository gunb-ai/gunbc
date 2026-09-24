set -u
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -2
G=./target/release/gunbc
r(){ echo "== $2"; timeout 900 $G run --source-root dag --source-root src/v2 --entry $1 --function $2 2>&1 | grep -E "returned|error:|cause:" | head -3; }
B=src/v2/test/claim/body_lowering; L=src/v2/test/claim/long
for f in b1_kind_renamed b1_kind_same_name; do r $B/zz_probe.dag $f; done
./target/release/claim_batch --source-root dag --source-root src/v2 \
  --entry $B/statement_let_bind_test.dag --functions statement_let_then_reference_lowers_holds,statement_let_chain_lowers_holds,unbound_statement_prefix_refuses_holds,single_statement_body_unchanged_holds,let_in_form_unchanged_holds,typed_statement_let_lowers_holds \
  --entry $B/data_initializer_fn_value_test.dag --functions data_record_arrow_lambda_field_is_not_retained_holds,data_record_fn_literal_field_is_not_retained_holds,lambda_in_fn_body_control_holds \
  --entry $L/wave1_gate1_a1_symbol_index_body_producer_witness_test.dag --functions wave1_gate1_a1_ingest_param_domain_parity_witness_holds,wave1_gate1_a1_domain_param_list_walk_witness_holds,wave1_gate1_a1_symbol_index_frontend_parity_witness_holds,wave1_gate1_a1_real_ingest_lowers_witness_holds \
  --entry $L/wave1_gate1_b1_bare_call_body_producer_witness_test.dag --functions wave1_gate1_b1_bare_call_lowers_to_transform_witness_holds,wave1_gate1_b1_bare_call_resolves_end_to_end_witness_holds,wave1_gate1_b1_builtin_call_resolves_end_to_end_witness_holds,wave1_gate1_b1_unbound_call_resolve_rejects_witness_holds,wave1_gate1_b1_symbol_index_frontend_parity_no_regression_witness_holds \
  2>&1 | grep -E "PASS|FAIL|error" | head -40
echo ALLDONE
