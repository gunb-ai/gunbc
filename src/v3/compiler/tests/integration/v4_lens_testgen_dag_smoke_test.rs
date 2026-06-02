//! **Layer:** integration
//!
//! T-19 Wave-0: `src/v4/lens/testgen.dag` parses and exposes claim-anchor-key-driven
//! `Generator` wiring (`kind` + `anchor` + `classification` + `slot: TestgenConcept`).
//! `ManualAnchorAbsent` is fail-closed on bootstrap via `Outcome` on `manual_test_claim_for_manual_anchor`.
//! `Generator.anchor` repeats **`ClaimAnchorKey`** from the selected **`TestClaim`** (single
//! carrier authority; manual rows use `ManualClaimAnchor`). `testgen_concept_for_manual_claim`
//! matches on `claim.anchor` so the slot projection stays aligned with the claim row.
//! **Note:** `compile_to_dag` on this module alone does not resolve `import v4.std.*` peers
//! (Import lowering is still M2-scoped); full merge compile lands with cross-file M2 per TASKS T-19.
//!
//! **TESTING.md:** Each `#[test]` is one slice (**#14812**). Where the surface AST exposes the
//! contract (`module` path, `TypeSum`/`Import` rows, typed `fn` return types, `Generator` fields),
//! assertions use **`parse_for_test`**. Remaining `str::contains` probes intentionally pin a few
//! Wave-0 wiring sentences inside `bootstrap_claim_generator_for_manual_anchor` / `Generator`
//! construction until M2 can compile this module end-to-end (codex **#14839** — bounded parse
//! ratchet, not a permanent substitute for `.dag` `TestClaim` coverage).
//!
//! **INVARIANTS §P5 checkable receipt (mechanism (b), SG-0 delta 0):** this file's row in
//! `sg0_census_test.rs` `EXPECTED_HAND_AUTHORED_TEST` is unchanged — no new hand-Rust path;
//! `sg0_v3_test_hand_authored_subratchet` enforces disk-vs-list parity on that invariant.
//! F.2-P1/F.2-P2 same-path tests expand inside this harness only. Explicit deferral:
//! **ROADMAP.md** **T-PB-B** / `pb_rust_tests_outside_residual_zero`
//! (`ROADMAP.md:43` Public Operational Lanes, `ROADMAP.md:63` Nine lanes); dissolve when T-22 runs
//! `lens_testgen/generator_provenance.dag` + `lens_testgen/shadow_ci_receipt.dag` `EqualsClaim`
//! end-to-end. PR #4265 added T-38B `lens_effect/effect_depends_on` pins the same way.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const VERIFICATION_DAG: &str = include_str!("../../../../v4/std/verification.dag");
const P9_REGISTRY_OWNER_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_cost/p9_llvm_instruction_cost_registry_owner.dag");
const P9_REGISTRY_OWNER_PATH: &str =
    "src/v4/test/claim/lens_cost/p9_llvm_instruction_cost_registry_owner.dag";
const NAT_LAW_DAG: &str = include_str!("../../../../v4/test/claim/manual/nat_law_anchors.dag");
const NAT_SUBSTRATE_DAG: &str = include_str!("../../../../v4/std/nat.dag");
const WITNESS_VALIDITY_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/witness_validity.dag");
const WITNESS_VALIDITY_PATH: &str = "src/v4/test/claim/generated/witness_validity.dag";
const LENS_TESTGEN_DAG_INPUT_SURFACE_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_testgen/dag_input_surface.dag");
const LENS_TESTGEN_DAG_INPUT_SURFACE_PATH: &str =
    "src/v4/test/claim/lens_testgen/dag_input_surface.dag";
const LENS_EFFECT_DEPENDS_ON_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_effect/effect_depends_on.dag");
const LENS_EFFECT_DEPENDS_ON_PATH: &str = "src/v4/test/claim/lens_effect/effect_depends_on.dag";
const LENS_TESTGEN_GENERATOR_PROVENANCE_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_testgen/generator_provenance.dag");
const LENS_TESTGEN_GENERATOR_PROVENANCE_PATH: &str =
    "src/v4/test/claim/lens_testgen/generator_provenance.dag";
const LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_testgen/shadow_ci_receipt.dag");
const LENS_TESTGEN_SHADOW_CI_RECEIPT_PATH: &str =
    "src/v4/test/claim/lens_testgen/shadow_ci_receipt.dag";
const ROADMAP: &str = include_str!("../../../../../ROADMAP.md");

const NAT_MANUAL_CLAIM_DATA: [&str; 6] = [
    "claim_nat_add_left_identity",
    "claim_nat_add_right_identity",
    "claim_nat_add_associativity",
    "claim_nat_mul_left_identity",
    "claim_nat_mul_annihilator",
    "claim_nat_mul_associativity",
];

#[test]
fn v4_lens_testgen_p9_registry_owner_claim_parses_and_checks_registry_exclusivity() {
    parse_module(P9_REGISTRY_OWNER_DAG, P9_REGISTRY_OWNER_PATH);
    assert!(
        P9_REGISTRY_OWNER_DAG.contains("lens_owned_fn_registry_v0")
            && P9_REGISTRY_OWNER_DAG.contains("p9_owned_fn_row_eq")
            && P9_REGISTRY_OWNER_DAG.contains("count_equal(")
            && P9_REGISTRY_OWNER_DAG.contains("item: lens_owned_fn_llvm_instruction_cost")
            && P9_REGISTRY_OWNER_DAG.contains("EqualsClaim {"),
        "P9 receipt must prove exactly one full registry row matches canonical fn+owner (B1)"
    );
}

#[test]
fn v4_lens_testgen_wave0_modules_tokenize_and_parse() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let verification = parse_module(VERIFICATION_DAG, "src/v4/std/verification.dag");
    let lens_testgen_claim = parse_module(
        LENS_TESTGEN_DAG_INPUT_SURFACE_DAG,
        LENS_TESTGEN_DAG_INPUT_SURFACE_PATH,
    );
    assert_eq!(
        module_paths(&testgen),
        vec![vec!["v4", "lens", "testgen"]],
        "T-19 authority module should remain v4.lens.testgen"
    );
    assert_eq!(
        module_paths(&verification),
        vec![vec!["v4", "std", "verification"]],
        "`TestClaim` schema should remain v4.std.verification"
    );
    assert_eq!(
        module_paths(&lens_testgen_claim),
        vec![vec![
            "v4",
            "test",
            "claim",
            "lens_testgen",
            "dag_input_surface"
        ]],
        "lens_testgen claim module should stay under recursive T-22 discovery"
    );
}

#[test]
fn v4_lens_testgen_dag_input_surface_claims_are_testclaim_data() {
    let module = parse_module(
        LENS_TESTGEN_DAG_INPUT_SURFACE_DAG,
        LENS_TESTGEN_DAG_INPUT_SURFACE_PATH,
    );
    assert!(
        function_count(&module, "language_behavior_generator_uses_conj_dag_input") == 1
            && function_count(&module, "language_behavior_generator_uses_disj_dag_input") == 1
            && function_count(&module, "language_behavior_generator_uses_transform_dag_input") == 1
            && function_count(&module, "language_behavior_generator_uses_dag_input") == 1
            && function_count(&module, "scheduled_language_behavior_generators_cover_dag_inputs")
                == 1
            && function_count(&module, "bootstrap_generator_has_conj_dag_input_surface") == 1,
        "{LENS_TESTGEN_DAG_INPUT_SURFACE_PATH}: claim file must prove scheduled + bootstrap .dag input surface"
    );
    assert!(
        LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("dag_language_model_surface_id")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("testgen_scheduled_language_behavior_generators")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("bootstrap_claim_generator_for_manual_anchor")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("data claim_lens_testgen_schedules_dag_input_surface: TestClaim = EqualsClaim")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("data claim_lens_testgen_bootstrap_generator_reifies_dag_input_surface: TestClaim = EqualsClaim")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains(
                "data witness_lens_testgen_schedules_dag_input_surface_green: Bool"
            )
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains(
                "data witness_lens_testgen_bootstrap_generator_reifies_dag_input_surface_green: Bool"
            )
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("for_all(")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains(
                "language_behavior_generator_uses_disj_dag_input"
            )
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains(
                "language_behavior_generator_uses_transform_dag_input"
            )
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("stub_empty_disj")
            && LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("stub_empty_transform")
            && !LENS_TESTGEN_DAG_INPUT_SURFACE_DAG.contains("compile-only until T-19"),
        "{LENS_TESTGEN_DAG_INPUT_SURFACE_PATH}: missing .dag input surface TestClaim wiring or green witnesses"
    );
}

// F.2-P1 / F.2-P2 — three same-path `#[test]` slices below; census row unchanged (see module doc).
#[test]
fn v4_lens_testgen_generator_carries_provenance_and_profile_fields() {
    // F.2-P1: Generator<C> gains a provenance bundle (GeneratorProvenance, authored in
    // v4.std.artifact — no duplicated artifact law here) + testgen-local profile_metadata.
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let prov_ty =
        generator_field_ty(&testgen, "provenance").expect("Generator should declare `provenance`");
    assert!(
        matches!(prov_ty, SurfaceType::Named { name: n, .. } if n == "GeneratorProvenance"),
        "Generator.provenance must be GeneratorProvenance (bundle authored in v4.std.artifact); got {prov_ty:?}"
    );
    let profile_ty = generator_field_ty(&testgen, "profile_metadata")
        .expect("Generator should declare `profile_metadata`");
    assert!(
        matches!(profile_ty, SurfaceType::Named { name: n, .. } if n == "GeneratorProfile"),
        "Generator.profile_metadata must be GeneratorProfile (testgen-local concept); got {profile_ty:?}"
    );
    let artifact_imports = import_names_for_path(&testgen, &["v4", "std", "artifact"])
        .expect("testgen must import provenance carriers from v4.std.artifact");
    for sym in [
        "GeneratorId",
        "GeneratorProvenance",
        "test_claim_generated_artifact",
    ] {
        assert!(
            artifact_imports.iter().any(|n| n == sym),
            "testgen must import `{sym}` from v4.std.artifact (single-authority cross-ref, no duplicated artifact law); got {artifact_imports:?}"
        );
    }
    assert!(
        TESTGEN_DAG.contains("fn scheduled_generators_carry_provenance()")
            && TESTGEN_DAG.contains("fn generator_carries_provenance"),
        "testgen must expose the provenance-integrity witness fns (close-criterion witness)"
    );
    assert!(
        TESTGEN_DAG.contains("fn testgen_scheduled_generators_outcome()")
            && TESTGEN_DAG.contains("fn testgen_scheduled_generators_roster_holds()")
            && TESTGEN_DAG.contains("type TestgenRunReceipt")
            && TESTGEN_DAG.contains("fn testgen_run_receipt_outcome(")
            && TESTGEN_DAG.contains("fn generator_matches_profile(")
            && !TESTGEN_DAG.contains("fn testgen_scheduled_generators()"),
        "testgen must expose F.2-P2 Outcome roster authority (no silent Rejected→Empty truncation)"
    );
    // Identity must be DERIVED from the row's canonical ClaimAnchorKey (single authority,
    // unique per row) — not a coarse static category symbol shared across distinct anchors.
    assert!(
        TESTGEN_DAG
            .contains("fn generator_id_for_claim_anchor(anchor: ClaimAnchorKey) -> GeneratorId")
            && TESTGEN_DAG
                .contains("fn generator_id_for_manual_anchor(key: ManualAnchorKey) -> GeneratorId"),
        "testgen must derive GeneratorId from the canonical anchor (no per-row collision)"
    );
    assert!(
        !TESTGEN_DAG.contains("testgen_gen_id_bootstrap_manual_anchor")
            && !TESTGEN_DAG.contains("testgen_gen_id_algebra_law"),
        "the shared static GeneratorIds (one id across distinct-anchor rows) must be gone — identity is anchor-derived"
    );
    // Provenance must not re-declare artifact law in the testgen lens.
    assert!(
        !TESTGEN_DAG.contains("type GeneratorProvenance")
            && !TESTGEN_DAG.contains("type GeneratorId"),
        "GeneratorProvenance/GeneratorId are authored in v4.std.artifact; testgen only consumes them"
    );
}

#[test]
fn v4_lens_testgen_generator_provenance_claim_parses_and_pins_witness() {
    let module = parse_module(
        LENS_TESTGEN_GENERATOR_PROVENANCE_DAG,
        LENS_TESTGEN_GENERATOR_PROVENANCE_PATH,
    );
    assert_eq!(
        module_paths(&module),
        vec![vec![
            "v4",
            "test",
            "claim",
            "lens_testgen",
            "generator_provenance"
        ]],
        "provenance claim module should stay under recursive T-22 discovery"
    );
    assert!(
        LENS_TESTGEN_GENERATOR_PROVENANCE_DAG.contains(
            "import v4.lens.testgen { scheduled_generators_carry_provenance }"
        ) && LENS_TESTGEN_GENERATOR_PROVENANCE_DAG.contains(
            "data claim_lens_testgen_scheduled_generators_carry_provenance: TestClaim = EqualsClaim"
        ) && LENS_TESTGEN_GENERATOR_PROVENANCE_DAG.contains(
            "data witness_lens_testgen_scheduled_generators_carry_provenance_green: Bool = scheduled_generators_carry_provenance()"
        ),
        "{LENS_TESTGEN_GENERATOR_PROVENANCE_PATH}: must pin the testgen provenance witness via EqualsClaim + green Bool routed through the lens helper"
    );
}

#[test]
fn v4_lens_testgen_p5_roadmap_t_pb_b_deferral_is_checkable() {
    assert!(
        ROADMAP.contains("### Nine lanes")
            && ROADMAP.contains("| **T-PB-B** | `pb_rust_tests_outside_residual_zero`")
            && ROADMAP.contains("T-PB-B / `pb_rust_tests_outside_residual_zero`"),
        "P5 deferral must bind to checkable T-PB-B authority (Nine lanes + Public Operational Lanes)"
    );
}

#[test]
fn v4_lens_testgen_shadow_ci_receipt_claim_parses_and_pins_witness() {
    let module = parse_module(
        LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG,
        LENS_TESTGEN_SHADOW_CI_RECEIPT_PATH,
    );
    assert_eq!(
        module_paths(&module),
        vec![vec![
            "v4",
            "test",
            "claim",
            "lens_testgen",
            "shadow_ci_receipt"
        ]],
        "shadow CI receipt claim module should stay under recursive T-22 discovery"
    );
    assert!(
        LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG.contains(
            "import v4.lens.testgen {"
        ) && LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG.contains("testgen_scheduled_generators_outcome")
            && LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG.contains("testgen_run_receipt_outcome")
            && LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG.contains(
                "data claim_lens_testgen_shadow_ci_run_receipt: TestClaim = EqualsClaim"
            )
            && LENS_TESTGEN_SHADOW_CI_RECEIPT_DAG.contains(
                "data witness_lens_testgen_shadow_ci_run_receipt_green: Bool = lens_testgen_shadow_ci_run_receipt_holds()"
            ),
        "{LENS_TESTGEN_SHADOW_CI_RECEIPT_PATH}: must pin F.2-P2 TestgenRunReceipt witness via EqualsClaim + green Bool"
    );
}

#[test]
fn v4_lens_effect_depends_on_routes_through_testclaim_run() {
    tokenize_for_test(LENS_EFFECT_DEPENDS_ON_DAG, LENS_EFFECT_DEPENDS_ON_PATH)
        .unwrap_or_else(|e| panic!("{LENS_EFFECT_DEPENDS_ON_PATH}: tokenize: {e:?}"));
    assert!(
        LENS_EFFECT_DEPENDS_ON_DAG.matches("fn effect_depends_on_claim_holds").count() == 1
            && LENS_EFFECT_DEPENDS_ON_DAG.matches("fn effect_context").count() == 1,
        "{LENS_EFFECT_DEPENDS_ON_PATH}: lens_effect family receipt must keep its local predicate and runtime context"
    );
    assert!(
        LENS_EFFECT_DEPENDS_ON_DAG.contains(
            "data claim_lens_effect_depends_on_runtime_verdict: TestClaim = EqualsClaim"
        ) && LENS_EFFECT_DEPENDS_ON_DAG.contains(
            "data subject_lens_effect_depends_on_runtime_verdict: TestClaimEvalSubject<Node> = eval_test_claim_subject("
        ) && LENS_EFFECT_DEPENDS_ON_DAG.contains(
            "data run_lens_effect_depends_on_runtime_verdict: TestClaimRun<Node, RuntimeValue> = run_test_claim("
        ) && LENS_EFFECT_DEPENDS_ON_DAG
            .contains("value: RuntimeUnitValue { unit_type: effect_runtime_type_node() }"),
        "{LENS_EFFECT_DEPENDS_ON_PATH}: missing T-38 subject roster/run_test_claim wiring"
    );
}

#[test]
fn v4_lens_testgen_wave0_verification_manual_anchor_key_only() {
    let verification = parse_module(VERIFICATION_DAG, "src/v4/std/verification.dag");
    assert!(
        import_names_for_path(&verification, &["v4", "std", "node"])
            .is_some_and(|names| names.iter().any(|n| n == "Symbol")),
        "verification.dag must import Symbol for diagnostic reason carriers (P2 resolve)"
    );
    assert!(
        VERIFICATION_DAG.contains("expected_value: Node")
            && VERIFICATION_DAG.contains("expected_rejection: NonEmptyDiagnostics"),
        "TestClaim variants must use polarity-specific carriers, not Outcome<Node> (P2 illegal-states)"
    );
    assert!(
        module_declares_type_sum_named(&verification, "ManualAnchorKey"),
        "substrate must declare `type ManualAnchorKey` (parsed `TypeSum`)"
    );
    assert!(
        !module_declares_type_sum_named(&verification, "PresentManualAnchorKey"),
        "substrate must not declare a mirrored present-only `PresentManualAnchorKey` sum (Practice 5 single carrier)"
    );
}

#[test]
fn v4_lens_testgen_wave0_function_inventory_matches_wave0() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let verification = parse_module(VERIFICATION_DAG, "src/v4/std/verification.dag");
    assert_eq!(
        function_count(&testgen, "bootstrap_claim_generator_for_manual_anchor"),
        1,
        "T-19 Wave-0: single generator entrypoint keyed by ManualAnchorKey"
    );
    assert_eq!(
        function_count(&testgen, "testgen_concept_for_manual_claim"),
        1,
        "T-19 Wave-0: concept projection must key off the selected `TestClaim`"
    );
    assert_eq!(
        function_count(&testgen, "testgen_concept_for_manual_anchor"),
        0,
        "legacy `testgen_concept_for_manual_anchor` must not return (claim-keyed projection only)"
    );
    assert_eq!(
        function_count(&testgen, "manual_test_claim_for_manual_anchor"),
        1,
        "T-19 Wave-0: manual TestClaim lookup should live in v4.lens.testgen"
    );
    assert_eq!(
        function_count(&testgen, "bootstrap_assert_kind_for_manual_anchor"),
        0,
        "AssertKind must not be re-authored in testgen; assertion shape lives on TestClaim coproduct"
    );
    assert_eq!(
        function_count(&verification, "present_manual_anchor_key"),
        0,
        "std must not host `present_manual_anchor_key` (Wave-0 fail-close lives in lens/testgen only)"
    );
    assert_eq!(
        function_count(&verification, "assert_kind_for_manual_anchor"),
        0,
        "AssertKind bootstrap mapping must not live in verification (single `ManualAnchorKey` axis)"
    );
    assert_eq!(
        function_count(&testgen, "present_manual_anchor_key_for_claim"),
        0,
        "lens/testgen must not host a mirrored present-key conversion table (codex duplicate-authority fix)"
    );
    let expected_outcome_rt = fn_return_type(&verification, "test_claim_expected_outcome")
        .expect("test_claim_expected_outcome should have an explicit return type");
    assert!(
        type_is_outcome_outcome_named(expected_outcome_rt, "Node"),
        "RoundTripClaim has no declared expected outcome: projection must fail-close as `Outcome<Outcome<Node>>`, not fabricate `Outcome<Node>`; got {expected_outcome_rt:?}"
    );
}

#[test]
fn v4_lens_testgen_wave0_nat_symbol_import_authority() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    assert!(
        !TESTGEN_DAG.contains("bootstrap_algebra")
            && !TESTGEN_DAG.contains("bootstrap_inhabitant"),
        "nat-law AlgebraLawSubject algebra/inhabitant must project from `v4.std.nat` substrate Symbol bundle (no shared bootstrap placeholders)"
    );
    let names = import_names_for_path(&testgen, &["v4", "std", "nat"]).expect(
        "testgen must import `v4.std.nat` for AlgebraLawSubject obligation carriers (parsed `Import`)",
    );
    for sym in [
        "NatAlgebraLawObligation",
        "nat_declared_algebra_law_obligations",
        "law_nat_add_associativity",
        "law_nat_add_left_identity",
        "law_nat_add_right_identity",
        "law_nat_mul_annihilator",
        "law_nat_mul_associativity",
        "law_nat_mul_left_identity",
    ] {
        assert!(
            names.iter().any(|n| n == sym),
            "import v4.std.nat must expose `{sym}`; got {names:?}"
        );
    }
    assert!(
        !TESTGEN_DAG.contains("fn present_manual_anchor_key("),
        "testgen must not define the retired `present_manual_anchor_key(` std-style helper"
    );
}

#[test]
fn v4_lens_testgen_wave0_outcome_return_surfaces() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");

    let manual_claim_rt = fn_return_type(&testgen, "manual_test_claim_for_manual_anchor")
        .expect("manual_test_claim_for_manual_anchor should have an explicit return type");
    assert!(
        type_is_outcome_named(manual_claim_rt, "TestClaim"),
        "manual claim lookup must return `Outcome<TestClaim>`; got {manual_claim_rt:?}"
    );

    let concept_rt = fn_return_type(&testgen, "testgen_concept_for_manual_claim")
        .expect("testgen_concept_for_manual_claim should have an explicit return type");
    assert!(
        type_is_outcome_named(concept_rt, "TestgenConcept"),
        "`TestgenConcept` scheduling projection must return `Outcome<TestgenConcept>`; got {concept_rt:?}"
    );

    let bootstrap_rt = fn_return_type(&testgen, "bootstrap_claim_generator_for_manual_anchor")
        .expect("bootstrap_claim_generator_for_manual_anchor should have an explicit return type");
    assert!(
        type_is_outcome_generator_testgen_concept(bootstrap_rt),
        "T-19 Wave-0 bootstrap must fail-close `ManualAnchorAbsent` via `Outcome<Generator<TestgenConcept>>`; got {bootstrap_rt:?}"
    );
}

#[test]
fn v4_lens_testgen_wave0_generator_anchor_field_is_claim_anchor_key() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let kind_ty = generator_field_ty(&testgen, "kind").expect("Generator should declare `kind`");
    assert!(
        matches!(
            kind_ty,
            SurfaceType::Named { name: n, .. } if n == "TestClaimCoproductVariant"
        ),
        "Generator.kind must carry the TestClaim coproduct variant authority; got {kind_ty:?}"
    );
    let anchor_ty =
        generator_field_ty(&testgen, "anchor").expect("Generator should declare `anchor`");
    assert!(
        matches!(
            anchor_ty,
            SurfaceType::Named { name: n, .. } if n == "ClaimAnchorKey"
        ),
        "Generator.anchor must be `ClaimAnchorKey` (same carrier as `TestClaim.anchor`, with manual/generated variants separated); got {anchor_ty:?}"
    );
}

#[test]
fn v4_lens_testgen_wave0_concept_projection_matches_claim_anchor() {
    assert!(
        TESTGEN_DAG.contains("fn testgen_concept_for_manual_claim")
            && TESTGEN_DAG.contains("match test_claim_anchor(c: claim)"),
        "concept projection must match on `test_claim_anchor(c: claim)` (single authority path with manual claim)"
    );
}

#[test]
fn v4_lens_testgen_wave0_generator_carries_claim_classification_and_anchor() {
    assert!(
        TESTGEN_DAG.contains("kind: test_claim_coproduct_variant(c: claim)"),
        "Generator must take claim kind from manual TestClaim via substrate helper"
    );
    assert!(
        TESTGEN_DAG.contains("classification: test_claim_classification(c: claim)"),
        "Generator must take classification from manual TestClaim via substrate helper"
    );
    assert!(
        TESTGEN_DAG.contains("anchor: test_claim_anchor(c: claim)"),
        "Generator must take ManualAnchorKey from manual TestClaim via substrate helper"
    );
}

#[test]
fn v4_lens_testgen_wave0_bootstrap_threads_claim_anchor_into_generator() {
    assert!(
        !TESTGEN_DAG.contains("fn present_manual_anchor_key_for_claim"),
        "bootstrap must not use a mirrored present-key conversion helper"
    );
    assert!(
        TESTGEN_DAG.contains("anchor: test_claim_anchor(c: claim)"),
        "Generator must wire `anchor` from the manual `TestClaim` row via substrate helper (single authority)"
    );
    assert!(
        TESTGEN_DAG.contains("match testgen_concept_for_manual_claim(claim: claim)"),
        "bootstrap must thread the same `claim` into the concept projection"
    );
}

#[test]
fn v4_lens_testgen_nat_law_manual_claims_use_compiles_stub() {
    assert_nat_manual_claim_blocks_use_compiles_stub(NAT_LAW_DAG);
}

#[test]
fn v4_lens_testgen_nat_substrate_carries_algebra_law_subject_symbols() {
    assert_nat_algebra_law_subject_symbols_in_substrate(NAT_SUBSTRATE_DAG);
}

#[test]
fn v4_lens_testgen_testgen_carries_six_nat_algebra_law_scheduling_arms() {
    assert_six_algebra_law_subject_paths_in_testgen(TESTGEN_DAG);
}

fn assert_nat_manual_claim_blocks_use_compiles_stub(nat_law_src: &str) {
    for claim in NAT_MANUAL_CLAIM_DATA {
        let block = nat_law_manual_claim_data_block(nat_law_src, claim);
        assert!(
            block.contains("CompilesClaim {"),
            "{claim}: nat-law manual stubs use placeholder `input`/`expected_value`; variant must stay `CompilesClaim` until T-22 law-shaped `Node` obligations land"
        );
    }
}

/// Slice one `data <claim>:` row through the byte before the next top-level `data ` row (openai-pro #14822).
fn nat_law_manual_claim_data_block<'a>(nat_law_src: &'a str, claim: &str) -> &'a str {
    let needle = format!("data {claim}:");
    let start = nat_law_src
        .find(&needle)
        .unwrap_or_else(|| panic!("{needle}: missing nat-law manual claim row"));
    let after_header = start + needle.len();
    let tail = &nat_law_src[after_header..];
    let rel = tail.find("\ndata ");
    let end = rel.map(|j| after_header + j).unwrap_or(nat_law_src.len());
    &nat_law_src[start..end]
}

/// `AlgebraLawSubject` nominal `Symbol` carriers live in `v4.std.nat` (substrate), not in claim stubs.
fn assert_nat_algebra_law_subject_symbols_in_substrate(nat_src: &str) {
    for sym in [
        "nat_algebra_law_subject_symbol_additive_monoid",
        "nat_algebra_law_subject_symbol_commutative_semiring",
        "nat_algebra_law_subject_symbol_inhabitant_nat",
    ] {
        let needle = format!("data {sym}:");
        assert!(
            nat_src.contains(&needle),
            "{needle}: missing nat substrate AlgebraLawSubject Symbol ground"
        );
    }
    for forbidden in [
        "nat_algebra_law_subject_symbol_add_operation",
        "nat_algebra_law_subject_symbol_mul_operation",
        "nat_algebra_law_subject_symbol_zero_value",
        "nat_algebra_law_subject_symbol_one_value",
        "nat_algebra_law_subject_symbol_two_value",
        "nat_algebra_law_subject_symbol_three_value",
    ] {
        assert!(
            !nat_src.contains(forbidden),
            "{forbidden}: Nat substrate must not export operation/value mirror symbols"
        );
    }
}

/// Pin the six Nat algebra-law anchors while allowing only the scheduled row and helper-owned
/// AlgebraLaw constructors.
fn assert_six_algebra_law_subject_paths_in_testgen(testgen_src: &str) {
    assert!(
        testgen_src.contains("fn testgen_concept_for_manual_claim"),
        "testgen_concept_for_manual_claim must exist for Wave-0 nat scheduling slice"
    );
    assert!(
        testgen_src.contains(
            "fn algebra_law_subject_for_manual_anchor(anchor: ManualAnchorKey) -> Outcome<AlgebraLawSubject>"
        ),
        "testgen must project Nat algebra-law subjects through one checked helper"
    );
    assert!(
        testgen_src.contains("match algebra_law_subject_for_manual_anchor(anchor: manual_anchor)"),
        "testgen_concept_for_manual_claim must consume the shared Nat algebra-law subject helper"
    );
    assert_eq!(
        testgen_src.matches("value: AlgebraLaw {").count(),
        2,
        "AlgebraLaw construction should stay limited to scheduled dispatch plus helper projection"
    );
    let helper = between(
        testgen_src,
        "fn algebra_law_subject_for_manual_anchor",
        "fn algebra_law_subject_atom",
    );
    for anchor in [
        "ManualNatAddLeftIdentity",
        "ManualNatAddRightIdentity",
        "ManualNatAddAssociativity",
        "ManualNatMulLeftIdentity",
        "ManualNatMulAnnihilator",
        "ManualNatMulAssociativity",
    ] {
        assert!(
            helper.contains(anchor),
            "shared algebra-law subject helper must cover {anchor}"
        );
    }
    let n = helper.matches("value: AlgebraLawSubject {").count();
    assert_eq!(
        n, 6,
        "expected six Nat AlgebraLawSubject projection arms in the shared helper; got {n}"
    );
}

fn module_declares_type_sum_named(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum {
                name: item_name, ..
            } if item_name == name
        )
    })
}

fn import_names_for_path<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
) -> Option<&'a [String]> {
    module.items.iter().find_map(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return None;
        };
        if item_path.len() != path.len() {
            return None;
        }
        item_path
            .iter()
            .zip(path.iter())
            .all(|(a, &b)| a.as_str() == b)
            .then_some(names.as_slice())
    })
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    text.split_once(start)
        .and_then(|(_, tail)| tail.split_once(end).map(|(middle, _)| middle))
        .unwrap_or_else(|| panic!("missing expected span from {start:?} to {end:?}"))
}

fn generator_field_ty<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    field_name: &str,
) -> Option<&'a SurfaceType> {
    for item in &module.items {
        let SurfaceItem::TypeRecord {
            name,
            fields,
            type_params,
            ..
        } = item
        else {
            continue;
        };
        if name != "Generator" || type_params.len() != 1 {
            continue;
        }
        for field in fields {
            if field.name == field_name {
                return Some(&field.ty);
            }
        }
    }
    None
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn fn_return_type<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Option<&'a SurfaceType> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn {
            name: item_name,
            return_type,
            ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name,
            return_type,
            ..
        } => (item_name == name).then_some(return_type),
        _ => None,
    })
}

fn type_is_generator_testgen_concept(ty: &SurfaceType) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Generator" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: inner } = &args[0] else {
        return false;
    };
    matches!(
        inner.as_ref(),
        SurfaceType::Named { name: slot, .. } if slot == "TestgenConcept"
    )
}

fn type_is_outcome_outcome_named(ty: &SurfaceType, inner_name: &str) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Outcome" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: outer_inner } = &args[0] else {
        return false;
    };
    type_is_outcome_named(outer_inner.as_ref(), inner_name)
}

fn type_is_outcome_named(ty: &SurfaceType, inner_name: &str) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Outcome" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: inner } = &args[0] else {
        return false;
    };
    matches!(
        inner.as_ref(),
        SurfaceType::Named { name: n, .. } if n == inner_name
    )
}

fn type_is_outcome_generator_testgen_concept(ty: &SurfaceType) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Outcome" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: inner } = &args[0] else {
        return false;
    };
    type_is_generator_testgen_concept(inner.as_ref())
}

// T-19 witness-validity generator category — structural guard for the new helper +
// generated corpus. Folded into this file (no new `EXPECTED_HAND_AUTHORED_TEST` census
// path) per INVARIANTS.md §P5 Dispatch-Discipline (b) same-path expansion.

#[test]
fn v4_lens_testgen_witness_validity_modules_tokenize_and_parse() {
    parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    parse_module(WITNESS_VALIDITY_DAG, WITNESS_VALIDITY_PATH);
}

#[test]
fn v4_lens_testgen_witness_validity_helper_returns_outcome_testclaim() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let rt = fn_return_type(&testgen, "testgen_emit_witness_validity_claim")
        .expect("testgen_emit_witness_validity_claim must be declared in v4.lens.testgen");
    assert!(
        type_is_outcome_named(rt, "TestClaim"),
        "testgen_emit_witness_validity_claim must return `Outcome<TestClaim>`; got {rt:?}"
    );
}

#[test]
fn v4_lens_testgen_witness_validity_module_imports_helper_from_lens_testgen() {
    let module = parse_module(WITNESS_VALIDITY_DAG, WITNESS_VALIDITY_PATH);
    let names = import_names_for_path(&module, &["v4", "lens", "testgen"]).expect(
        "witness_validity.dag must import from `v4.lens.testgen` (rows route through helper)",
    );
    assert!(
        names
            .iter()
            .any(|n| n == "testgen_emit_witness_validity_claim"),
        "witness_validity.dag must import `testgen_emit_witness_validity_claim`; got {names:?}"
    );
}

#[test]
fn v4_lens_testgen_witness_validity_module_pins_four_row_corpus_via_helper() {
    // Behavior-pinning ratchet (codex 2026-05-22): pin the exact 4-row corpus this PR
    // delivers — 1 positive + 3 negative arms — so dropping one row regresses the test.
    let helper_calls = WITNESS_VALIDITY_DAG
        .matches("testgen_emit_witness_validity_claim(")
        .count();
    assert_eq!(
        helper_calls, 4,
        "witness_validity.dag must contain exactly 4 helper-routed rows; got {helper_calls}"
    );
    let row_data_decls = WITNESS_VALIDITY_DAG
        .matches("data row_witness_validity_")
        .count();
    assert_eq!(
        row_data_decls, 4,
        "witness_validity.dag must declare exactly 4 `data row_witness_validity_*` rows; got {row_data_decls}"
    );
}

#[test]
fn v4_lens_testgen_witness_validity_module_authors_no_testclaim_literals() {
    for literal in [
        "EqualsClaim {",
        "DiagnosticClaim {",
        "CompilesClaim {",
        "RoundTripClaim {",
    ] {
        assert!(
            !WITNESS_VALIDITY_DAG.contains(literal),
            "witness_validity.dag must not author `{literal}` literals — claim polarity is decided by verify_witness inside the helper (tautology-skip discipline)"
        );
    }
}

fn function_count(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> usize {
    module
        .items
        .iter()
        .filter(|item| match item {
            SurfaceItem::Fn {
                name: item_name, ..
            }
            | SurfaceItem::FnExternalBody {
                name: item_name, ..
            } => item_name == name,
            _ => false,
        })
        .count()
}
