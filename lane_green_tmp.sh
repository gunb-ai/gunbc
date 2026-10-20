#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): regen to fixed point, then GREEN claims, then
# emit the regenerated mirror as a base64 patch (the runner has no push credentials).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== BUILD (pre-regen) ==="
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2

PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 2>&1 | grep -E "changed_paths|convergence_stages|RebuildScopeRefused|error:" | head -8
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet 2>&1 | tail -2
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet 2>&1 | tail -2
  WRITTEN=$(git status --porcelain | wc -l | tr -d ' ')
  echo "round $ROUND dirty_files=$WRITTEN"
  if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
  PREV=$WRITTEN
done
echo "--- mirror diffstat ---"
git diff --stat -- src/v1/stage0/src | tail -8
echo "leftover literal-12 prefix arms (want 0): $(grep -c 'parse_expr_bp(token_stream_advance(tokens.clone(), 1), ctx.clone(), 12)' src/v1/stage0/src/v1_compiler_parse.rs || true)"
echo "find_prefix_binding_power in mirror: $(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs || true)"

echo "=== GREEN (regenerated mirror) ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for pair in \
 "src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold" \
 "src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree" \
 "src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports" \
 "src/v2/test/claim/manual/unary_prefix_v1_shape_probe.dag unary_prefix_v1_shape_probe_reports" \
 "src/v2/test/claim/manual/unary_prefix_v2_shape_probe.dag unary_prefix_v2_shape_probe_reports"; do
  set -- $pair
  echo "--- $1 :: $2 ---"
  ./target/release/claim_batch --source-root src --source-root src/v2 --source-root dag --entry $1 --function $2 > /tmp/green.log 2>&1; echo "exit=$?"
  grep -oE "ARM\[[^]]*\][^\"]*|refused: [^\n]*|FAIL [a-z_]*|PASS [a-z_]*|✓ [a-z_]*" /tmp/green.log | head -16
  grep -aiE "error:" /tmp/green.log | head -6
done

echo "=== PATCH BASE64 (regenerated mirror) ==="
git add -A src/v1/stage0/src
echo "BEGIN-PATCH"
git diff --cached | base64 -w0
echo
echo "END-PATCH"
