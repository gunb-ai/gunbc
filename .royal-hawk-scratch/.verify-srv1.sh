set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
verification_root=/home/briansrls/royal-hawk-241
mkdir -p "$verification_root"
verification_run=$(mktemp -d "$verification_root/verification.XXXXXX")
git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$verification_run/repo"
git -C "$verification_run/repo" fetch origin royal-hawk-241/share-native-context-main
git -C "$verification_run/repo" checkout --detach fabd35f5a2476de5b7bd76b2010394c39c4522ef
mkdir "$verification_run/runner-temp"
printf '%s\n' "$verification_run/verification.log"
cd "$verification_run/repo"
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$verification_run/runner-temp" CTRL_BUILD_MODE=local bash -c '
set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc
failed=0
for witness in shared_context_duplicate_root_refusal_preserves_every_diagnostic_for_each_subject shared_context_malformed_root_refusal_preserves_every_diagnostic_for_each_subject shared_context_resolves_two_subjects_and_preserves_missing_subject_location resolve_dual_imports_do_not_collide_on_header_metadata resolve_with_admission_threads_validate_diagnostics_on_accept; do
  if env PATH="$PATH" RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/admission_fail_closed_test.dag --function "$witness" --claim-run; then
    printf "VERIFY %s PASS\n" "$witness"
  else
    result=$?
    printf "VERIFY %s FAIL rc=%s\n" "$witness" "$result"
    failed=1
  fi
done
for witness in shared_root_validation_refusal_stays_at_prepare_for_each_identity the_requested_declaration_is_selected_and_evaluated a_nested_same_named_graft_edge_is_not_selected an_absent_declaration_is_not_found a_sibling_modules_declaration_is_not_selected; do
  if env PATH="$PATH" RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/native_decl_selection_test.dag --function "$witness" --claim-run; then
    printf "VERIFY %s PASS\n" "$witness"
  else
    result=$?
    printf "VERIFY %s FAIL rc=%s\n" "$witness" "$result"
    failed=1
  fi
done
exit "$failed"
' > "$verification_run/verification.log" 2>&1
printf '%s\n' "$verification_run/verification.log"
