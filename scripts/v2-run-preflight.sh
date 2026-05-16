#!/usr/bin/env bash
# scripts/v2-run-preflight.sh
#
# INTERIM (T-24 bridge): Move 1 / Move 3 pre-flight helper. The structural
# authority for CI remains `v2-compiler compile` (see .github/workflows/ci.yml
# v4 job). v2 `run` over src/v4 is **not** a green gate today — see
# src/v4/test/v2_run_preflight/MOVE1_COVERAGE.txt (deferred until T-22).
#
# Usage: from repo root, with v2-compiler already built:
#   V2_COMPILER=target/release/v2-compiler bash scripts/v2-run-preflight.sh

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

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

echo "::notice::v2 \`run\` TestClaim execution gate is DEFERRED until T-22 (v4 eval). Evidence: src/v4/test/v2_run_preflight/MOVE1_COVERAGE.txt"

exit 0
