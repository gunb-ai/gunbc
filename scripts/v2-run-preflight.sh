#!/usr/bin/env bash
# scripts/v2-run-preflight.sh
#
# Retired T-24 bridge helper. The structural v4 bootstrap authority remains
# `v2-compiler compile`; the T-22 manual TestClaim corpus gate now runs in CI
# through `scripts/v4-testclaim-corpus-gate.sh` and pins the modeled
# TestClaimRun surface.
#
# Usage: from repo root, with v2-compiler already built:
#   V2_COMPILER=target/release/v2-compiler bash scripts/v2-run-preflight.sh
#
# When the caller has already run `v2-compiler compile` on src/v4, set
# V2_PREFLIGHT_SKIP_COMPILE=1 to skip the redundant full-graph compile.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

if [[ "${V2_PREFLIGHT_SKIP_COMPILE:-0}" == "1" ]]; then
  echo "=== v2-run-preflight: compile skipped (V2_PREFLIGHT_SKIP_COMPILE=1; prior step already compiled src/v4) ==="
else
  bin="${V2_COMPILER:-target/release/v2-compiler}"
  if [[ ! -x "$bin" ]]; then
    echo "error: v2-compiler not found at $bin (build with: cargo build -p v2-compiler --release)" >&2
    exit 1
  fi

  out="${V2_PREFLIGHT_COMPILE_OUT:-/tmp/v4-preflight-compile-out}"
  rm -rf "$out"
  mkdir -p "$out"

  echo "=== v2-run-preflight: compile src/v4 (interim bridge; T-24 = workflow/ci.dag) ==="
  "$bin" compile --source-root src/v4 --output-dir "$out" --target dag
  echo "compile: OK"
fi

echo "::notice::v2 run preflight bridge is retired for CI; T-22 TestClaim corpus gate now pins the modeled TestClaimRun surface. Evidence: src/v4/test/v2_run_preflight/MOVE1_COVERAGE.txt"

exit 0
