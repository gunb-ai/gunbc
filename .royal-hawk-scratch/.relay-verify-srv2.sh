set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
relay_head=a2bd14c40938aaa500977905dceed6eb2ab247bd
relay_root=$(mktemp -d /home/briansrls/royal-hawk-241/relay.XXXXXX)
printf '%s\n' "$relay_root/relay.log"
git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$relay_root/repo"
git -C "$relay_root/repo" fetch origin "$relay_head"
git -C "$relay_root/repo" checkout --detach "$relay_head"
mkdir "$relay_root/runner-temp"
cd "$relay_root/repo"
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$relay_root/runner-temp" GITHUB_SHA="$relay_head" CTRL_BUILD_MODE=local bash -c '
set -euo pipefail
cargo build --release --bin claim_executor
./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2
cargo test --release -p v1-compiler-tests --lib native_driver_cost
cargo build --release -p v1-compiler --bin gunbc
env RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/claim.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/native_driver_cost_partition_witness_test.dag --function exclusive_sum_is_every_row_the_law_folds --claim-run
printf "EXCLUSIVE_SUM_PASS\n"
env RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/docs.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/docs_projection_gate.dag --function main
printf "DOCS_PROJECTION_PASS\n"
' > "$relay_root/relay.log" 2>&1 && relay_status=0 || relay_status=$?
git diff --binary > "$relay_root/generated.patch"
git status --short > "$relay_root/generated-status.txt"
printf 'RELAY_VERIFY_DONE rc=%s root=%s\n' "$relay_status" "$relay_root"
exit "$relay_status"
