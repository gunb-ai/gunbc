#!/usr/bin/env python3
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

lit_path = ROOT / "src/v4/std/refinement_authoritative_literals.dag"
lit = lit_path.read_text()

rows = []
for m in re.finditer(
    r"data (authoritative_\w+): (\w+) = discharge_(\w+)\(\s*receipt: (\w+)\(value: (-?\d+)\)\s*\)",
    lit,
    re.MULTILINE,
):
    rows.append(
        {
            "name": m.group(1),
            "type": m.group(2),
            "discharge": m.group(3),
            "literal_fn": m.group(4),
            "value": m.group(5),
        }
    )

replacements = {
    r["name"]: f"discharge_{r['discharge']}(receipt: {r['literal_fn']}(value: {r['value']}))"
    for r in rows
}

fn_end = lit.index("\ndata authoritative_")
lit_path.write_text(lit[:fn_end].rstrip() + "\n")

for fp in (ROOT / "src/v4/extdeps/formatters").glob("*.dag"):
    text = fp.read_text()
    if "refinement_authoritative_literals" not in text:
        continue

    used_names = [n for n in replacements if re.search(rf"\b{n}\b", text)]
    if not used_names:
        continue

    used_discharges = sorted(
        {f"discharge_{r['discharge']}" for r in rows if r["name"] in used_names}
    )
    used_literals = sorted({r["literal_fn"] for r in rows if r["name"] in used_names})

    for name in sorted(used_names, key=len, reverse=True):
        text = re.sub(rf"\b{name}\b", replacements[name], text)

    ref_types = []
    m = re.search(r"import v4\.std\.refinement \{([^}]+)\}", text, re.DOTALL)
    if m:
        ref_types = [x.strip() for x in m.group(1).split(",") if x.strip()]

    text = re.sub(
        r"import v4\.std\.refinement_authoritative_literals \{[^}]+\}\n",
        "import v4.std.refinement_authoritative_literals {\n  "
        + ",\n  ".join(used_discharges)
        + ",\n}\n",
        text,
        count=1,
    )

    if m:
        existing = set(ref_types)
        for lf in used_literals:
            existing.add(lf)
        new_import = "import v4.std.refinement {\n  " + ",\n  ".join(sorted(existing)) + ",\n}\n"
        text = re.sub(r"import v4\.std\.refinement \{[^}]+\}\n", new_import, text, count=1, flags=re.DOTALL)
    else:
        insert = "import v4.std.refinement {\n  " + ",\n  ".join(used_literals) + ",\n}\n"
        text = text.replace(
            "import v4.std.refinement_authoritative_literals {\n  "
            + ",\n  ".join(used_discharges)
            + ",\n}\n",
            "import v4.std.refinement_authoritative_literals {\n  "
            + ",\n  ".join(used_discharges)
            + ",\n}\n"
            + insert,
        )

    fp.write_text(text)
    print(f"updated {fp.name}: {len(used_names)} inlines")

witness_path = ROOT / "src/v4/test/claim/manual/refinement_formatter_literals_accept.dag"

positive_values = sorted(
    {int(r["value"]) for r in rows if r["discharge"] == "positive_int_catalog_row"},
    key=lambda v: (v < 0, v),
)
non_negative_values = sorted(
    {int(r["value"]) for r in rows if r["discharge"] == "non_negative_int_catalog_row"},
    key=lambda v: (v < 0, v),
)
at_least_minus_one_values = sorted(
    {int(r["value"]) for r in rows if r["discharge"] == "at_least_minus_one_int_catalog_row"},
    key=lambda v: (v < 0, v),
)
at_least_minus_two_values = sorted(
    {int(r["value"]) for r in rows if r["discharge"] == "at_least_minus_two_int_catalog_row"},
    key=lambda v: (v < 0, v),
)
exact_one_values = sorted(
    {int(r["value"]) for r in rows if r["discharge"] == "exact_one_int_catalog_row"},
    key=lambda v: (v < 0, v),
)


def gen_family(make_fn, literal_fn, discharge_fn, match_fn, values):
    out = []
    for v in values:
        safe = str(v).replace("-", "minus_")
        witness = f"witness_{literal_fn}_{safe}_discharge"
        claim = f"claim_{literal_fn}_{safe}_discharge"
        out.append(
            f"""fn {witness}() -> Bool {{
  match {make_fn}(value: {v}) {{
    Accepted {{ value: accepted, diagnostics: _ }} =>
      {match_fn}(
        literal: {discharge_fn}(receipt: {literal_fn}(value: {v})),
        accepted: accepted
      )
    Rejected {{ diagnostics: _ }} => false
  }}
}}


fn {claim}_input(ok: Bool) -> Node {{
  if ok {{ stub_empty_conj }} else {{ stub_empty_disj }}
}}

data {claim}: TestClaim = EqualsClaim {{
  label: "manual/refinement: {literal_fn}(value: {v}) catalog discharge matches {make_fn}",
  anchor: manual_claim_anchor(anchor: ManualAnchorAbsent),
  lhs: {claim}_input(ok: {witness}()),
  rhs: stub_empty_conj,
  classification: TestClassification {{ tier: Tier1, unit: Unit }},
}}

"""
        )
    return "".join(out)


body = """// src/v4/test/claim/manual/refinement_formatter_literals_accept.dag
// Scope: formatter upstream default literals — inline catalog discharge matches Accepted make_*.
// Status: T-25-core authoritative literal discharge witnesses.


module v4.test.claim.manual.refinement_formatter_literals_accept


import v4.std.refinement {
  PositiveInt,
  NonNegativeInt,
  AtLeastMinusOneInt,
  AtLeastMinusTwoInt,
  ExactOneInt,
  make_at_least_minus_one_int_at_authoritative,
  make_at_least_minus_two_int_at_authoritative,
  make_exact_one_int_at_authoritative,
  make_non_negative_int_at_authoritative,
  make_positive_int_at_authoritative,
  at_least_minus_one_int_authoritative_literal,
  at_least_minus_two_int_authoritative_literal,
  exact_one_int_authoritative_literal,
  non_negative_int_authoritative_literal,
  positive_int_authoritative_literal,
  refined_base,
}
import v4.std.refinement_authoritative_literals {
  discharge_at_least_minus_one_int_catalog_row,
  discharge_at_least_minus_two_int_catalog_row,
  discharge_exact_one_int_catalog_row,
  discharge_non_negative_int_catalog_row,
  discharge_positive_int_catalog_row,
}
import v4.std.diagnostic { Accepted, Rejected }
import v4.std.node { Node }
import v4.std.verification {
  EqualsClaim,
  ManualAnchorAbsent,
  TestClaim,
  TestClassification,
  Tier1,
  Unit,
  manual_claim_anchor,
}
import v4.test.claim.manual.connective_anchors {
  stub_empty_conj,
  stub_empty_disj,
}

fn positive_int_authoritative_literal_matches_accepted(
  literal: PositiveInt,
  accepted: PositiveInt,
) -> Bool {
  refined_base(r: literal.refined) == refined_base(r: accepted.refined)
}

fn non_negative_int_authoritative_literal_matches_accepted(
  literal: NonNegativeInt,
  accepted: NonNegativeInt,
) -> Bool {
  refined_base(r: literal.refined) == refined_base(r: accepted.refined)
}

fn at_least_minus_one_int_authoritative_literal_matches_accepted(
  literal: AtLeastMinusOneInt,
  accepted: AtLeastMinusOneInt,
) -> Bool {
  refined_base(r: literal.refined) == refined_base(r: accepted.refined)
}

fn at_least_minus_two_int_authoritative_literal_matches_accepted(
  literal: AtLeastMinusTwoInt,
  accepted: AtLeastMinusTwoInt,
) -> Bool {
  refined_base(r: literal.refined) == refined_base(r: accepted.refined)
}

fn exact_one_int_authoritative_literal_matches_accepted(
  literal: ExactOneInt,
  accepted: ExactOneInt,
) -> Bool {
  refined_base(r: literal.refined) == refined_base(r: accepted.refined)
}

"""

body += gen_family(
    "make_positive_int_at_authoritative",
    "positive_int_authoritative_literal",
    "discharge_positive_int_catalog_row",
    "positive_int_authoritative_literal_matches_accepted",
    positive_values,
)
body += gen_family(
    "make_non_negative_int_at_authoritative",
    "non_negative_int_authoritative_literal",
    "discharge_non_negative_int_catalog_row",
    "non_negative_int_authoritative_literal_matches_accepted",
    non_negative_values,
)
body += gen_family(
    "make_at_least_minus_one_int_at_authoritative",
    "at_least_minus_one_int_authoritative_literal",
    "discharge_at_least_minus_one_int_catalog_row",
    "at_least_minus_one_int_authoritative_literal_matches_accepted",
    at_least_minus_one_values,
)
body += gen_family(
    "make_at_least_minus_two_int_at_authoritative",
    "at_least_minus_two_int_authoritative_literal",
    "discharge_at_least_minus_two_int_catalog_row",
    "at_least_minus_two_int_authoritative_literal_matches_accepted",
    at_least_minus_two_values,
)
body += gen_family(
    "make_exact_one_int_at_authoritative",
    "exact_one_int_authoritative_literal",
    "discharge_exact_one_int_catalog_row",
    "exact_one_int_authoritative_literal_matches_accepted",
    exact_one_values,
)

witness_path.write_text(body)
print("done")
