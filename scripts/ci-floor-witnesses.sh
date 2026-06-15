#!/usr/bin/env bash
# CI floor +/- witnesses (operator bankruptcy — 3-gate floor).
#
# Usage:
#   scripts/ci-floor-witnesses.sh green          # all three GREEN paths
#   scripts/ci-floor-witnesses.sh red-gate1    # gate-1: non-allowlisted parse break -> RED
#   scripts/ci-floor-witnesses.sh red-gate3     # gate-3: perturb on selection semantics -> RED
#   scripts/ci-floor-witnesses.sh red-floor      # overall: planted break in ci.dag -> floor RED

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

gunbc="${V2_COMPILER:-target/release/gunbc}"
claim_gate="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"

ensure_bins() {
  test -x "$gunbc" || cargo build -p v2-compiler --release
  test -x "$claim_gate" || cargo build -p ci_claim_gate --release
}

# Mirror gunbc.tools.ci_runner run_logged_script (dsl/gunbc/tools/ci_runner.dag):
# redirect stdout/stderr to RUNNER_TEMP log, tail 120 on failure, propagate ec.
run_logged_script_mirror() {
  local log_label="$1"
  shift
  [ -n "${RUNNER_TEMP:-}" ] || RUNNER_TEMP=/tmp
  local log="$RUNNER_TEMP/gunbc-${log_label}.log"
  set +e
  "$@" >"$log" 2>&1
  local ec=$?
  tail -n 120 "$log"
  return "$ec"
}

witness_green_gate1() {
  echo "::group::witness gate-1 GREEN (full-tree minus 2-file allowlist)"
  bash scripts/v2-dsl-full-tree-parse-gate.sh
  echo "::endgroup::"
}

witness_red_gate1() {
  echo "::group::witness gate-1 RED (new break outside allowlist)"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  cp -a dsl "$tmp/dsl"
  local plant='let planted_gate1_red_witness: Int = 0'
  printf '\n%s\n' "$plant" >>"$tmp/dsl/gunbc/ci.dag"
  echo "PLANTED: ${plant} in non-allowlisted $tmp/dsl/gunbc/ci.dag"
  set +e
  "$gunbc" compile --source-root "$tmp/dsl" --output-dir "$tmp/out" --target rust >"$tmp/log" 2>&1
  local status=$?
  set -e
  echo "OBSERVED: gunbc compile exit ${status} (expect non-zero)"
  if [[ "$status" -eq 0 ]]; then
    echo "error: expected compile failure on planted top-level let" >&2
    cat "$tmp/log" >&2
    return 1
  fi
  if ! rg -q 'planted_gate1_red_witness' "$tmp/log"; then
    echo "error: planted break did not surface in compile diagnostics" >&2
    cat "$tmp/log" >&2
    return 1
  fi
  echo "DETECTED RED: compile diagnostic references planted_gate1_red_witness:"
  rg 'planted_gate1_red_witness|error\[' "$tmp/log" || true
  echo "gate-1 RED witness OK"
  echo "::endgroup::"
}

witness_green_gate3() {
  echo "::group::witness gate-3 GREEN (fail-closed selection semantics)"
  "$claim_gate" \
    --source-root src/v4 \
    --gate-entry src/v4/test/claim/workflow/probe_selector_ci_runner.dag \
    --rows-fn probe_selector_ci_runner_rows_tsv \
    --notice-title "gate-3 fail-closed selection"
  echo "::endgroup::"
}

witness_red_gate3() {
  echo "::group::witness gate-3 RED (perturb via run_logged_script redirect — live gate-3 ec shape)"
  echo "CHECK: redirect wrapper propagates non-zero exit (honest-bar; must not fail-open)"
  set +e
  run_logged_script_mirror gate3-exit-guard false
  local guard_ec=$?
  set -e
  echo "OBSERVED: false through redirect exited ${guard_ec} (expect non-zero)"
  if [[ "$guard_ec" -eq 0 ]]; then
    echo "error: redirect wrapper swallowed gate-3 failure exit code" >&2
    return 1
  fi
  echo "exit-status propagation OK"
  echo "PLANTED: ci-claim-gate --perturb-check rewrites each probe_selector witness body to false"
  echo "OBSERVED: perturb through same redirect path live gate-3 uses (run_logged_script in ci_runner.dag)"
  set +e
  run_logged_script_mirror gate3 \
    "$claim_gate" \
    --source-root src/v4 \
    --gate-entry src/v4/test/claim/workflow/probe_selector_ci_runner.dag \
    --rows-fn probe_selector_ci_runner_rows_tsv \
    --notice-title "gate-3 perturb" \
    --perturb-check
  local ec=$?
  set -e
  echo "OBSERVED: perturb-check through redirect exited ${ec} (expect 0 when RED teeth bite)"
  if [[ "$ec" -ne 0 ]]; then
    echo "error: perturb-check failed through redirect wrapper" >&2
    return 1
  fi
  echo "DETECTED RED: perturb-check completed through redirect — every row went RED under perturb and was caught"
  echo "gate-3 RED witness OK"
  echo "::endgroup::"
}

witness_green_floor() {
  echo "::group::witness overall floor GREEN (clean tree)"
  "$gunbc" run --source-root dsl --entry dsl/gunbc/tools/ci_runner.dag --function run_ci_pipeline
  echo "::endgroup::"
}

witness_red_floor() {
  echo "::group::witness overall floor RED (planted break in ci.dag)"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  cp -a dsl "$tmp/dsl"
  local plant='let planted_floor_red_witness: Int = 0'
  printf '\n%s\n' "$plant" >>"$tmp/dsl/gunbc/ci.dag"
  echo "PLANTED: ${plant} in $tmp/dsl/gunbc/ci.dag"
  set +e
  "$gunbc" run --source-root "$tmp/dsl" --entry dsl/gunbc/tools/ci_runner.dag --function run_ci_pipeline >"$tmp/log" 2>&1
  local status=$?
  set -e
  echo "OBSERVED: run_ci_pipeline exit ${status} (expect non-zero)"
  if [[ "$status" -eq 0 ]]; then
    echo "error: expected run_ci_pipeline failure on planted ci.dag break" >&2
    cat "$tmp/log" >&2
    return 1
  fi
  echo "DETECTED RED: run_ci_pipeline failed on planted break; log tail:"
  tail -20 "$tmp/log" || true
  echo "overall floor RED witness OK"
  echo "::endgroup::"
}

mode="${1:-green}"
ensure_bins

case "$mode" in
  green)
    witness_green_gate1
    witness_green_gate3
    witness_green_floor
    echo "all floor witnesses GREEN"
    ;;
  --red-demo|red-demo)
    witness_red_gate1
    witness_red_gate3
    witness_red_floor
    echo "floor RED teeth witnesses OK (gate-1 + gate-3 + overall)"
    ;;
  red-gate1)
    witness_red_gate1
    ;;
  red-gate3)
    witness_red_gate3
    ;;
  red-floor)
    witness_red_floor
    ;;
  *)
    echo "usage: $0 {green|red-gate1|red-gate3|red-floor}" >&2
    exit 2
    ;;
esac
