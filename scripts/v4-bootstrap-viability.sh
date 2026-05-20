#!/usr/bin/env bash
# scripts/v4-bootstrap-viability.sh
#
# Fail-closed CI gate: v2-compiler must complete compile over src/v4 with exit 0
# and emit the "compiled: N files emitted, 0 diagnostics" receipt line.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/v2-compiler}"
if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin" >&2
  exit 1
fi

out="${V4_BOOTSTRAP_OUT:-/tmp/v4-stage1}"
log="${V4_BOOTSTRAP_LOG:-/tmp/v4-stage1.log}"
rm -rf "$out"
mkdir -p "$out"

set +e
"$bin" compile --source-root src/v4 --output-dir "$out" --target dag 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
set -e

if [[ "$status" -ne 0 ]]; then
  exit "$status"
fi

if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" >/dev/null; then
  echo "error: v4 bootstrap compile did not emit a clean compiled receipt" >&2
  exit 1
fi

echo "Bootstrap viability OK — v2 compiled all v4 modules."
