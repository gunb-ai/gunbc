set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
verification_root=/home/briansrls/royal-hawk-241
mkdir -p "$verification_root"
verification_run=$(mktemp -d "$verification_root/verification.XXXXXX")
git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$verification_run/repo"
git -C "$verification_run/repo" fetch origin royal-hawk-241/keyed-module-roots
git -C "$verification_run/repo" checkout --detach e9423d7e0180d3a2c0ea3f99535f88107056d555
mkdir "$verification_run/runner-temp"
printf '%s\n' "$verification_run/verification.log"
cd "$verification_run/repo"
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$verification_run/runner-temp" CTRL_BUILD_MODE=local bash -c '
set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc
failed=0
for witness in validated_subject_index_matches_general_lookup keyed_validation_preserves_general_ambiguity_diagnostic keyed_validation_equal_duplicate_precedes_malformed keyed_validation_unequal_duplicate_precedes_malformed keyed_validation_malformed_precedes_duplicate validate_module_roots_malformed_root_is_rejected resolve_module_not_found_preserves_missing_module_identity resolve_duplicate_module_roots_is_rejected; do
  if env PATH="$PATH" RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/admission_fail_closed_test.dag --function "$witness" --claim-run; then
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
