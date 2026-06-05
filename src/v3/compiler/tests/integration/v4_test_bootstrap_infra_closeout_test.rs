//! **Layer:** integration
//!
//! Closeout ratchets for the v4 T-19/T-20/T-22 test/bootstrap-infra lane. These
//! checks stay at the parse-surface boundary: they prove structural authorities
//! exist and remain joined; T-22 rows are parse/substrate ratchets only (not execution).
//! This hand-Rust ratchet retires when T-22 generated harness coverage
//! expresses the same bootstrap closeout checks as `.dag` `TestClaim` rows.
//! **P5 receipt for same-path expansion (INVARIANTS.md §P5 Mechanism (b)):** explicit
//! deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:43,63); this file remains inside
//! the SG-0 T-PB-B test subset (`sg0_census_test.rs:1065`) and dissolves when these
//! T-22 closeout checks are emitted as `.dag` `TestClaim` rows or generated harness
//! coverage.
//!
//! **PR #4295 P5 receipt (+0 SG-0 paths):** same-path expansion in this file for
//! `check_t19_testgen_activation` — structural migration of the activation gate
//! formerly enforced by `scripts/check_t19_testgen_activation.py` (deleted on main
//! in #4252, operator 2026-06-01 CI hygiene; not re-deleted in this PR). No new
//! `EXPECTED_HAND_AUTHORED_TEST` census row (ROADMAP.md:43,63; `sg0_census_test.rs:1065`);
//! dissolves when T-22 generated harness or `.dag` TestClaim rows own these substrate checks.
//!
//! **PR #4335 P5 receipt (INVARIANTS.md §P5 Mechanism (b) — +0 SG-0 paths):**
//! disposition (3) explicit deferral — **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:43,63); same-path expansion for
//! `rr_a_step2_bootstrap_evaluator_corpus_harness_entry` under existing census row
//! `sg0_census_test.rs:1065` (no new `EXPECTED_HAND_AUTHORED_TEST` entry). Dissolves when
//! modeled `.dag` `TestClaim` rows or generated harness coverage own these substrate checks.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceModule, SurfaceRecordField, SurfaceType,
    SurfaceVariant, TypeAngleArg,
};
use v3_compiler::tokenize_for_test;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const TESTGEN_PATH: &str = "src/v4/lens/testgen.dag";
const BOOTSTRAP_DAG: &str = include_str!("../../../../v4/workflow/bootstrap.dag");
const BOOTSTRAP_PATH: &str = "src/v4/workflow/bootstrap.dag";
const CLI_DAG: &str = include_str!("../../../../v4/workflow/cli.dag");
const CLI_PATH: &str = "src/v4/workflow/cli.dag";
const CONNECTIVE_ANCHORS_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/connective_anchors.dag");
const CONNECTIVE_ANCHORS_PATH: &str = "src/v4/test/claim/manual/connective_anchors.dag";
const NAT_LAW_ANCHORS_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/nat_law_anchors.dag");
const NAT_LAW_ANCHORS_PATH: &str = "src/v4/test/claim/manual/nat_law_anchors.dag";
const MANIFEST_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/manual_anchor_manifest.dag");
const MANIFEST_PATH: &str = "src/v4/test/claim/manual/manual_anchor_manifest.dag";
const DIAGNOSTIC_ASSERT_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/diagnostic_assert_eval.dag");
const DIAGNOSTIC_ASSERT_EVAL_PATH: &str = "src/v4/test/claim/manual/diagnostic_assert_eval.dag";
const SG1B_FAILCLOSED_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/sg1b_signature_realization_failclosed.dag");
const SG1B_FAILCLOSED_PATH: &str =
    "src/v4/test/claim/manual/sg1b_signature_realization_failclosed.dag";
const REFINEMENT_BRAND_DISJOINTNESS_IS_SUGAR_SCOPE_DAG: &str = include_str!(
    "../../../../v4/test/claim/manual/refinement_brand_disjointness_is_sugar_scope.dag"
);
const REFINEMENT_BRAND_DISJOINTNESS_IS_SUGAR_SCOPE_PATH: &str =
    "src/v4/test/claim/manual/refinement_brand_disjointness_is_sugar_scope.dag";
const EVAL_DAG: &str = include_str!("../../../../v4/compiler/05_eval.dag");
const RUNTIME_DAG: &str = include_str!("../../../../v4/std/runtime.dag");
const LBE_GENERATED_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/language_behavior_equivalence.dag");
const LBE_GENERATED_PATH: &str = "src/v4/test/claim/generated/language_behavior_equivalence.dag";
const ALGEBRA_LAW_GENERATED_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/algebra_law_conformance.dag");
const ALGEBRA_LAW_GENERATED_PATH: &str = "src/v4/test/claim/generated/algebra_law_conformance.dag";
const TESTGEN_WISHLIST_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/testgen_category_wishlist.dag");
const TESTGEN_WISHLIST_PATH: &str = "src/v4/test/claim/generated/testgen_category_wishlist.dag";
const COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/coproduct_exhaustiveness.dag");
const COPRODUCT_EXHAUSTIVENESS_GENERATED_PATH: &str =
    "src/v4/test/claim/generated/coproduct_exhaustiveness.dag";
const REFINEMENT_GENERATED_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/refinement_preservation.dag");
const REFINEMENT_GENERATED_PATH: &str = "src/v4/test/claim/generated/refinement_preservation.dag";
const IDEMPOTENT_OPERATION_GENERATED_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/idempotent_operation_conformance.dag");
const IDEMPOTENT_OPERATION_GENERATED_PATH: &str =
    "src/v4/test/claim/generated/idempotent_operation_conformance.dag";
const EFFECTS_DAG: &str = include_str!("../../../../v4/std/effects.dag");
const EFFECTS_PATH: &str = "src/v4/std/effects.dag";
const DAG_INPUT_SURFACE_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_testgen/dag_input_surface.dag");
const DAG_INPUT_SURFACE_PATH: &str = "src/v4/test/claim/lens_testgen/dag_input_surface.dag";
const LBE_MANIFEST_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/lbe_anchor_manifest.dag");
const LBE_MANIFEST_PATH: &str = "src/v4/test/claim/generated/lbe_anchor_manifest.dag";
const REFINEMENT_MANIFEST_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/refinement_preservation_anchor_manifest.dag");
const REFINEMENT_MANIFEST_PATH: &str =
    "src/v4/test/claim/generated/refinement_preservation_anchor_manifest.dag";
const VERIFICATION_DAG: &str = include_str!("../../../../v4/std/verification.dag");
const VERIFICATION_PATH: &str = "src/v4/std/verification.dag";

#[test]
fn language_behavior_equivalence_generated_claims_parse() {
    parse_module(LBE_GENERATED_DAG, LBE_GENERATED_PATH);
}

#[test]
fn t19_algebra_law_generated_claims_parse_and_use_testgen_emit() {
    parse_module(ALGEBRA_LAW_GENERATED_DAG, ALGEBRA_LAW_GENERATED_PATH);

    assert!(
        TESTGEN_DAG.contains("fn testgen_emit_algebra_law_claim")
            && TESTGEN_DAG.contains("if lhs == rhs")
            && TESTGEN_DAG.contains("t19_algebra_law_tautological_sides")
            && TESTGEN_DAG
                .contains("type AlgebraLawCase { anchor: ClaimAnchorKey, subject: AlgebraLawSubject }")
            && TESTGEN_DAG.contains(
                "fn algebra_law_claim_term(subject: AlgebraLawSubject, expression: Node) -> Node"
            )
            && TESTGEN_DAG
                .contains("fn algebra_law_manual_claim_case(anchor: ManualAnchorKey) -> Outcome<AlgebraLawCase>")
            && TESTGEN_DAG.contains(
                "fn algebra_law_subject_for_manual_anchor(anchor: ManualAnchorKey) -> Outcome<AlgebraLawSubject>"
            )
            && TESTGEN_DAG.contains("match algebra_law_manual_claim_case(anchor: anchor)")
            && TESTGEN_DAG.contains("match algebra_law_subject_for_manual_anchor(anchor: manual_anchor)")
            && TESTGEN_DAG
                .contains("lhs: algebra_law_claim_term(subject: law_case.subject, expression: lhs)")
            && TESTGEN_DAG
                .contains("rhs: algebra_law_claim_term(subject: law_case.subject, expression: rhs)")
            && TESTGEN_DAG.contains("Rejected {")
            && TESTGEN_DAG.contains("value: EqualsClaim {"),
        "testgen must reject tautological algebra-law sides and derive AlgebraLawSubject from the checked algebra-law anchor"
    );
    assert!(
        ALGEBRA_LAW_GENERATED_DAG.contains("testgen_emit_algebra_law_claim")
            && ALGEBRA_LAW_GENERATED_DAG.contains("Nat,")
            && ALGEBRA_LAW_GENERATED_DAG.contains("nat_add,")
            && ALGEBRA_LAW_GENERATED_DAG.contains("nat_mul")
            && ALGEBRA_LAW_GENERATED_DAG.contains("feature:t19-nat-expression-node-encoding")
            && ALGEBRA_LAW_GENERATED_DAG.contains("generated_nat_add_left_identity_claim")
            && ALGEBRA_LAW_GENERATED_DAG.contains("generated_nat_add_associativity_claim")
            && ALGEBRA_LAW_GENERATED_DAG.contains("generated_nat_mul_annihilator_claim")
            && ALGEBRA_LAW_GENERATED_DAG
                .contains("length(xs: generated_algebra_law_claim_rows()) == 3"),
        "algebra-law generator slice must produce at least three sample TestClaim rows"
    );
    assert!(
        ALGEBRA_LAW_GENERATED_DAG.contains("fn t19_generated_nat_add")
            && ALGEBRA_LAW_GENERATED_DAG.contains("fn t19_generated_nat_mul")
            && ALGEBRA_LAW_GENERATED_DAG
                .contains("operation: algebra_law_generated_nat_add_application")
            && ALGEBRA_LAW_GENERATED_DAG
                .contains("operation: algebra_law_generated_nat_mul_application")
            && ALGEBRA_LAW_GENERATED_DAG
                .contains("result: nat_add(a: t19_generated_nat_zero_value(), b: t19_generated_nat_one_value())")
            && ALGEBRA_LAW_GENERATED_DAG.contains("rhs: t19_generated_nat_one()")
            && ALGEBRA_LAW_GENERATED_DAG
                .contains("result: nat_mul(a: t19_generated_nat_zero_value(), b: t19_generated_nat_three_value())")
            && ALGEBRA_LAW_GENERATED_DAG.contains("rhs: t19_generated_nat_zero()"),
        "algebra-law samples must route generated Nat expression results through canonical nat_add/nat_mul, not exported operation/value mirror symbols"
    );
    for forbidden in [
        "nat_algebra_law_subject_symbol_add_operation",
        "nat_algebra_law_subject_symbol_mul_operation",
        "nat_algebra_law_subject_symbol_zero_value",
        "nat_algebra_law_subject_symbol_one_value",
        "nat_algebra_law_subject_symbol_two_value",
        "nat_algebra_law_subject_symbol_three_value",
    ] {
        assert!(
            !ALGEBRA_LAW_GENERATED_DAG.contains(forbidden),
            "algebra-law generated corpus must not consume {forbidden}"
        );
    }
}

#[test]
fn t19_non_tautological_generator_wishlist_parse_and_pins_dispatch_rows() {
    parse_module(TESTGEN_WISHLIST_DAG, TESTGEN_WISHLIST_PATH);

    assert!(
        TESTGEN_WISHLIST_DAG.contains("type TestgenOracleBasis")
            && TESTGEN_WISHLIST_DAG
                .contains("feature:t19-generator-oracle-basis-carrier")
            && TESTGEN_WISHLIST_DAG.contains(
                "bound task: src/v4/TASKS.md#t-19-lenstestgendag--producer-of-testclaim-corpus-from-substrate",
            )
            && TESTGEN_WISHLIST_DAG.contains("dissolve-on-arrival: delete TestgenOracleBasis")
            && TESTGEN_WISHLIST_DAG.contains("StructuralConstructionWitness")
            && TESTGEN_WISHLIST_DAG.contains("AlgebraLawWitness")
            && TESTGEN_WISHLIST_DAG.contains("DiagnosticNegativeFixture")
            && TESTGEN_WISHLIST_DAG.contains("LensObservationFixture")
            && TESTGEN_WISHLIST_DAG.contains("RoundTripDifferential")
            && TESTGEN_WISHLIST_DAG.contains("FrozenIoSnapshot")
            && TESTGEN_WISHLIST_DAG.contains("RefinementProjectionWitness")
            && TESTGEN_WISHLIST_DAG.contains("dispatch_key: Symbol"),
        "T-19 wishlist rows must name an independent oracle basis plus a dispatch key"
    );
    assert!(
        TESTGEN_WISHLIST_DAG.contains("generated_claim_anchor")
            && TESTGEN_WISHLIST_DAG.contains("GeneratedCoproductExhaustiveness")
            && TESTGEN_WISHLIST_DAG.contains("omitted_variant: TestClaimDiagnosticClaimVariant"),
        "DiagnosticExhaustiveness dispatched row must use the generated coproduct-exhaustiveness anchor and omitted variant"
    );

    let pending = between(
        TESTGEN_WISHLIST_DAG,
        "fn testgen_pending_non_tautological_generator_wishlist",
        "fn testgen_dispatched_non_tautological_generators",
    );
    assert_eq!(
        pending.matches("TestgenWishlistRow {").count(),
        3,
        "wishlist must dispatch the three pending non-dispatched TestgenConcept categories"
    );
    for generator in [
        "generator: wishlist_type_construction_generator()",
        "generator: wishlist_lens_applicability_generator()",
        "generator: wishlist_bidirectional_roundtrip_generator()",
    ] {
        assert!(
            pending.contains(generator),
            "pending wishlist must include {generator}"
        );
    }
    assert!(
        !pending.contains("slot: LanguageBehaviorEquivalence"),
        "LBE has already shipped generated runner receipts and must stay out of pending wishlist rows"
    );
    assert!(
        !pending.contains("slot: AlgebraLaw"),
        "AlgebraLaw has generated corpus rows and must stay out of pending wishlist rows"
    );
    assert!(
        !pending.contains("slot: DiagnosticExhaustiveness"),
        "DiagnosticExhaustiveness has generated corpus rows and must stay out of pending wishlist rows"
    );

    let dispatched = between(
        TESTGEN_WISHLIST_DAG,
        "fn testgen_dispatched_non_tautological_generators",
        "fn pending_non_tautological_generator_count_is_three",
    );
    assert_eq!(
        dispatched.matches("TestgenWishlistRow {").count(),
        4,
        "wishlist must record the four already-dispatched generator rows"
    );
    assert!(
        dispatched.contains("generator: dispatched_language_behavior_equivalence_generator()")
            && dispatched.contains("oracle: FrozenIoSnapshot"),
        "dispatched row must keep LBE tied to the frozen I/O snapshot oracle"
    );
    assert!(
        dispatched.contains("generator: dispatched_algebra_law_generator()")
            && dispatched.contains("oracle: AlgebraLawWitness")
            && TESTGEN_WISHLIST_DAG.contains("anchor: manual_claim_anchor(anchor: ManualNatAddAssociativity)"),
        "dispatched row must keep AlgebraLaw tied to the emitted algebra-law anchor and witness oracle"
    );
    assert!(
        dispatched.contains("generator: dispatched_diagnostic_exhaustiveness_generator()")
            && dispatched.contains("oracle: DiagnosticNegativeFixture")
            && dispatched.contains("dispatch_key: t19_dispatched_diagnostic_exhaustiveness")
            && between(
                TESTGEN_WISHLIST_DAG,
                "fn dispatched_diagnostic_exhaustiveness_generator",
                "fn wishlist_lens_applicability_generator",
            )
            .contains("classification: TestClassification { tier: Tier1, layer: Unit }"),
        "dispatched row must keep DiagnosticExhaustiveness tied to the emitted diagnostic oracle and Tier1 emitted claim classification"
    );
    assert!(
        dispatched.contains("generator: dispatched_refinement_preservation_generator()")
            && dispatched.contains("oracle: RefinementProjectionWitness")
            && TESTGEN_WISHLIST_DAG
                .contains("anchor: manual_claim_anchor(anchor: ManualRefinementNonEmptyListBase)"),
        "dispatched row must keep RefinementPreservation tied to the emitted refinement anchor and projection witness oracle"
    );
}

#[test]
fn refinement_preservation_generated_claims_parse() {
    parse_module(REFINEMENT_GENERATED_DAG, REFINEMENT_GENERATED_PATH);
}

#[test]
fn refinement_preservation_receipts_present() {
    assert!(
        TESTGEN_DAG.contains("RefinementPreservation { subject: RefinementPreservationSubject }")
            && TESTGEN_DAG.contains("fn testgen_emit_refinement_preservation_claim")
            && TESTGEN_DAG.contains("fn testgen_scheduled_refinement_preservation_generators")
            && TESTGEN_DAG.contains("-> Outcome<List<Generator<TestgenConcept>>>")
            && !TESTGEN_DAG.contains("Rejected { diagnostics: _ } => Empty")
            && TESTGEN_DAG.contains("-> Outcome<RefinementPreservationSubject>")
            && TESTGEN_DAG.contains("refined: Refined<List<Node>>")
            && TESTGEN_DAG.contains("refined_base(r: subject.refined)")
            && TESTGEN_DAG.contains("ManualRefinementNonEmptyListBase")
            && TESTGEN_DAG.contains(
                "bootstrap_claim_generator_for_manual_anchor(key: ManualRefinementNonEmptyListBase)"
            )
            && REFINEMENT_GENERATED_DAG
                .contains("refinement_preservation_subject_nonempty_list_base()")
            && REFINEMENT_GENERATED_DAG
                .contains("refined_base(r: subject.refined) == subject.original")
            && REFINEMENT_GENERATED_DAG.contains(
                "data claim_refinement_nonempty_list_base_preserved: Outcome<TestClaim>"
            )
            && REFINEMENT_GENERATED_DAG.contains(
                "data witness_refinement_preserves_nonempty_list_base: Bool"
            )
            && REFINEMENT_GENERATED_DAG.contains(
                "data witness_refinement_preservation_scheduler_emits_one_generator: Bool"
            ),
        "generated refinement-preservation corpus must derive a TestClaim through testgen_emit, prove refined_base preserves the accepted base, and prove the mandatory scheduler emits the refinement key"
    );
}

#[test]
fn idempotent_operation_generated_claims_parse_and_pin_emission() {
    parse_module(EFFECTS_DAG, EFFECTS_PATH);
    parse_module(
        IDEMPOTENT_OPERATION_GENERATED_DAG,
        IDEMPOTENT_OPERATION_GENERATED_PATH,
    );
    assert!(
        IDEMPOTENT_OPERATION_GENERATED_DAG.contains("testgen_emit_idempotent_operation_claim")
            && IDEMPOTENT_OPERATION_GENERATED_DAG
                .contains("generated_read_idempotent_operation_claim")
            && IDEMPOTENT_OPERATION_GENERATED_DAG
                .contains("generated_upsert_idempotent_operation_claim")
            && IDEMPOTENT_OPERATION_GENERATED_DAG
                .contains("generated_delete_idempotent_operation_claim")
            && IDEMPOTENT_OPERATION_GENERATED_DAG
                .contains("generated_idempotent_operation_sample_count_is_three"),
        "idempotent-operation generator slice must produce at least three sample TestClaim rows"
    );
    assert!(
        IDEMPOTENT_OPERATION_GENERATED_DAG.contains("generated_label_only_skip_pins_rejection")
            && IDEMPOTENT_OPERATION_GENERATED_DAG.contains("generated_label_only_skip_is_rejected")
            && IDEMPOTENT_OPERATION_GENERATED_DAG.contains("import v4.std.node { Symbol }")
            && IDEMPOTENT_OPERATION_GENERATED_DAG.contains("Accepted { value: _, diagnostics: _ } => false")
            && IDEMPOTENT_OPERATION_GENERATED_DAG.contains("idempotent_operation_tautology_skip")
            && IDEMPOTENT_OPERATION_GENERATED_DAG.contains("sample_label_only_subject")
            && EFFECTS_DAG.contains("type ComposableIdempotentOperationSubject")
            && EFFECTS_DAG.contains("Composable(ComposableIdempotentOperationSubject)")
            && EFFECTS_DAG.contains("LabelOnlyIdempotentInhabitance")
            && EFFECTS_DAG.contains("fn idempotent_operation_apply_twice(state: Node, subject: ComposableIdempotentOperationSubject)")
            && EFFECTS_DAG.contains("fn idempotent_operation_apply_once(state: Node, subject: ComposableIdempotentOperationSubject)")
            && EFFECTS_DAG.contains("ComputationNode { behavior: Transform }")
            && EFFECTS_DAG.contains("key_source_path_param_value_field")
            && TESTGEN_DAG.contains("idempotent_operation_apply_twice(state: sample_state"),
        "idempotent-operation claims must model f(f(x))==f(x) via nested Transform application in v4.std.effects, with an explicit label-only skip path in the generated corpus"
    );
    assert!(
        EFFECTS_DAG.contains("type IdempotentShape")
            && EFFECTS_DAG.contains("type EffectShape")
            && EFFECTS_DAG.contains("ReadIdempotentSample")
            && EFFECTS_DAG.contains("fn idempotent_operation_witness_node")
            && EFFECTS_DAG.contains("classified_idempotent_effect_node"),
        "v4.std.effects must carry canonical IdempotentShape/EffectShape witnesses for generator subjects"
    );
    assert!(
        TESTGEN_DAG.contains("fn testgen_emit_idempotent_operation_claim")
            && TESTGEN_DAG.contains("import v4.std.effects")
            && TESTGEN_DAG.contains("ComposableIdempotentOperationSubject")
            && TESTGEN_DAG.contains("Composable(inner) =>")
            && TESTGEN_DAG.contains("idempotent_operation_apply_twice")
            && TESTGEN_DAG.contains("fn testgen_scheduled_idempotent_operation_subjects"),
        "testgen lens must emit idempotent-operation claims from v4.std.effects composable subjects, not parallel Symbol/Node subjects"
    );
}

#[test]
fn language_behavior_equivalence_run_test_claim_receipts_present() {
    assert!(
        LBE_GENERATED_DAG.contains("run_test_claim(")
            && LBE_GENERATED_DAG.contains("fn lbe_claim_from_testgen_emit")
            && LBE_GENERATED_DAG.contains("-> Outcome<TestClaim>")
            && LBE_GENERATED_DAG.contains("Fail { actual: Rejected { diagnostics:")
            && LBE_GENERATED_DAG
                .contains("data run_lbe_conj_via_run_test_claim: TestClaimRun<Node, RuntimeValue>")
            && LBE_GENERATED_DAG
                .contains("data run_lbe_disj_via_run_test_claim: TestClaimRun<Node, RuntimeValue>")
            && LBE_GENERATED_DAG.contains(
                "data run_lbe_transform_via_run_test_claim: TestClaimRun<Node, RuntimeValue>"
            )
            && LBE_GENERATED_DAG.contains("run_test_claim_assert(")
            && LBE_GENERATED_DAG.contains("witness_lbe_conj_snapshot_pass")
            && LBE_GENERATED_DAG.contains("witness_lbe_disj_snapshot_pass")
            && LBE_GENERATED_DAG.contains("witness_lbe_transform_snapshot_pass")
            && LBE_GENERATED_DAG.contains("testgen_scheduled_language_behavior_generators"),
        "generated LBE corpus must wire frozen-snapshot mocks through run_test_claim_assert and run_test_claim"
    );
    assert!(
        TESTGEN_DAG.contains("LanguageBehaviorEquivalence {")
            && TESTGEN_DAG.contains("type FrozenLanguageBehaviorSnapshot")
            && TESTGEN_DAG.contains("fn testgen_emit_language_behavior_equivalence_claim")
            && TESTGEN_DAG.contains("lbe_label_conj_dag_surface"),
        "testgen lens must emit LBE claims with frozen snapshot + I/O mock carriers"
    );
}

#[test]
fn coproduct_exhaustiveness_generated_claim_parse_and_witnesses_present() {
    parse_module(
        COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG,
        COPRODUCT_EXHAUSTIVENESS_GENERATED_PATH,
    );
    assert!(
        TESTGEN_DAG.contains("fn coproduct_exhaustiveness_subject_testclaim_compiles")
            && TESTGEN_DAG.contains("fn coproduct_exhaustiveness_subject_testclaim_diagnostic")
            && TESTGEN_DAG.contains("fn coproduct_exhaustiveness_subject_testclaim_equals")
            && TESTGEN_DAG.contains("fn coproduct_exhaustiveness_subject_testclaim_roundtrip")
            && TESTGEN_DAG.contains("fn testgen_emit_coproduct_exhaustiveness_claim")
            && TESTGEN_DAG.contains("fn testgen_scheduled_coproduct_exhaustiveness_generators")
            && TESTGEN_DAG.contains("slot: DiagnosticExhaustiveness")
            && TESTGEN_DAG.contains("value: DiagnosticClaim {")
            && TESTGEN_DAG.contains("anchor: generated_claim_anchor(anchor: anchor)"),
        "T-19 DiagnosticExhaustiveness must emit coproduct-exhaustiveness TestClaim data from lens/testgen"
    );
    assert!(
        TESTGEN_DAG.contains("fn coproduct_exhaustiveness_input(anchor: GeneratedAnchorKey) -> Node")
            && TESTGEN_DAG.contains("fn coproduct_exhaustiveness_anchor_omitted_variant(anchor: GeneratedAnchorKey) -> TestClaimCoproductVariant")
            && TESTGEN_DAG.contains("variant: coproduct_exhaustiveness_anchor_omitted_variant(")
            && TESTGEN_DAG.contains("anchor: anchor")
            && TESTGEN_DAG.contains("at: node_locus(node: input)")
            && !TESTGEN_DAG.contains("NodeLocus { node: input }"),
        "coproduct-exhaustiveness generation must carry omitted variant into the input node and use canonical node_locus"
    );
    assert!(
        COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG
            .contains("fn generated_coproduct_exhaustiveness_claim() -> Outcome<TestClaim>")
            && COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG
                .contains("testgen_emit_coproduct_exhaustiveness_claim")
            && COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG
                .contains("witness_coproduct_exhaustiveness_diagnostic_claim")
            && COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG
                .contains("witness_coproduct_exhaustiveness_uses_generated_anchor")
            && COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG
                .contains("witness_coproduct_exhaustiveness_all_variants_emit")
            && COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG
                .contains("witness_coproduct_exhaustiveness_generator_count")
            && COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG.contains(
                "length(xs: testgen_scheduled_coproduct_exhaustiveness_generators()) == 4"
            ),
        "generated coproduct-exhaustiveness corpus must consume the testgen emit helper and expose all four missing-arm witnesses"
    );
}

#[test]
fn t22_diagnostic_assert_eval_witnesses_parse() {
    parse_module(DIAGNOSTIC_ASSERT_EVAL_DAG, DIAGNOSTIC_ASSERT_EVAL_PATH);
}

#[test]
fn sg1b_signature_realization_failclosed_receipt_parses() {
    parse_module(SG1B_FAILCLOSED_DAG, SG1B_FAILCLOSED_PATH);
}

#[test]
fn claim1_refinement_brand_disjointness_is_sugar_scope_parses() {
    parse_module(
        REFINEMENT_BRAND_DISJOINTNESS_IS_SUGAR_SCOPE_DAG,
        REFINEMENT_BRAND_DISJOINTNESS_IS_SUGAR_SCOPE_PATH,
    );
    assert!(
        REFINEMENT_BRAND_DISJOINTNESS_IS_SUGAR_SCOPE_DAG
            .contains("witness_claim1_v4_frontend_not_in_measurement_loop")
            && REFINEMENT_BRAND_DISJOINTNESS_IS_SUGAR_SCOPE_DAG.contains("NOT a measurement"),
        "Claim #1 scope receipt must document v4 frontend honesty (not a vacuous-oracle lock)"
    );
}

#[test]
fn t22_eval_diagnostic_assert_not_deferred_in_substrate() {
    assert!(
        !EVAL_DAG.contains("eval_rejected_assert_kind_deferred"),
        "removed deferred scaffold must not return"
    );
    assert!(
        EVAL_DAG.contains("DiagnosticClaim { expected_rejection: expected")
            && EVAL_DAG.contains("Rejected { diagnostics: expected }"),
        "DiagnosticClaim must execute with polarity-specific rejection carrier (P2)"
    );
    assert!(
        EVAL_DAG.contains("verdict: aggregate_verdicts(") && EVAL_DAG.contains("rs: ["),
        "run_test_claim must route through aggregate_verdicts"
    );
    assert!(
        EVAL_DAG.contains("CompilesClaim { expected_value: expected")
            && EVAL_DAG.contains("Accepted { value: expected, diagnostics: None }"),
        "CompilesClaim must compare actual against declared accepted Node (P2/P3 fail-closed)"
    );
    assert!(
        EVAL_DAG.contains("eval_round_trip_claim_input_for_verdict(input: input)")
            && EVAL_DAG.contains("run_test_claim_round_trip_verdict_runtime(")
            && EVAL_DAG.contains("dag_round_trip_wave1_authorities_ready()"),
        "RoundTripClaim must admit witness input structurally (IRT-3), not runtime-eval TypeNode, and re-derive wave-1 readiness from dag.dag authorities (P2; not Deferred)"
    );
    assert!(
        !EVAL_DAG.contains("eval_rejected_roundtrip_deferred"),
        "removed RoundTripClaim Deferred scaffold must not return"
    );
    assert!(
        !EVAL_DAG.contains("Accepted { value: inputs.root"),
        "eval_node must not fabricate Accepted{{value:inputs.root}} on unrealized eval (CI-signal-integrity: would falsely Pass CompilesClaim/EqualsClaim where expected==input)"
    );
    assert!(
        RUNTIME_DAG.contains("type RuntimeValueNodeProjection")
            && RUNTIME_DAG.contains("RuntimeValueNodeUnrepresentable")
            && RUNTIME_DAG.contains("fn runtime_value_node_projection(value: RuntimeValue) -> RuntimeValueNodeProjection")
            && EVAL_DAG.contains("fn eval_node(tree: InferredTree, inputs: Inputs) -> Outcome<Node>")
            && EVAL_DAG.contains("runtime_value_node_projection(value: value)")
            && EVAL_DAG.contains("eval_rejected_runtime_value_node_unrepresentable"),
        "eval_node must consume std/runtime RuntimeValue-to-Node projection and fail closed when no faithful Node projection exists"
    );
    assert!(
        !RUNTIME_DAG.contains("fn runtime_value_node(value: RuntimeValue) -> Node")
            && !EVAL_DAG.contains("runtime_value_node(value: value)"),
        "eval_node must not accept a hollow RuntimeValue-to-type alias as a realized Node"
    );
    assert!(
        !EVAL_DAG.contains("eval_node_unrealized")
            && !EVAL_DAG.contains("nd.head.reason == eval_node_unrealized"),
        "eval_node_unrealized fail-closed scaffold must dissolve once eval_node projects RuntimeValue into Node"
    );
}

/// T-19 testgen activation ratchet (INVARIANTS §P5 same-path expansion; +0 SG-0).
/// Replaces `scripts/check_t19_testgen_activation.py` after #4252 removed `scripts/`.
/// Substrate parse + structural substring receipts only.
#[test]
fn check_t19_testgen_activation() {
    parse_module(DAG_INPUT_SURFACE_DAG, DAG_INPUT_SURFACE_PATH);
    parse_module(LBE_MANIFEST_DAG, LBE_MANIFEST_PATH);
    parse_module(REFINEMENT_MANIFEST_DAG, REFINEMENT_MANIFEST_PATH);
    parse_module(LBE_GENERATED_DAG, LBE_GENERATED_PATH);
    parse_module(TESTGEN_WISHLIST_DAG, TESTGEN_WISHLIST_PATH);
    parse_module(ALGEBRA_LAW_GENERATED_DAG, ALGEBRA_LAW_GENERATED_PATH);
    parse_module(
        COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG,
        COPRODUCT_EXHAUSTIVENESS_GENERATED_PATH,
    );
    parse_module(REFINEMENT_GENERATED_DAG, REFINEMENT_GENERATED_PATH);
    parse_module(
        IDEMPOTENT_OPERATION_GENERATED_DAG,
        IDEMPOTENT_OPERATION_GENERATED_PATH,
    );

    require_substrings(
        "lens_testgen/dag_input_surface",
        DAG_INPUT_SURFACE_DAG,
        &[
            "witness_lens_testgen_schedules_dag_input_surface_green",
            "witness_lens_testgen_bootstrap_generator_reifies_dag_input_surface_green",
            "scheduled_language_behavior_generators_cover_dag_inputs()",
            "for_all(",
            "language_behavior_generator_uses_disj_dag_input",
            "language_behavior_generator_uses_transform_dag_input",
            "bootstrap_generator_has_conj_dag_input_surface()",
            "dag_language_model_surface_id",
            "testgen_scheduled_language_behavior_generators",
            "bootstrap_claim_generator_for_manual_anchor",
        ],
    );
    forbid_substrings(
        "lens_testgen/dag_input_surface",
        DAG_INPUT_SURFACE_DAG,
        &["compile-only until T-19"],
    );

    require_substrings(
        "lbe_anchor_manifest.dag",
        LBE_MANIFEST_DAG,
        &[
            "ManualLbeConjDagSurface",
            "ManualLbeDisjDagSurface",
            "ManualLbeTransformDagSurface",
        ],
    );
    require_substrings(
        "refinement_preservation_anchor_manifest.dag",
        REFINEMENT_MANIFEST_DAG,
        &["ManualRefinementNonEmptyListBase"],
    );

    parse_module(VERIFICATION_DAG, VERIFICATION_PATH);

    require_substrings(
        VERIFICATION_PATH,
        VERIFICATION_DAG,
        &[
            "ManualLbeConjDagSurface",
            "ManualLbeDisjDagSurface",
            "ManualLbeTransformDagSurface",
            "type TestClaimCoproductVariant",
            "feature:testclaim-coproduct-reflection; bound task: src/v4/TASKS.md#t-19-lenstestgendag--producer-of-testclaim-corpus-from-substrate",
            "follow-up: delete this mirror when T-19 projects arm keys from TestClaim",
            "GeneratedCoproductExhaustiveness { omitted_variant: TestClaimCoproductVariant }",
            "ManualRefinementNonEmptyListBase",
        ],
    );

    require_substrings(
        "testgen.dag",
        TESTGEN_DAG,
        &[
            "| LanguageBehaviorEquivalence {",
            "type LanguageBehaviorEquivalenceSubject",
            "type FrozenLanguageBehaviorSnapshot",
            "type LanguageBehaviorIoMock",
            "fn testgen_emit_language_behavior_equivalence_claim",
            "fn testgen_emit_algebra_law_claim",
            "if lhs == rhs",
            "t19_algebra_law_tautological_sides",
            "fn testgen_emit_idempotent_operation_claim",
            "fn testgen_scheduled_language_behavior_generators",
            "fn testgen_scheduled_idempotent_operation_subjects",
            "dag_language_model_surface_id",
            "ManualLbeConjDagSurface",
            "ManualRefinementNonEmptyListBase",
            "fn testgen_emit_coproduct_exhaustiveness_claim",
            "fn testgen_scheduled_coproduct_exhaustiveness_generators",
            "| RefinementPreservation { subject: RefinementPreservationSubject }",
            "fn testgen_emit_refinement_preservation_claim",
            "refined_base(r: subject.refined)",
        ],
    );

    let concept_body = between(TESTGEN_DAG, "type TestgenConcept", "type Generator");
    assert!(
        concept_body.contains("LanguageBehaviorEquivalence"),
        "LanguageBehaviorEquivalence must be a TestgenConcept variant"
    );
    assert!(
        concept_body.contains("RefinementPreservation"),
        "RefinementPreservation must be a TestgenConcept variant"
    );
    assert!(
        !concept_body.contains("IdempotentOperationSubject"),
        "IdempotentOperationSubject must stay outside the closed seven-way TestgenConcept coproduct"
    );

    let pending = between(
        TESTGEN_WISHLIST_DAG,
        "fn testgen_pending_non_tautological_generator_wishlist",
        "fn testgen_dispatched_non_tautological_generators",
    );
    assert_eq!(
        pending.matches("TestgenWishlistRow {").count(),
        3,
        "generator wishlist must carry exactly three pending non-dispatched rows"
    );
    for shipped in ["slot: AlgebraLaw", "slot: DiagnosticExhaustiveness"] {
        assert!(
            !pending.contains(shipped),
            "{shipped} has generated rows and must stay out of pending wishlist rows"
        );
    }

    let dispatched = between(
        TESTGEN_WISHLIST_DAG,
        "fn testgen_dispatched_non_tautological_generators",
        "fn pending_non_tautological_generator_count_is_three",
    );
    assert_eq!(
        dispatched.matches("TestgenWishlistRow {").count(),
        4,
        "generator wishlist must carry exactly four dispatched rows"
    );
    assert!(
        dispatched.contains("generator: dispatched_language_behavior_equivalence_generator()")
            && dispatched.contains("generator: dispatched_algebra_law_generator()")
            && dispatched.contains("generator: dispatched_diagnostic_exhaustiveness_generator()")
            && dispatched.contains("generator: dispatched_refinement_preservation_generator()"),
        "dispatched wishlist rows must include LBE, AlgebraLaw, DiagnosticExhaustiveness, and RefinementPreservation"
    );

    require_substrings(
        "language_behavior_equivalence.dag",
        LBE_GENERATED_DAG,
        &[
            "run_test_claim_assert",
            "run_test_claim(",
            "witness_lbe_conj_snapshot_pass",
            "witness_lbe_disj_snapshot_pass",
            "witness_lbe_transform_snapshot_pass",
            "witness_testgen_schedules_three_lbe_generators",
            "lbe_io_mock_conj_dag_surface",
        ],
    );

    require_substrings(
        "coproduct_exhaustiveness.dag",
        COPRODUCT_EXHAUSTIVENESS_GENERATED_DAG,
        &[
            "length(xs: testgen_scheduled_coproduct_exhaustiveness_generators()) == 4",
            "witness_coproduct_exhaustiveness_generator_count",
        ],
    );

    require_substrings(
        "refinement_preservation.dag",
        REFINEMENT_GENERATED_DAG,
        &[
            "witness_refinement_preserves_nonempty_list_base",
            "refined_base(r: subject.refined) == subject.original",
        ],
    );

    assert!(
        ALGEBRA_LAW_GENERATED_DAG
            .matches("data generated_nat_")
            .count()
            >= 3,
        "algebra-law generator must produce at least three sample TestClaim rows"
    );
}

#[test]
fn testgen_concept_surface_stays_closed_and_classified() {
    let module = parse_module(TESTGEN_DAG, TESTGEN_PATH);

    assert_eq!(
        variant_name_set(type_sum(&module, "TestgenConcept")),
        expected_name_set(&[
            "TypeConstruction",
            "AlgebraLaw",
            "DiagnosticExhaustiveness",
            "LensApplicability",
            "BidirectionalRoundtrip",
            "LanguageBehaviorEquivalence",
            "RefinementPreservation",
        ]),
        "T-19 scheduling arms must stay the closed seven-way set (LBE + refinement-preservation activation)"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "Generator")),
        expected_field_type_map(&[
            ("kind", "TestClaimCoproductVariant"),
            ("classification", "TestClassification"),
            ("anchor", "ClaimAnchorKey"),
            ("slot", "C"),
            ("provenance", "GeneratorProvenance"),
            ("profile_metadata", "GeneratorProfile"),
        ]),
        "Generator<C> must carry claim kind, anchor, classification, slot, provenance bundle, and profile metadata (F.2-P1 #4316)"
    );
}

#[test]
fn manual_manifest_matches_claim_anchor_discriminants() {
    let connective = parse_module(CONNECTIVE_ANCHORS_DAG, CONNECTIVE_ANCHORS_PATH);
    let nat_laws = parse_module(NAT_LAW_ANCHORS_DAG, NAT_LAW_ANCHORS_PATH);
    let manifest = parse_module(MANIFEST_DAG, MANIFEST_PATH);

    let manifest_keys = manifest_anchor_values(&manifest);
    let claim_keys = claim_anchor_values(&[&connective, &nat_laws]);

    assert_eq!(
        manifest_keys.len(),
        17,
        "T-19 manifest is the seventeen live anchors"
    );
    assert_eq!(
        claim_keys, manifest_keys,
        "manual TestClaim.anchor values must join to the manifest by the same ManualAnchorKey discriminants"
    );
    assert!(
        !claim_keys.contains("ManualAnchorAbsent"),
        "the seventeen live manual anchors must not route through the absent sentinel"
    );
}

#[test]
fn claim_corpus_has_no_direct_manual_anchor_assignments() {
    let root = workspace_root().join("src/v4/test/claim");
    let mut dag_files = Vec::new();
    collect_dag_files(&root, &mut dag_files);
    dag_files.push(workspace_root().join("src/v4/lens/testgen.dag"));

    let mut offenders = Vec::new();
    for path in dag_files {
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let file_str = path
            .strip_prefix(workspace_root())
            .unwrap_or(path.as_path())
            .to_str()
            .unwrap_or("")
            .to_string();
        let Ok(tokens) = tokenize_for_test(&source, &file_str) else {
            continue;
        };
        let Ok(module) = parse_for_test(&tokens, &file_str) else {
            continue;
        };
        for item in &module.items {
            let SurfaceItem::Data {
                name: data_name,
                ty: SurfaceType::Named { name: ty_name, .. },
                body: Some(body),
                ..
            } = item
            else {
                continue;
            };
            if ty_name == "TestClaim" && testclaim_anchor_is_direct_var(body) {
                offenders.push(format!("{file_str}: data {data_name}"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "TestClaim.anchor fields must use manual_claim_anchor(...) after the ClaimAnchorKey split:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn t20_bootstrap_plan_keeps_self_hosting_chain_as_data() {
    let module = parse_module(BOOTSTRAP_DAG, BOOTSTRAP_PATH);

    assert_eq!(
        record_field_type_map(type_record(&module, "CompileStage")),
        expected_field_type_map(&[
            ("consumes", "List<Symbol>"),
            ("produces", "Symbol"),
            ("produces_hash", "BootstrapHashPin"),
            ("compiled_by", "Symbol"),
        ]),
        "CompileStage must keep the compiler-of-record as a structural field"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "BootstrapHashPin")),
        expected_field_type_map(&[("digest", "Hash"), ("pin", "Symbol")]),
        "BootstrapHashPin must keep digest and symbolic pin as structured fields"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "FixptStage1Stage2")),
        expected_field_type_map(&[
            ("left", "Symbol"),
            ("left_hash", "BootstrapHashPin"),
            ("right", "Symbol"),
            ("right_hash", "BootstrapHashPin"),
            ("pinned_hash", "BootstrapHashPin"),
        ]),
        "FixptStage1Stage2 must carry only the canonical compared hashes and fixed-point pin"
    );
    assert!(
        !BOOTSTRAP_DAG.contains("type BootstrapFixptWitness")
            && BOOTSTRAP_DAG.contains("fn bootstrap_fixpt_witness(f: FixptStage1Stage2) -> Witness<FixptStage1Stage2>")
            && BOOTSTRAP_DAG.contains("data bootstrap_plan_fixpt_witness: Witness<FixptStage1Stage2>"),
        "fixpt proof must flow through canonical Witness<FixptStage1Stage2> without an N-wrapper record"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "BootstrapPlan")),
        expected_field_type_map(&[
            ("seed", "CompileStage"),
            ("self0", "CompileStage"),
            ("self1", "CompileStage"),
            ("fixpt", "FixptStage1Stage2"),
        ]),
        "BootstrapPlan must stay the seed/self0/self1/fixpt chain"
    );

    let plan = data_expr(&module, "bootstrap_plan");
    let call_args = match plan {
        SurfaceExpr::Call { target, args, .. } => {
            assert_eq!(target, "bootstrap_plan_well_formed");
            args
        }
        other => panic!("bootstrap_plan must call bootstrap_plan_well_formed, got {other:?}"),
    };
    assert_eq!(
        call_args.len(),
        1,
        "bootstrap_plan_well_formed takes one plan"
    );
    let bootstrap_plan_arg = match &call_args[0] {
        SurfaceExpr::Record { fields, .. } => record_field_expr(fields, "p"),
        other => other,
    };
    let bootstrap_plan_fields = match bootstrap_plan_arg {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "BootstrapPlan");
            fields
        }
        other => panic!("bootstrap_plan argument must be a BootstrapPlan record, got {other:?}"),
    };

    assert_compile_stage(
        record_field_expr(bootstrap_plan_fields, "seed"),
        &["v4_dag_source"],
        "v4_stage0_binary",
        ("v4_stage0_hash", "v4_stage0_hash_pin"),
        "v2_pipeline",
    );
    assert_compile_stage(
        record_field_expr(bootstrap_plan_fields, "self0"),
        &["v4_dag_source", "v4_stage0_binary"],
        "v4_stage1_binary",
        ("v4_stage1_hash", "v4_stage1_hash_pin"),
        "v4_stage0_binary",
    );
    assert_compile_stage(
        record_field_expr(bootstrap_plan_fields, "self1"),
        &["v4_dag_source", "v4_stage1_binary"],
        "v4_stage2_binary",
        ("v4_stage1_hash", "v4_stage2_hash_pin"),
        "v4_stage1_binary",
    );
    assert_fixpt(
        record_field_expr(bootstrap_plan_fields, "fixpt"),
        "v4_stage1_binary",
        ("v4_stage1_hash", "v4_stage1_hash_pin"),
        "v4_stage2_binary",
        ("v4_stage1_hash", "v4_stage2_hash_pin"),
        ("v4_stage1_hash", "pinned_v4_fixed_point_hash_pin"),
    );
    assert!(
        BOOTSTRAP_DAG.contains("data v4_stage2_hash: Hash = v4_stage2_hash")
            && BOOTSTRAP_DAG.contains(
                "data pinned_v4_fixed_point_hash: Hash = pinned_v4_fixed_point_hash"
            ),
        "bootstrap must declare stage-2 and pinned digest carriers as independent Hash facts (A2+A3)"
    );
    assert!(
        BOOTSTRAP_DAG.contains("p.self0.produces_hash.digest == p.self1.produces_hash.digest")
            && BOOTSTRAP_DAG
                .contains("p.fixpt.left_hash.digest == p.self0.produces_hash.digest")
            && BOOTSTRAP_DAG
                .contains("p.fixpt.right_hash.digest == p.self1.produces_hash.digest")
            && BOOTSTRAP_DAG
                .contains("p.fixpt.pinned_hash.digest == p.self0.produces_hash.digest")
            && BOOTSTRAP_DAG.contains("match bootstrap_fixpt_witness(f: p.fixpt)")
            && BOOTSTRAP_DAG.contains("Holds { value: _ } =>")
            && BOOTSTRAP_DAG.contains("data bootstrap_plan_fixpt_witness: Witness<FixptStage1Stage2>")
            && !BOOTSTRAP_DAG.contains("BootstrapFixptWitness")
            && !BOOTSTRAP_DAG.contains("fn bootstrap_fixpt_holds")
            && !BOOTSTRAP_DAG.contains("p.fixpt.witness"),
        "bootstrap_plan_well_formed must enforce digest convergence through canonical fixpt fields and consume the derived witness directly (A2+A3/P2)"
    );
    assert!(
        !BOOTSTRAP_DAG.contains("p.fixpt.left_hash.pin == p.fixpt.right_hash.pin")
            && !BOOTSTRAP_DAG.contains("p.fixpt.left_hash.pin == p.fixpt.pinned_hash.pin"),
        "fixpt pins identify independent carrier slots; digest equality proves convergence (A2+A3)"
    );
    const CONNECTIVE_ANCHORS: &str =
        include_str!("../../../../v4/test/claim/manual/connective_anchors.dag");
    assert!(
        CONNECTIVE_ANCHORS.contains("claim_arrow_empty_rejected")
            && CONNECTIVE_ANCHORS.contains("claim_transform_empty_rejected")
            && CONNECTIVE_ANCHORS.contains("claim_branch_empty_rejected")
            && CONNECTIVE_ANCHORS.contains("claim_loop_empty_rejected")
            && CONNECTIVE_ANCHORS.contains("claim_bind_zero_children_rejected"),
        "A1 arity gate must pin rejection receipts for Arrow/Transform/Branch/Loop/Bind shapes (TESTING.md regression discipline)"
    );
    assert!(
        BOOTSTRAP_DAG.contains("bootstrap_plan_fixpt_digest_mismatch_rejects")
            && BOOTSTRAP_DAG.contains("produces_hash: BootstrapHashPin { digest: v4_stage2_hash, pin: v4_stage2_hash_pin")
            && BOOTSTRAP_DAG.contains("right_hash: BootstrapHashPin { digest: v4_stage2_hash, pin: v4_stage2_hash_pin"),
        "mismatch regression must wire stage-2 and fixpt.right through v4_stage2_hash (not an unrelated seed digest)"
    );
}

#[test]
fn rr_a_step2_bootstrap_evaluator_corpus_harness_entry() {
    let _ = parse_module(BOOTSTRAP_DAG, BOOTSTRAP_PATH);
    let _ = parse_module(CLI_DAG, CLI_PATH);

    assert!(
        BOOTSTRAP_DAG.contains("type BootstrapEvaluatorCorpusHarnessEntry")
            && BOOTSTRAP_DAG.contains("fn bootstrap_evaluator_corpus_harness_entry()")
            && BOOTSTRAP_DAG.contains("entry_fn: run_manual_testclaim_corpus_eval")
            && BOOTSTRAP_DAG.contains("data run_manual_testclaim_corpus_eval: Symbol = run_manual_testclaim_corpus_eval")
            && BOOTSTRAP_DAG.contains("entry_module: v4_test_claim_workflow_testclaim_corpus_runner")
            && BOOTSTRAP_DAG.contains("runtime_model: v4_evaluator_runtime_wave1()")
            && BOOTSTRAP_DAG.contains("stage0_binary: v4_stage0_binary")
            && BOOTSTRAP_DAG.contains("data witness_bootstrap_evaluator_corpus_harness_well_formed: Bool"),
        "RR-A §5.2: bootstrap must model stage0 corpus harness with wave1 runtime pin and run_manual_testclaim_corpus_eval entry"
    );
    assert!(
        CLI_DAG.contains("type GunbcTestCorpusHarnessRoute")
            && CLI_DAG.contains(
                "data gunbc_test_manual_corpus_harness_route: GunbcTestCorpusHarnessRoute"
            )
            && CLI_DAG.contains("harness: bootstrap_manual_corpus_harness")
            && CLI_DAG.contains(".harness.entry_fn == run_manual_testclaim_corpus_eval")
            && CLI_DAG.contains("fn gunbc_test_manual_corpus_harness_route_well_formed()"),
        "cli.dag must expose harness route typed separately from GunbcTestRoute.selection_fn (P2)"
    );
}

/// Returns true when a `data x: TestClaim = …` body directly assigns a ManualAnchorKey
/// variant to the `anchor` field (not wrapped in `manual_claim_anchor`).
/// Helper-function bodies (arity_rejection_claim, testgen_emit_*, etc.) are Call
/// expressions and are never flagged — they wrap the anchor internally.
fn testclaim_anchor_is_direct_var(body: &SurfaceExpr) -> bool {
    let SurfaceExpr::VariantRecord { fields, .. } = body else {
        return false;
    };
    let Some(anchor_expr) = fields
        .iter()
        .find_map(|f| (f.name == "anchor").then_some(&f.value))
    else {
        return false;
    };
    matches!(anchor_expr, SurfaceExpr::Var { .. })
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn collect_dag_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|e| panic!("read_dir {}: {e}", root.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "dag") {
            out.push(path);
        }
    }
}

fn parse_module(source: &str, file: &str) -> SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn type_sum<'a>(module: &'a SurfaceModule, name: &str) -> &'a [SurfaceVariant] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(variants.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum {name}"))
}

fn type_record<'a>(module: &'a SurfaceModule, name: &str) -> &'a [SurfaceField] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record {name}"))
}

fn data_expr<'a>(module: &'a SurfaceModule, name: &str) -> &'a SurfaceExpr {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(body),
                ..
            } if item_name == name => Some(body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data {name}"))
}

fn variant_name_set(variants: &[SurfaceVariant]) -> BTreeSet<&str> {
    variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect()
}

fn expected_name_set(names: &[&'static str]) -> BTreeSet<&'static str> {
    names.iter().copied().collect()
}

fn record_field_type_map(fields: &[SurfaceField]) -> BTreeMap<&str, String> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), surface_type_name(&field.ty)))
        .collect()
}

fn expected_field_type_map(
    fields: &[(&'static str, &'static str)],
) -> BTreeMap<&'static str, String> {
    fields
        .iter()
        .map(|(name, ty)| (*name, (*ty).to_string()))
        .collect()
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    TypeAngleArg::TypeExpr { ty } => surface_type_name(ty),
                    TypeAngleArg::WidthNatLiteral { decimal, .. } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        SurfaceType::Optional { inner, .. } => format!("?{}", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

fn manifest_anchor_values(module: &SurfaceModule) -> BTreeSet<&str> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Data {
                ty: SurfaceType::Named { name: ty_name, .. },
                body: Some(SurfaceExpr::Var { name, .. }),
                ..
            } if ty_name == "ManualAnchorKey" => Some(name.as_str()),
            _ => None,
        })
        .collect()
}

fn claim_anchor_values<'a>(modules: &[&'a SurfaceModule]) -> BTreeSet<&'a str> {
    modules
        .iter()
        .flat_map(|module| module.items.iter())
        .filter_map(|item| match item {
            SurfaceItem::Data {
                ty: SurfaceType::Named { name: ty_name, .. },
                body: Some(body),
                ..
            } if ty_name == "TestClaim" => Some(claim_anchor_name(body)),
            _ => None,
        })
        .collect()
}

fn claim_anchor_name(body: &SurfaceExpr) -> &str {
    let anchor_expr = match body {
        SurfaceExpr::VariantRecord { fields, .. } => record_field_expr(fields, "anchor"),
        SurfaceExpr::Call { target, args, .. } if target == "arity_rejection_claim" => {
            let SurfaceExpr::Record { fields, .. } = args
                .first()
                .unwrap_or_else(|| panic!("arity_rejection_claim must take one named-arg record"))
            else {
                panic!(
                    "arity_rejection_claim args must desugar to Record, got {:?}",
                    args.first()
                );
            };
            record_field_expr(fields, "anchor")
        }
        other => panic!(
            "TestClaim data body must be variant record or arity_rejection_claim, got {other:?}"
        ),
    };
    match anchor_expr {
        SurfaceExpr::Var { name, .. } => name.as_str(),
        SurfaceExpr::Call { target, args, .. } if target == "manual_claim_anchor" => {
            let SurfaceExpr::Record { fields, .. } = args
                .first()
                .unwrap_or_else(|| panic!("manual_claim_anchor must take one named-arg record"))
            else {
                panic!(
                    "manual_claim_anchor args must desugar to Record, got {:?}",
                    args.first()
                );
            };
            match record_field_expr(fields, "anchor") {
                SurfaceExpr::Var { name, .. } => name.as_str(),
                other => panic!(
                    "manual claim anchor wrapper must carry a discriminant var, got {other:?}"
                ),
            }
        }
        other => panic!("TestClaim.anchor must be a discriminant var, got {other:?}"),
    }
}

fn record_field_expr<'a>(fields: &'a [SurfaceRecordField], name: &str) -> &'a SurfaceExpr {
    fields
        .iter()
        .find_map(|field| (field.name == name).then_some(&field.value))
        .unwrap_or_else(|| panic!("missing record field {name}"))
}

fn require_substrings(label: &str, text: &str, needles: &[&str]) {
    let missing: Vec<_> = needles
        .iter()
        .copied()
        .filter(|n| !text.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "{label}: missing required substrings: {missing:?}"
    );
}

fn forbid_substrings(label: &str, text: &str, needles: &[&str]) {
    let present: Vec<_> = needles
        .iter()
        .copied()
        .filter(|n| text.contains(n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: forbidden substrings present: {present:?}"
    );
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    text.split_once(start)
        .and_then(|(_, tail)| tail.split_once(end).map(|(middle, _)| middle))
        .unwrap_or_else(|| panic!("missing expected span from {start:?} to {end:?}"))
}

fn assert_compile_stage(
    expr: &SurfaceExpr,
    expected_consumes: &[&str],
    expected_produces: &str,
    expected_produces_hash: (&str, &str),
    expected_compiled_by: &str,
) {
    let fields = match expr {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "CompileStage");
            fields
        }
        other => panic!("expected CompileStage record, got {other:?}"),
    };

    assert_eq!(
        list_var_names(record_field_expr(fields, "consumes")),
        expected_consumes,
        "CompileStage.consumes drifted"
    );
    assert_eq!(
        var_name(record_field_expr(fields, "produces")),
        expected_produces,
        "CompileStage.produces drifted"
    );
    assert_hash_pin(
        record_field_expr(fields, "produces_hash"),
        expected_produces_hash,
        "CompileStage.produces_hash",
    );
    assert_eq!(
        var_name(record_field_expr(fields, "compiled_by")),
        expected_compiled_by,
        "CompileStage.compiled_by drifted"
    );
}

fn assert_fixpt(
    expr: &SurfaceExpr,
    expected_left: &str,
    expected_left_hash: (&str, &str),
    expected_right: &str,
    expected_right_hash: (&str, &str),
    expected_pinned_hash: (&str, &str),
) {
    let fields = match expr {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "FixptStage1Stage2");
            fields
        }
        other => panic!("expected FixptStage1Stage2 record, got {other:?}"),
    };

    assert_eq!(var_name(record_field_expr(fields, "left")), expected_left);
    assert_hash_pin(
        record_field_expr(fields, "left_hash"),
        expected_left_hash,
        "FixptStage1Stage2.left_hash",
    );
    assert_eq!(var_name(record_field_expr(fields, "right")), expected_right);
    assert_hash_pin(
        record_field_expr(fields, "right_hash"),
        expected_right_hash,
        "FixptStage1Stage2.right_hash",
    );
    assert_hash_pin(
        record_field_expr(fields, "pinned_hash"),
        expected_pinned_hash,
        "FixptStage1Stage2.pinned_hash",
    );
}

fn assert_hash_pin(expr: &SurfaceExpr, expected: (&str, &str), label: &str) {
    let fields = match expr {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "BootstrapHashPin", "{label} target drifted");
            fields
        }
        other => panic!("{label} must be a BootstrapHashPin record, got {other:?}"),
    };

    assert_eq!(
        var_name(record_field_expr(fields, "digest")),
        expected.0,
        "{label}.digest drifted"
    );
    assert_eq!(
        var_name(record_field_expr(fields, "pin")),
        expected.1,
        "{label}.pin drifted"
    );
}

fn list_var_names(expr: &SurfaceExpr) -> Vec<&str> {
    match expr {
        SurfaceExpr::List { elements, .. } => elements.iter().map(var_name).collect(),
        other => panic!("expected list of symbols, got {other:?}"),
    }
}

fn var_name(expr: &SurfaceExpr) -> &str {
    match expr {
        SurfaceExpr::Var { name, .. } => name,
        other => panic!("expected symbol var, got {other:?}"),
    }
}
