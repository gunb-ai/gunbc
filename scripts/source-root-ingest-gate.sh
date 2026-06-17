#!/usr/bin/env bash
# scripts/source-root-ingest-gate.sh — host transport for Lane 3a SourceRootIngest gate.
#
# Emits an ephemeral manifest via discover_source_root_ingest, then runs the
# gate-only real_ingest_test witnesses (manifest overlay required).
#
# Usage: source-root-ingest-gate.sh [repo_root]
#   repo_root defaults to git toplevel or pwd.

set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

MANIFEST="$ROOT/target/v2-source-root-ingest-manifest.dag"
MANIFEST_DIR="$(dirname "$MANIFEST")"

cargo run -p v1-compiler --release --bin discover_source_root_ingest -- \
  --source-root src/v2 \
  --scan-dir src/v2/test/fixture/program_assembly \
  --emit-dag-manifest "$MANIFEST"

GUNBC="$ROOT/target/release/gunbc"
if [ ! -x "$GUNBC" ]; then
  cargo build -p v1-compiler --release --bin gunbc
fi

"$GUNBC" run \
  --source-root src/v2 \
  --source-root "$MANIFEST_DIR" \
  --entry src/v2/test/claim/program_assembly/real_ingest_test.dag \
  --function program_assembly_real_ingest_module_roots_parse_holds \
  --claim-run

"$GUNBC" run \
  --source-root src/v2 \
  --source-root "$MANIFEST_DIR" \
  --entry src/v2/test/claim/program_assembly/real_ingest_test.dag \
  --function program_assembly_real_ingest_host_manifest_receipt_holds \
  --claim-run

"$GUNBC" run \
  --source-root src/v2 \
  --source-root "$MANIFEST_DIR" \
  --entry src/v2/test/claim/program_assembly/real_ingest_test.dag \
  --function program_assembly_real_ingest_validate_module_roots_red_on_parsed_roots \
  --claim-run
