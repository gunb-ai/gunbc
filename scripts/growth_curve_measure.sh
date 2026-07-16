#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cargo build --release -p v1-compiler --bin gunbc >/dev/null 2>&1
GUNBC="$PWD/target/release/gunbc"

run_point() {
  local label="$1"
  local entry="$2"
  local outdir="/tmp/gunbc-growth-${label}"
  rm -rf "$outdir"
  echo "=== POINT $label entry=$entry ==="
  /usr/bin/time -f "wall_s=%e" env GUNBC_GLOBAL_BARE_Q2_BISECT=all \
    "$GUNBC" compile --target dag --output-dir "$outdir" "$entry" 2>&1 \
    | rg "global-bare-q2-bisect|resolved [0-9]+ sources|modules to reconcile|error:" || true
  echo
}

export GUNBC_GLOBAL_BARE_Q2_BISECT=all

# 8 std homonym modules (single-module closures)
run_point "std-algebra" "src/v2/std/algebra.dag"
run_point "std-node" "src/v2/std/node.dag"
run_point "std-integer" "src/v2/std/integer.dag"

# Growing importer closures
run_point "v2-infer" "src/v2/compiler/04_infer.dag"
run_point "v2-compile" "src/v2/compiler/compile.dag"
run_point "ci-spec" "dag/gunbc/ci_spec.dag"
