#!/usr/bin/env bash
# Verify that committed stage0 matches what regenerate-stage0.sh would produce.
#
# This is the CI gate that prevents dual-representation drift:
# - Editing stage0 Rust without updating .dag source → fails
# - Editing .dag source without regenerating stage0 → fails
#
# Uses the same regeneration logic as regenerate-stage0.sh (no duplication).
#
# Usage: ./scripts/check-stage0-freshness.sh
#   Exit 0 = stage0 is fresh (matches regeneration output)
#   Exit 1 = stage0 is stale (diff shown)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CHECK_DIR="$ROOT/.freshness-check"
rm -rf "$CHECK_DIR"

# Regenerate into temp dir using the same script (single implementation)
"$SCRIPT_DIR/regenerate-stage0.sh" --output-dir "$CHECK_DIR"

echo "=== Comparing ==="
DIFF_OUTPUT=$(diff -rq "$CHECK_DIR/src/" "$ROOT/src/v2/stage0/src/" 2>&1 || true)

rm -rf "$CHECK_DIR"

if [ -z "$DIFF_OUTPUT" ]; then
    echo "=== Stage0 is fresh — matches regeneration output. ==="
    exit 0
else
    echo "=== Stage0 is STALE — does not match regeneration output. ==="
    echo ""
    echo "$DIFF_OUTPUT"
    echo ""
    echo "Run ./scripts/regenerate-stage0.sh to update stage0."
    exit 1
fi
