#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): B2 = regen to fixed point WITH FULL LOGS
# + emit mirror patch. (B1 j45d9581d: regen wrote NOTHING, dirty=4 were untracked debris;
# je484d35d ran the same lines earlier and changed_paths=4 was real. Need unfiltered output.)
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== 0. CLEAN + BASELINE ==="
git clean -x -d --force >/dev/null
git status --porcelain | sed 's/^/dirty-before: /'

echo "=== 1. BUILD claim_executor + gunbc ==="
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | tail -2

echo "=== 2. REGEN ROUND 1 (full log tail 60) ==="
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen1.log 2>&1
echo "rc=$?"
tail -60 /tmp/regen1.log
echo "--- git status after round 1 ---"
git status --porcelain | sed 's/^/dirty: /'
echo "--- diffstat mirrors ---"
git diff --stat -- src/v1/stage0/src | tail -8
echo "witness find_prefix_binding_power: $(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs)"
echo "witness prefix_operators std_syntax: $(grep -c prefix_operators src/v1/stage0/src/std_syntax.rs)"
echo "witness min_bp: $(grep -n 'min_bp' src/v1/stage0/src/v1_compiler_parse.rs | head -5)"

echo "=== 3. PATCH BASE64 (emit even mid-lane; round 2 below only if mirrors changed) ==="
git add -A src/v1/stage0/src
echo "BEGIN-PATCH"
git diff --cached | base64 -w0
echo
echo "END-PATCH"
