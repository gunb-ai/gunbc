set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
instrument_head=e239e31e0eaa2dbea09c031d59092d0441e0a50a
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
' > "$instrument_root/instrument.log" 2>&1 && instrument_status=0 || instrument_status=$?
git diff --binary > "$instrument_root/generated.patch"
git status --short > "$instrument_root/generated-status.txt"
printf 'INSTRUMENT_DONE rc=%s root=%s\n' "$instrument_status" "$instrument_root"
exit "$instrument_status"
