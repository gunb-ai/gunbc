#!/bin/bash
set -uo pipefail
echo "=== HEAD ==="; git rev-parse HEAD | cut -c1-11; git branch --show-current
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/parse/data_initializer_layout_parse.dag --function a_continuation_line_data_initializer_parses_on_both_routes > /tmp/cl.log 2>&1
echo "claim rc=$?"
grep -aE 'data-initializer layout refused|resolve failed|error' /tmp/cl.log | head -6 | cut -c1-420
echo "=== DONE ==="
