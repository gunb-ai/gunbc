#!/usr/bin/env bash
# .github/ci-floor/v4-typescript-structural-emit-probe.sh
#
# M0 measurement probe: v4 structural TypeScript emit for MVP-1 add-fn, then Node exercise
# of the canonical emitted source. Documents the first real gap between translate-time emit
# (green) and emit-host execute+eval wiring (missing typescript row).
#
# Authority: ctrl#1489 emit spine M0 lane; mirrors v4-rust-full-tree-emit-probe.sh receipt shape.
#
# Env:
#   V2_COMPILER              — gunbc binary (default: target/release/gunbc)
#   V4_TYPESCRIPT_STRUCTURAL_EMIT_PROBE_OUT        — probe summary dir (default: /tmp/v4-ts-emit-probe)
#   V4_TYPESCRIPT_STRUCTURAL_EMIT_PROBE_LOG        — claim-run log (default: ${OUT}.claim-run.log)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_TYPESCRIPT_STRUCTURAL_EMIT_PROBE_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-ts-emit-probe"
else
  out="${V4_TYPESCRIPT_STRUCTURAL_EMIT_PROBE_OUT:-/tmp/v4-ts-emit-probe}"
fi
claim_log="${V4_TYPESCRIPT_STRUCTURAL_EMIT_PROBE_LOG:-${out}.claim-run.log}"
summary="${out}.m0-probe-summary.txt"
node_log="${out}.node-run.log"

entry="src/v4/test/claim/manual/mvp1_typescript_add_translate.dag"
witness="mvp1_ts_emit_add_fn_accepts_holds"
canonical_source='function add(x: number, y: number): number { return x + y; }'

if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "error: node not found on PATH (required for M0 TS execute probe)" >&2
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

echo "=== M0: gunbc --claim-run structural TS emit (${witness}) ==="
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

echo "=== M0: tsc typecheck on canonical ts_mvp1 add-fn source ==="
tsc_log="${out}.tsc.log"
ts_fixture="${out}/add_probe.ts"
cat > "$ts_fixture" <<EOF
${canonical_source}
EOF

tsc_pkg="typescript@5.9.2"
set +e
npx -y -p "$tsc_pkg" tsc --strict --noEmit --target ES2022 --module ES2022 "$ts_fixture" >"$tsc_log" 2>&1
tsc_status=$?
set -e

echo "=== M0: Node runtime on tsc-emitted JS for canonical add-fn ==="
js_out_dir="${out}/tsc-out"
mkdir -p "$js_out_dir"
js_fixture="${js_out_dir}/add_probe.ts"
cat > "$js_fixture" <<EOF
${canonical_source}
const result = add(2, 3);
if (result !== 5) {
  console.error("add(2,3) expected 5 got", result);
  process.exit(1);
}
EOF

set +e
npx -y -p "$tsc_pkg" tsc --target ES2022 --module ES2022 --outDir "$js_out_dir" "$js_fixture" >>"$tsc_log" 2>&1
tsc_emit_status=$?
node "$js_out_dir/add_probe.js" >"$node_log" 2>&1
node_status=$?
set -e

first_gap="emit_host_typescript_row_missing"
gap_detail="v4 translate emit is green (${witness}=${witness_result:-<unknown>}); canonical TS typechecks (tsc) and runs on Node only after tsc emit (raw .ts is not Node-executable). emit_host.dag has no typescript authority pin / run_emit_host_typescript / emit_host_runner Node+tsc row — run_emit_host rejects with emit_host_run_unsupported_target before execute+eval compare (RTADD/D1)."

{
  echo "M0 v4 TypeScript emit probe"
  echo "==========================="
  echo "structural claim-run exit: ${claim_status}"
  echo "structural witness (${witness}): ${witness_result:-<missing>}"
  echo "canonical source: ${canonical_source}"
  echo "tsc --noEmit exit: ${tsc_status}"
  echo "tsc emit exit: ${tsc_emit_status}"
  echo "node runtime exit (on tsc output): ${node_status}"
  echo ""
  echo "first_real_gap: ${first_gap}"
  echo "gap_detail: ${gap_detail}"
  echo ""
  echo "logs: claim-run=${claim_log} tsc=${tsc_log} node=${node_log}"
} | tee "$summary"

probe_body="$(head -20 "$summary")"
emit_notice "M0 v4 TS emit probe" "$probe_body"

echo "=== M0 probe summary written to ${summary} ==="

if [[ "$claim_status" -ne 0 ]]; then
  exit "$claim_status"
fi

if [[ "$witness_result" != "true" ]]; then
  echo "error: M0 probe requires structural witness true (got ${witness_result:-<missing>})" >&2
  exit 1
fi

if [[ "$tsc_status" -ne 0 ]]; then
  echo "error: M0 probe requires tsc --noEmit pass on canonical source (exit=${tsc_status})" >&2
  cat "$tsc_log" >&2 || true
  exit "$tsc_status"
fi

if [[ "$tsc_emit_status" -ne 0 || "$node_status" -ne 0 ]]; then
  echo "error: M0 probe requires tsc emit + Node runtime pass (tsc=${tsc_emit_status} node=${node_status})" >&2
  cat "$tsc_log" >&2 || true
  cat "$node_log" >&2 || true
  exit 1
fi

exit 0
