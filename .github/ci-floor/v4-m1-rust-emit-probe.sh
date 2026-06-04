#!/usr/bin/env bash
# .github/ci-floor/v4-m1-rust-emit-probe.sh
#
# M1 fail-closed gate: v2-compiler --target rust over full src/v4 must exit 0 and emit a
# clean `compiled: N files emitted, 0 diagnostics` receipt. Missing compiler, v2 emit
# failure, a missing receipt, or nonzero diagnostics fail the step.
#
# Authority: src/v4/workflow/ci.dag (T-24) + src/v4/TASKS.md T-24; r3 gates #98/#100 interim bridge.
# Pattern: .github/ci-floor/v4-bootstrap-viability.sh (compile + log receipt parsing).
#
# Env:
#   V2_COMPILER              — v2-compiler binary (default: target/release/gunbc)
#   V4_M1_RUST_EMIT_OUT       — rust emit output dir (default: /tmp/v4-rust-emit)
#   V4_M1_RUST_EMIT_LOG       — v2 compile log (default: ${OUT}.compile.log)
#   V4_M1_DAG_EMIT_OUT        — optional dag emit output dir for shared rust+dag closure
#   V4_M1_DAG_EMIT_LOG        — dag compile receipt log (default: ${DAG_OUT}.compile.log)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_M1_RUST_EMIT_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-rust-emit"
else
  out="${V4_M1_RUST_EMIT_OUT:-/tmp/v4-rust-emit}"
fi
compile_log="${V4_M1_RUST_EMIT_LOG:-${out}.compile.log}"
summary="${out}.m1-probe-summary.txt"
dag_out="${V4_M1_DAG_EMIT_OUT:-}"
dag_log="${V4_M1_DAG_EMIT_LOG:-}"
shared_out=""
if [[ -n "$dag_out" ]]; then
  shared_out="$(dirname "$out")/v4-shared-closure"
  dag_log="${dag_log:-${dag_out}.compile.log}"
fi

if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  exit 1
fi

if [[ -n "$dag_out" ]]; then
  rm -rf "$shared_out" "$out" "$dag_out"
else
  rm -rf "$out"
fi
mkdir -p "$(dirname "$compile_log")"
if [[ -n "$dag_log" ]]; then
  mkdir -p "$(dirname "$dag_log")"
fi

emit_notice() {
  local title="$1"
  local body="$2"
  local escaped="${body//$'\n'/%0A}"
  escaped="${escaped//\r/}"
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
    echo "::notice title=${title}::${escaped}"
  fi
}

if [[ -n "$dag_out" ]]; then
  echo "=== M1: v2-compiler compile --target rust+dag src/v4 (single source closure) ==="
else
  echo "=== M1: v2-compiler compile --target rust src/v4 ==="
fi
set +e
if [[ -n "$dag_out" ]]; then
  "$bin" compile --source-root src/v4 --output-dir "$shared_out" --target rust+dag 2>&1 | tee "$compile_log"
else
  "$bin" compile --source-root src/v4 --output-dir "$out" --target rust 2>&1 | tee "$compile_log"
fi
compile_status=${PIPESTATUS[0]}
set -e

if [[ -n "$dag_out" && -d "$shared_out/rust" && -d "$shared_out/dag" ]]; then
  mv "$shared_out/rust" "$out"
  mv "$shared_out/dag" "$dag_out"
  rmdir "$shared_out" 2>/dev/null || true
  cp "$compile_log" "$dag_log"
fi

if [[ -n "$dag_out" ]]; then
  dag_artifact="$dag_out/dag-artifact.json"
  if [[ ! -s "$dag_artifact" ]]; then
    echo "error: M1 shared rust+dag probe requires non-empty DAG artifact: $dag_artifact" >&2
    exit 1
  fi
fi

compiled_receipt=""
if [[ -f "$compile_log" ]]; then
  compiled_receipt="$(grep -E '^compiled: [0-9]+ files emitted, [0-9]+ diagnostics$' "$compile_log" | tail -1 || true)"
fi

files_emitted=0
v2_diagnostics=0
if [[ -n "$compiled_receipt" ]]; then
  files_emitted="$(echo "$compiled_receipt" | sed -n 's/^compiled: \([0-9]*\) files emitted, \([0-9]*\) diagnostics$/\1/p')"
  v2_diagnostics="$(echo "$compiled_receipt" | sed -n 's/^compiled: \([0-9]*\) files emitted, \([0-9]*\) diagnostics$/\2/p')"
fi

rs_on_disk=0
if [[ -d "$out" ]]; then
  rs_on_disk="$(find "$out" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
fi

v2_error_lines=0
v2_error_categories=""
if [[ -f "$compile_log" ]]; then
  v2_error_lines="$(grep -cE '^[[:space:]]*error(\[[^]]+\])?: ' "$compile_log" 2>/dev/null || true)"
  v2_error_lines="${v2_error_lines:-0}"
  v2_error_categories="$(
    grep -oE '^[[:space:]]*error(\[[^]]+\])?: ' "$compile_log" 2>/dev/null \
      | sed 's/^[[:space:]]*//' | sort | uniq -c | sort -rn | head -20 || true
  )"
fi

{
  echo "M1 v4 full-tree rust emit probe"
  echo "==========================="
  echo "v2 compile exit: ${compile_status}"
  echo "v2 receipt: ${compiled_receipt:-<missing>}"
  echo "v2 stderr error lines: ${v2_error_lines}"
  echo ".rs files on disk: ${rs_on_disk}"
  echo ""
  if [[ -n "$v2_error_categories" ]]; then
    echo "v2 error prefix histogram (top 20):"
    echo "$v2_error_categories"
    echo ""
  fi
  echo "logs: compile=${compile_log}"
} | tee "$summary"

probe_body="$(head -20 "$summary")"
emit_notice "M1 v4 rust emit probe" "$probe_body"

if [[ "$compile_status" -ne 0 ]]; then
  emit_notice "M1 compile failed" "exit=${compile_status}; see ${compile_log}"
elif [[ "$v2_diagnostics" != "0" ]]; then
  emit_notice "M1 compile diagnostics" "${compiled_receipt:-see log}"
fi

echo "=== M1 probe summary written to ${summary} ==="

if [[ "$compile_status" -ne 0 ]]; then
  exit "$compile_status"
fi

if [[ -z "$compiled_receipt" ]]; then
  echo "error: M1 probe requires a compiled receipt line (compiled: N files emitted, M diagnostics)" >&2
  exit 1
fi

if [[ "$v2_diagnostics" != "0" ]]; then
  echo "error: M1 probe requires 0 diagnostics (${compiled_receipt})" >&2
  exit 1
fi

if [[ "$files_emitted" -lt 1 ]]; then
  echo "error: M1 probe requires at least one emitted file (${compiled_receipt})" >&2
  exit 1
fi

exit 0
