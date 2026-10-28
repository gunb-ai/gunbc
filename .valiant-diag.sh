#!/usr/bin/env bash
# DIAGNOSTIC: current std-core errors with source context. The worktree already carries the
# adopted gen-1 candidate mirrors.
set -uo pipefail
cd "$(dirname "$0")"
cargo build --release -p v1-stage0-std-core > /tmp/sc.out 2>&1
echo "ERROR_LINES=$(grep -cE '^error' /tmp/sc.out)"
grep -nE "^error|^ *-->" /tmp/sc.out | sed 's/^[[0-9;]*//g' | head -70
echo "=== EXPECTED/FOUND PAIRS ==="
grep -A3 -E "^error\[" /tmp/sc.out | grep -E "expected|found|-->" | sed 's/^[[0-9;]*//g' | head -60
