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


def test_constant_system_function_call_carries_optional_arguments_list() -> None:
    body = block(VERILOG.read_text(encoding="utf-8"), "ConstantSystemFunctionCall")
    assert "system_function_name: Symbol" in body
    assert "arguments: List<ConstantExpression>" in body


def test_xnor_operator_spelling_variants_are_preserved() -> None:
    source = VERILOG.read_text(encoding="utf-8")

    unary = block(source, "ConstantUnaryOperator")
    assert "| ConstUnaryReductionXnorTildeCaret" in unary
    assert "| ConstUnaryReductionXnorCaretTilde" in unary
    assert "| ConstUnaryReductionXnor\n" not in unary

    binary = block(source, "ConstantBinaryOperator")
    assert "| ConstBinaryBitwiseXnorTildeCaret" in binary
    assert "| ConstBinaryBitwiseXnorCaretTilde" in binary
    assert "| ConstBinaryBitwiseXnor\n" not in binary


def test_constant_range_expression_carries_plain_expression_alternative() -> None:
    body = block(VERILOG.read_text(encoding="utf-8"), "ConstantRangeExpression")
    assert "= ConstantRangeSingle { expression: ConstantExpression }" in body
    assert "| ConstantRange { msb: ConstantExpression, lsb: ConstantExpression }" in body
    assert (
        "| ConstantIndexedRangePlus { base: ConstantExpression, "
        "width: ConstantExpression }"
    ) in body
    assert (
        "| ConstantIndexedRangeMinus { base: ConstantExpression, "
        "width: ConstantExpression }"
    ) in body


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


def test_constant_select_shapes_match_parameter_and_specparam_grammar() -> None:
    select = block(VERILOG.read_text(encoding="utf-8"), "ConstantSelect")
    assert "= NoConstantSelect" in select
    assert "| ConstantSelectRange { range: ConstantRangeExpression }" in select
    assert "bit_selects: List<ConstantExpression>" not in select

    primary = block(VERILOG.read_text(encoding="utf-8"), "ConstantPrimary")
    assert "| ConstantParameterReference { name: Symbol, select: ConstantSelect }" in primary
    assert (
        "| ConstantSpecparamReference { name: Symbol, select: ConstantSelect }"
    ) in primary


def test_vector_range_uses_constant_expression_endpoints() -> None:
    body = block(VERILOG.read_text(encoding="utf-8"), "VectorRange")
    assert "msb: ConstantExpression" in body
    assert "lsb: ConstantExpression" in body
    assert "msb_lexeme" not in body
    assert "lsb_lexeme" not in body


if __name__ == "__main__":
    test_constant_function_call_carries_attributes()
    test_constant_system_function_call_carries_optional_arguments_list()
    test_xnor_operator_spelling_variants_are_preserved()
    test_constant_range_expression_carries_plain_expression_alternative()
    test_unary_constant_expression_takes_constant_primary()
    test_constant_primary_carries_parenthesized_single_expression()
    test_constant_select_shapes_match_parameter_and_specparam_grammar()
    test_vector_range_uses_constant_expression_endpoints()
    print("OK: scripts/test_verilog_constexpr_model.py")
