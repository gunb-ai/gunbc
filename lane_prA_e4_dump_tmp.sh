#!/bin/bash
# Scratch (never lands): prA lane E4 = OBSERVE the emitted Rust for the four unary-prefix
# discriminator bodies (E3 executed: bang poles true, both minus poles false => the minus
# substrings are calibrated wrong; guessing again costs a 12-min battery run).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }

echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin unary_prefix_emit_dump_tmp 2>&1 | tail -3
[ -x ./target/release/unary_prefix_emit_dump_tmp ] || { echo "FATAL: bin missing"; exit 4; }
./target/release/unary_prefix_emit_dump_tmp
