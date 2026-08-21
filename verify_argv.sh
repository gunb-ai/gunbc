set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
run() { echo "=== $2 ==="; $B run --source-root dag --source-root src/v2 --entry "$1" --function "$2" 2>&1 | grep -E "returned|cause: |error:" | head -4 || true; }
W=dag/test/claim/live_deploy/operations_witness_test.dag
for f in witness_dpkg_status_argv_matches_transport_row witness_apt_install_command_matches_transport witness_privileged_apt_install_command witness_systemctl_restart_command_matches_transport witness_systemctl_enable_command_matches_transport witness_systemctl_disable_now_command_matches_transport witness_systemctl_daemon_reload_command_privileged live_deploy_operations_RED_apt_install_wrong_package_golden operations_RED_argv_element_with_a_space_keeps_its_boundary operations_RED_command_is_not_a_bare_space_join; do run $W $f; done
E=dag/test/claim/live_deploy/emit_test.dag
for f in witness_dependency_ensure_is_idempotent witness_apply_script_contains_systemd_and_tailscale witness_tree_sync_restart_diagnoses_on_failure; do run $E $f; done
C=dag/test/claim/host_axis_caps_witness_test.dag
for f in witness_cap_upsert_script_quotes_content_through_posix_authority witness_cap_upsert_script_carries_no_hand_spelled_printf_quoting witness_control_shadow_revert_script_matches_extdeps_argv; do run $C $f; done
