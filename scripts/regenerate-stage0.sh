#!/usr/bin/env bash
# Regenerate stage0 using the v2 compiler (self-compile).
#
# FF-9: The compiler resolves imports transitively from source roots.
# No manual file lists. The compiler follows imports and loads only
# what's needed.
#
# Usage:
#   ./scripts/regenerate-stage0.sh              # regenerate in-place
#   ./scripts/regenerate-stage0.sh --output-dir DIR  # regenerate to DIR (no copy, no verify)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STAGE0_DIR="$ROOT/src/v2/stage0"

# Parse args
OUTPUT_ONLY=""
if [ "${1:-}" = "--output-dir" ]; then
    OUTPUT_ONLY="${2:?--output-dir requires a path}"
fi

echo "=== Building stage0 (v2-compiler) ==="
cargo build -p v2-compiler --release

STAGE0_CMD="cargo run -p v2-compiler --release --"

OUTPUT_DIR="${OUTPUT_ONLY:-$ROOT/.regen-output}"
GENERATED_SOURCE_ROOT="$ROOT/.regen-generated-source-root"
rm -rf "$OUTPUT_DIR"
rm -rf "$GENERATED_SOURCE_ROOT"

echo "=== Generating method-template projection source root ==="
cargo run -p v3-compiler --bin emit_method_template_projection -- "$GENERATED_SOURCE_ROOT"

echo "=== Compiling .dag source with v2 compiler (FF-9: import-driven resolution) ==="
$STAGE0_CMD compile \
    --source-root "$ROOT/src/v2" \
    --source-root "$ROOT/dsl" \
    --source-root "$GENERATED_SOURCE_ROOT" \
    --output-dir "$OUTPUT_DIR"

# lib.rs, main.rs, and compiler_tests.rs are now fully emitted by the
# compiler (no hand-maintained files). The emitter produces lib.rs with
# module declarations derived from emitted files, main.rs with FF-9
# import resolution and diagnostic rendering, and compiler_tests.rs
# with the full test harness.

# --output-dir mode: leave output in place, caller handles it
if [ -n "$OUTPUT_ONLY" ]; then
    echo "=== Regeneration output in $OUTPUT_DIR ==="
    exit 0
fi

echo "=== Copying to stage0 ==="
for f in "$OUTPUT_DIR"/src/*.rs; do
    cp "$f" "$STAGE0_DIR/src/$(basename "$f")"
done

echo "=== Verifying stage0 compiles ==="
if ! cargo check -p v2-compiler 2>&1 | tail -5; then
    echo ""
    echo "=== Stage0 has compilation errors. ==="
    cargo check -p v2-compiler 2>&1 | grep "^error" | wc -l
    echo " errors remaining."
    exit 1
fi

echo "=== Pass 2: self-hosting check (regenerated binary re-compiles itself) ==="
cargo build -p v2-compiler --release
PASS2_DIR="$ROOT/.regen-pass2"
rm -rf "$PASS2_DIR"
$STAGE0_CMD compile \
    --source-root "$ROOT/src/v2" \
    --source-root "$ROOT/dsl" \
    --source-root "$GENERATED_SOURCE_ROOT" \
    --output-dir "$PASS2_DIR"

if ! diff -r "$PASS2_DIR" "$OUTPUT_DIR" > /dev/null 2>&1; then
    echo "=== FIXED-POINT FAILURE: pass 1 != pass 2 ==="
    echo ""
    echo "The regenerated binary does not reproduce its own output."
    echo "This means a schema change or emitter change altered the"
    echo "generated code in a way that changes subsequent output."
    echo ""
    echo "To resolve: run ./scripts/regenerate-stage0.sh --output-dir /tmp/pass1"
    echo "then copy to stage0, rebuild, and run regen again until convergence."
    echo ""
    echo "--- Diff (first 30 lines): ---"
    diff -r "$PASS2_DIR" "$OUTPUT_DIR" | head -30 || true
    rm -rf "$PASS2_DIR"
    rm -rf "$OUTPUT_DIR"
    exit 1
else
    echo "=== Fixed point verified (pass 1 == pass 2). ==="
fi

rm -rf "$PASS2_DIR"
rm -rf "$OUTPUT_DIR"
rm -rf "$GENERATED_SOURCE_ROOT"

# Apply workspace fmt so CI fmt-check doesn't see drift. The v2
# compiler's emitter doesn't produce rustfmt-canonical output;
# running fmt after the fixed-point check above keeps regen output
# stable while ensuring committed stage0 is fmt-compliant.
echo "=== Applying cargo fmt --all ==="
cargo fmt --all

echo "=== Done. Stage0 regenerated via v2 self-compile. ==="
