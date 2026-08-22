set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
E=dag/test/claim/live_deploy/emit_test.dag
for f in witness_apply_script_contains_systemd_and_tailscale witness_tree_sync_restart_diagnoses_on_failure witness_apply_script_wires_memory_cap_deploy_consumer; do echo "=== $f ==="; $B run --source-root dag --source-root src/v2 --entry $E --function $f 2>&1 | grep -E "returned|cause: |error:" | head -3 || true; done
