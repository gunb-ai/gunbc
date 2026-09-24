set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
G=./target/release/gunbc
r(){ echo "== $1 $2"; timeout 900 $G run --source-root dag --source-root src/v2 --entry $1 --function $2 2>&1 | tail -4; }
B=src/v2/test/claim/body_lowering; L=src/v2/test/claim/long
r $B/zz_probe.dag p1; r $B/zz_probe.dag p2; r $B/zz_probe.dag p3
for f in ingest_param_domain_parity domain_param_list_walk symbol_index_frontend_parity real_ingest_lowers; do r $L/wave1_gate1_a1_symbol_index_body_producer_witness_test.dag wave1_gate1_a1_${f}_witness_holds; done
for f in bare_call_lowers_to_transform bare_call_resolves_end_to_end builtin_call_resolves_end_to_end unbound_call_resolve_rejects symbol_index_frontend_parity_no_regression; do r $L/wave1_gate1_b1_bare_call_body_producer_witness_test.dag wave1_gate1_b1_${f}_witness_holds; done
