#!/bin/bash
# Scratch (never lands): acceptance lane for fix/concept-route-data-row-parse.
# The main-break control (route rc=1 at 3db8f9e with the exact TypeError) is banked in
# lane D job j965bfcdd. Here: at the FIX head the route must flip rc=1 -> rc=2 and the
# #14559 consumer claim must stay green.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current

echo "=== BUILD gunbc + claim_batch ==="
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1

echo "=== 1. PARSE STANDALONE (host route, both files, at fix) ==="
cp src/v1/stage0/src/bin/unary_prefix_digest.rs /tmp/digest.rs 2>/dev/null || true
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
mkdir -p /tmp/fixfiles && cp src/v2/test/claim/manual/java_grammar_structure.dag src/v2/test/claim/manual/rust_grammar_structure.dag /tmp/fixfiles/
./target/release/unary_prefix_digest /tmp/fixfiles 2>&1 | grep -a "PARSE-FAIL\|ITEMS"

echo "=== 2. REAL ROUTE at fix: whole_tree_extract_smoke (must be rc=2 = returned true) ==="
./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
  --entry src/v2/lens/grounding.dag --function whole_tree_extract_smoke > /tmp/route_fix.log 2>&1
echo "route@fix rc=$?"
grep -a "TypeError\|failed to parse\|returned \`true\`" /tmp/route_fix.log | head -3

echo "=== 3. #14559 consumer claim stays green ==="
./target/release/claim_batch --source-root src --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/lex_token_class_membership_readers.dag --function lex_token_class_membership_readers_hold 2>&1 | tail -3
echo "readers-claim rc=$?"

echo "=== 4. the two grammar claims evaluate ==="
./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
  --claim-run --entry src/v2/test/claim/manual/java_grammar_structure.dag 2>&1 | tail -2
echo "java-claim rc=$?"
./target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src" --source-root "$ROOT/src/v2" \
  --claim-run --entry src/v2/test/claim/manual/rust_grammar_structure.dag 2>&1 | tail -2
echo "rust-claim rc=$?"

echo "=== DONE ==="
