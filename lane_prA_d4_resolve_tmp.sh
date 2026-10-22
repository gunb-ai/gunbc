#!/bin/bash
# Scratch: D4 = full refusal text for the agreement claim's resolve failure.
set -uo pipefail
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports \
  > /tmp/ag4.log 2>&1
echo "rc=$?"
grep -a -v 'floor-phase\|floor-drain\|memory-cgroup' /tmp/ag4.log | head -50 | cut -c1-300
