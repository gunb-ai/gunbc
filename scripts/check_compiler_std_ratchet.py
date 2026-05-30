#!/usr/bin/env python3
"""Compiler-std consolidation ratchet (#642)."""

from __future__ import annotations

import re
import sys

from ci_process import repo_root


def inventory_bucket_duplicates() -> list[tuple[str, str, str]]:
    seen: dict[str, str] = {}
    duplicates: list[tuple[str, str, str]] = []
    for label, inventory in (
        ("tracked", TRACKED_ROWS),
        ("positive", POSITIVE_ROWS),
        ("exempt", EXEMPT_ROWS),
    ):
        for row in inventory:
            previous = seen.get(row)
            if previous is not None:
                duplicates.append((row, previous, label))
            else:
                seen[row] = label
    return duplicates


BASELINE_TRACKED_TOTAL = 4
SURFACES = ["src/v3/compiler/*.dag", "src/v3/lenses/*.dag"]

POSITIVE_ROWS = [
    "src/v3/compiler/pipeline.dag:CompilerHostRealization",
    "src/v3/compiler/pipeline.dag:PipelineSnapshotKind",
    "src/v3/compiler/pipeline.dag:PipelineStageBinding",
    "src/v3/compiler/regen.dag:LensRegistryEntry",
    "src/v3/lenses/complexity.dag:Certainty",
    "src/v3/lenses/complexity.dag:ComplexityEnforcedApplication",
    "src/v3/lenses/complexity.dag:ComplexityEntry",
    "src/v3/lenses/complexity.dag:ComplexitySummary",
    "src/v3/lenses/complexity.dag:DominanceOutcome",
    "src/v3/lenses/cost.dag:CostBasisKind",
    "src/v3/lenses/cost.dag:CostBasisDeclaration",
    "src/v3/lenses/cost.dag:SymbolicCostEntry",
    "src/v3/lenses/effect_enumeration.dag:StructuralEffectShape",
    "src/v3/lenses/effect_enumeration.dag:EffectFact",
    "src/v3/lenses/effect_enumeration.dag:CoverageGap",
    "src/v3/lenses/effect_enumeration.dag:RedundantReadError",
    "src/v3/lenses/effect_enumeration.dag:TransactionalPattern",
    "src/v3/lenses/effect_enumeration.dag:EffectEnumerationReport",
    "src/v3/lenses/effect_enumeration.dag:EffectEnumerationAuthoritySite",
    "src/v3/lenses/infer_helpers.dag:TemplateArgumentBinding",
    "src/v3/lenses/parallelism.dag:LinearBranchesLookup",
    "src/v3/lenses/parallelism.dag:ParallelismMode",
    "src/v3/lenses/provenance.dag:Origin",
    "src/v3/lenses/structural_resolution.dag:UnresolvedArrowBody",
    "src/v3/lenses/structural_resolution.dag:NameKeyedReference",
    "src/v3/lenses/unused_parameters.dag:UnusedParameter",
    "src/v3/lenses/variant_payload.dag:VariantPayloadShape",
    "src/v3/lenses/variant_payload.dag:VariantPayloadShapeLookup",
]

EXEMPT_ROWS = [
    "src/v3/compiler/parse_tables.dag:BinaryOpLevel",
    "src/v3/compiler/parse_tables.dag:BinaryOpRow",
    "src/v3/compiler/parse_tables.dag:TopLevelItemKwRow",
    "src/v3/compiler/parse_tables.dag:SoftKeywordIdentRow",
    "src/v3/compiler/parse_tables.dag:BracketRow",
    "src/v3/compiler/parse_tables.dag:PrimaryPrefixRow",
    "src/v3/compiler/parse_tables.dag:PrimaryAtomRow",
]

TRACKED_ROWS = [
    "src/v3/lenses/infer_helpers.dag:TemplateArgumentsMatch",
    "src/v3/lenses/infer_helpers.dag:TemplateArgumentCursor",
    "src/v3/lenses/infer_helpers.dag:NormalizedInstantiationArgs",
    "src/v3/lenses/parallelism.dag:NonCommutingPairLookup",
]


def type_rows(root) -> list[str]:
    rows: list[str] = []
    for pattern in SURFACES:
        for path in sorted(root.glob(pattern)):
            rel = path.relative_to(root).as_posix()
            for line in path.read_text(encoding="utf-8").splitlines():
                match = re.match(r"^type ([A-Z][^ {=]*)", line)
                if match:
                    rows.append(f"{rel}:{match.group(1)}")
    return rows


def main() -> int:
    root = repo_root()
    rows = type_rows(root)
    if not rows:
        print("compiler-std ratchet: no type rows found on the configured surfaces", file=sys.stderr)
        return 1

    classified = [*TRACKED_ROWS, *POSITIVE_ROWS, *EXEMPT_ROWS]
    unknown = sorted(row for row in rows if row not in classified)
    row_counts = {row: rows.count(row) for row in rows}
    violations = False

    duplicate_inventory_rows = inventory_bucket_duplicates()

    missing_or_duplicate_live: list[tuple[str, str, int]] = []
    for label, inventory in (
        ("tracked", TRACKED_ROWS),
        ("positive", POSITIVE_ROWS),
        ("exempt", EXEMPT_ROWS),
    ):
        for row in inventory:
            count = row_counts.get(row, 0)
            if count != 1:
                missing_or_duplicate_live.append((label, row, count))

    if unknown:
        print("compiler-std ratchet: unclassified compiler/lens type rows:", file=sys.stderr)
        for row in unknown:
            print(f"  {row}", file=sys.stderr)
        violations = True
    if duplicate_inventory_rows:
        print("compiler-std ratchet: inventory rows classified in multiple buckets:", file=sys.stderr)
        for row, previous, current in duplicate_inventory_rows:
            print(f"  {row} ({previous}, {current})", file=sys.stderr)
        violations = True
    if missing_or_duplicate_live:
        print("compiler-std ratchet: inventory drift.", file=sys.stderr)
        print("Each classified declaration must exist exactly once in the live counted set.", file=sys.stderr)
        for label, row, count in missing_or_duplicate_live:
            print(f"  {label}: {row} (matches in live set: {count})", file=sys.stderr)
        violations = True
    if len(classified) != len(rows):
        print("compiler-std ratchet: row classification drift.", file=sys.stderr)
        print(f"Counted rows: {len(rows)}", file=sys.stderr)
        print(f"Tracked rows: {len(TRACKED_ROWS)}", file=sys.stderr)
        print(f"Positive-def rows: {len(POSITIVE_ROWS)}", file=sys.stderr)
        print(f"Exempt rows: {len(EXEMPT_ROWS)}", file=sys.stderr)
        print("The explicit row inventory must cover the full counted surface.", file=sys.stderr)
        violations = True
    if len(TRACKED_ROWS) != BASELINE_TRACKED_TOTAL:
        print(
            f"compiler-std ratchet: tracked total {len(TRACKED_ROWS)} != baseline {BASELINE_TRACKED_TOTAL}",
            file=sys.stderr,
        )
        violations = True

    if violations:
        return 1

    print(
        "compiler-std ratchet: OK "
        f"(total={len(rows)}, positive={len(POSITIVE_ROWS)}, exempt={len(EXEMPT_ROWS)}, "
        f"tracked={len(TRACKED_ROWS)}/{BASELINE_TRACKED_TOTAL})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
