#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-gate.sh
#
# T-22 manual TestClaim corpus gate. Compiles src/v4 to the modeled .dag
# artifact, then verifies that the manual claim corpus is in the compile closure
# and that the T-22 TestClaimRun surface is present in the artifact.
#
# This intentionally avoids generated-Rust source-shape matching. It is still a
# structural gate: full emitted Rust execution remains owned by the M1 rust emit
# path until src/v4 emits cargo-clean Rust.
#
# Env:
#   V2_COMPILER             - v2-compiler binary (default: target/release/v2-compiler)
#   V4_TESTCLAIM_OUT        - artifact output dir (default: $RUNNER_TEMP/v4-testclaim-corpus or /tmp)
#   V4_TESTCLAIM_LOG        - compiler log path (default: ${OUT}.log)
#   V4_TESTCLAIM_TIMEOUT_SECS - optional timeout (CI default: 240)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/v2-compiler}"
if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for dag-artifact inspection" >&2
  exit 1
fi

manual_dir="src/v4/test/claim/manual"
if [[ ! -d "$manual_dir" ]]; then
  echo "error: missing manual TestClaim corpus directory: $manual_dir" >&2
  exit 1
fi

mapfile -t manual_files < <(find "$manual_dir" -maxdepth 1 -type f -name '*.dag' | sort)
if [[ "${#manual_files[@]}" -eq 0 ]]; then
  echo "error: manual TestClaim corpus has no .dag files under $manual_dir" >&2
  exit 1
fi

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
tmp_root="${RUNNER_TEMP:-/tmp}"
out="${V4_TESTCLAIM_OUT:-${tmp_root}/v4-testclaim-corpus-${run_suffix}}"
log="${V4_TESTCLAIM_LOG:-${out}.log}"
rm -rf "$out"
mkdir -p "$out" "$(dirname "$log")"

compile_timeout="${V4_TESTCLAIM_TIMEOUT_SECS:-}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "$compile_timeout" ]]; then
  compile_timeout=240
fi

echo "=== T-22: compile src/v4 manual TestClaim corpus (--target dag) ==="
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
  echo "error: v4 TestClaim corpus compile exited $status (log: $log)" >&2
  exit "$status"
fi

if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" >/dev/null; then
  echo "error: v4 TestClaim corpus compile did not emit a clean compiled receipt" >&2
  exit 1
fi

artifact="${out}/dag-artifact.json"
if [[ ! -s "$artifact" ]]; then
  echo "error: expected dag artifact at $artifact" >&2
  exit 1
fi

node_names="${out}/dag-node-names.txt"
jq -r '.nodes[] | .name' "$artifact" > "$node_names"

require_node() {
  local name="$1"
  if ! grep -Fx "$name" "$node_names" >/dev/null; then
    echo "error: dag artifact missing node name: $name" >&2
    exit 1
  fi
}

require_artifact_text() {
  local text="$1"
  if ! grep -F "$text" "$artifact" >/dev/null; then
    echo "error: dag artifact missing text: $text" >&2
    exit 1
  fi
}

for file in "${manual_files[@]}"; do
  stem="$(basename "$file" .dag)"
  require_node "v4.test.claim.manual.${stem}"
done

mapfile -t run_rows < <(
  grep -R -h -E '^data[[:space:]]+run_[A-Za-z0-9_]+:[[:space:]]+TestClaimRun' "$manual_dir" \
    | sed -E 's/^data[[:space:]]+(run_[A-Za-z0-9_]+):.*/\1/' \
    | sort -u
)
if [[ "${#run_rows[@]}" -eq 0 ]]; then
  echo "error: manual corpus has no TestClaimRun data rows" >&2
  exit 1
fi

for row in "${run_rows[@]}"; do
  require_node "$row"
done

for name in \
  TestClaimRun \
  TestClaimEvalSubject \
  run_test_claim \
  eval_test_claim_subject \
  run_test_claim_assert
do
  require_node "$name"
done

require_artifact_text "run_test_claim_runtime_assert"

require_node "v4.test.claim.manual.eval_runtime_mvp"
require_node "claim_eval_mvp2_test_claim_route"
require_node "run_eval_mvp2_test_claim_route"

echo "T-22 TestClaim corpus gate OK: ${#manual_files[@]} manual .dag files compiled; ${#run_rows[@]} TestClaimRun rows present."
