#!/usr/bin/env bash
# Regenerate stage0 using the v2 compiler (self-compile).
#
# This replaces the v1 assemble_stage0 binary. The v2 compiler compiles
# its own .dag source to produce new stage0 Rust code.
#
# The v2 emitter generates different module names than the v1 emitter
# (e.g., v2_compiler_compile.rs vs compile.rs). This script renames
# files and patches use statements for workspace compatibility.
#
# Usage: ./scripts/regenerate-stage0.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STAGE0_DIR="$ROOT/src/v2/stage0"

echo "=== Building stage0 (v2-compiler) ==="
cargo build -p v2-compiler --release

STAGE0_BIN="$ROOT/target/release/v2-compiler"

# Prepare sources in temp directory (same layout as bootstrap test)
SOURCES_DIR=$(mktemp -d)
OUTPUT_DIR=$(mktemp -d)
trap "rm -rf $SOURCES_DIR $OUTPUT_DIR" EXIT

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
"$STAGE0_BIN" compile --source-dir "$SOURCES_DIR" --output-dir "$OUTPUT_DIR"

echo "=== Renaming modules for workspace compatibility ==="

# Rename v2 emitter module files to match workspace convention.
# v2 emitter prefixes with full module path; workspace uses short names.
cd "$OUTPUT_DIR/src"
for f in v2_compiler_*.rs; do
    newname="${f#v2_compiler_}"
    mv "$f" "$newname"
done
[ -f v2_std_core.rs ] && mv v2_std_core.rs v2_core.rs
[ -f extdeps_languages_rust_emit.rs ] && mv extdeps_languages_rust_emit.rs rust_emit.rs
[ -f extdeps_languages_python_emit.rs ] && mv extdeps_languages_python_emit.rs python_emit.rs
[ -f extdeps_languages_go_emit.rs ] && mv extdeps_languages_go_emit.rs go_emit.rs
cd "$ROOT"

# Patch use/mod statements: replace v2 emitter module paths with short names
sed -i '' \
    -e 's/v2_compiler_//g' \
    -e 's/v2_std_core/v2_core/g' \
    -e 's/extdeps_languages_rust_emit/rust_emit/g' \
    -e 's/extdeps_languages_python_emit/python_emit/g' \
    -e 's/extdeps_languages_go_emit/go_emit/g' \
    "$OUTPUT_DIR"/src/*.rs

# Keep std_types.rs — v2 emitter generates it from dsl/std/types.dag

# Preserve hand-maintained files from committed stage0
cp "$STAGE0_DIR/src/lib.rs" "$OUTPUT_DIR/src/lib.rs"
cp "$STAGE0_DIR/src/v2_rt.rs" "$OUTPUT_DIR/src/v2_rt.rs" 2>/dev/null || true
cp "$STAGE0_DIR/src/generated_tests.rs" "$OUTPUT_DIR/src/generated_tests.rs" 2>/dev/null || true

echo "=== Copying to stage0 ==="
for f in "$OUTPUT_DIR"/src/*.rs; do
    cp "$f" "$STAGE0_DIR/src/$(basename "$f")"
done

echo "=== Verifying stage0 compiles ==="
if cargo check -p v2-compiler 2>&1 | tail -5; then
    echo "=== Done. Stage0 regenerated via v2 self-compile. ==="
else
    echo ""
    echo "=== Stage0 has compilation errors. ==="
    cargo check -p v2-compiler 2>&1 | grep "^error" | wc -l
    echo " errors remaining."
    exit 1
fi
