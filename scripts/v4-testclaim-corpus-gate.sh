#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-gate.sh
#
# T-22 manual TestClaim corpus structural bridge. Compiles src/v4 to emitted Rust
# and the modeled .dag artifact, then verifies that the manual claim corpus is in
# the compile closure and that the T-22 TestClaimRun surface is present in the
# artifact.
#
# 🟡 gated — feature:t38-testclaim-corpus-ci-gate — scaffold:TASKS T-38 / INVARIANTS §P5
# Owner lane: T-22/T-38 evaluation harness closeout; operator-authorized CI receipt only.
# Dissolve-on-arrival: delete this script and the paired CI step when the modeled runner
#   executes `src/v4/test/claim/manual/*.dag` in CI and emits structured TestClaimRun
#   verdicts directly from .dag/workflow-as-data, with no shell-owned artifact inspection.
# Exit condition: removal when that runner is green on main CI for 14 consecutive days.
#
# This intentionally avoids generated-Rust source-shape matching. It is still a
# structural bridge: full emitted Rust execution remains owned by the M1 rust emit
# path until src/v4 emits cargo-clean Rust.
#
# Env:
#   V2_COMPILER             - v2-compiler binary (default: target/release/v2-compiler)
#   V4_TESTCLAIM_OUT        - dag artifact output dir (default: $RUNNER_TEMP/v4-testclaim-corpus or /tmp)
#   V4_TESTCLAIM_LOG        - dag compiler log path (default: ${OUT}.log)
#   V4_TESTCLAIM_RUST_OUT   - rust emit output dir (default: ${OUT}-rust)
#   V4_TESTCLAIM_RUST_LOG   - rust emit compiler log path (default: ${RUST_OUT}.log)
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
rust_out="${V4_TESTCLAIM_RUST_OUT:-${out}-rust}"
rust_log="${V4_TESTCLAIM_RUST_LOG:-${rust_out}.log}"
rm -rf "$out" "$rust_out"
mkdir -p "$out" "$rust_out" "$(dirname "$log")" "$(dirname "$rust_log")"

compile_timeout="${V4_TESTCLAIM_TIMEOUT_SECS:-}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "$compile_timeout" ]]; then
  compile_timeout=240
fi

run_compile() {
  local target="$1"
  local output_dir="$2"
  local output_log="$3"

  set +e
  if [[ -n "$compile_timeout" ]]; then
    timeout --preserve-status "$compile_timeout" \
      "$bin" compile --source-root src/v4 --output-dir "$output_dir" --target "$target" 2>&1 | tee "$output_log"
  else
    "$bin" compile --source-root src/v4 --output-dir "$output_dir" --target "$target" 2>&1 | tee "$output_log"
  fi
  status=${PIPESTATUS[0]}
  set -e

  if [[ "$status" -ne 0 ]]; then
    echo "error: v4 TestClaim corpus compile --target ${target} exited $status (log: $output_log)" >&2
    exit "$status"
  fi

  if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$output_log" >/dev/null; then
    echo "error: v4 TestClaim corpus compile --target ${target} did not emit a clean compiled receipt" >&2
    exit 1
  fi
}

echo "=== T-22: compile src/v4 manual TestClaim corpus (--target rust) ==="
run_compile rust "$rust_out" "$rust_log"

echo "=== T-22: compile src/v4 manual TestClaim corpus (--target dag) ==="
run_compile dag "$out" "$log"

artifact="${out}/dag-artifact.json"
if [[ ! -s "$artifact" ]]; then
  echo "error: expected dag artifact at $artifact" >&2
  exit 1
fi

module_names="${out}/dag-module-names.txt"
item_names="${out}/dag-item-registry-keys.txt"
jq -e 'has("modules") and has("item_registry_keys") and has("files")' "$artifact" >/dev/null
jq -r '. as $root | .modules[] | .module["$ref"] as $id | $root.nodes[$id].name' "$artifact" > "$module_names"
jq -r '.item_registry_keys[]' "$artifact" > "$item_names"

require_module() {
  local name="$1"
  if ! grep -Fx "$name" "$module_names" >/dev/null; then
    echo "error: dag artifact missing module: $name" >&2
    exit 1
  fi
}

require_item() {
  local name="$1"
  if ! grep -Fx "$name" "$item_names" >/dev/null; then
    echo "error: dag artifact missing item_registry_key: $name" >&2
    exit 1
  fi
}

for file in "${manual_files[@]}"; do
  stem="$(basename "$file" .dag)"
  require_module "v4.test.claim.manual.${stem}"
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
  require_item "$row"
done

for name in \
  TestClaimRun \
  TestClaimEvalSubject \
  run_test_claim \
  eval_test_claim_subject \
  run_test_claim_assert
do
  require_item "$name"
done

require_item "run_test_claim_runtime_assert"

require_module "v4.test.claim.manual.eval_runtime_mvp"
require_item "claim_eval_mvp2_test_claim_route"
require_item "run_eval_mvp2_test_claim_route"

echo "T-22 TestClaim corpus structural bridge PASS: ${#manual_files[@]} manual .dag files compiled; ${#run_rows[@]} TestClaimRun rows present; rust emit clean; no TestClaim verdicts evaluated."
