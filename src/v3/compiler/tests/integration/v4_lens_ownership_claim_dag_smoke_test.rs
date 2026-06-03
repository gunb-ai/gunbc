//! **Layer:** integration
//!
//! T-38B wire: `src/v4/test/claim/lens_ownership/*` — subject roster +
//! `run_test_claim` + family receipt over v4 evaluator wave-1 wedge.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (peer v4 smoke posture).
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:57); dissolves when T-38B
//! `.dag` TestClaim execution replaces this hand-Rust parse harness.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const RESOURCE_DEPENDENCY_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_ownership/resource_dependency.dag");
const RESOURCE_DEPENDENCY_PATH: &str =
    "src/v4/test/claim/lens_ownership/resource_dependency.dag";
const SUBJECT_ROSTER_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_ownership/subject_roster.dag");
const SUBJECT_ROSTER_PATH: &str = "src/v4/test/claim/lens_ownership/subject_roster.dag";
const FAMILY_RECEIPT_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_ownership/family_receipt.dag");
const FAMILY_RECEIPT_PATH: &str = "src/v4/test/claim/lens_ownership/family_receipt.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_path(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<&str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn v4_lens_ownership_claim_dags_tokenize_and_parse() {
    let _ = parse_module(RESOURCE_DEPENDENCY_DAG, RESOURCE_DEPENDENCY_PATH);
    let _ = parse_module(SUBJECT_ROSTER_DAG, SUBJECT_ROSTER_PATH);
    let _ = parse_module(FAMILY_RECEIPT_DAG, FAMILY_RECEIPT_PATH);
}

#[test]
fn v4_lens_ownership_t38b_wiring() {
    assert!(
        SUBJECT_ROSTER_DAG.contains("subject_lens_ownership_resource_dependency")
            && SUBJECT_ROSTER_DAG.contains("run_lens_ownership_resource_dependency_receipt")
            && SUBJECT_ROSTER_DAG.contains("run_test_claim(")
            && SUBJECT_ROSTER_DAG.contains("eval_context_v4_evaluator_wave1"),
        "{SUBJECT_ROSTER_PATH}: T-38B subject + run_test_claim over wave-1 eval context"
    );
    assert!(
        SUBJECT_ROSTER_DAG.contains("lens_ownership_subject_rows")
            && SUBJECT_ROSTER_DAG.contains("subject_lens_ownership_resource_dependency"),
        "{SUBJECT_ROSTER_PATH}: subject roster"
    );
    assert!(
        FAMILY_RECEIPT_DAG.contains("lens_ownership_runtime_value_rows")
            && FAMILY_RECEIPT_DAG.contains("run_lens_ownership_resource_dependency_receipt"),
        "{FAMILY_RECEIPT_PATH}: family receipt"
    );
    let roster = parse_module(SUBJECT_ROSTER_DAG, SUBJECT_ROSTER_PATH);
    assert_eq!(
        module_path(&roster),
        vec!["v4", "test", "claim", "lens_ownership", "subject_roster"],
        "{SUBJECT_ROSTER_PATH}: module path"
    );
}
