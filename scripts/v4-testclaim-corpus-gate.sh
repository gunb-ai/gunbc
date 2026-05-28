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

check_generated_rust_receipt() {
  local eval_rs="${rust_out}/src/v4_compiler_eval.rs"
  local fixture_rs="${rust_out}/src/v4_test_claim_manual_eval_runtime_mvp.rs"
  require_file "$eval_rs"
  require_file "$fixture_rs"

  python3 - "$eval_rs" "$fixture_rs" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path


class ReceiptError(Exception):
    pass


def generated_function(source: str, name: str, path: Path) -> str:
    marker = f"pub fn {name}("
    start = source.find(marker)
    if start == -1:
        raise ReceiptError(f"{path}: generated function not found: {name}")
    brace = source.find("{", start)
    if brace == -1:
        raise ReceiptError(f"{path}: generated function has no body: {name}")

    depth = 0
    in_line_comment = False
    in_block_comment = False
    in_string = False
    in_char = False
    escaped = False
    for index in range(brace, len(source)):
        ch = source[index]
        nxt = source[index + 1] if index + 1 < len(source) else ""

        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            continue
        if in_block_comment:
            if ch == "*" and nxt == "/":
                in_block_comment = False
            continue
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            continue
        if in_char:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == "'":
                in_char = False
            continue

        if ch == "/" and nxt == "/":
            in_line_comment = True
            continue
        if ch == "/" and nxt == "*":
            in_block_comment = True
            continue
        if ch == '"':
            in_string = True
            continue
        if ch == "'":
            in_char = True
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]

    raise ReceiptError(f"{path}: unterminated generated function: {name}")


def require(source: str, needle: str, path: Path, label: str) -> None:
    if needle not in source:
        raise ReceiptError(f"{path}: missing {label}: {needle}")


def require_order(source: str, needles: list[str], path: Path, label: str) -> None:
    offset = 0
    for needle in needles:
        found = source.find(needle, offset)
        if found == -1:
            raise ReceiptError(f"{path}: missing ordered {label}: {needle}")
        offset = found + len(needle)


def check_generated_eval(eval_rs: Path) -> None:
    source = eval_rs.read_text()
    require_order(
        generated_function(source, "eval", eval_rs),
        [
            "pub fn eval(tree: Rc<InferredTree>, interpretation: Rc<InterpretationAlgebra>, inputs: Rc<Inputs>) -> Rc<Outcome>",
            "well_formed(tree.root.clone())",
            "well_formed(inputs.root.clone())",
            "eval_runtime_node(inputs.root.clone(), tree.clone(), interpretation, empty_evaluation_environment(), eval_runtime())",
        ],
        eval_rs,
        "compiled eval(tree, interpretation, inputs) dispatch",
    )
    require_order(
        generated_function(source, "eval_runtime_node", eval_rs),
        [
            "fold_node(node, Rc::new(NodeFold",
            "init: Rc::new(|n0| eval_fold_init",
            "step: Rc::new(|acc, e, child| eval_fold_step",
            "eval_fold_state_value(folded)",
        ],
        eval_rs,
        "compiled runtime-node fold",
    )
    require_order(
        generated_function(source, "eval_fold_init", eval_rs),
        [
            "structural: eval_structural_node_for_eval",
            "child_values: Rc::new(Outcome::Accepted",
            "value: if ((node.children.clone().len() as i64) == 0)",
            "eval_interpret_node",
            "outcome_rejected",
            "eval_rejected_pending_children",
        ],
        eval_rs,
        "compiled leaf evaluation posture",
    )
    require_order(
        generated_function(source, "eval_fold_step", eval_rs),
        [
            "eval_edge_is_runtime_argument(edge)",
            "eval_append_child_value",
            "eval_absorb_child_diagnostics",
            "eval_maybe_complete_node",
        ],
        eval_rs,
        "compiled child value accumulation",
    )
    require_order(
        generated_function(source, "eval_computation_node", eval_rs),
        [
            "interpretation_behavior_dispatch(interpretation.clone(), behavior)",
            "RuntimeBehaviorInterpreter::ValueRuntimeInterpreter",
            "RuntimeBehaviorInterpreter::TransformRuntimeInterpreter",
            "RuntimeBehaviorInterpreter::BranchRuntimeInterpreter",
            "RuntimeBehaviorInterpreter::LoopRuntimeInterpreter",
            "RuntimeBehaviorInterpreter::BindRuntimeInterpreter",
            "environment",
            "runtime",
            "eval_accept_runtime_value_with_facts",
        ],
        eval_rs,
        "compiled behavior dispatch through runtime-owned InterpretationAlgebra",
    )
    require_order(
        generated_function(source, "eval_branch_node", eval_rs),
        [
            "eval_first_runtime_argument",
            "(branch.choose_branch)",
            "eval_runtime_node(chosen.clone(), tree.clone(), interpretation.clone(), environment.clone(), runtime.clone())",
        ],
        eval_rs,
        "compiled Branch interpreter chooses and resumes through selected subgraph",
    )
    require_order(
        generated_function(source, "eval_bind_node", eval_rs),
        [
            "eval_bind_key",
            "eval_bind_value_argument",
            "(bind.bind_value)",
            "eval_bind_body",
            "eval_runtime_node(body.clone(), tree.clone(), interpretation.clone(), bound_environment.clone(), runtime.clone())",
        ],
        eval_rs,
        "compiled Bind interpreter extends environment and resumes through body",
    )


def check_generated_fixture(fixture_rs: Path) -> None:
    source = fixture_rs.read_text()

    literal_body = generated_function(source, "eval_mvp2_literal_node", fixture_rs)
    require(literal_body, "behavior: Behavior::Value", fixture_rs, "literal Value node")
    require(literal_body, "EdgeLabel::Named", fixture_rs, "non-runtime literal type edge")

    root_body = generated_function(source, "eval_mvp2_add_subgraph", fixture_rs)
    require(root_body, "behavior: Behavior::Transform", fixture_rs, "root Transform node")
    require(root_body, "eval_mvp2_literal_node(eval_mvp2_left_symbol())", fixture_rs, "left child")
    require(root_body, "eval_mvp2_literal_node(eval_mvp2_right_symbol())", fixture_rs, "right child")
    if root_body.count("EdgeLabel::Positional") != 2:
        raise ReceiptError(f"{fixture_rs}: root Transform must have exactly two positional runtime children")

    two_body = generated_function(source, "eval_mvp2_two_value", fixture_rs)
    if two_body.count("eval_mvp2_byte()") != 2:
        raise ReceiptError(f"{fixture_rs}: two-value fixture must carry exactly two bytes")

    five_body = generated_function(source, "eval_mvp2_five_value", fixture_rs)
    if five_body.count("eval_mvp2_byte()") != 5:
        raise ReceiptError(f"{fixture_rs}: five-value fixture must carry exactly five bytes")

    require_order(
        generated_function(source, "eval_mvp2_allocate_literal", fixture_rs),
        ["Outcome::Accepted", "value: eval_mvp2_two_value()", "diagnostics: None"],
        fixture_rs,
        "compiled literal interpreter returns two-byte RuntimeValue",
    )
    require_order(
        generated_function(source, "eval_mvp2_arg_is_two_literal", fixture_rs),
        [
            "RuntimeValue::RuntimePrimitive",
            "p.primitive_type.clone() == eval_mvp2_i64_node()",
            "p.bytes.clone().len() as i64) == 2",
        ],
        fixture_rs,
        "compiled two-byte argument predicate",
    )
    require_order(
        generated_function(source, "eval_mvp2_args_are_two_literals", fixture_rs),
        [
            "args.clone().len() as i64) == 2",
            "eval_mvp2_arg_is_two_literal(left.clone())",
            "eval_mvp2_arg_is_two_literal(right.clone())",
        ],
        fixture_rs,
        "compiled two-argument predicate",
    )
    require_order(
        generated_function(source, "eval_mvp2_call_primitive", fixture_rs),
        [
            "if eval_mvp2_args_are_two_literals(args)",
            "Outcome::Accepted",
            "value: eval_mvp2_five_value()",
            "diagnostics: None",
            "Outcome::Rejected",
        ],
        fixture_rs,
        "compiled transform interpreter fails closed and accepts five",
    )
    require_order(
        generated_function(source, "eval_mvp2_interpretation_algebra", fixture_rs),
        [
            "allocate_literal: Rc::new(eval_mvp2_allocate_literal)",
            "call_primitive: Rc::new(eval_mvp2_call_primitive)",
        ],
        fixture_rs,
        "compiled InterpretationAlgebra binds fixture interpreters",
    )
    require_order(
        generated_function(source, "eval_mvp2_actual", fixture_rs),
        [
            "eval(eval_mvp2_inferred_tree(), eval_mvp2_interpretation_algebra(), Rc::new(Inputs",
            "root: eval_mvp2_add_subgraph()",
        ],
        fixture_rs,
        "compiled eval(tree, interpretation, inputs) invocation",
    )
    require_order(
        generated_function(source, "witness_eval_mvp2_add_accepts_five", fixture_rs),
        [
            "match (*eval_mvp2_actual()).clone()",
            "Outcome::Accepted { ref value, diagnostics: None",
            "RuntimeValue::RuntimePrimitive",
            "p.primitive_type.clone() == eval_mvp2_i64_node()",
            "p.bytes.clone().len() as i64) == 5",
            "_ => false",
        ],
        fixture_rs,
        "compiled witness asserts Accepted RuntimePrimitive with five bytes",
    )


try:
    check_generated_eval(Path(sys.argv[1]))
    check_generated_fixture(Path(sys.argv[2]))
except ReceiptError as err:
    raise SystemExit(f"error: {err}") from err
PY
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
