#!/usr/bin/env python3
"""Compiler-std consolidation ratchet (#642)."""

from __future__ import annotations

import re
import sys

from ci_process import repo_root


BASELINE_TRACKED_TOTAL = 4
SURFACES = ["src/v3/compiler/*.dag", "src/v3/lenses/*.dag"]

POSITIVE_ROWS = {
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
}

EXEMPT_ROWS = {
    "src/v3/compiler/parse_tables.dag:BinaryOpLevel",
    "src/v3/compiler/parse_tables.dag:BinaryOpRow",
    "src/v3/compiler/parse_tables.dag:TopLevelItemKwRow",
    "src/v3/compiler/parse_tables.dag:SoftKeywordIdentRow",
    "src/v3/compiler/parse_tables.dag:BracketRow",
    "src/v3/compiler/parse_tables.dag:PrimaryPrefixRow",
    "src/v3/compiler/parse_tables.dag:PrimaryAtomRow",
}

TRACKED_ROWS = {
    "src/v3/lenses/infer_helpers.dag:TemplateArgumentsMatch",
    "src/v3/lenses/infer_helpers.dag:TemplateArgumentCursor",
    "src/v3/lenses/infer_helpers.dag:NormalizedInstantiationArgs",
    "src/v3/lenses/parallelism.dag:NonCommutingPairLookup",
}


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

    classified = TRACKED_ROWS | POSITIVE_ROWS | EXEMPT_ROWS
    unknown = sorted(row for row in rows if row not in classified)
    missing_tracked = sorted(row for row in TRACKED_ROWS if row not in rows)
    missing_positive = sorted(row for row in POSITIVE_ROWS if row not in rows)
    missing_exempt = sorted(row for row in EXEMPT_ROWS if row not in rows)
    violations = False

    if unknown:
        print("compiler-std ratchet: unclassified compiler/lens type rows:", file=sys.stderr)
        for row in unknown:
            print(f"  {row}", file=sys.stderr)
        violations = True
    if missing_tracked or missing_positive or missing_exempt:
        print("compiler-std ratchet: classified rows missing from source:", file=sys.stderr)
        for label, missing in (
            ("tracked", missing_tracked),
            ("positive", missing_positive),
            ("exempt", missing_exempt),
        ):
            for row in missing:
                print(f"  {label}: {row}", file=sys.stderr)
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
