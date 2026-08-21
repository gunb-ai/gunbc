set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
run() { echo "=== $2 ==="; $B run --source-root dag --source-root src/v2 --entry "$1" --function "$2" 2>&1 | grep -E "returned|cause: |error:" | head -3 || true; }
run dag/test/claim/fleet_converge_cli_witness_test.dag witness_converge_cli_run_srv2_fails_closed_on_knob_frontier
run dag/test/claim/fleet_converge_cli_witness_test.dag witness_converge_cli_srv3_routes_to_slice1_not_realize_frontier
run dag/test/claim/srv3_runner_memory_converge_witness_test.dag witness_provision_routes_through_host_effect_apply
run dag/test/claim/srv3_runner_memory_converge_witness_test.dag witness_srv3_memory_knobs_precede_runner_width_widen
run dag/test/claim/srv3_runner_memory_converge_witness_test.dag witness_readback_verdict_drifts_RED_when_live_mismatch
run dag/test/claim/host_axis_caps_witness_test.dag witness_deploy_memory_cap_author_script_derives_allocation_bytes
run dag/test/claim/host_axis_caps_witness_test.dag witness_cap_upsert_script_quotes_content_through_posix_authority
run dag/test/claim/host_axis_caps_witness_test.dag witness_cap_upsert_script_carries_no_hand_spelled_printf_quoting
run dag/test/claim/host_axis_caps_witness_test.dag witness_cap_author_before_shadow_revert_enforced_in_axis_caps
run dag/test/claim/fleet_converge_privilege_hold_witness_test.dag witness_control_shadow_revert_script_is_revert_not_set_property
run dag/test/claim/fleet_converge_privilege_hold_witness_test.dag witness_shadow_revert_blocked_without_derived_dropins_on_disk
run dag/test/claim/host_converge_slice1_witness_test.dag witness_control_shadow_revert_script_matches_extdeps_argv
echo "=== grep: any surviving reference to the deleted module in code positions ==="
grep -rn 'host_converge_realize\.' dag src --include=*.dag | grep -v '// ' | head -5 || true
