#!/bin/bash
# Scratch (never lands): parity follow-up + the manager's SKIPPED-1 justification, one dispatch.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current

echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin unary_prefix_skip_justifier_tmp --bin claim_batch 2>&1 | tail -2

echo "=== 1. SKIPPED-1 JUSTIFICATION + host-route parity (bin) ==="
./target/release/unary_prefix_skip_justifier_tmp src/v1/stage0/tests/fixtures/fact_cardinality_split_brace.dag 2>&1 | grep -aE "TOKEN CENSUS|token |PREFIX-TOKEN-COUNT|HOST-PARSE|HOST-PARITY|:1:|expected"

echo "=== 2. fixture consumers that REQUIRE the malformed shape (grep receipts) ==="
grep -n "include_str!(\"../tests/fixtures/fact_cardinality_split_brace.dag\")" src/v1/stage0/src/cli_run.rs | sed 's/^/consumer: /'
grep -rn "fact_cardinality_split_brace" src/v2/test/claim/namespace_step0_canary_v2_witness.dag | sed 's/^/consumer: /'

echo "=== 3. v2-ROUTE parity (claim_batch report channel) ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/data_decl_layout_parity_probe.dag --function data_decl_layout_parity_reports > /tmp/parity.log 2>&1
echo "parity rc=$?"
grep -aoE "V2-ROUTE joined: [^\"]*" /tmp/parity.log | head -2
tail -2 /tmp/parity.log | head -c 800
echo "=== DONE ==="
