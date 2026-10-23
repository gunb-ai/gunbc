set -u
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
F=$(grep -o "^test fn cns_[a-z_]*" src/v2/test/claim/cast_nominal_to_string_probe.dag | sed 's/test fn //' | paste -sd,)
timeout 2400 target/release/claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/cast_nominal_to_string_probe.dag --functions $F --eval-budget-ms 1200000 2>&1 | grep -E "^(PASS|FAIL)"
