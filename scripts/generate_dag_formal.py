#!/usr/bin/env python3
"""Generate dag_wave1_formal_productions() body for dag.dag patch."""
from __future__ import annotations

from pathlib import Path

lines: list[str] = []


def T(tok: str) -> str:
    return f"dag_formal_terminal(token_class: dag_token_{tok})"


def N(nt: str) -> str:
    return f"dag_formal_nonterminal(production: dag_production_{nt})"


def fp(lhs: str, *rhs: str) -> None:
    if not rhs:
        lines.append(f"    dag_formal_production(lhs: dag_production_{lhs}, rhs: []),")
    else:
        inner = ",\n        ".join(rhs)
        lines.append(
            f"    dag_formal_production(\n"
            f"      lhs: dag_production_{lhs},\n"
            f"      rhs: [\n        {inner}\n      ]\n    ),"
        )


def comma_list(lhs: str, item_nt: str, *, trailing: bool = True) -> None:
    """item_nt is production symbol name without dag_production_ prefix."""
    suf = f"{lhs}_suffix"
    fp(lhs, N(item_nt))
    fp(lhs, N(item_nt), T("comma"), N(item_nt), N(suf))
    fp(suf)
    fp(suf, T("comma"), N(item_nt), N(suf))
    if trailing:
        fp(lhs, N(item_nt), T("comma"))
        fp(suf, T("comma"))


def optional_nt(lhs: str, inner_nt: str) -> None:
    fp(lhs)
    fp(lhs, N(inner_nt))


def repeat_suffix(lhs: str, elem_rhs: list[str]) -> None:
    suf = f"{lhs}_suffix"
    fp(lhs, *elem_rhs)
    fp(suf)
    inner = ",\n        ".join(elem_rhs + [N(suf)])
    lines.append(
        f"    dag_formal_production(\n"
        f"      lhs: dag_production_{lhs},\n"
        f"      rhs: [\n        {inner}\n      ]\n    ),"
    )


# --- productions ---
fp("module_header", T("kw_module"), N("qualified_name"))

# qualified_name: ident (. ident)*
fp("qualified_name", T("ident"))
repeat_suffix(
    "qualified_name",
    [T("dot"), T("ident")],
)

fp("generic_params", T("lt"), N("comma_ident_list"), T("gt"))
comma_list("comma_ident_list", "ident")

fp("typed_param", T("ident"), T("colon"), N("type_expr"))

fp("param_list", T("lparen"), T("rparen"))
optional_nt("param_list_opt_typed_params", "comma_typed_param_list")
fp("param_list", T("lparen"), N("param_list_opt_typed_params"), T("rparen"))
comma_list("comma_typed_param_list", "typed_param")

# qualified_type: qname optional < optional type_list >
fp("qualified_type", N("qualified_name"))
fp("qualified_type", N("qualified_name"), N("qualified_type_generics"))
fp("qualified_type_generics", T("lt"), T("gt"))
optional_nt("qualified_type_generics_type_list", "comma_type_expr_list")
fp("qualified_type_generics", T("lt"), N("qualified_type_generics_type_list"), T("gt"))
comma_list("comma_type_expr_list", "type_expr")

fp("fn_type", T("kw_fn"), T("lparen"), T("rparen"), T("arrow"), N("type_expr"))
optional_nt("fn_type_param_types", "comma_type_expr_list")
fp("fn_type", T("kw_fn"), T("lparen"), N("fn_type_param_types"), T("rparen"), T("arrow"), N("type_expr"))

fp("type_expr", N("qualified_type"))
fp("type_expr", N("fn_type"))

fp("import_block", T("lbrace"), N("comma_ident_list"), T("rbrace"))

fp("field_decl", T("ident"), T("colon"), N("type_expr"))

fp("field_decl_block", T("lbrace"), T("rbrace"))
optional_nt("field_decl_block_opt", "comma_field_decl_list")
fp("field_decl_block", T("lbrace"), N("field_decl_block_opt"), T("rbrace"))
comma_list("comma_field_decl_list", "field_decl")

fp("fn_body", T("lbrace"), T("rbrace"))
repeat_suffix("fn_body", [N("stmt")])

fp("stmt", N("let_expr"))
fp("stmt", N("expr"))

fp("let_expr", T("kw_let"), T("ident"), T("eq"), N("expr"))

# binary_expr: pipe chain
fp("binary_expr", N("unary_expr"))
repeat_suffix("binary_expr_pipe", [T("pipe_gt"), N("binary_expr_or_base")])
fp("binary_expr_or_base", N("unary_expr"))
repeat_suffix("binary_expr_or", [T("or_or"), N("binary_expr_and_base")])
fp("binary_expr_and_base", N("unary_expr"))
repeat_suffix("binary_expr_and", [T("and_and"), N("binary_expr_equality_base")])
fp("binary_expr_equality_base", N("binary_expr_comparison_base"))
fp("binary_expr_equality", T("eq_eq"), N("binary_expr_comparison_base"))
fp("binary_expr_equality", T("neq"), N("binary_expr_comparison_base"))
fp("binary_expr_comparison_base", N("binary_expr_additive_base"))
fp("binary_expr_comparison", T("gte"), N("binary_expr_additive_base"))
fp("binary_expr_comparison", T("lte"), N("binary_expr_additive_base"))
fp("binary_expr_comparison", T("lt"), N("binary_expr_additive_base"))
fp("binary_expr_comparison", T("gt"), N("binary_expr_additive_base"))
fp("binary_expr_additive_base", N("unary_expr"))
repeat_suffix("binary_expr_additive", [T("plus"), N("unary_expr")])
repeat_suffix("binary_expr_additive", [T("minus"), N("unary_expr")])
fp("binary_expr_multiplicative_base", N("unary_expr"))
repeat_suffix("binary_expr_multiplicative", [T("star"), N("unary_expr")])
repeat_suffix("binary_expr_multiplicative", [T("slash"), N("unary_expr")])
repeat_suffix("binary_expr_multiplicative", [T("percent"), N("unary_expr")])

# Wire binary_expr to start of chain - use unary as base; simplify: binary_expr = pipe helper top
# Remap: production binary_expr should match grammar - use single entry pointing to or_base chain
# Delete intermediate binary_expr_pipe rules with wrong lhs - fix architecture

lines.clear()

# Simpler approach: lhs dag_production_binary_expr for entire precedence chain using suffix NTs on same lhs
# Actually use dedicated suffix production names that are only in formal grammar

def bin_chain():
    """Emit binary_expr as dag_production_binary_expr with suffix nonterminals."""
    fp("binary_expr", N("unary_expr"))
    # pipe
    suf = "binary_expr"
    fp("binary_expr", N("unary_expr"))
    lines.append(
        "    dag_formal_production(\n"
        "      lhs: dag_production_binary_expr,\n"
        "      rhs: [\n"
        "        dag_formal_nonterminal(production: dag_production_binary_expr),\n"
        "        dag_formal_terminal(token_class: dag_token_pipe_gt),\n"
        "        dag_formal_nonterminal(production: dag_production_unary_expr)\n"
        "      ]\n"
        "    ),"
    )


# Restart with cleaner structure - all on dag_production_binary_expr like rust additive on same lhs

lines.clear()


def repeat_on_lhs(lhs: str, elem: list[str]) -> None:
    suf = f"{lhs}_repeat_suffix"
    fp(lhs, *elem)
    fp(suf)
    inner = ",\n        ".join(elem + [N(suf)])
    lines.append(
        f"    dag_formal_production(\n"
        f"      lhs: dag_production_{lhs},\n"
        f"      rhs: [\n        {inner}\n      ]\n    ),"
    )


# module_header
fp("module_header", T("kw_module"), N("qualified_name"))

fp("qualified_name", T("ident"))
repeat_on_lhs("qualified_name", [T("dot"), T("ident")])

fp("generic_params", T("lt"), N("comma_ident_list"), T("gt"))
comma_list("comma_ident_list", "ident")

fp("typed_param", T("ident"), T("colon"), N("type_expr"))

fp("param_list", T("lparen"), T("rparen"))
fp("param_list", T("lparen"), N("comma_typed_param_list"), T("rparen"))
comma_list("comma_typed_param_list", "typed_param")

fp("qualified_type", N("qualified_name"))
fp("qualified_type", N("qualified_name"), T("lt"), T("gt"))
fp("qualified_type", N("qualified_name"), T("lt"), N("comma_type_expr_list"), T("gt"))
comma_list("comma_type_expr_list", "type_expr")

fp("fn_type", T("kw_fn"), T("lparen"), T("rparen"), T("arrow"), N("type_expr"))
fp("fn_type", T("kw_fn"), T("lparen"), N("comma_type_expr_list"), T("rparen"), T("arrow"), N("type_expr"))

fp("type_expr", N("qualified_type"))
fp("type_expr", N("fn_type"))

fp("import_block", T("lbrace"), N("comma_ident_list"), T("rbrace"))

fp("field_decl", T("ident"), T("colon"), N("type_expr"))

fp("field_decl_block", T("lbrace"), T("rbrace"))
fp("field_decl_block", T("lbrace"), N("comma_field_decl_list"), T("rbrace"))
comma_list("comma_field_decl_list", "field_decl")

fp("fn_body", T("lbrace"), T("rbrace"))
repeat_on_lhs("fn_body", [N("stmt")])

fp("stmt", N("let_expr"))
fp("stmt", N("expr"))

fp("let_expr", T("kw_let"), T("ident"), T("eq"), N("expr"))

# unary
fp("unary_expr", N("postfix_expr"))
fp("unary_expr", T("bang"), N("unary_expr"))

# postfix
fp("postfix_expr", N("primary_expr"))
repeat_on_lhs(
    "postfix_expr",
    [T("dot"), T("ident")],
)

# arg list optional in call - postfix call suffix optional
fp("postfix_call_suffix", T("lparen"), T("rparen"))
fp("postfix_call_suffix", T("lparen"), N("comma_arg_list"), T("rparen"))
comma_list("comma_arg_list", "arg")

fp("postfix_expr", N("primary_expr"), T("dot"), T("ident"))
fp("postfix_expr", N("primary_expr"), T("dot"), T("ident"), N("postfix_call_suffix"))

# primary
fp("primary_expr", T("ident"))
fp("primary_expr", T("ident"), N("primary_ident_suffix"))
fp("primary_ident_suffix")
fp("primary_ident_suffix", N("postfix_call_suffix"))
fp(
    "primary_ident_suffix",
    T("lbrace"),
    N("comma_field_init_list"),
    T("rbrace"),
)
comma_list("comma_field_init_list", "field_init")

fp("primary_expr", T("int_literal"))
fp("primary_expr", T("string_literal"))
fp("primary_expr", T("kw_true"))
fp("primary_expr", T("kw_false"))
fp("primary_expr", T("lbracket"), N("comma_expr_list"), T("rbracket"))
comma_list("comma_expr_list", "expr")
fp("primary_expr", T("lbrace"), N("comma_field_init_list"), T("rbrace"))
fp("primary_expr", T("lparen"), N("expr"), T("rparen"))

fp("block_expr", N("fn_body"))

fp("arg", T("ident"), T("colon"), N("expr"))
fp("arg", N("expr"))

fp("field_init", T("ident"), T("colon"), N("expr"))

fp("match_expr", T("kw_match"), N("binary_expr"), T("lbrace"), T("rbrace"))
repeat_on_lhs("match_expr", [N("match_arm")])

fp("match_arm", N("pattern"), T("fat_arrow"), N("expr"))

fp("pattern", N("constructor_pattern"))
fp("pattern", T("kw_true"))
fp("pattern", T("kw_false"))
fp("pattern", T("int_literal"))
fp("pattern", T("string_literal"))

fp("constructor_pattern", T("ident"))
fp("constructor_pattern", T("ident"), N("constructor_pattern_suffix"))
fp("constructor_pattern_suffix")
fp(
    "constructor_pattern_suffix",
    T("lbrace"),
    N("comma_field_pattern_list"),
    T("rbrace"),
)
fp(
    "constructor_pattern_suffix",
    T("lparen"),
    N("comma_pattern_list"),
    T("rparen"),
)
comma_list("comma_field_pattern_list", "field_pattern")
comma_list("comma_pattern_list", "pattern")

fp("field_pattern", T("ident"), T("colon"), N("pattern"))
fp("field_pattern", T("ident"))

fp("if_expr", N("if_then_form"))
fp("if_expr", N("if_block_form"))

fp("if_then_form", T("kw_if"), N("expr"), T("kw_then"), N("expr"), T("kw_else"), N("expr"))

fp(
    "if_block_form",
    T("kw_if"),
    N("expr"),
    N("fn_body"),
    T("kw_else"),
    N("fn_body"),
)
fp(
    "if_block_form",
    T("kw_if"),
    N("expr"),
    N("fn_body"),
    T("kw_else"),
    N("if_expr"),
)

fp("fn_literal", T("kw_fn"), T("lparen"), T("rparen"), N("fn_body"))
fp("fn_literal", T("kw_fn"), T("lparen"), N("comma_ident_list"), T("rparen"), N("fn_body"))
fp(
    "fn_literal",
    T("kw_fn"),
    T("lparen"),
    T("rparen"),
    T("arrow"),
    N("type_expr"),
    N("fn_body"),
)
fp(
    "fn_literal",
    T("kw_fn"),
    T("lparen"),
    N("comma_ident_list"),
    T("rparen"),
    T("arrow"),
    N("type_expr"),
    N("fn_body"),
)

fp("import_decl", T("kw_import"), N("qualified_name"))
fp("import_decl", T("kw_import"), N("qualified_name"), N("import_block"))

fp("type_variant", N("type_expr"))
fp("type_variant", N("type_expr"), N("field_decl_block"))

fp("type_alias_rhs", N("type_variant"))
repeat_on_lhs("type_alias_rhs", [T("pipe"), N("type_variant")])

fp("type_decl", T("kw_type"), T("ident"))
fp("type_decl", T("kw_type"), T("ident"), N("generic_params"))
fp("type_decl", T("kw_type"), T("ident"), N("field_decl_block"))
fp("type_decl", T("kw_type"), T("ident"), T("eq"), N("type_alias_rhs"))
fp("type_decl", T("kw_type"), T("ident"), N("generic_params"), N("field_decl_block"))
fp("type_decl", T("kw_type"), T("ident"), N("generic_params"), T("eq"), N("type_alias_rhs"))

fp("data_decl", T("kw_data"), T("ident"), T("colon"), N("type_expr"), T("eq"), N("expr"))

fp("fn_decl", T("kw_fn"), T("ident"), N("param_list"), N("fn_body"))
fp("fn_decl", T("kw_fn"), T("ident"), N("param_list"), T("arrow"), N("type_expr"), N("fn_body"))
fp("fn_decl", T("kw_fn"), T("ident"), N("generic_params"), N("param_list"), N("fn_body"))
fp(
    "fn_decl",
    T("kw_fn"),
    T("ident"),
    N("generic_params"),
    N("param_list"),
    T("arrow"),
    N("type_expr"),
    N("fn_body"),
)

fp("top_level_item", N("import_decl"))
fp("top_level_item", N("type_decl"))
fp("top_level_item", N("data_decl"))
fp("top_level_item", N("fn_decl"))

fp("module", N("module_header"))
repeat_on_lhs("module", [N("top_level_item")])

# binary_expr precedence - all on dag_production_binary_expr
lines_bin: list[str] = []


def fpb(*rhs: str) -> None:
    fp("binary_expr", *rhs)


# Reset binary - remove duplicate binary_expr from lines and add proper chain
lines = [l for l in lines if "dag_production_binary_expr" not in l or "dag_production_binary_expr_repeat" in l or "dag_production_binary_expr," not in l]
# Actually filter is messy - regenerate from scratch only non-binary

# Simpler: append binary rules at end before closing
binary_rules = """
    dag_formal_production(lhs: dag_production_binary_expr, rhs: [dag_formal_nonterminal(production: dag_production_unary_expr)]),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_pipe_gt),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_or_or),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_and_and),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_eq_eq),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_neq),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_gte),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_lte),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_lt),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_gt),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_plus),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_minus),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_star),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_slash),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
    dag_formal_production(
      lhs: dag_production_binary_expr,
      rhs: [
        dag_formal_nonterminal(production: dag_production_binary_expr),
        dag_formal_terminal(token_class: dag_token_percent),
        dag_formal_nonterminal(production: dag_production_unary_expr)
      ]
    ),
"""

# Fix match_expr to use binary_expr
lines = [l for l in lines if "dag_production_match_expr," not in l or "match_expr_arms" in l]
for i, l in enumerate(lines):
    if "lhs: dag_production_match_expr," in l and "kw_match" in l:
        lines[i] = l.replace("dag_production_expr", "dag_production_binary_expr").replace(
            "production: dag_production_expr", "production: dag_production_binary_expr"
        )

# Fix module repeat - module uses top_level_item not module for repeat lhs
lines = [l for l in lines if "dag_production_module_repeat" not in l]

body = "\n".join(lines) + binary_rules

# suffix data symbols
suffix_syms = [
    "qualified_name_repeat_suffix",
    "comma_ident_list_suffix",
    "comma_typed_param_list_suffix",
    "comma_type_expr_list_suffix",
    "comma_field_decl_list_suffix",
    "fn_body_repeat_suffix",
    "postfix_expr_repeat_suffix",
    "comma_arg_list_suffix",
    "comma_field_init_list_suffix",
    "comma_expr_list_suffix",
    "match_expr_arms_repeat_suffix",
    "comma_field_pattern_list_suffix",
    "comma_pattern_list_suffix",
    "type_alias_rhs_repeat_suffix",
    "module_repeat_suffix",
    "primary_ident_suffix",
    "postfix_call_suffix",
    "constructor_pattern_suffix",
    "if_then_form",
    "if_block_form",
    "param_list_opt_typed_params",
    "comma_ident_list",
    "comma_typed_param_list",
    "comma_type_expr_list",
    "comma_field_decl_list",
    "comma_arg_list",
    "comma_field_init_list",
    "comma_expr_list",
    "comma_field_pattern_list",
    "comma_pattern_list",
    "match_expr_arms",
    "constructor_pattern",
]

print("// AUTO-GENERATED suffix symbols count:", len(suffix_syms))
print("// production lines:", len(lines))
Path("/tmp/dag_formal_body.txt").write_text(body)
print("wrote /tmp/dag_formal_body.txt", len(body), "chars")
