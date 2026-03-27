#!/usr/bin/env bash
# Regenerate stage0 using the v2 compiler (self-compile).
#
# This replaces the v1 assemble_stage0 binary. The v2 compiler compiles
# its own .dag source to produce new stage0 Rust code.
#
# Usage: ./scripts/regenerate-stage0.sh
#
# Prerequisites: stage0 must build (cargo build -p v2-compiler --release)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STAGE0_DIR="$ROOT/src/v2/stage0"

echo "=== Building stage0 (v2-compiler) ==="
cargo build -p v2-compiler --release

STAGE0_BIN="$ROOT/target/release/v2-compiler"

# Prepare sources in temp directory (same layout as bootstrap test)
SOURCES_DIR=$(mktemp -d)
trap "rm -rf $SOURCES_DIR" EXIT

# Copy v2 compiler .dag files
for f in "$ROOT"/src/v2/*.dag; do
    cp "$f" "$SOURCES_DIR/"
done

# Copy language extdeps
for lang in rust python go; do
    mkdir -p "$SOURCES_DIR/dsl/extdeps/languages/$lang"
    cp "$ROOT/dsl/extdeps/languages/$lang/emit.dag" "$SOURCES_DIR/dsl/extdeps/languages/$lang/"
done

# Copy std types
mkdir -p "$SOURCES_DIR/dsl/std"
cp "$ROOT/dsl/std/types.dag" "$SOURCES_DIR/dsl/std/"

echo "=== Compiling .dag source with v2 compiler ==="
OUTPUT_DIR=$(mktemp -d)
"$STAGE0_BIN" compile --source-dir "$SOURCES_DIR" --output-dir "$OUTPUT_DIR"

echo "=== Updating stage0 ==="
# Preserve hand-maintained files
cp "$STAGE0_DIR/src/v2_rt.rs" "$OUTPUT_DIR/src/v2_rt.rs" 2>/dev/null || true
cp "$STAGE0_DIR/src/generated_tests.rs" "$OUTPUT_DIR/src/generated_tests.rs" 2>/dev/null || true

# Copy generated source files (preserve Cargo.toml — v2 emitter generates
# a different package name; the committed Cargo.toml is authoritative)
for f in "$OUTPUT_DIR"/src/*.rs; do
    basename=$(basename "$f")
    cp "$f" "$STAGE0_DIR/src/$basename"
done

rm -rf "$OUTPUT_DIR"

echo "=== Verifying stage0 compiles ==="
cargo check -p v2-compiler

echo "=== Done. Stage0 regenerated via v2 self-compile. ==="
echo "Run 'cargo test -p v2-compiler-tests' to verify."
