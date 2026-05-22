#!/usr/bin/env python3
"""T-19 testgen activation gate — generated TestClaim corpus + runner receipts.

Verifies TestgenConcept arms, testgen emission helpers, and generated claim modules
that exercise run_test_claim / run_test_claim_assert (post-T-22 eval decomposition).

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
REFINEMENT_GENERATED = ROOT / "src/v4/test/claim/generated/refinement_preservation.dag"
REFINEMENT_MANIFEST = ROOT / "src/v4/test/claim/generated/refinement_preservation_anchor_manifest.dag"
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
    for path in (TESTGEN, LBE_GENERATED, LBE_MANIFEST, REFINEMENT_GENERATED, REFINEMENT_MANIFEST, VERIFICATION):
        _require(path)

    testgen = _read(TESTGEN)
    lbe = _read(LBE_GENERATED)
    manifest = _read(LBE_MANIFEST)
    refinement = _read(REFINEMENT_GENERATED)
    refinement_manifest = _read(REFINEMENT_MANIFEST)
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
            "T19ManualRefinementNonEmptyListBase",
            "dag_language_model_surface_id",
            "| RefinementPreservation { subject: RefinementPreservationSubject }",
            "type RefinementPreservationSubject",
            "fn testgen_emit_refinement_preservation_claim",
            "fn refinement_preservation_subject_nonempty_list_base",
            "-> Outcome<RefinementPreservationSubject>",
            "refined: Refined<List<Node>>",
            "original: List<Node>",
            "refine(",
            "refined_base(r: subject.refined)",
            "t19_refinement_label_nonempty_list_base",
        ),
    )

    _require_substrings(
        "verification.dag",
        verification,
        (
            "T19ManualLbeConjDagSurface",
            "T19ManualLbeDisjDagSurface",
            "T19ManualLbeTransformDagSurface",
            "T19ManualRefinementNonEmptyListBase",
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
        "refinement_preservation.dag",
        refinement,
        (
            "fn refinement_preservation_claim_from_testgen_emit",
            "-> Outcome<TestClaim>",
            "testgen_emit_refinement_preservation_claim",
            "RefinementPreservationSubject",
            "refined_base(r: subject.refined) == subject.original",
            "refinement_preservation_subject_nonempty_list_base()",
            "data claim_refinement_nonempty_list_base_preserved: Outcome<TestClaim>",
            "witness_refinement_preserves_nonempty_list_base",
            "T19ManualRefinementNonEmptyListBase",
        ),
    )

    _require_substrings(
        "refinement_preservation_anchor_manifest.dag",
        refinement_manifest,
        ("T19ManualRefinementNonEmptyListBase",),
    )

    if "LanguageBehaviorEquivalence" not in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit("LanguageBehaviorEquivalence must be a TestgenConcept variant, not free text only")

    if "RefinementPreservation" not in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit("RefinementPreservation must be a TestgenConcept variant, not free text only")

    print("OK: T-19 testgen activation (LBE + refinement-preservation generated receipts).")


if __name__ == "__main__":
    main()
