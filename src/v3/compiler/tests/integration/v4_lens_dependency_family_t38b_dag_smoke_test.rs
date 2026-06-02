//! **Layer:** integration
//!
//! T-38B wire: `src/v4/test/claim/lens_ownership/*` and
//! `src/v4/test/claim/lens_parallelism/*` — subject roster + `run_test_claim` +
//! family receipt over the eval_mvp2 runtime wedge. Extends the lens_idempotency
//! T-38B posture to the two remaining single-worked-claim dependency-lens
//! families that share the `ClassifiedDependencyView<_>` projection shape.
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

const OWNERSHIP_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_ownership/resource_dependency.dag");
const OWNERSHIP_CLAIM_PATH: &str = "src/v4/test/claim/lens_ownership/resource_dependency.dag";
const OWNERSHIP_ROSTER_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_ownership/subject_roster.dag");
const OWNERSHIP_ROSTER_PATH: &str = "src/v4/test/claim/lens_ownership/subject_roster.dag";
const OWNERSHIP_RECEIPT_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_ownership/family_receipt.dag");
const OWNERSHIP_RECEIPT_PATH: &str = "src/v4/test/claim/lens_ownership/family_receipt.dag";

const PARALLELISM_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_parallelism/data_dependency.dag");
const PARALLELISM_CLAIM_PATH: &str = "src/v4/test/claim/lens_parallelism/data_dependency.dag";
const PARALLELISM_ROSTER_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_parallelism/subject_roster.dag");
const PARALLELISM_ROSTER_PATH: &str = "src/v4/test/claim/lens_parallelism/subject_roster.dag";
const PARALLELISM_RECEIPT_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_parallelism/family_receipt.dag");
const PARALLELISM_RECEIPT_PATH: &str = "src/v4/test/claim/lens_parallelism/family_receipt.dag";

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
fn v4_lens_dependency_family_t38b_dags_tokenize_and_parse() {
    let _ = parse_module(OWNERSHIP_CLAIM_DAG, OWNERSHIP_CLAIM_PATH);
    let _ = parse_module(OWNERSHIP_ROSTER_DAG, OWNERSHIP_ROSTER_PATH);
    let _ = parse_module(OWNERSHIP_RECEIPT_DAG, OWNERSHIP_RECEIPT_PATH);
    let _ = parse_module(PARALLELISM_CLAIM_DAG, PARALLELISM_CLAIM_PATH);
    let _ = parse_module(PARALLELISM_ROSTER_DAG, PARALLELISM_ROSTER_PATH);
    let _ = parse_module(PARALLELISM_RECEIPT_DAG, PARALLELISM_RECEIPT_PATH);
}

#[test]
fn v4_lens_ownership_t38b_wiring() {
    assert!(
        OWNERSHIP_CLAIM_DAG.contains("subject_ownership_resource_dependency_receipt")
            && OWNERSHIP_CLAIM_DAG.contains("run_ownership_resource_dependency_receipt")
            && OWNERSHIP_CLAIM_DAG.contains("run_test_claim(")
            && OWNERSHIP_CLAIM_DAG.contains("eval_mvp2_context"),
        "{OWNERSHIP_CLAIM_PATH}: T-38B subject + run_test_claim over eval_mvp2"
    );
    assert!(
        OWNERSHIP_ROSTER_DAG.contains("lens_ownership_subject_rows")
            && OWNERSHIP_ROSTER_DAG.contains("subject_ownership_resource_dependency_receipt"),
        "{OWNERSHIP_ROSTER_PATH}: subject roster"
    );
    assert!(
        OWNERSHIP_RECEIPT_DAG.contains("lens_ownership_runtime_value_rows")
            && OWNERSHIP_RECEIPT_DAG.contains("run_ownership_resource_dependency_receipt"),
        "{OWNERSHIP_RECEIPT_PATH}: family receipt"
    );
    let roster = parse_module(OWNERSHIP_ROSTER_DAG, OWNERSHIP_ROSTER_PATH);
    assert_eq!(
        module_path(&roster),
        vec!["v4", "test", "claim", "lens_ownership", "subject_roster"],
        "{OWNERSHIP_ROSTER_PATH}: module path"
    );
}

#[test]
fn v4_lens_parallelism_t38b_wiring() {
    assert!(
        PARALLELISM_CLAIM_DAG.contains("subject_parallelism_data_dependency_receipt")
            && PARALLELISM_CLAIM_DAG.contains("run_parallelism_data_dependency_receipt")
            && PARALLELISM_CLAIM_DAG.contains("run_test_claim(")
            && PARALLELISM_CLAIM_DAG.contains("eval_mvp2_context"),
        "{PARALLELISM_CLAIM_PATH}: T-38B subject + run_test_claim over eval_mvp2"
    );
    assert!(
        PARALLELISM_ROSTER_DAG.contains("lens_parallelism_subject_rows")
            && PARALLELISM_ROSTER_DAG.contains("subject_parallelism_data_dependency_receipt"),
        "{PARALLELISM_ROSTER_PATH}: subject roster"
    );
    assert!(
        PARALLELISM_RECEIPT_DAG.contains("lens_parallelism_runtime_value_rows")
            && PARALLELISM_RECEIPT_DAG.contains("run_parallelism_data_dependency_receipt"),
        "{PARALLELISM_RECEIPT_PATH}: family receipt"
    );
    let roster = parse_module(PARALLELISM_ROSTER_DAG, PARALLELISM_ROSTER_PATH);
    assert_eq!(
        module_path(&roster),
        vec!["v4", "test", "claim", "lens_parallelism", "subject_roster"],
        "{PARALLELISM_ROSTER_PATH}: module path"
    );
}
