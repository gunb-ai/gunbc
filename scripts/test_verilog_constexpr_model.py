#!/usr/bin/env python3
"""Structural checks for the Verilog constant_expression P8 carrier."""

from __future__ import annotations

import pathlib
import re


ROOT = pathlib.Path(__file__).resolve().parents[1]
VERILOG = ROOT / "src/v4/extdeps/languages/verilog.dag"


def block(source: str, name: str) -> str:
    match = re.search(rf"^type {name}\b(?P<body>.*?)(?=^\S|\Z)", source, re.M | re.S)
    assert match is not None, f"missing type {name}"
    return match.group("body")


def test_constant_function_call_carries_attributes() -> None:
    body = block(VERILOG.read_text(encoding="utf-8"), "ConstantFunctionCall")
    assert "function_name: Symbol" in body
    assert "attributes: AttributeInstanceClause" in body
    assert "arguments: List<ConstantExpression>" in body


def test_unary_constant_expression_takes_constant_primary() -> None:
    body = block(VERILOG.read_text(encoding="utf-8"), "ConstantExpression")
    assert (
        "| ConstantUnaryExpression { attributes: AttributeInstanceClause, "
        "op: ConstantUnaryOperator, operand: ConstantPrimary }"
    ) in body
    assert "operand: ConstantExpression" not in body


def test_constant_primary_carries_parenthesized_single_expression() -> None:
    body = block(VERILOG.read_text(encoding="utf-8"), "ConstantPrimary")
    assert (
        "| ConstantParenthesizedExpressionPrimary { expression: ConstantExpression }"
    ) in body
    assert (
        "| ConstantMinTypMaxPrimary { min: ConstantExpression, "
        "typ: ConstantExpression, max: ConstantExpression }"
    ) in body


if __name__ == "__main__":
    test_constant_function_call_carries_attributes()
    test_unary_constant_expression_takes_constant_primary()
    test_constant_primary_carries_parenthesized_single_expression()
    print("OK: scripts/test_verilog_constexpr_model.py")
