#!/usr/bin/env bash
# scripts/v2-compiler-closure-nv2-gate.sh — N_v2 substrate path (v2 emit_for_target via interpreter).
#
# Emits scoped ingest manifest + entry admission, then runs claim-run witnesses.
# Layer roots: gunbc.ci_layer_roots.witness_layer_roots via scripts/lib/witness_layer_roots.sh
#   (authority carrier: v2.compiler.self_host.closure_witness_layer_roots).
#
# Usage: v2-compiler-closure-nv2-gate.sh [repo_root]

set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

# shellcheck source=lib/witness_layer_roots.sh
source "$ROOT/scripts/lib/witness_layer_roots.sh"
witness_layer_roots_load "$ROOT"

ENTRY="src/v2/compiler/00_compile.dag"
MANIFEST="$ROOT/target/v2-compiler-closure-ingest-manifest.dag"
MANIFEST_DIR="$(dirname "$MANIFEST")"
GUNBC="$ROOT/target/release/gunbc"

cargo build -p v1-compiler --release --bin discover_source_root_ingest --bin gunbc

cargo run -p v1-compiler --release --bin discover_source_root_ingest -- \
  --source-root src/v2 \
  --entry "$ENTRY" \
  --emit-dag-manifest "$MANIFEST"

run_claim() {
  local fn="$1"
  local -a root_args=()
  local r
  for r in "${WITNESS_LAYER_ROOTS[@]}"; do
    root_args+=(--source-root "${ROOT}/${r}")
  done
  "$GUNBC" run \
    "${root_args[@]}" \
    --source-root "$MANIFEST_DIR" \
    --entry src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag \
    --function "$fn" \
    --claim-run
}

run_claim compiler_closure_scoped_ingest_module_count_ok
run_claim compiler_closure_scoped_ingest_parses
run_claim compiler_closure_v2_emit_from_scoped_ingest_accepts

echo "N_v2 substrate claim-run witnesses: PASS"
