set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
run() { echo "=== $2 ==="; $B run --source-root dag --source-root src/v2 --entry "$1" --function "$2" 2>&1 | grep -E "returned|cause: |error:" | head -3 || true; }
run dag/test/claim/host_axis_caps_witness_test.dag witness_control_shadow_revert_script_matches_extdeps_argv
run dag/test/claim/host_axis_caps_witness_test.dag control_shadow_residue_teardowns_owned_unit
run dag/test/claim/host_converge_slice1_witness_test.dag witness_slice1_knob_is_per_slot_memory_max_on_srv1
run dag/test/claim/host_converge_slice1_witness_test.dag witness_verdict_uses_converge_target_not_string_flatten
run dag/test/claim/fleet_converge_privilege_hold_witness_test.dag witness_operator_roster_not_widened_to_whole_binary_install_or_systemctl
run dag/test/claim/fleet_converge_privilege_hold_witness_test.dag witness_cap_author_before_shadow_revert_enforced_in_slice1
