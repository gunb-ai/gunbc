#!/bin/bash
# Scratch (never lands): F2 = v2 head-labelled shape dumps (Transform heads by atom identity).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
echo "=== V2 SHAPES (head-labelled) ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v2_shape_probe_tmp.dag --function unary_prefix_v2_shape_reports > /tmp/v2shapes2.log 2>&1
echo "v2-shapes rc=$?"
grep -aoE "BODY\[[^]]*\] [^\"]*" /tmp/v2shapes2.log | head -10
tail -3 /tmp/v2shapes2.log | head -c 900
