set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
instrument_head=a6f5b7b0f6503795b8b1b9d390b348af7e2f1626
instrument_root=$(mktemp -d /home/briansrls/royal-hawk-241/instrument.XXXXXX)
printf '%s\n' "$instrument_root/instrument.log"
git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$instrument_root/repo"
git -C "$instrument_root/repo" fetch origin "$instrument_head"
git -C "$instrument_root/repo" checkout --detach "$instrument_head"
mkdir "$instrument_root/runner-temp"
cd "$instrument_root/repo"
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$instrument_root/runner-temp" GITHUB_SHA="$instrument_head" CTRL_BUILD_MODE=local bash -c '
set -euo pipefail
# Match required_regen_host::seed_cargo_build exactly; package selection can change feature unification.
cargo build --release --bin claim_executor
./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2
cargo test --release -p v1-compiler-tests --lib native_driver_cost
cargo build --release --bin gunbc
scope_failed=0
for witness in native_context_identity_changes_with_one_byte_at_same_path native_context_identity_rejects_substituted_placeholder_producer native_context_identity_repeats_for_same_read_population; do
  if RUNNER_TEMP=$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX") ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/native_context_identity_test.dag --function "$witness" --claim-run; then
    echo "SCOPE_VERIFY $witness PASS"
  else
    echo "SCOPE_VERIFY $witness FAIL"
    scope_failed=1
  fi
done
test "$scope_failed" -eq 0
' > "$instrument_root/instrument.log" 2>&1 && instrument_status=0 || instrument_status=$?
git diff --binary > "$instrument_root/generated.patch"
git status --short > "$instrument_root/generated-status.txt"
printf 'INSTRUMENT_DONE rc=%s root=%s\n' "$instrument_status" "$instrument_root"
exit "$instrument_status"
