#!/usr/bin/env bash
# Uniform `.dag`-driven layering-imports CI Bool-witness gate transport.
#
# Row authority lives in the gate model (`--gate-entry`); host enumerates import
# facts; `ci-claim-gate` evaluates `--rows-fn` via the v2 interpreter and runs
# green + optional perturb passes. Replaces scripts/check_v4_layering_imports.py.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
scan_bin="${LAYERING_IMPORTS_SCAN:-target/release/layering_imports_scan}"
discover_sh="$root/scripts/v4-layering-imports-discover.sh"
perturb=()

case "${1:-}" in
  --perturb-check) perturb=(--perturb-check) ;;
  "") ;;
  *)
    echo "usage: $0 [--perturb-check]" >&2
    exit 2
    ;;
esac

if [[ ! -x "$bin" ]]; then
  echo "error: ci-claim-gate not found at $bin (build: cargo build -p ci_claim_gate --release)" >&2
  exit 2
fi

if [[ ! -x "$scan_bin" ]]; then
  echo "error: layering_imports_scan not found at $scan_bin (build: cargo build -p layering_imports_scan --release)" >&2
  exit 2
fi

manifest="$("$discover_sh")"
manifest_dir="$(dirname "$manifest")"

exec "$bin" \
  --source-root src/v4 \
  --source-root "$manifest_dir" \
  --gate-entry src/v4/workflow/layering_imports_gate.dag \
  --rows-fn layering_imports_claim_run_rows_tsv \
  --notice-title "layering imports" \
  "${perturb[@]}"
