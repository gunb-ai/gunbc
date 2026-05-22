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
EFFECTS = ROOT / "src/v4/std/effects.dag"
LBE_GENERATED = ROOT / "src/v4/test/claim/generated/language_behavior_equivalence.dag"
LBE_MANIFEST = ROOT / "src/v4/test/claim/generated/lbe_anchor_manifest.dag"
<<<<<<< HEAD
COPRODUCT_EXHAUSTIVENESS_GENERATED = (
    ROOT / "src/v4/test/claim/generated/coproduct_exhaustiveness.dag"
=======
IDEMPOTENT_OPERATION_GENERATED = (
    ROOT / "src/v4/test/claim/generated/idempotent_operation_conformance.dag"
>>>>>>> origin/main
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
<<<<<<< HEAD
        LBE_GENERATED,
        LBE_MANIFEST,
        COPRODUCT_EXHAUSTIVENESS_GENERATED,
=======
        EFFECTS,
        LBE_GENERATED,
        LBE_MANIFEST,
        IDEMPOTENT_OPERATION_GENERATED,
>>>>>>> origin/main
        VERIFICATION,
    ):
        _require(path)

    testgen = _read(TESTGEN)
    effects = _read(EFFECTS)
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
            "fn testgen_emit_idempotent_operation_claim",
            "fn testgen_scheduled_language_behavior_generators",
            "fn testgen_scheduled_idempotent_operation_subjects",
            "import v4.std.effects",
            "idempotent_operation_apply_twice(state: t19_sample_state",
            "idempotent_operation_apply_twice",
            "idempotent_operation_apply_once",
            "t19_lbe_label_conj_dag_surface",
            "T19ManualLbeConjDagSurface",
            "T19ManualLbeDisjDagSurface",
            "T19ManualLbeTransformDagSurface",
            "dag_language_model_surface_id",
            "fn coproduct_exhaustiveness_subject_testclaim_compiles",
            "fn testgen_emit_coproduct_exhaustiveness_claim",
            "fn testgen_scheduled_coproduct_exhaustiveness_generators",
            "coproduct_exhaustiveness_subject_testclaim_diagnostic",
            "coproduct_exhaustiveness_subject_testclaim_equals",
            "coproduct_exhaustiveness_subject_testclaim_roundtrip",
            "t19_coproduct_exhaustiveness_missing_variant",
            "fn coproduct_exhaustiveness_anchor_omitted_variant",
            "variant: coproduct_exhaustiveness_anchor_omitted_variant(anchor: anchor)",
            "t19_anchor: t19_generated_claim_anchor(anchor: anchor)",
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
            "type TestClaimCoproductVariant",
            "T19GeneratedCoproductExhaustiveness { omitted_variant: TestClaimCoproductVariant }",
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

<<<<<<< HEAD
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
=======
    idempotent = _read(IDEMPOTENT_OPERATION_GENERATED)
    _require_substrings(
        "idempotent_operation_conformance.dag",
        idempotent,
        (
            "import v4.std.effects",
            "testgen_emit_idempotent_operation_claim",
            "generated_read_idempotent_operation_claim",
            "generated_upsert_idempotent_operation_claim",
            "generated_delete_idempotent_operation_claim",
            "generated_label_only_skip_pins_rejection",
            "generated_label_only_skip_is_rejected",
            "import v4.std.node { Symbol }",
            "t19_idempotent_operation_tautology_skip",
            "t19_sample_read_subject",
            "t19_sample_upsert_subject",
            "t19_sample_delete_subject",
            "t19_sample_label_only_subject",
            "generated_idempotent_operation_sample_count_is_three",
>>>>>>> origin/main
        ),
    )

    if "LanguageBehaviorEquivalence" not in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit("LanguageBehaviorEquivalence must be a TestgenConcept variant, not free text only")

<<<<<<< HEAD
    print("OK: T-19 testgen activation (LBE + coproduct-exhaustiveness generated receipts).")
=======
    if "IdempotentOperationSubject" in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit(
            "IdempotentOperationSubject must stay outside the closed six-way TestgenConcept coproduct"
        )

    _require_substrings(
        "effects.dag",
        effects,
        (
            "type IdempotentShape",
            "type EffectShape",
            "type IdempotentOperationSubject",
            "type ComposableIdempotentOperationSubject",
            "Composable(ComposableIdempotentOperationSubject)",
            "fn idempotent_operation_apply_node(state: Node, subject: ComposableIdempotentOperationSubject)",
            "ReadIdempotentSample",
            "UpsertIdempotentSample",
            "DeleteIdempotentSample",
            "LabelOnlyIdempotentInhabitance",
            "fn idempotent_operation_witness_node",
            "fn idempotent_operation_apply_twice",
            "fn idempotent_operation_apply_once",
            "ComputationNode { behavior: Transform }",
            "key_source_path_param_value_field",
            "fn classified_idempotent_effect_node",
            "IsIdempotent(IdempotentShape)",
        ),
    )

    print(
        "OK: T-19 testgen activation "
        "(LBE sixth category + idempotent-operation generator slice + generated runner receipts)."
    )
>>>>>>> origin/main


if __name__ == "__main__":
    main()
