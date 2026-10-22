#!/bin/bash
# Scratch (never lands): F3 = both-parser shape dumps incl. cast child order and dot node kind.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
for probe in v1 v2; do
  echo "=== ${probe} SHAPES ==="
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry src/v2/test/claim/manual/unary_prefix_${probe}_shape_probe_tmp.dag \
    --function unary_prefix_${probe}_shape_reports > /tmp/${probe}shapes3.log 2>&1
  echo "${probe} rc=$?"
  grep -aoE "BODY\[[^]]*\] [^\"]*" /tmp/${probe}shapes3.log | head -30
  tail -3 /tmp/${probe}shapes3.log | head -c 500
done
