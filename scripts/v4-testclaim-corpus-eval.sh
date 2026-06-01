#!/usr/bin/env bash
# scripts/v4-testclaim-corpus-eval.sh
#
# P5 Layer 2 / T-38: TestClaim corpus structural CI host transport.
# Consumes upstream M1 rust emit + bootstrap dag artifacts (no host-owned compile).
#
# 🟡 gated — feature:t38-testclaim-corpus-eval — bind src/v4/TASKS.md T-38 +
# docs/planning/v4-p5-structural-bridge-replacement-worksheet-2026-05-30.md §1.4 —
# structural-slice: positive-Y `ci_upsert_testclaim_corpus_eval_*` sole structural authority.
# T-38-PR2: verdict SURFACE migrated — receipt now reports the per-row TestClaimRun roster + the
# modeled corpus tally witness (witness_manual_corpus_gate_closed: non-empty AND zero Fail AND zero
# Deferred), retiring the opaque blocked_m1_subset string. The witness is an AUTHORING-TIME const
# over the modeled CorpusEvalReport; per-row RUNTIME execution (cargo-clean M1 emitted subset OR
# bootstrap-evaluator corpus path) is load-bearing/out-of-lane and tracked as escalated upward debt.
#
# Authority: src/v4/workflow/ci.dag (`TestClaimCorpusEvalCommand` +
# `testclaim_corpus_eval_ci_live_workflow_signal`) +
# src/v4/test/claim/workflow/testclaim_corpus_runner.dag
#
# Env (composition-edge upstream — wired from ci.yml):
#   V4_M1_RUST_EMIT_OUT / V4_M1_RUST_EMIT_LOG — M1 `m1_rust_emit_probe_execution`
#   V4_BOOTSTRAP_OUT / V4_BOOTSTRAP_LOG — bootstrap dag emit (workflow-local)
#   V2_COMPILER — v2-compiler binary (presence check only; compile forbidden here)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

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

rust_out="${V4_M1_RUST_EMIT_OUT:-}"
rust_log="${V4_M1_RUST_EMIT_LOG:-}"
dag_out="${V4_BOOTSTRAP_OUT:-}"
dag_log="${V4_BOOTSTRAP_LOG:-}"

if [[ -z "$rust_out" || -z "$rust_log" || -z "$dag_out" || -z "$dag_log" ]]; then
  echo "error: corpus eval requires upstream artifact env (V4_M1_RUST_EMIT_OUT/LOG, V4_BOOTSTRAP_OUT/LOG)" >&2
  exit 1
fi

has_clean_compile_receipt() {
  local compile_log="$1"
  [[ -f "$compile_log" ]] \
    && grep -qE '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$compile_log"
}

if ! has_clean_compile_receipt "$rust_log"; then
  echo "error: M1 upstream rust emit missing clean compile receipt (log: $rust_log)" >&2
  exit 1
fi

if ! has_clean_compile_receipt "$dag_log"; then
  echo "error: bootstrap upstream dag emit missing clean compile receipt (log: $dag_log)" >&2
  exit 1
fi

echo "=== T-22/P5: structural receipt over upstream M1 rust (${rust_out}) + bootstrap dag (${dag_out}) ==="

require_file() {
  local path="$1"
  if [[ ! -s "$path" ]]; then
    echo "error: expected generated file at $path" >&2
    exit 1
  fi
}

check_generated_rust_receipt() {
  local eval_rs="${rust_out}/src/v4_compiler_eval.rs"
  local fixture_rs="${rust_out}/src/v4_test_claim_manual_eval_runtime_mvp.rs"
  local runner_rs="${rust_out}/src/v4_test_claim_workflow_testclaim_corpus_runner.rs"
  local roster_rs="${rust_out}/src/v4_test_claim_manual_manual_corpus_roster.rs"
  local corpus_eval_rs="${rust_out}/src/v4_test_claim_workflow_manual_corpus_eval.rs"
  require_file "$eval_rs"
  require_file "$fixture_rs"
  require_file "$runner_rs"
  require_file "$roster_rs"
  require_file "$corpus_eval_rs"

  python3 - "$eval_rs" "$fixture_rs" "$runner_rs" "$roster_rs" "$corpus_eval_rs" <<'PY'
from __future__ import annotations

import sys
import re
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


def check_modeled_runner(runner_rs: Path, roster_rs: Path) -> None:
    runner_required = [
        "pub fn run_manual_testclaim_corpus_eval",
        "pub struct CorpusEvalReport",
        "pub struct CorpusEvalEntry",
        "manual_corpus_node_runtime_value_rows",
        "test_claim_run_claim",
        "test_claim_run_verdict",
    ]
    roster_required = [
        "manual_corpus_node_runtime_value_rows",
        "run_eval_mvp2_test_claim_route",
        "run_rust_language_model_emit_mechanical_reverification_claim",
        "run_rust_language_model_emit_subsumption_reverifies",
    ]

    def check(path: Path, needles: list[str]) -> None:
        source = path.read_text()
        for needle in needles:
            if needle not in source:
                raise ReceiptError(f"{path}: missing modeled-runner symbol: {needle}")

    check(runner_rs, runner_required)
    check(roster_rs, roster_required)


def check_generated_corpus_eval(corpus_eval_rs: Path) -> None:
    source = corpus_eval_rs.read_text()
    for name in [
        "manual_corpus_all_pass",
        "manual_corpus_gate",
        "witness_manual_corpus_gate_closed",
    ]:
        require(source, f"pub fn {name}", corpus_eval_rs, f"generated corpus eval function {name}")

    all_pass_body = generated_function(source, "manual_corpus_all_pass", corpus_eval_rs)
    require_order(
        all_pass_body,
        [
            "let tally = corpus_report_tally(report);",
        ],
        corpus_eval_rs,
        "generated tally source for zero Fail/Deferred predicate",
    )
    normalized_all_pass = "".join(all_pass_body.split())
    fail_deferred_conjunction = re.compile(
        r"tally\.fail={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\b[^&|;=]*&&"
        r"[^&|;]*tally\.deferred={2}[^&|;=!A-Za-z0-9_:]*(?:Nat::)?[Zz]ero\b[^&|;=]*(?:;|\})"
    )
    if not fail_deferred_conjunction.search(normalized_all_pass):
        raise ReceiptError(
            f"{corpus_eval_rs}: manual_corpus_all_pass must directly conjoin "
            "the zero Fail check with the zero Deferred check via &&"
        )
    gate_body = generated_function(source, "manual_corpus_gate", corpus_eval_rs)
    normalized_gate = "".join(gate_body.split())
    inline_empty_gate = re.compile(
        r"if(?<!!)is_empty\([^)]*report[^)]*entries[^)]*\)"
        r"\{false\}else\{manual_corpus_all_pass\([^)]*report[^)]*\)"
    )
    if not inline_empty_gate.search(normalized_gate):
        raise ReceiptError(
            f"{corpus_eval_rs}: manual_corpus_gate must mirror the modeled inline "
            "is_empty(report.entries) false fallback before manual_corpus_all_pass"
        )
    require_order(
        generated_function(source, "witness_manual_corpus_gate_closed", corpus_eval_rs),
        [
            "manual_corpus_gate(run_manual_testclaim_corpus_eval())",
        ],
        corpus_eval_rs,
        "generated witness folds modeled TestClaimRun roster",
    )


try:
    check_generated_eval(Path(sys.argv[1]))
    check_generated_fixture(Path(sys.argv[2]))
    check_modeled_runner(Path(sys.argv[3]), Path(sys.argv[4]))
    check_generated_corpus_eval(Path(sys.argv[5]))
except ReceiptError as err:
    raise SystemExit(f"error: {err}") from err
PY
}

check_generated_rust_receipt

artifact="${dag_out}/dag-artifact.json"
if [[ ! -s "$artifact" ]]; then
  echo "error: expected dag artifact at $artifact" >&2
  exit 1
fi

module_names="${dag_out}/dag-module-names.txt"
item_names="${dag_out}/dag-item-registry-keys.txt"
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

# T-38-PR2 verdict surface: the modeled corpus tally over run_manual_testclaim_corpus_eval.
# Authoring-time const witness (src/v4/test/claim/workflow/manual_corpus_eval.dag); a Fail or
# Deferred row holds witness_manual_corpus_gate_closed open, so those verdict facts are not
# silently dropped. This is the SURFACE — per-row runtime execution is the escalated residual gate.
require_module "v4.test.claim.workflow.testclaim_corpus_runner"
require_module "v4.test.claim.workflow.manual_corpus_eval"

corpus_eval_src="src/v4/test/claim/workflow/manual_corpus_eval.dag"
if ! grep -q '^data witness_manual_corpus_gate_closed: Bool = manual_corpus_gate(' "$corpus_eval_src"; then
  echo "error: ${corpus_eval_src}: missing authoring-time corpus gate witness binding" >&2
  exit 1
fi
if ! grep -Eq '\(tally\.fail == Zero\) && \(tally\.deferred == Zero\)' "$corpus_eval_src"; then
  echo "error: ${corpus_eval_src}: corpus gate must require zero Fail AND zero Deferred (no dropped verdict facts)" >&2
  exit 1
fi

# JSON array of the full manual-corpus TestClaimRun registry (structural item-registry obligation).
run_registry_json="$(printf '%s\n' "${run_rows[@]}" | jq -R . | jq -s -c .)"

# Verdict-surface roster = EXACTLY the rows witness_manual_corpus_gate_closed folds, i.e. the
# explicit `manual_corpus_node_runtime_value_rows` consumed by run_manual_testclaim_corpus_eval().
# This is a strict subset of the full manual registry above; the receipt keeps the two distinct
# so the tally witness is never paired with rows it does not cover (P2 boundary discipline).
roster_src="src/v4/test/claim/manual/manual_corpus_roster.dag"
mapfile -t surface_rows < <(
  sed -n '/data manual_corpus_node_runtime_value_rows:/,/]/p' "$roster_src" \
    | grep -oE 'run_[A-Za-z0-9_]+' | sort -u
)
if [[ "${#surface_rows[@]}" -eq 0 ]]; then
  echo "error: ${roster_src}: could not parse manual_corpus_node_runtime_value_rows roster" >&2
  exit 1
fi
# The witness's tally is only meaningful if the runner actually folds that exact roster.
runner_src="src/v4/test/claim/workflow/testclaim_corpus_runner.dag"
if ! grep -q 'runs: manual_corpus_node_runtime_value_rows' "$runner_src"; then
  echo "error: ${runner_src}: run_manual_testclaim_corpus_eval must fold manual_corpus_node_runtime_value_rows" >&2
  exit 1
fi
# Every verdict-surface row must also be present in the compiled item registry (fail-closed).
for row in "${surface_rows[@]}"; do
  require_item "$row"
done
surface_roster_json="$(printf '%s\n' "${surface_rows[@]}" | jq -R . | jq -s -c .)"

files_emitted="$(grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$rust_log" | tail -1 | sed -n 's/^compiled: \([0-9]*\) files emitted, 0 diagnostics$/\1/p')"

cat <<JSON
{
  "schema": "scripts/v4-testclaim-corpus-eval.sh::host_verdict_surface_receipt_v3",
  "execution_status": "authoring_time_verdict_surface",
  "verdict_surface_source": "authoring-time const witnesses over the modeled corpus runner (CorpusEvalReport / VerdictTally); NOT CI-executed runtime verdicts",
  "residual_runtime_gate": "per-row runtime TestClaimRun execution (cargo-clean M1 emitted subset OR bootstrap-evaluator corpus path) is load-bearing and out of this author lane; tracked as escalated upward debt",
  "manual_dag_files": ${#manual_files[@]},
  "manual_corpus_registry_rows": ${#run_rows[@]},
  "manual_corpus_registry_roster": ${run_registry_json},
  "verdict_surface_roster_rows": ${#surface_rows[@]},
  "verdict_surface_roster": ${surface_roster_json},
  "verdict_surface_roster_note": "the rows manual_corpus_node_runtime_value_rows that run_manual_testclaim_corpus_eval folds; a strict subset of manual_corpus_registry_roster — the tally witness covers ONLY these rows",
  "corpus_tally_gate_witness": "witness_manual_corpus_gate_closed",
  "corpus_tally_gate_witness_status": "binding present in source with predicate verified by grep; NOT evaluated by this host transport (compile does not gate on Bool witness truth) — see residual_runtime_gate",
  "corpus_gate_predicate": "non-empty roster AND zero Fail AND zero Deferred",
  "rust_emit_files_emitted": ${files_emitted},
  "structural_witness": "PASS"
}
JSON

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "### T-22 TestClaim corpus eval — verdict surface (modeled CiUpsertStep)"
    echo ""
    echo "**execution_status:** \`authoring_time_verdict_surface\` — the modeled corpus tally witness (\`witness_manual_corpus_gate_closed\`, predicate: non-empty AND zero Fail AND zero Deferred) covers the ${#surface_rows[@]}-row \`manual_corpus_node_runtime_value_rows\` roster; the full manual TestClaimRun registry is ${#run_rows[@]} rows. The witness is an authoring-time const verified by source-grep, NOT evaluated by this transport (compile does not gate on Bool witness truth); per-row runtime execution remains the escalated residual gate."
    echo ""
    echo "| receipt | status |"
    echo "| --- | --- |"
    echo "| upstream M1 rust emit (0-diagnostic) | PASS |"
    echo "| upstream bootstrap dag emit (0-diagnostic) | PASS |"
    echo "| manual corpus modules + TestClaimRun registry | PASS (${#manual_files[@]} files, ${#run_rows[@]} runs) |"
    echo "| MVP generated-Rust + modeled runner structural witness | PASS |"
    echo "| corpus tally gate witness (covers ${#surface_rows[@]}-row roster) | DECLARED — source-present + predicate verified; NOT CI-evaluated |"
  } >> "$GITHUB_STEP_SUMMARY"
fi

echo "T-22 TestClaim corpus verdict-surface eval PASS: ${#manual_files[@]} manual .dag files; ${#run_rows[@]} TestClaimRun registry rows; verdict-surface tally witness covers ${#surface_rows[@]}-row roster (source-present, not CI-evaluated); per-row runtime execution = escalated residual gate."
exit 0
