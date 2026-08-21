set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
run() { echo "=== $2 ==="; $B run --source-root dag --source-root src/v2 --entry "$1" --function "$2" 2>&1 | grep -E "returned|cause: |error:" | head -3 || true; }
W=dag/test/claim/fleet_converge_plan_witness_test.dag
run $W generation_observation_separates_absent_from_unreadable
run $W generation_admission_admits_absent_as_first_and_refuses_unreadable
run $W locked_apply_script_serializes_generation_under_flock
run $W locked_apply_inner_program_is_quoted_not_spliced
run $W locked_apply_guard_does_not_coerce_an_unreadable_store_to_zero
run $W locked_apply_guard_compares_the_modelled_prior_generation
run $W apply_shell_carries_dissolve_on_marker_not_host_gate
run $W parse_nonneg_generation_refuses_corrupt_and_negative
echo "=== EMITTED locked apply script ==="
$B run --source-root dag --source-root src/v2 --entry dag/gunbc/fleet_converge_plan.dag --function fleet_converge_plan_locked_apply_probe 2>&1 | head -6 || true
