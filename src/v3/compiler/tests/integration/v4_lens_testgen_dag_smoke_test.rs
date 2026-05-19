//! **Layer:** integration
//!
//! T-19 Wave-0: `src/v4/lens/testgen.dag` parses and exposes manual-anchor-key-driven
//! `Generator` wiring (`kind` + `t19_anchor` + `classification` + `slot: TestgenConcept`).
//! `T19ManualAnchorAbsent` is fail-closed on bootstrap via `Outcome` on `manual_test_claim_for_manual_anchor`.
//! `Generator` metadata (`kind`, `classification`, `t19_anchor: T19PresentManualAnchorKey`) is wired from
//! the selected manual `TestClaim` through `t19_present_manual_anchor_key_for_claim` so the carrier cannot
//! structurally represent `T19ManualAnchorAbsent` (P2 / Practice 2); lookup/bootstrap input remains `T19ManualAnchorKey`.
//! `testgen_concept_for_manual_claim` matches on `claim.t19_anchor` so the slot projection cannot split-brain
//! from the claim authority path.
//! **Note:** `compile_to_dag` on this module alone does not resolve `import v4.std.*` peers
//! (Import lowering is still M2-scoped); full merge compile lands with cross-file M2 per TASKS T-19.
//!
//! **TESTING.md:** each `#[test]` below pins one structural slice (split per `openai-pro` review **#14812**).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const VERIFICATION_DAG: &str = include_str!("../../../../v4/std/verification.dag");
const NAT_LAW_DAG: &str = include_str!("../../../../v4/test/claim/manual/nat_law_anchors.dag");
const NAT_SUBSTRATE_DAG: &str = include_str!("../../../../v4/std/nat.dag");

const NAT_MANUAL_CLAIM_DATA: [&str; 6] = [
    "claim_nat_add_left_identity",
    "claim_nat_add_right_identity",
    "claim_nat_add_associativity",
    "claim_nat_mul_left_identity",
    "claim_nat_mul_annihilator",
    "claim_nat_mul_associativity",
];

#[test]
fn v4_lens_testgen_wave0_modules_tokenize_and_parse() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let verification = parse_module(VERIFICATION_DAG, "src/v4/std/verification.dag");
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
}

#[test]
fn v4_lens_testgen_wave0_verification_declares_present_anchor_key() {
    assert!(
        VERIFICATION_DAG.contains("type T19PresentManualAnchorKey"),
        "substrate must declare `T19PresentManualAnchorKey` (present-only manual anchor carrier)"
    );
}

#[test]
fn v4_lens_testgen_wave0_function_inventory_matches_wave0() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let verification = parse_module(VERIFICATION_DAG, "src/v4/std/verification.dag");
    assert_eq!(
        function_count(&testgen, "bootstrap_claim_generator_for_manual_anchor"),
        1,
        "T-19 Wave-0: single generator entrypoint keyed by T19ManualAnchorKey"
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
        "AssertKind must not be re-authored in testgen; use manual `TestClaim.kind`"
    );
    assert_eq!(
        function_count(&verification, "t19_present_manual_anchor_key"),
        0,
        "std must not host `t19_present_manual_anchor_key` (Wave-0 fail-close lives in lens/testgen only)"
    );
    assert_eq!(
        function_count(&verification, "assert_kind_for_manual_anchor"),
        0,
        "AssertKind bootstrap mapping must not live in verification (single `T19ManualAnchorKey` axis)"
    );
    assert_eq!(
        function_count(&testgen, "t19_present_manual_anchor_key_for_claim"),
        1,
        "present-anchor narrowing join must live in v4.lens.testgen (single Outcome<T19PresentManualAnchorKey> projection)"
    );
}

#[test]
fn v4_lens_testgen_wave0_nat_symbol_import_authority() {
    assert!(
        !TESTGEN_DAG.contains("t19_bootstrap_algebra")
            && !TESTGEN_DAG.contains("t19_bootstrap_inhabitant"),
        "nat-law AlgebraLawSubject algebra/inhabitant must project from `v4.std.nat` substrate Symbol bundle (no shared bootstrap placeholders)"
    );
    assert!(
        TESTGEN_DAG.contains("import v4.std.nat {")
            && TESTGEN_DAG.contains("nat_algebra_law_subject_symbol_additive_monoid")
            && TESTGEN_DAG.contains("nat_algebra_law_subject_symbol_commutative_semiring")
            && TESTGEN_DAG.contains("nat_algebra_law_subject_symbol_inhabitant_nat"),
        "testgen must import the three `AlgebraLawSubject` Symbol carriers from `v4.std.nat` (single substrate authority)"
    );
    assert!(
        !TESTGEN_DAG.contains("fn t19_present_manual_anchor_key("),
        "testgen must not define the retired `t19_present_manual_anchor_key(` std-style helper (use `t19_present_manual_anchor_key_for_claim` in lens only)"
    );
}

#[test]
fn v4_lens_testgen_wave0_outcome_return_surfaces() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let present_anchor_rt = fn_return_type(&testgen, "t19_present_manual_anchor_key_for_claim")
        .expect("t19_present_manual_anchor_key_for_claim should have an explicit return type");
    assert!(
        type_is_outcome_named(present_anchor_rt, "T19PresentManualAnchorKey"),
        "present-anchor narrowing must return `Outcome<T19PresentManualAnchorKey>`; got {present_anchor_rt:?}"
    );

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
        "T-19 Wave-0 bootstrap must fail-close `T19ManualAnchorAbsent` via `Outcome<Generator<TestgenConcept>>`; got {bootstrap_rt:?}"
    );
}

#[test]
fn v4_lens_testgen_wave0_generator_t19_anchor_field_is_present_key() {
    let testgen = parse_module(TESTGEN_DAG, "src/v4/lens/testgen.dag");
    let anchor_ty =
        generator_t19_anchor_field_ty(&testgen).expect("Generator should declare `t19_anchor`");
    assert!(
        matches!(
            anchor_ty,
            SurfaceType::Named { name: n, .. } if n == "T19PresentManualAnchorKey"
        ),
        "Generator.t19_anchor must be `T19PresentManualAnchorKey` (absent sentinel excluded from successful carrier); got {anchor_ty:?}"
    );
}

#[test]
fn v4_lens_testgen_wave0_concept_projection_matches_claim_t19_anchor() {
    assert!(
        TESTGEN_DAG.contains("fn testgen_concept_for_manual_claim")
            && TESTGEN_DAG.contains("match claim.t19_anchor"),
        "concept projection must match on `claim.t19_anchor` (single authority path with manual claim)"
    );
}

#[test]
fn v4_lens_testgen_wave0_generator_carries_claim_kind_and_classification() {
    assert!(
        TESTGEN_DAG.contains("classification: claim.classification"),
        "Generator must take classification from manual TestClaim"
    );
    assert!(
        TESTGEN_DAG.contains("kind: claim.kind"),
        "Generator must take AssertKind from manual TestClaim.kind"
    );
}

#[test]
fn v4_lens_testgen_wave0_bootstrap_narrows_present_anchor_before_generator() {
    assert!(
        TESTGEN_DAG.contains("fn t19_present_manual_anchor_key_for_claim")
            && TESTGEN_DAG.contains("match t19_present_manual_anchor_key_for_claim(claim: claim)"),
        "bootstrap must narrow `claim.t19_anchor` through `t19_present_manual_anchor_key_for_claim` before constructing `Generator`"
    );
    assert!(
        TESTGEN_DAG.contains("t19_anchor: present_anchor"),
        "Generator must wire `t19_anchor` from the narrowed present key (not raw `claim.t19_anchor`)"
    );
    assert!(
        TESTGEN_DAG.contains("match testgen_concept_for_manual_claim(claim: claim)"),
        "bootstrap must thread the same `claim` into the concept projection"
    );
}

#[test]
fn v4_lens_testgen_nat_law_manual_claims_use_equals() {
    assert_nat_manual_claim_blocks_use_equals(NAT_LAW_DAG);
}

#[test]
fn v4_lens_testgen_nat_substrate_carries_algebra_law_subject_symbols() {
    assert_nat_algebra_law_subject_symbols_in_substrate(NAT_SUBSTRATE_DAG);
}

#[test]
fn v4_lens_testgen_testgen_carries_six_nat_algebra_law_scheduling_arms() {
    assert_six_algebra_law_constructor_sites_in_testgen(TESTGEN_DAG);
}

fn assert_nat_manual_claim_blocks_use_equals(nat_law_src: &str) {
    for claim in NAT_MANUAL_CLAIM_DATA {
        let needle = format!("data {claim}:");
        let i = nat_law_src
            .find(&needle)
            .unwrap_or_else(|| panic!("{needle}: missing nat-law manual claim row"));
        let end = nat_law_src.len().min(i.saturating_add(700));
        let tail = &nat_law_src[i..end];
        assert!(
            tail.contains("kind: Equals"),
            "{claim}: nat-law manual stubs must use `kind: Equals` (AlgebraLaw Wave-0 pairing)"
        );
    }
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
}

/// Count `value: AlgebraLaw {` sites in `testgen.dag` (file-wide; currently exactly six nat scheduling arms).
fn assert_six_algebra_law_constructor_sites_in_testgen(testgen_src: &str) {
    assert!(
        testgen_src.contains("fn testgen_concept_for_manual_claim"),
        "testgen_concept_for_manual_claim must exist for Wave-0 nat scheduling slice"
    );
    let n = testgen_src.matches("value: AlgebraLaw {").count();
    assert_eq!(
        n, 6,
        "expected six nat `value: AlgebraLaw {{...}}` constructor sites in testgen.dag; got {n}"
    );
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn generator_t19_anchor_field_ty(
    module: &v3_compiler::parse_surface::SurfaceModule,
) -> Option<&SurfaceType> {
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
            if field.name == "t19_anchor" {
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
