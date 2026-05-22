#!/usr/bin/env python3
"""T-19 testgen activation gate — generated TestClaim corpus receipts.

Verifies the six-way TestgenConcept arm, testgen emission helpers, and generated claim modules
that exercise LBE runner receipts plus DiagnosticExhaustiveness coproduct-exhaustiveness emission.

Run: python3 scripts/check_t19_testgen_activation.py
Self-test: python3 scripts/test_check_t19_testgen_activation.py
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

TESTGEN = ROOT / "src/v4/lens/testgen.dag"
LBE_GENERATED = ROOT / "src/v4/test/claim/generated/language_behavior_equivalence.dag"
LBE_MANIFEST = ROOT / "src/v4/test/claim/generated/lbe_anchor_manifest.dag"
COPRODUCT_EXHAUSTIVENESS_GENERATED = (
    ROOT / "src/v4/test/claim/generated/coproduct_exhaustiveness.dag"
)
VERIFICATION = ROOT / "src/v4/std/verification.dag"


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _require(path: Path) -> None:
    if not path.is_file():
        raise SystemExit(f"missing required path: {path}")


def _require_substrings(label: str, text: str, needles: tuple[str, ...]) -> None:
    missing = [n for n in needles if n not in text]
    if missing:
        raise SystemExit(f"{label}: missing required substrings: {missing!r}")


def main() -> None:
    for path in (
        TESTGEN,
        LBE_GENERATED,
        LBE_MANIFEST,
        COPRODUCT_EXHAUSTIVENESS_GENERATED,
        VERIFICATION,
    ):
        _require(path)

    testgen = _read(TESTGEN)
    lbe = _read(LBE_GENERATED)
    manifest = _read(LBE_MANIFEST)
    coproduct_exhaustiveness = _read(COPRODUCT_EXHAUSTIVENESS_GENERATED)
    verification = _read(VERIFICATION)

    _require_substrings(
        "testgen.dag",
        testgen,
        (
            "| LanguageBehaviorEquivalence {",
            "type LanguageBehaviorEquivalenceSubject",
            "type FrozenLanguageBehaviorSnapshot",
            "type LanguageBehaviorIoMock",
            "fn testgen_emit_language_behavior_equivalence_claim",
            "fn testgen_scheduled_language_behavior_generators",
            "t19_lbe_label_conj_dag_surface",
            "T19ManualLbeConjDagSurface",
            "T19ManualLbeDisjDagSurface",
            "T19ManualLbeTransformDagSurface",
            "dag_language_model_surface_id",
            "type DiagnosticExhaustivenessSubject",
            "fn coproduct_exhaustiveness_subject_testclaim_compiles",
            "fn testgen_emit_coproduct_exhaustiveness_claim",
            "fn testgen_scheduled_coproduct_exhaustiveness_generators",
            "coproduct_exhaustiveness_subject_testclaim_diagnostic",
            "coproduct_exhaustiveness_subject_testclaim_equals",
            "coproduct_exhaustiveness_subject_testclaim_roundtrip",
            "t19_coproduct_exhaustiveness_missing_variant",
            "omitted_variant: test_claim_coproduct_variant_symbol(variant: subject.omitted_variant)",
            "t19_coproduct_exhaustiveness_omitted_variant_edge",
            "node_locus(node: input)",
        ),
    )

    _require_substrings(
        "verification.dag",
        verification,
        (
            "T19ManualLbeConjDagSurface",
            "T19ManualLbeDisjDagSurface",
            "T19ManualLbeTransformDagSurface",
            "T19GeneratedCoproductExhaustiveness { omitted_variant: Symbol }",
        ),
    )

    _require_substrings(
        "language_behavior_equivalence.dag",
        lbe,
        (
            "fn lbe_claim_from_testgen_emit",
            "-> Outcome<TestClaim>",
            "testgen_emit_language_behavior_equivalence_claim",
            "Fail { actual: Rejected { diagnostics:",
            "run_test_claim_assert",
            "run_test_claim(",
            "data run_lbe_conj_via_run_test_claim: TestClaimRun<Node, RuntimeValue>",
            "data run_lbe_disj_via_run_test_claim: TestClaimRun<Node, RuntimeValue>",
            "data run_lbe_transform_via_run_test_claim: TestClaimRun<Node, RuntimeValue>",
            "witness_lbe_conj_snapshot_pass",
            "witness_lbe_disj_snapshot_pass",
            "witness_lbe_transform_snapshot_pass",
            "witness_testgen_schedules_three_lbe_generators",
            "lbe_io_mock_conj_dag_surface",
            "LanguageBehaviorIoMock",
        ),
    )

    _require_substrings(
        "lbe_anchor_manifest.dag",
        manifest,
        (
            "T19ManualLbeConjDagSurface",
            "T19ManualLbeDisjDagSurface",
            "T19ManualLbeTransformDagSurface",
        ),
    )

    _require_substrings(
        "coproduct_exhaustiveness.dag",
        coproduct_exhaustiveness,
        (
            "fn generated_coproduct_exhaustiveness_claim() -> Outcome<TestClaim>",
            "testgen_emit_coproduct_exhaustiveness_claim",
            "T19GeneratedCoproductExhaustiveness { omitted_variant: _ }",
            "witness_coproduct_exhaustiveness_diagnostic_claim",
            "witness_coproduct_exhaustiveness_uses_generated_anchor",
            "witness_coproduct_exhaustiveness_all_variants_emit",
            "witness_coproduct_exhaustiveness_generator_count",
            "length(xs: testgen_scheduled_coproduct_exhaustiveness_generators()) == 4",
        ),
    )

    if "LanguageBehaviorEquivalence" not in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit("LanguageBehaviorEquivalence must be a TestgenConcept variant, not free text only")

    print("OK: T-19 testgen activation (LBE + coproduct-exhaustiveness generated receipts).")


if __name__ == "__main__":
    main()
