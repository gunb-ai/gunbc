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
# Exclude hand-maintained files (not generated, survive regen).
# These are declared in 05_emit_rust.dag via hand_maintained_mods.
DIFF_EXCLUDE="--exclude=v2_interpreter.rs --exclude=cli_run.rs --exclude=rest_transport_facts.rs"
DIFF_OUTPUT=$(diff -rq $DIFF_EXCLUDE "$CHECK_DIR/src/" "$ROOT/src/v2/stage0/src/" 2>&1 || true)

if [ -z "$DIFF_OUTPUT" ]; then
    rm -rf "$CHECK_DIR"
    echo "=== Stage0 is fresh — matches regeneration output. ==="
    exit 0
else
    echo "=== Stage0 is STALE — does not match regeneration output. ==="
    echo ""
    echo "$DIFF_OUTPUT"
    echo ""
    # Show actual content diff for the first differing file
    for f in "$CHECK_DIR"/src/*.rs; do
        fname=$(basename "$f")
        committed="$ROOT/src/v2/stage0/src/$fname"
        if [ -f "$committed" ] && ! diff -q "$f" "$committed" > /dev/null 2>&1; then
            echo "=== Diff for $fname ==="
            diff -u "$committed" "$f" | head -40
            break
        fi
    done
    rm -rf "$CHECK_DIR"
    echo ""
    echo "Run ./scripts/regenerate-stage0.sh to update stage0."
    exit 1
fi
