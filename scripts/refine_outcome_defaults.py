#!/usr/bin/env python3
"""Remove catalog discharge; keep Outcome on formatter default fields."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Delete literals module discharge file
lit = ROOT / "src/v4/std/refinement_authoritative_literals.dag"
if lit.exists():
    lit.unlink()

FORMATTER_FILES = [
    ROOT / "src/v4/extdeps/formatters/black.dag",
    ROOT / "src/v4/extdeps/formatters/prettier.dag",
    ROOT / "src/v4/extdeps/formatters/swift_format.dag",
    ROOT / "src/v4/extdeps/formatters/rustfmt.dag",
    ROOT / "src/v4/extdeps/formatters/clang_format.dag",
]

CARRIER_TO_LITERAL = {
    "positive_int": "positive_int_authoritative_literal",
    "non_negative_int": "non_negative_int_authoritative_literal",
    "at_least_minus_one_int": "at_least_minus_one_int_authoritative_literal",
    "at_least_minus_two_int": "at_least_minus_two_int_authoritative_literal",
    "exact_one_int": "exact_one_int_authoritative_literal",
}

CARRIER_TO_TYPE = {
    "positive_int": "PositiveInt",
    "non_negative_int": "NonNegativeInt",
    "at_least_minus_one_int": "AtLeastMinusOneInt",
    "at_least_minus_two_int": "AtLeastMinusTwoInt",
    "exact_one_int": "ExactOneInt",
}

discharge_pat = re.compile(
    r"discharge_(\w+)_catalog_row\(receipt: (\w+)_authoritative_literal\(value: (-?\d+)\), catalog_value: -?\d+\)"
)


def update_formatter(path: Path) -> None:
    text = path.read_text()

    # Remove refinement_authoritative_literals import block
    text = re.sub(
        r"import v4\.std\.refinement_authoritative_literals \{[^}]+\}\n",
        "",
        text,
    )

    # Add Outcome import if missing
    if "Outcome" not in text:
        if "import v4.std.diagnostic" in text:
            text = re.sub(
                r"import v4\.std\.diagnostic \{([^}]+)\}",
                lambda m: f"import v4.std.diagnostic {{{m.group(1).strip()}, Outcome}}",
                text,
                count=1,
            )
        else:
            text = text.replace(
                "import v4.std.patch",
                "import v4.std.diagnostic { Outcome }\nimport v4.std.patch",
                1,
            )

    used_literals = set()
    for m in discharge_pat.finditer(text):
        used_literals.add(m.group(2))

    # Replace discharge calls with authoritative_literal
    text = discharge_pat.sub(
        lambda m: f"{CARRIER_TO_LITERAL[m.group(1)]}(value: {m.group(3)})",
        text,
    )

    # Outcome-wrap refined carrier types in struct/patch/nested types
    for carrier, typename in CARRIER_TO_TYPE.items():
        if carrier not in used_literals and carrier.replace("_", "") not in text:
            continue
        outcome_type = f"Outcome<{typename}>"
        # Field declarations: name: PositiveInt -> Outcome<PositiveInt>
        text = re.sub(
            rf"(: )({typename})(\s|$|\|)",
            rf"\1{outcome_type}\3",
            text,
        )
        text = re.sub(
            rf"FieldPatch<{typename}>",
            f"FieldPatch<{outcome_type}>",
            text,
        )
        text = re.sub(
            rf"count: {typename}",
            f"count: {outcome_type}",
            text,
        )

    path.write_text(text)
    print(f"updated {path.name}: literals={sorted(used_literals)}")


def update_witness(path: Path) -> None:
    text = path.read_text()
    text = re.sub(
        r"import v4\.std\.refinement_authoritative_literals \{[^}]+\}\n",
        "",
        text,
    )
    text = discharge_pat.sub(
        lambda m: f"{CARRIER_TO_LITERAL[m.group(1)]}(value: {m.group(3)})",
        text,
    )
    # Witness helpers: compare via nested match on Outcome literal
    for carrier, typename in CARRIER_TO_TYPE.items():
        literal_fn = CARRIER_TO_LITERAL[carrier]
        match_fn = f"{carrier}_authoritative_literal_matches_accepted"
        old_block = f"""      {match_fn}(
        literal: {literal_fn}(value:"""
        if old_block not in text:
            continue
        # Replace pattern: literal: fn(value: N) -> nested match
        pat = re.compile(
            rf"({match_fn}\(\s*)literal: {literal_fn}\(value: (-?\d+)\), catalog_value: -?\d+\)",
            re.MULTILINE,
        )
        text = pat.sub(
            rf"""\1literal_out: {literal_fn}(value: \2)) {{
      Accepted {{ value: literal, diagnostics: _ }} =>
        {match_fn}(literal: literal,""",
            text,
        )
        # Close extra brace before accepted: accepted
        text = text.replace(
            f"{match_fn}(literal: literal,\n        accepted: accepted",
            f"{match_fn}(literal: literal, accepted: accepted",
        )
        # Add Rejected on literal_out after accepted line - messy; do simpler replace

    # Simpler full rewrite of witness compare calls
    for carrier, typename in CARRIER_TO_TYPE.items():
        literal_fn = CARRIER_TO_LITERAL[carrier]
        match_fn = f"{carrier}_authoritative_literal_matches_accepted"
        text = re.sub(
            rf"{match_fn}\(\s*literal: {literal_fn}\(value: (-?\d+)\), catalog_value: -?\d+\),\s*accepted: accepted\s*\)",
            rf"""match {literal_fn}(value: \1) {{
        Accepted {{ value: literal, diagnostics: _ }} =>
          {match_fn}(literal: literal, accepted: accepted)
        Rejected {{ diagnostics: _ }} => false
      }}""",
            text,
        )

    path.write_text(text)
    print("updated witness file")


def update_refinement_dag() -> None:
    path = ROOT / "src/v4/std/refinement.dag"
    text = path.read_text()
    text = text.replace(
        "// v4.std.refinement_authoritative_literals (catalog discharge is module-local there).\n",
        "// Formatter upstream defaults: *_authoritative_literal (Outcome) on config fields; witnesses in refinement_formatter_literals_accept.dag.\n",
    )
    path.write_text(text)


for f in FORMATTER_FILES:
    update_formatter(f)
update_witness(ROOT / "src/v4/test/claim/manual/refinement_formatter_literals_accept.dag")
update_refinement_dag()
