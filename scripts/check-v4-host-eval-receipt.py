#!/usr/bin/env python3
"""CI receipt for the v4 T-22 host-eval MVP fixture."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EVAL = ROOT / "src/v4/compiler/05_eval.dag"
FIXTURE = ROOT / "src/v4/test/claim/manual/eval_runtime_mvp.dag"


def require(source: str, needle: str, path: Path, label: str) -> None:
    if needle not in source:
        raise SystemExit(f"{path}: missing {label}: {needle}")


def require_order(source: str, needles: list[str], path: Path, label: str) -> None:
    offset = 0
    for needle in needles:
        found = source.find(needle, offset)
        if found == -1:
            raise SystemExit(f"{path}: missing ordered {label}: {needle}")
        offset = found + len(needle)


def main() -> None:
    eval_source = EVAL.read_text()
    fixture_source = FIXTURE.read_text()

    require(
        eval_source,
        "fn eval(tree: InferredTree, interpretation: InterpretationAlgebra, inputs: Inputs) -> Outcome<RuntimeValue>",
        EVAL,
        "T-22 eval entrypoint over tree, runtime interpretation, and inputs",
    )
    require(
        eval_source,
        "eval_runtime_node(\n        node: inputs.root,\n        tree: tree,\n        interpretation: interpretation,",
        EVAL,
        "eval dispatch into runtime node interpreter",
    )
    require(
        eval_source,
        "eval_accept_runtime_value_with_facts",
        EVAL,
        "runtime-value conformance check",
    )

    require(
        fixture_source,
        "data witness_eval_mvp2_add_accepts_five: Bool = match eval_mvp2_actual()",
        FIXTURE,
        "host-eval runtime-value witness",
    )
    require_order(
        fixture_source,
        [
            "fn eval_mvp2_actual() -> Outcome<RuntimeValue>",
            "eval(",
            "tree: eval_mvp2_inferred_tree()",
            "interpretation: eval_mvp2_interpretation_algebra()",
            "inputs: Inputs { root: eval_mvp2_add_subgraph() }",
        ],
        FIXTURE,
        "eval(tree, interpretation, inputs) invocation",
    )
    require_order(
        fixture_source,
        [
            "Accepted { value: RuntimePrimitive { value: p }, diagnostics: None }",
            "(p.primitive_type == eval_mvp2_i64_node()) && (count(p.bytes) == 5)",
        ],
        FIXTURE,
        "expected RuntimeValue assertion",
    )
    require(
        fixture_source,
        "fn eval_mvp2_call_primitive(",
        FIXTURE,
        "runtime transform interpreter body",
    )
    require(
        fixture_source,
        "Accepted { value: eval_mvp2_five_value(), diagnostics: None }",
        FIXTURE,
        "five-byte runtime result",
    )

    print("T-22 host-eval receipt: eval(tree, interpretation, inputs) is wired to RuntimeValue witness")


if __name__ == "__main__":
    main()
