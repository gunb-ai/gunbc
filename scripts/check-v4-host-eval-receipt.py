#!/usr/bin/env python3
"""CI receipt for the v4 T-22 host-eval MVP fixture.

This is intentionally compiler-backed instead of a raw source-shape grep:
the receipt first asks the bootstrap compiler to resolve and emit the v4
program, then checks the generated evaluator/witness surface that CI would
consume:

  eval(tree, interpretation, inputs) evaluates two Value children to two-byte
  RuntimePrimitive values, dispatches the Transform node through the supplied
  InterpretationAlgebra, and the witness accepts exactly a five-byte result.

Scaffold boundary - INVARIANTS P5 (Progress Is Dissolution): delete this
script and the ci.yml step when the modeled T-22 runner executes
src/v4/test/claim/manual/eval_runtime_mvp.dag on main CI and reports the same
RuntimeValue witness through TestClaimRun or workflow-as-data, with no
scripts-owned generated-Rust receipt standing between the claim and the gate.
"""

from __future__ import annotations

import os
import re
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOT = ROOT / "src/v4"
DEFAULT_COMPILER = ROOT / "target/release/v2-compiler"


class ReceiptError(Exception):
    pass


def require(source: str, needle: str, path: Path, label: str) -> None:
    if needle not in source:
        raise ReceiptError(f"{path}: missing {label}: {needle}")


def require_regex(source: str, pattern: str, path: Path, label: str) -> None:
    if re.search(pattern, source, re.MULTILINE) is None:
        raise ReceiptError(f"{path}: missing {label}: /{pattern}/")


def require_order(source: str, needles: list[str], path: Path, label: str) -> None:
    offset = 0
    for needle in needles:
        found = source.find(needle, offset)
        if found == -1:
            raise ReceiptError(f"{path}: missing ordered {label}: {needle}")
        offset = found + len(needle)


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


def run_v4_compile(output_dir: Path) -> None:
    compiler = Path(os.environ.get("V2_COMPILER", str(DEFAULT_COMPILER)))
    if not compiler.exists():
        raise ReceiptError(
            f"{compiler}: missing v2 compiler; build with `cargo build -p v2-compiler --release`"
        )
    cmd = [
        str(compiler),
        "compile",
        "--source-root",
        str(SOURCE_ROOT),
        "--output-dir",
        str(output_dir),
        "--target",
        "rust",
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        raise ReceiptError(
            "v4 bootstrap compile failed before host-eval receipt\n"
            f"command: {' '.join(cmd)}\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )
    output = proc.stdout + proc.stderr
    require_regex(
        output,
        r"^compiled: [0-9]+ files emitted, 0 diagnostics$",
        Path("v2-compiler"),
        "zero-diagnostic v4 Rust emission",
    )


def check_generated_eval(eval_rs: Path) -> None:
    source = eval_rs.read_text()
    eval_body = generated_function(source, "eval", eval_rs)
    require_order(
        eval_body,
        [
            "pub fn eval(tree: &Rc<InferredTree>, interpretation: Rc<InterpretationAlgebra>, inputs: &Rc<Inputs>) -> Rc<Outcome>",
            "well_formed(tree.root.clone())",
            "well_formed(inputs.root.clone())",
            "eval_runtime_node(inputs.root.clone(), tree.clone(), interpretation, empty_evaluation_environment(), eval_runtime())",
        ],
        eval_rs,
        "compiled eval(tree, interpretation, inputs) dispatch",
    )

    runtime_body = generated_function(source, "eval_runtime_node", eval_rs)
    require_order(
        runtime_body,
        [
            "fold_node(node, &Rc::new(NodeFold",
            "init: Rc::new(|n0| eval_fold_init",
            "step: Rc::new(|acc, e, child| eval_fold_step",
            "eval_fold_state_value(folded)",
        ],
        eval_rs,
        "compiled runtime-node fold",
    )

    init_body = generated_function(source, "eval_fold_init", eval_rs)
    require_order(
        init_body,
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

    step_body = generated_function(source, "eval_fold_step", eval_rs)
    require_order(
        step_body,
        [
            "eval_edge_is_runtime_argument(edge)",
            "eval_append_child_value",
            "eval_absorb_child_diagnostics",
            "eval_maybe_complete_node",
        ],
        eval_rs,
        "compiled child value accumulation",
    )

    computation_body = generated_function(source, "eval_computation_node", eval_rs)
    require_order(
        computation_body,
        [
            "v2_rt::interpret(eval_behavior_dispatch(behavior)",
            "tree",
            "interpretation",
            "environment",
            "runtime",
            "eval_accept_runtime_value_with_facts",
        ],
        eval_rs,
        "compiled behavior dispatch through runtime-owned InterpretationAlgebra",
    )

    branch_body = generated_function(source, "eval_branch_node", eval_rs)
    require_order(
        branch_body,
        [
            "eval_first_runtime_argument",
            "v2_rt::choose_branch",
            "eval_runtime_node(chosen.clone(), tree.clone(), interpretation.clone(), environment.clone(), runtime.clone())",
        ],
        eval_rs,
        "compiled Branch interpreter chooses and resumes through selected subgraph",
    )

    bind_body = generated_function(source, "eval_bind_node", eval_rs)
    require_order(
        bind_body,
        [
            "eval_bind_key",
            "eval_bind_value_arg",
            "v2_rt::bind_value",
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

    allocate_body = generated_function(source, "eval_mvp2_allocate_literal", fixture_rs)
    require_order(
        allocate_body,
        ["Outcome::Accepted", "value: eval_mvp2_two_value()", "diagnostics: None"],
        fixture_rs,
        "compiled literal interpreter returns two-byte RuntimeValue",
    )

    arg_body = generated_function(source, "eval_mvp2_arg_is_two_literal", fixture_rs)
    require_order(
        arg_body,
        [
            "RuntimeValue::RuntimePrimitive",
            "p.primitive_type.clone() == eval_mvp2_i64_node()",
            "p.bytes.clone().len() as i64) == 2",
        ],
        fixture_rs,
        "compiled two-byte argument predicate",
    )

    args_body = generated_function(source, "eval_mvp2_args_are_two_literals", fixture_rs)
    require_order(
        args_body,
        [
            "args.clone().len() as i64) == 2",
            "eval_mvp2_arg_is_two_literal(left.clone())",
            "eval_mvp2_arg_is_two_literal(right.clone())",
        ],
        fixture_rs,
        "compiled two-argument predicate",
    )

    call_body = generated_function(source, "eval_mvp2_call_primitive", fixture_rs)
    require_order(
        call_body,
        [
            "if eval_mvp2_args_are_two_literals(&args)",
            "Outcome::Accepted",
            "value: eval_mvp2_five_value()",
            "diagnostics: None",
            "Outcome::Rejected",
        ],
        fixture_rs,
        "compiled transform interpreter fails closed and accepts five",
    )

    algebra_body = generated_function(source, "eval_mvp2_interpretation_algebra", fixture_rs)
    require_order(
        algebra_body,
        [
            "allocate_literal: Rc::new(eval_mvp2_allocate_literal)",
            "call_primitive: Rc::new(eval_mvp2_call_primitive)",
        ],
        fixture_rs,
        "compiled InterpretationAlgebra binds fixture interpreters",
    )

    actual_body = generated_function(source, "eval_mvp2_actual", fixture_rs)
    require_order(
        actual_body,
        [
            "eval(&eval_mvp2_inferred_tree(), eval_mvp2_interpretation_algebra(), &Rc::new(Inputs",
            "root: eval_mvp2_add_subgraph()",
        ],
        fixture_rs,
        "compiled eval(tree, interpretation, inputs) invocation",
    )

    witness_body = generated_function(source, "witness_eval_mvp2_add_accepts_five", fixture_rs)
    require_order(
        witness_body,
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


def main() -> None:
    try:
        with tempfile.TemporaryDirectory(prefix="gunbc-v4-host-eval-") as tmp:
            output_dir = Path(tmp)
            run_v4_compile(output_dir)
            check_generated_eval(output_dir / "src/v4_compiler_eval.rs")
            check_generated_fixture(output_dir / "src/v4_test_claim_manual_eval_runtime_mvp.rs")
    except ReceiptError as err:
        raise SystemExit(f"error: {err}") from err

    print(
        "T-22 host-eval receipt: compiled eval(tree, interpretation, inputs) "
        "drives the MVP RuntimeValue witness to Accepted five-byte result"
    )


if __name__ == "__main__":
    main()
