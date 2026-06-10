#!/usr/bin/env bash
# .github/ci-floor/v4-mvp1-typescript-emit-host-probe.sh
#
# Brief T execution gate: gunbc structural emit for mvp1 ts add-fn, then
# `emit_host_runner` tsc+Node transport on the authority pin (not hand-wrapped fixture).
#
# Authority: emit_host_typescript_authority_pin / ts_mvp1_source_text; mirrors
# v4-m1-rust-emit-probe.sh receipt shape.
#
# Env:
#   V2_COMPILER              — gunbc binary (default: target/release/gunbc)
#   V4_MVP1_TS_EMIT_HOST_OUT — probe summary dir (default: /tmp/v4-ts-emit-host-probe)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_MVP1_TS_EMIT_HOST_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-ts-emit-host-probe"
else
  out="${V4_MVP1_TS_EMIT_HOST_OUT:-/tmp/v4-ts-emit-host-probe}"
fi
claim_log="${out}.claim-run.log"
runner_log="${out}.emit-host-runner.log"
summary="${out}.mvp1-probe-summary.txt"

entry="src/v4/test/claim/manual/mvp1_typescript_add_translate.dag"
witness="mvp1_ts_emit_add_fn_accepts_holds"

if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "error: node not found on PATH (required for typescript emit-host probe)" >&2
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "error: npx not found on PATH (required for tsc transport)" >&2
  exit 1
fi

rm -rf "$out"
mkdir -p "$out"

emit_notice() {
  local title="$1"
  local body="$2"
  local escaped="${body//$'\n'/%0A}"
  escaped="${escaped//\r/}"
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
    echo "::notice title=${title}::${escaped}"
  fi
}

echo "=== MVP1 TS: gunbc structural emit (${witness}) ==="
set +e
"$bin" run \
  --source-root src/v4 \
  --entry "$entry" \
  --function "$witness" \
  --claim-run 2>&1 | tee "$claim_log"
claim_status=${PIPESTATUS[0]}
set -e

witness_result=""
if [[ -f "$claim_log" ]]; then
  witness_result="$(grep -E '^(true|false)$' "$claim_log" | tail -1 || true)"
fi

echo "=== MVP1 TS: emit_host_runner authority-pin execute (tsc + Node) ==="
set +e
bash .github/ci-floor/with-sccache-retry.sh \
  cargo test -p emit_host_runner typescript_mvp1_authority_pin_executes_on_node -- --exact --nocapture \
  2>&1 | tee "$runner_log"
runner_status=${PIPESTATUS[0]}
set -e

{
  echo "MVP1 v4 TypeScript emit-host probe"
  echo "=================================="
  echo "structural claim-run exit: ${claim_status}"
  echo "structural witness (${witness}): ${witness_result:-<missing>}"
  echo "emit_host_runner exit: ${runner_status}"
  echo ""
  echo "logs: claim-run=${claim_log} runner=${runner_log}"
} | tee "$summary"

probe_body="$(head -15 "$summary")"
emit_notice "MVP1 v4 TS emit-host probe" "$probe_body"

echo "=== MVP1 probe summary written to ${summary} ==="

if [[ "$claim_status" -ne 0 ]]; then
  exit "$claim_status"
fi

if [[ "$witness_result" != "true" ]]; then
  echo "error: probe requires structural witness true (got ${witness_result:-<missing>})" >&2
  exit 1
fi

if [[ "$runner_status" -ne 0 ]]; then
  echo "error: emit_host_runner typescript execute gate failed (exit=${runner_status})" >&2
  cat "$runner_log" >&2 || true
  exit "$runner_status"
fi

exit 0
