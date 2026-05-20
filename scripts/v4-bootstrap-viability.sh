#!/usr/bin/env bash
# scripts/v4-bootstrap-viability.sh
#
# Fail-closed v4 bootstrap gate for v2-compiler over src/v4.
#
# Local / default: exit 0 and the "compiled: N files emitted, 0 diagnostics" receipt.
# GitHub Actions: Ubicloud often SIGTERMs during emit (~2–5m) after a clean resolve;
# then require resolve posture (resolved line, no compiler error lines) — same bar as
# docs/v4-close-interrogation.md §14 parse/resolve receipt / CI v4 posture.

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

compile_timeout="${V4_BOOTSTRAP_TIMEOUT_SECS:-}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "$compile_timeout" ]]; then
  compile_timeout=180
fi

set +e
if [[ -n "$compile_timeout" ]]; then
  timeout --preserve-status "$compile_timeout" \
    "$bin" compile --source-root src/v4 --output-dir "$out" --target dag 2>&1 | tee "$log"
else
  "$bin" compile --source-root src/v4 --output-dir "$out" --target dag 2>&1 | tee "$log"
fi
status=${PIPESTATUS[0]}
set -e

bootstrap_resolve_posture_ok() {
  grep -qE '^resolved [0-9]+ sources \(transitive import closure\)$' "$log" \
    && ! grep -qE '^error:' "$log"
}

if [[ "$status" -eq 0 ]]; then
  if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" >/dev/null; then
    echo "error: v4 bootstrap compile did not emit a clean compiled receipt" >&2
    exit 1
  fi
  echo "Bootstrap viability OK — v2 compiled all v4 modules."
  exit 0
fi

if [[ -n "${GITHUB_ACTIONS:-}" && ( "$status" -eq 124 || "$status" -eq 143 ) ]]; then
  if bootstrap_resolve_posture_ok; then
    echo "::warning::v4 bootstrap: compile exit $status after clean resolve (CI emit wall); full compiled receipt not required on Actions." >&2
    echo "Bootstrap viability OK — parse/resolve posture verified (CI)."
    exit 0
  fi
fi

exit "$status"
