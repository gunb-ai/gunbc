#!/usr/bin/env bash
# scripts/v2-compiler-closure-nv2-gate.sh — N_v2 substrate path (v2 emit_for_target via interpreter).
#
# Emits scoped 53-module ingest manifest + entry admission, then runs claim-run witnesses.
#
# Usage: v2-compiler-closure-nv2-gate.sh [repo_root]

set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

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
  "$GUNBC" run \
    --source-root src/v2 \
    --source-root "$MANIFEST_DIR" \
    --entry src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag \
    --function "$fn" \
    --claim-run
}

run_claim compiler_closure_scoped_ingest_module_count_ok
run_claim compiler_closure_scoped_ingest_parses
run_claim compiler_closure_v2_emit_from_scoped_ingest_accepts

echo "N_v2 substrate claim-run witnesses: PASS"
