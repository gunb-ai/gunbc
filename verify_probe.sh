set -uo pipefail
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2 || true
B=./target/release/gunbc
for f in witness_cap_upsert_script_quotes_content_through_posix_authority \
         witness_cap_upsert_script_does_not_emit_raw_apostrophe_content \
         witness_cap_upsert_script_carries_no_hand_spelled_printf_quoting \
         witness_runner_memory_cap_author_script_derived \
         witness_cap_author_before_shadow_revert_enforced_in_axis_caps \
         witness_reversed_deploy_script_fails_author_before_shadow_revert_gate \
         witness_padded_wrong_order_script_fails_author_before_shadow_revert_gate \
         witness_control_shadow_revert_script_matches_extdeps_argv \
         control_shadow_residue_teardowns_owned_unit; do
  echo "=== $f ==="
  $B run --source-root dag --source-root src/v2 --entry dag/test/claim/host_axis_caps_witness_test.dag --function $f 2>&1 | grep -E "returned|cause: |error" | head -2 || true
done
echo "=== emit_test memory cap consumer ==="
$B run --source-root dag --source-root src/v2 --entry dag/test/claim/live_deploy/emit_test.dag --function witness_apply_script_wires_memory_cap_deploy_consumer 2>&1 | grep -E "returned|cause: |error" | head -2 || true
echo "=== EMITTED converge author script ==="
$B run --source-root dag --source-root src/v2 --entry dag/gunbc/host_axis_caps.dag --function runner_memory_cap_author_script 2>&1 | head -8 || true
echo "=== EMITTED deploy author script ==="
$B run --source-root dag --source-root src/v2 --entry dag/gunbc/host_axis_caps.dag --function deploy_runner_memory_cap_author_script 2>&1 | head -8 || true
