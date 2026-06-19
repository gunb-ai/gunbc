#!/usr/bin/env bash
# scripts/gap4-locate-slice.sh — one-shot gap-4 locate on 7-module slice manifest (DIAGNOSTIC).
#
# Uses nv2 gate entry (full overlay closure) + gap4_probe_first_ingest_reject.
# Authority: scripts/lib/witness_layer_roots.sh

set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

# shellcheck source=lib/witness_layer_roots.sh
source "$ROOT/scripts/lib/witness_layer_roots.sh"
witness_layer_roots_load "$ROOT"

MANIFEST_DIR="${MANIFEST_DIR:-$(mktemp -d /tmp/gap4-slice-manifest-XXXXXX)}"
if [[ -z "${MANIFEST_DIR_PERSIST:-}" ]]; then
  cp "$ROOT/target/v2-compiler-closure-slice-manifest.dag" \
    "$MANIFEST_DIR/host_source_root_ingest_manifest.dag"
fi
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"

cargo build -p v1-compiler --release --bin gunbc >/dev/null
cargo test -p v1-compiler-tests emit_representative_slice_manifest -- --ignored --nocapture >/dev/null

declare -a root_args=()
for r in "${WITNESS_LAYER_ROOTS[@]}"; do
  root_args+=(--source-root "${ROOT}/${r}")
done

GATE="src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag"

echo "=== gap4 slice locate (manifest_dir=$MANIFEST_DIR) ==="
"$GUNBC" run \
  "${root_args[@]}" \
  --source-root "$MANIFEST_DIR" \
  --entry "$GATE" \
  --function gap4_probe_first_ingest_reject

echo "=== file_path ==="
"$GUNBC" run \
  "${root_args[@]}" \
  --source-root "$MANIFEST_DIR" \
  --entry "$GATE" \
  --function gap4_probe_found_file_path \
  --claim-run

echo "=== byte_start ==="
"$GUNBC" run \
  "${root_args[@]}" \
  --source-root "$MANIFEST_DIR" \
  --entry "$GATE" \
  --function gap4_probe_found_byte_start \
  --claim-run
