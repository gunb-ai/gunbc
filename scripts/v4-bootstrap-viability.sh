#!/usr/bin/env bash
# scripts/v4-bootstrap-viability.sh
#
# Fail-closed v4 bootstrap gate: v2-compiler full compile over src/v4.
# Success requires exit 0 and the compiler's `compiled: N files emitted, 0 diagnostics` line.
# When V4_BOOTSTRAP_REUSE_LOG is set, this validates the DAG half emitted by the
# upstream rust+dag closure instead of running a second source-closure compile.
#
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin" >&2
  exit 1
fi

out="${V4_BOOTSTRAP_OUT:-/tmp/v4-stage1}"
log="${V4_BOOTSTRAP_LOG:-/tmp/v4-stage1.log}"
reuse_log="${V4_BOOTSTRAP_REUSE_LOG:-}"

if [[ -n "$reuse_log" ]]; then
  if [[ ! -d "$out" ]]; then
    echo "error: reused v4 bootstrap output dir missing: $out" >&2
    exit 1
  fi
  if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$reuse_log" >/dev/null; then
    echo "error: reused v4 bootstrap compile did not emit a clean compiled receipt" >&2
    exit 1
  fi
  mkdir -p "$(dirname "$log")"
  cp "$reuse_log" "$log"
  echo "Bootstrap viability OK — reused upstream single-closure DAG emit."
  exit 0
fi

rm -rf "$out"
mkdir -p "$out"

compile_timeout="${V4_BOOTSTRAP_TIMEOUT_SECS:-}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "$compile_timeout" ]]; then
  # 900s: full src/v4 --target dag on loaded self-hosted runners after M1 emits.
  compile_timeout=900
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

if [[ "$status" -ne 0 ]]; then
  echo "error: v4 bootstrap compile exited $status (log: $log)" >&2
  exit "$status"
fi

if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" >/dev/null; then
  echo "error: v4 bootstrap compile did not emit a clean compiled receipt" >&2
  exit 1
fi

echo "Bootstrap viability OK — v2 compiled all v4 modules."
exit 0
