#!/bin/bash
# Scratch: D8 = decode suite at final head + RFM-file resolve check via a full-closure claim.
set -uo pipefail
echo "=== HEAD ==="; git rev-parse HEAD
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|test result|error\[|error:' | head -14 | cut -c1-220
echo "=== RFM FILE RESOLVES (agreement claim, closure covers its modules) ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports > /tmp/c8_agree.log 2>&1
echo "claim@agreement rc=$?"; grep -aE 'unary-prefix|resolve failed' /tmp/c8_agree.log | head -3 | cut -c1-200
echo "=== DONE ==="
