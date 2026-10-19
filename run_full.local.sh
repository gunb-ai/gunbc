set -uo pipefail
export GUNBC_MEMORY_BUDGET_BYTES=24000000000 GITHUB_ACTIONS=true
cargo build --release -p v1-compiler --bin claim_batch 2>&1|tail -1
./target/release/claim_batch --hermetic --source-root src/v2 --source-root dag --entry dag/probe/fullcorpus.dag --functions fullcorpus_resolves > /tmp/full_out.txt 2>&1; grep -E "^(PASS|FAIL)|error:" /tmp/full_out.txt | cut -c1-260
