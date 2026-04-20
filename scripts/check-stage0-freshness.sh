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

# `cargo fmt` needs hand-maintained modules that lib.rs references (same as
# bootstrap `copy_stage0_support_modules` / regenerate in-place + workspace fmt).
STAGE0_SRC="$ROOT/src/v2/stage0/src"
for name in v2_interpreter.rs cli_run.rs rest_transport_facts.rs; do
  if [ -f "$STAGE0_SRC/$name" ]; then
    cp "$STAGE0_SRC/$name" "$CHECK_DIR/src/$name"
  fi
done

# Full in-place regen ends with `cargo fmt --all` on the workspace; `--output-dir`
# mode exits before that, so normalize the temp crate the same way before diff.
cargo fmt --all --manifest-path "$CHECK_DIR/Cargo.toml"

echo "=== Comparing ==="
# Exclude hand-maintained files (not generated, survive regen).
# These are declared in 05_emit_rust.dag via hand_maintained_mods.
#
# rest_transport_facts.rs: bounded substrate seed (INVARIANTS.md). Dissolution:
# remove when REST op facts come from the resolved graph / single declaration
# export so tests do not need a parallel AST walk (see module header in-file).
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
