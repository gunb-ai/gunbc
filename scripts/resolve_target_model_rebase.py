#!/usr/bin/env python3
"""Resolve target_model.dag rebase conflicts: keep BOTH EffectApply and Conditional."""
from pathlib import Path
import re
import sys

p = Path(sys.argv[1] if len(sys.argv) > 1 else "src/v4/std/compilers/target_model.dag")
text = p.read_text()
if "<<<<<<<" not in text:
    sys.exit(0)

# Template enum: both arms
text = re.sub(
    r"<<<<<<< [^\n]+\n  \| ValueEffectApply\n=======\n  \| ValueConditional\n>>>>>>> [^\n]+",
    "  | ValueEffectApply\n  | ValueConditional",
    text,
)

# Projection fields: both
text = re.sub(
    r"<<<<<<< [^\n]+\n  effect_apply_form: TargetEffectApplyShape\n=======\n  conditional_form: TargetConditionalShape\n>>>>>>> [^\n]+",
    "  effect_apply_form: TargetEffectApplyShape\n  conditional_form: TargetConditionalShape",
    text,
)

# Types: EffectApply block + Conditional shape (no stray closing brace)
text = re.sub(
    r"<<<<<<< [^\n]+\n(// T6 — per-language applied-effect projection\.[\s\S]*?separator: Symbol\n)\n=======\ntype TargetConditionalShape \{[^}]+\}\n>>>>>>> [^\n]+\n\}",
    r"\1\n\ntype TargetConditionalShape {\n  if_token: Symbol\n  then_token: Symbol\n  else_token: Symbol\n}",
    text,
    count=1,
)

# Decode chain nested bind
old_decode = re.compile(
    r"<<<<<<< [^\n]+\n"
    r"(\s+name: target_value_expr_field_effect_apply\n"
    r"\s+\),\n"
    r"\s+f: fn\(effect_bundle\) \{[\s\S]*?"
    r"f: fn\(effect_apply_form\) \{\n)"
    r"=======\n"
    r"\s+name: target_value_expr_field_conditional\n"
    r"\s+\),\n"
    r"\s+f: fn\(conditional_bundle\) \{[\s\S]*?"
    r"f: fn\(conditional_form\) \{\n"
    r">>>>>>> [^\n]+\n"
    r"(\s+outcome_accepted\(\n"
    r"\s+TargetValueExpressionProjection \{[\s\S]*?"
    r"callable_apply_form: callable_apply_form,\n)"
    r"<<<<<<< [^\n]+\n"
    r"\s+effect_apply_form: effect_apply_form\n"
    r"=======\n"
    r"\s+conditional_form: conditional_form\n"
    r">>>>>>> [^\n]+\n"
    r"(\s+\}\n"
    r"\s+\)\n)",
    re.M,
)

def decode_repl(m):
    indent = "                                      "
    return (
        m.group(1)
        + indent
        + "bind_outcome(\n"
        + indent
        + "  o: projection_bundle_child(\n"
        + indent
        + "    bundle: bundle,\n"
        + indent
        + "    name: target_value_expr_field_conditional\n"
        + indent
        + "  ),\n"
        + indent
        + "  f: fn(conditional_bundle) {\n"
        + indent
        + "    bind_outcome(\n"
        + indent
        + "      o: decode_conditional_shape_bundle(\n"
        + indent
        + "        bundle: conditional_bundle\n"
        + indent
        + "      ),\n"
        + indent
        + "      f: fn(conditional_form) {\n"
        + m.group(2)
        + "                                          effect_apply_form: effect_apply_form,\n"
        + "                                          conditional_form: conditional_form\n"
        + m.group(3)
    )

text, n = old_decode.subn(decode_repl, text, count=1)

# Effect helpers + conditional constructor: keep both blocks
while True:
    m = re.search(
        r"<<<<<<< [^\n]+\n// Carrier wire-schema guard for the effect-apply[\s\S]*?"
        r"=======\n(fn target_value_expression_conditional\([\s\S]*?\n\}\n\n)"
        r">>>>>>> [^\n]+\n",
        text,
    )
    if not m:
        break
    effect = m.group(0).split("=======\n")[0]
    effect = re.sub(r"^<<<<<<< [^\n]+\n", "", effect)
    cond = m.group(1)
    text = text[: m.start()] + effect + cond + text[m.end() :]

# Emit match arms
text = re.sub(
    r"<<<<<<< [^\n]+\n"
    r"    TargetValueExprEffectApply =>\n"
    r"      target_value_expression_effect_apply_to_concrete_tokens\(\n"
    r"        expr: expr,\n"
    r"        projection: projection\n"
    r"=======\n"
    r"    TargetValueExprSymbolIdentity =>\n"
    r"      target_value_expression_symbol_identity_to_concrete_tokens\(expr: expr\)\n"
    r"    TargetValueExprConditional =>\n"
    r"      target_value_expression_conditional_to_concrete_tokens\(\n"
    r"        expr: expr,\n"
    r"        projection: projection,\n"
    r"        catalog: catalog\n"
    r">>>>>>> [^\n]+\n"
    r"      \)",
    "    TargetValueExprEffectApply =>\n"
    "      target_value_expression_effect_apply_to_concrete_tokens(\n"
    "        expr: expr,\n"
    "        projection: projection\n"
    "      )\n"
    "    TargetValueExprSymbolIdentity =>\n"
    "      target_value_expression_symbol_identity_to_concrete_tokens(expr: expr)\n"
    "    TargetValueExprConditional =>\n"
    "      target_value_expression_conditional_to_concrete_tokens(\n"
    "        expr: expr,\n"
    "        projection: projection,\n"
    "        catalog: catalog\n"
    "      )",
    text,
    count=1,
)

# Template reject arms
text = re.sub(
    r"<<<<<<< [^\n]+\n    ValueEffectApply =>\n=======\n    ValueConditional =>\n>>>>>>> [^\n]+",
    "    ValueEffectApply =>\n    ValueConditional =>",
    text,
)

# Projection bundle serialize: both edges
while True:
    m = re.search(
        r"<<<<<<< [^\n]+\n"
        r"(        name: target_value_expr_field_effect_apply,[\s\S]*?)"
        r"=======\n"
        r"(        name: target_value_expr_field_conditional,[\s\S]*?)"
        r">>>>>>> [^\n]+\n"
        r"(        \)\n)",
        text,
    )
    if not m:
        break
    main_part = m.group(1).rstrip()
    if not main_part.endswith(")"):
        main_part += "\n        )"
    cond_part = m.group(2).rstrip()
    text = (
        text[: m.start()]
        + main_part
        + "\n      ),\n      target_model_named_edge(\n"
        + cond_part
        + "\n"
        + m.group(3)
        + text[m.end() :]
    )

# Constructor-only conflict (effect_apply already present)
text = re.sub(
    r"<<<<<<< [^\n]+\n=======\n(fn target_value_expression_conditional\([\s\S]*?\n\}\n\n)>>>>>>> [^\n]+\n",
    r"\1",
    text,
)

remaining = text.count("<<<<<<<")
if remaining:
    print(f"WARNING: {remaining} conflicts remain", file=sys.stderr)
    for i, line in enumerate(text.splitlines(), 1):
        if line.startswith("<<<<<<<"):
            print(f"  line {i}", file=sys.stderr)
    sys.exit(1)

p.write_text(text)
print("resolved", p)
