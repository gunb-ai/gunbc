#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-gate.sh
#
# T-22 manual TestClaim corpus structural bridge. Compiles src/v4 to emitted Rust
# and the modeled .dag artifact, then verifies the manual claim corpus closure,
# the T-22 TestClaimRun artifact surface, and the legacy eval-runtime MVP
# generated-Rust receipt while modeled verdict execution is still absent.
#
# 🟡 gated — feature:t38-testclaim-corpus-ci-gate — scaffold:TASKS T-38 / INVARIANTS §P5
# Owner lane: T-22/T-38 evaluation harness closeout; operator-authorized CI receipt only.
# Dissolve-on-arrival: delete this script and the paired CI step when the modeled runner
#   executes `src/v4/test/claim/manual/*.dag` in CI and emits structured TestClaimRun
#   verdicts directly from .dag/workflow-as-data, with no shell-owned artifact inspection.
# Exit condition: removal when that runner is green on main CI for 14 consecutive days.
#
# This is still a structural bridge: full emitted Rust execution remains owned by
# the M1 rust emit path until src/v4 emits cargo-clean Rust, and TestClaim verdict
# execution remains owned by the T-38 modeled runner closeout.
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

require_file() {
  local path="$1"
  if [[ ! -s "$path" ]]; then
    echo "error: expected generated Rust file at $path" >&2
    exit 1
  fi
}

require_contains() {
  local path="$1"
  local needle="$2"
  local label="$3"
  if ! grep -F "$needle" "$path" >/dev/null; then
    echo "error: $path missing ${label}: $needle" >&2
    exit 1
  fi
}

check_generated_rust_receipt() {
  local eval_rs="${rust_out}/src/v4_compiler_eval.rs"
  local fixture_rs="${rust_out}/src/v4_test_claim_manual_eval_runtime_mvp.rs"
  require_file "$eval_rs"
  require_file "$fixture_rs"

  require_contains "$eval_rs" "pub fn eval(tree: Rc<InferredTree>, interpretation: Rc<InterpretationAlgebra>, inputs: Rc<Inputs>) -> Rc<Outcome>" "eval entrypoint"
  require_contains "$eval_rs" "well_formed(tree.root.clone())" "eval tree well_formed check"
  require_contains "$eval_rs" "well_formed(inputs.root.clone())" "eval input well_formed check"
  require_contains "$eval_rs" "eval_runtime_node(inputs.root.clone(), tree.clone(), interpretation, empty_evaluation_environment(), eval_runtime())" "eval runtime dispatch"
  require_contains "$eval_rs" "fold_node(node, Rc::new(NodeFold" "runtime node fold"
  require_contains "$eval_rs" "init: Rc::new(|n0| eval_fold_init" "runtime fold init"
  require_contains "$eval_rs" "step: Rc::new(|acc, e, child| eval_fold_step" "runtime fold step"
  require_contains "$eval_rs" "eval_fold_state_value(folded)" "runtime fold result"
  require_contains "$eval_rs" "interpretation_behavior_dispatch(interpretation.clone(), behavior)" "runtime behavior dispatch"
  require_contains "$eval_rs" "RuntimeBehaviorInterpreter::ValueRuntimeInterpreter" "Value interpreter dispatch"
  require_contains "$eval_rs" "RuntimeBehaviorInterpreter::TransformRuntimeInterpreter" "Transform interpreter dispatch"
  require_contains "$eval_rs" "RuntimeBehaviorInterpreter::BranchRuntimeInterpreter" "Branch interpreter dispatch"
  require_contains "$eval_rs" "RuntimeBehaviorInterpreter::LoopRuntimeInterpreter" "Loop interpreter dispatch"
  require_contains "$eval_rs" "RuntimeBehaviorInterpreter::BindRuntimeInterpreter" "Bind interpreter dispatch"
  require_contains "$eval_rs" "eval_accept_runtime_value_with_facts" "runtime value acceptance"

  require_contains "$fixture_rs" "behavior: Behavior::Value" "MVP literal Value behavior"
  require_contains "$fixture_rs" "EdgeLabel::Named" "MVP non-runtime literal type edge"
  require_contains "$fixture_rs" "behavior: Behavior::Transform" "MVP root Transform behavior"
  require_contains "$fixture_rs" "eval_mvp2_literal_node(eval_mvp2_left_symbol())" "MVP left runtime child"
  require_contains "$fixture_rs" "eval_mvp2_literal_node(eval_mvp2_right_symbol())" "MVP right runtime child"
  require_contains "$fixture_rs" "allocate_literal: Rc::new(eval_mvp2_allocate_literal)" "MVP InterpretationAlgebra literal interpreter"
  require_contains "$fixture_rs" "call_primitive: Rc::new(eval_mvp2_call_primitive)" "MVP InterpretationAlgebra transform interpreter"
  require_contains "$fixture_rs" "if eval_mvp2_args_are_two_literals(args)" "MVP transform fail-closed predicate"
  require_contains "$fixture_rs" "value: eval_mvp2_five_value()" "MVP five-byte accepted value"
  require_contains "$fixture_rs" "eval(eval_mvp2_inferred_tree(), eval_mvp2_interpretation_algebra(), Rc::new(Inputs" "MVP eval invocation"
  require_contains "$fixture_rs" "root: eval_mvp2_add_subgraph()" "MVP eval input root"
  require_contains "$fixture_rs" "match (*eval_mvp2_actual()).clone()" "MVP witness evaluates actual"
  require_contains "$fixture_rs" "Outcome::Accepted { ref value, diagnostics: None" "MVP witness requires accepted value"
  require_contains "$fixture_rs" "RuntimeValue::RuntimePrimitive" "MVP witness requires RuntimePrimitive"
  require_contains "$fixture_rs" "p.primitive_type.clone() == eval_mvp2_i64_node()" "MVP witness primitive type"
  require_contains "$fixture_rs" "p.bytes.clone().len() as i64) == 5" "MVP witness five-byte result"
}

check_generated_rust_receipt

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

echo "T-22 TestClaim corpus structural bridge PASS: ${#manual_files[@]} manual .dag files compiled; ${#run_rows[@]} TestClaimRun rows present; rust emit and MVP generated-Rust receipt clean; no TestClaim verdicts evaluated."
