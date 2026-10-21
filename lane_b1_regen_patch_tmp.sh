#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): B1 = regen to fixed point + emit mirror patch.
# (A/RED receipts banked in job je484d35d; B2 = GREEN claim runs at the regenerated head.)
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== 1. BUILD claim_executor + gunbc ==="
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | tail -2

echo "=== 2. REGEN (heal sequence to fixed point) ==="
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 2>&1 | grep -E "changed_paths|convergence_stages|RebuildScopeRefused|error:|✗" | head -8
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet 2>&1 | tail -2
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet 2>&1 | tail -2
  WRITTEN=$(git status --porcelain | wc -l | tr -d ' ')
  echo "round $ROUND dirty_files=$WRITTEN"
  if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
  PREV=$WRITTEN
done
echo "--- mirror diffstat ---"
git diff --stat -- src/v1/stage0/src | tail -8
echo "--- mirror grep witnesses ---"
echo "find_prefix_binding_power: $(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs)"
echo "literal-12 prefix arms remaining: $(grep -n 'min_bp: 12' src/v1/stage0/src/v1_compiler_parse.rs | wc -l)"
echo "prefix_operators in std_syntax mirror: $(grep -c prefix_operators src/v1/stage0/src/std_syntax.rs)"

echo "=== 3. PATCH BASE64 ==="
git add -A src/v1/stage0/src
echo "BEGIN-PATCH"
git diff --cached | base64 -w0
echo
echo "END-PATCH"
