set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
W=dag/test/claim/fleet_converge_plan_witness_test.dag
for f in locked_apply_script_serializes_generation_under_flock locked_apply_inner_program_is_quoted_not_spliced locked_apply_guard_does_not_coerce_an_unreadable_store_to_zero locked_apply_guard_compares_the_modelled_prior_generation locked_apply_stale_diagnostic_keeps_observed_value_in_one_word; do echo "=== $f ==="; $B run --source-root dag --source-root src/v2 --entry $W --function $f 2>&1 | grep -E "returned|cause: |error:" | head -3 || true; done
