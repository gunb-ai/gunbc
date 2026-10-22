#!/bin/bash
# Scratch (never lands): E5 = E4's dump, grepped to the probe bodies + surrounding lines.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
cargo build --release -p v1-compiler --bin unary_prefix_emit_dump_tmp 2>&1 | tail -1
./target/release/unary_prefix_emit_dump_tmp | grep -aE "BEGIN|END|fn probe_case|->|let |return|-|\!" | grep -avE "^use |^pub |^fn [a-z_]+\(|assert|panic|format!" | head -60
