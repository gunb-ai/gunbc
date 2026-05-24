#!/usr/bin/env python3
"""T-19 testgen activation gate — generated TestClaim corpus + runner receipts.

Verifies TestgenConcept arms, testgen emission helpers, and generated claim modules
that exercise run_test_claim / run_test_claim_assert, refinement-preservation,
and DiagnosticExhaustiveness coproduct-exhaustiveness emission.

Run: python3 scripts/check_testgen_activation.py
Self-test: python3 scripts/test_check_testgen_activation.py
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

TESTGEN = ROOT / "src/v4/lens/testgen.dag"
EFFECTS = ROOT / "src/v4/std/effects.dag"
LBE_GENERATED = ROOT / "src/v4/test/claim/generated/language_behavior_equivalence.dag"
LBE_MANIFEST = ROOT / "src/v4/test/claim/generated/lbe_anchor_manifest.dag"
GENERATOR_WISHLIST = ROOT / "src/v4/test/claim/generated/testgen_category_wishlist.dag"
ALGEBRA_GENERATED = ROOT / "src/v4/test/claim/generated/algebra_law_conformance.dag"
COPRODUCT_EXHAUSTIVENESS_GENERATED = (
    ROOT / "src/v4/test/claim/generated/coproduct_exhaustiveness.dag"
)
REFINEMENT_GENERATED = ROOT / "src/v4/test/claim/generated/refinement_preservation.dag"
REFINEMENT_MANIFEST = ROOT / "src/v4/test/claim/generated/refinement_preservation_anchor_manifest.dag"
IDEMPOTENT_OPERATION_GENERATED = (
    ROOT / "src/v4/test/claim/generated/idempotent_operation_conformance.dag"
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
        COPRODUCT_EXHAUSTIVENESS_GENERATED,
        EFFECTS,
        LBE_GENERATED,
        LBE_MANIFEST,
        GENERATOR_WISHLIST,
        ALGEBRA_GENERATED,
        REFINEMENT_GENERATED,
        REFINEMENT_MANIFEST,
        IDEMPOTENT_OPERATION_GENERATED,
        VERIFICATION,
    ):
        _require(path)

    testgen = _read(TESTGEN)
    effects = _read(EFFECTS)
    lbe = _read(LBE_GENERATED)
    manifest = _read(LBE_MANIFEST)
    wishlist = _read(GENERATOR_WISHLIST)
    algebra = _read(ALGEBRA_GENERATED)
    coproduct_exhaustiveness = _read(COPRODUCT_EXHAUSTIVENESS_GENERATED)
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
            "fn testgen_emit_algebra_law_claim",
            "if lhs == rhs",
            "t19_algebra_law_tautological_sides",
            "type AlgebraLawCase { anchor: ClaimAnchorKey, subject: AlgebraLawSubject }",
            "fn algebra_law_manual_claim_case(anchor: ManualAnchorKey) -> Outcome<AlgebraLawCase>",
            "t19_algebra_law_anchor_mismatch",
            "match algebra_law_manual_claim_case(anchor: anchor)",
            "fn algebra_law_claim_term(subject: AlgebraLawSubject, expression: Node) -> Node",
            "lhs: algebra_law_claim_term(subject: law_case.subject, expression: lhs)",
            "rhs: algebra_law_claim_term(subject: law_case.subject, expression: rhs)",
            "fn testgen_emit_idempotent_operation_claim",
            "fn testgen_scheduled_language_behavior_generators",
            "fn testgen_scheduled_idempotent_operation_subjects",
            "import v4.std.effects",
            "idempotent_operation_apply_twice(state: sample_state",
            "idempotent_operation_apply_twice",
            "idempotent_operation_apply_once",
            "lbe_label_conj_dag_surface",
            "ManualLbeConjDagSurface",
            "ManualLbeDisjDagSurface",
            "ManualLbeTransformDagSurface",
            "ManualRefinementNonEmptyListBase",
            "dag_language_model_surface_id",
            "fn coproduct_exhaustiveness_subject_testclaim_compiles",
            "fn testgen_emit_coproduct_exhaustiveness_claim",
            "fn testgen_scheduled_coproduct_exhaustiveness_generators",
            "coproduct_exhaustiveness_subject_testclaim_diagnostic",
            "coproduct_exhaustiveness_subject_testclaim_equals",
            "coproduct_exhaustiveness_subject_testclaim_roundtrip",
            "coproduct_exhaustiveness_missing_variant",
            "fn coproduct_exhaustiveness_anchor_omitted_variant",
            "variant: coproduct_exhaustiveness_anchor_omitted_variant(anchor: anchor)",
            "anchor: generated_claim_anchor(anchor: anchor)",
            "coproduct_exhaustiveness_omitted_variant_edge",
            "node_locus(node: input)",
            "| RefinementPreservation { subject: RefinementPreservationSubject }",
            "type RefinementPreservationSubject",
            "fn testgen_emit_refinement_preservation_claim",
            "fn refinement_preservation_subject_nonempty_list_base",
            "-> Outcome<RefinementPreservationSubject>",
            "refined: Refined<List<Node>>",
            "original: List<Node>",
            "refine(",
            "refined_base(r: subject.refined)",
            "refinement_label_nonempty_list_base",
        ),
    )

    _require_substrings(
        "verification.dag",
        verification,
        (
            "ManualLbeConjDagSurface",
            "ManualLbeDisjDagSurface",
            "ManualLbeTransformDagSurface",
            "type TestClaimCoproductVariant",
            "feature:testclaim-coproduct-reflection; bound task: src/v4/TASKS.md#t-19-lenstestgendag--producer-of-testclaim-corpus-from-substrate",
            "follow-up: delete this mirror when T-19 projects arm keys from TestClaim",
            "GeneratedCoproductExhaustiveness { omitted_variant: TestClaimCoproductVariant }",
            "ManualRefinementNonEmptyListBase",
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
            "ManualLbeConjDagSurface",
            "ManualLbeDisjDagSurface",
            "ManualLbeTransformDagSurface",
        ),
    )

    _require_substrings(
        "testgen_category_wishlist.dag",
        wishlist,
        (
            "fn testgen_pending_non_tautological_generator_wishlist",
            "fn testgen_dispatched_non_tautological_generators",
            "kind: TestClaimCompilesClaimVariant",
            "kind: TestClaimDiagnosticClaimVariant",
            "kind: TestClaimEqualsClaimVariant",
            "kind: TestClaimRoundTripClaimVariant",
            "type TestgenOracleBasis",
            "StructuralConstructionWitness",
            "AlgebraLawWitness",
            "DiagnosticNegativeFixture",
            "LensObservationFixture",
            "RoundTripDifferential",
            "FrozenIoSnapshot",
            "RefinementProjectionWitness",
            "dispatch_key: Symbol",
            "feature:t19-generator-oracle-basis-carrier",
            "bound task: src/v4/TASKS.md#t-19-lenstestgendag--producer-of-testclaim-corpus-from-substrate",
            "dissolve-on-arrival: delete TestgenOracleBasis",
            "data claim_testgen_wishlist_formalized: TestClaim",
        ),
    )

    _require_substrings(
        "algebra_law_conformance.dag",
        algebra,
        (
            "testgen_emit_algebra_law_claim",
            "nat_algebra_law_subject_symbol_add_operation",
            "nat_algebra_law_subject_symbol_mul_operation",
            "nat_algebra_law_subject_symbol_zero_value",
            "generated_nat_add_left_identity_claim",
            "generated_nat_add_associativity_claim",
            "generated_nat_mul_annihilator_claim",
            "fn t19_generated_nat_add",
            "fn t19_generated_nat_mul",
            "operation: nat_algebra_law_subject_symbol_add_operation",
            "operation: nat_algebra_law_subject_symbol_mul_operation",
            "lhs: t19_generated_nat_add(left: t19_generated_nat_zero(), right: t19_generated_nat_one())",
            "rhs: t19_generated_nat_one()",
            "lhs: t19_generated_nat_mul(left: t19_generated_nat_zero(), right: t19_generated_nat_three())",
            "rhs: t19_generated_nat_zero()",
            "fn generated_algebra_law_claim_rows",
            "length(xs: generated_algebra_law_claim_rows()) == 3",
        ),
    )

    _require_substrings(
        "coproduct_exhaustiveness.dag",
        coproduct_exhaustiveness,
        (
            "fn generated_coproduct_exhaustiveness_claim() -> Outcome<TestClaim>",
            "testgen_emit_coproduct_exhaustiveness_claim",
            "GeneratedCoproductExhaustiveness { omitted_variant: _ }",
            "witness_coproduct_exhaustiveness_diagnostic_claim",
            "witness_coproduct_exhaustiveness_uses_generated_anchor",
            "witness_coproduct_exhaustiveness_all_variants_emit",
            "witness_coproduct_exhaustiveness_generator_count",
            "length(xs: testgen_scheduled_coproduct_exhaustiveness_generators()) == 4",
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
            "ManualRefinementNonEmptyListBase",
        ),
    )

    _require_substrings(
        "refinement_preservation_anchor_manifest.dag",
        refinement_manifest,
        ("ManualRefinementNonEmptyListBase",),
    )

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
            "idempotent_operation_tautology_skip",
            "sample_read_subject",
            "sample_upsert_subject",
            "sample_delete_subject",
            "sample_label_only_subject",
            "generated_idempotent_operation_sample_count_is_three",
        ),
    )

    if "LanguageBehaviorEquivalence" not in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit("LanguageBehaviorEquivalence must be a TestgenConcept variant, not free text only")

    if "RefinementPreservation" not in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit("RefinementPreservation must be a TestgenConcept variant, not free text only")

    if "IdempotentOperationSubject" in testgen.split("type TestgenConcept")[1].split("type Generator")[0]:
        raise SystemExit(
            "IdempotentOperationSubject must stay outside the closed seven-way TestgenConcept coproduct"
        )

    pending_rows = wishlist.split("fn testgen_pending_non_tautological_generator_wishlist")[1].split(
        "fn testgen_dispatched_non_tautological_generators"
    )[0]
    if pending_rows.count("TestgenWishlistRow {") != 4:
        raise SystemExit("generator wishlist must carry exactly four pending non-dispatched rows")
    if "slot: AlgebraLaw" in pending_rows:
        raise SystemExit("AlgebraLaw has generated rows and must stay out of pending wishlist rows")

    dispatched_rows = wishlist.split("fn testgen_dispatched_non_tautological_generators")[1].split(
        "fn pending_non_tautological_generator_count_is_four"
    )[0]
    if dispatched_rows.count("TestgenWishlistRow {") != 3:
        raise SystemExit("generator wishlist must carry exactly three dispatched rows")
    if (
        "generator: dispatched_language_behavior_equivalence_generator()" not in dispatched_rows
        or "generator: dispatched_algebra_law_generator()" not in dispatched_rows
        or "generator: dispatched_refinement_preservation_generator()" not in dispatched_rows
    ):
        raise SystemExit("dispatched wishlist rows must include LBE, AlgebraLaw, and RefinementPreservation")
    if "ManualNatAddAssociativity" not in wishlist:
        raise SystemExit("dispatched AlgebraLaw wishlist row must carry the emitted algebra-law anchor")
    if "ManualRefinementNonEmptyListBase" not in wishlist:
        raise SystemExit("dispatched RefinementPreservation wishlist row must carry the emitted refinement anchor")

    if algebra.count("data generated_nat_") < 3:
        raise SystemExit("algebra-law generator must produce at least three sample TestClaim rows")

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
        "(LBE + algebra-law + refinement-preservation + idempotent-operation + "
        "coproduct-exhaustiveness generated receipts)."
    )


if __name__ == "__main__":
    main()
