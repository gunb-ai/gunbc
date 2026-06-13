#!/usr/bin/env bash
# Uniform `.dag`-driven CI Bool-witness gate transport.
#
# Row authority lives in the gate model (`--gate-entry`); this script invokes the
# `ci-claim-gate` host, which evaluates `--rows-fn` via the v2 interpreter and
# runs green + optional perturb passes. Replaces per-gate awk/grep roster projection.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
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

exec "$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/workflow/lens_ci_gate.dag \
  --rows-fn lens_ci_claim_run_rows_tsv \
  --notice-title "v4 lens CI" \
  "${perturb[@]}"
