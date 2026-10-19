export GUNBC_MEMORY_BUDGET_BYTES=24000000000 GITHUB_ACTIONS=true
cargo build --release -p v1-compiler --bin claim_batch 2>&1|tail -1
./target/release/claim_batch --hermetic --source-root src/v2 --source-root dag --source-root probe --entry probe/p1.dag --functions a1,a2,a3,a4 2>&1 | grep -E "error|probe" | sort -u | cut -c1-250 | head -20
