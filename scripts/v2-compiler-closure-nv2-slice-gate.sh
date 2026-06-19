#!/usr/bin/env bash
# scripts/v2-compiler-closure-nv2-slice-gate.sh — bounded representative slice for Q1 correctness.
#
# Exercises gap-1/2/3 constructs (caret, comma-optional lists, repeat cons-spine /
# qualified names) plus normalize/infer pipeline modules — without the full 59-module
# interpreter marathon (Q2 scale finding is separate).
#
# Usage: v2-compiler-closure-nv2-slice-gate.sh [repo_root]

set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

MANIFEST="$ROOT/target/v2-compiler-closure-slice-manifest.dag"
MANIFEST_DIR="$(dirname "$MANIFEST")"
GUNBC="$ROOT/target/release/gunbc"

# Representative slice (10 modules from the scoped 00_compile closure):
#   gap-1 caret ^IDENT  → dag.dag, 01_tokenize.dag
#   gap-2 comma-optional lists → dag.dag, grammar.dag, 02_parse.dag
#   gap-3 repeat/cons + qual names → 02_parse, grammar, qualified_name, program_assembly
#   normalize/infer depth → 03_normalize, 04_infer, program_assembly
SLICE_PATHS=(
  "src/v2/compiler/01_tokenize.dag"
  "src/v2/compiler/02_parse.dag"
  "src/v2/compiler/03_normalize.dag"
  "src/v2/compiler/03_name_resolve.dag"
  "src/v2/compiler/04_infer.dag"
  "src/v2/compiler/program_assembly.dag"
  "src/v2/extdeps/languages/dag.dag"
  "src/v2/std/grammar.dag"
  "src/v2/std/qualified_name.dag"
  "src/v2/std/node.dag"
)

cargo build -p v1-compiler --release --bin discover_source_root_ingest --bin gunbc

cargo test -p v1-compiler-tests emit_representative_slice_manifest -- --ignored --nocapture

run_claim() {
  local fn="$1"
  "$GUNBC" run \
    --source-root dsl \
    --source-root src/v2 \
    --source-root "$MANIFEST_DIR" \
    --entry src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag \
    --function "$fn" \
    --claim-run
}

echo "=== N_v2 representative slice (${#SLICE_PATHS[@]} modules) ==="
run_claim compiler_closure_scoped_ingest_module_count_ok
run_claim compiler_closure_scoped_ingest_parses
echo "N_v2 slice claim-run: PASS (compiler_closure_scoped_ingest_parses TRUE on bounded slice)"
