#!/bin/bash
# Scratch (never lands): F1 = v1 + v2 shape dumps for the cast/dot/pipe agreement arms.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
echo "=== V1 SHAPES (report channel) ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_shape_probe_tmp.dag --function unary_prefix_v1_shape_reports > /tmp/v1shapes.log 2>&1
echo "v1-shapes rc=$?"
grep -aoE "BODY\[[^]]*\] [^\"]*" /tmp/v1shapes.log | head -10
tail -3 /tmp/v1shapes.log | head -c 600
exit 0
echo "=== V2 SHAPES (report channel) ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v2_shape_probe_tmp.dag --function unary_prefix_v2_shape_reports > /tmp/v2shapes.log 2>&1
echo "v2-shapes rc=$?"
grep -aoE "BODY\[[^]]*\] [^\"]*" /tmp/v2shapes.log | head -10
tail -3 /tmp/v2shapes.log | head -c 600
