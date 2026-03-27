#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEMO_DIR="$ROOT/demo"
OUTPUT_DIR="$DEMO_DIR/output"
STAGE0_BIN="$ROOT/target/release/v2-compiler"

# ── Build compiler if needed ─────────────────────────────────────────────
if [[ ! -f "$STAGE0_BIN" ]]; then
  echo "Building v2 compiler (first time takes ~40s)..."
  cargo build -p v2-compiler --release --manifest-path "$ROOT/Cargo.toml" 2>&1
fi

# ── Clean and compile ────────────────────────────────────────────────────
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "=== Compiling .dag -> Python ==="
echo ""
"$STAGE0_BIN" compile \
  --source-dir "$DEMO_DIR/src" \
  --output-dir "$OUTPUT_DIR" \
  --target python

echo ""
echo "=== Generated Python ==="
echo ""
cat "$OUTPUT_DIR/rpg_actuary.py"

echo ""
echo "=== Source .dag file ==="
echo ""
echo "See: demo/src/rpg_actuary.dag"
echo ""
echo "Key idea: you write types + pure functions in .dag,"
echo "and the compiler generates idiomatic Python (or Rust, or Go)."
echo "Types like Int aren't magic -- they're OrderedRing<Word64>,"
echo "composed from algebraic structures over machine words."
