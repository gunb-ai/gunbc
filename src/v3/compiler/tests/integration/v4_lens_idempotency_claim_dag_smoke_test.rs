//! **Layer:** integration
//!
//! T-38B wire: `src/v4/test/claim/lens_idempotency/*` — subject roster +
//! `run_test_claim` + family receipt over eval_mvp2 runtime wedge.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (peer v4 smoke posture).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const WRITE_EFFECT_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_idempotency/write_effect.dag");
const WRITE_EFFECT_PATH: &str = "src/v4/test/claim/lens_idempotency/write_effect.dag";
const SUBJECT_ROSTER_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_idempotency/subject_roster.dag");
const SUBJECT_ROSTER_PATH: &str = "src/v4/test/claim/lens_idempotency/subject_roster.dag";
const FAMILY_RECEIPT_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_idempotency/family_receipt.dag");
const FAMILY_RECEIPT_PATH: &str = "src/v4/test/claim/lens_idempotency/family_receipt.dag";

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
fn v4_lens_idempotency_claim_dags_tokenize_and_parse() {
    let _ = parse_module(WRITE_EFFECT_DAG, WRITE_EFFECT_PATH);
    let _ = parse_module(SUBJECT_ROSTER_DAG, SUBJECT_ROSTER_PATH);
    let _ = parse_module(FAMILY_RECEIPT_DAG, FAMILY_RECEIPT_PATH);
}

#[test]
fn v4_lens_idempotency_t38b_wiring() {
    assert!(
        WRITE_EFFECT_DAG.contains("subject_idempotency_write_effect_receipt")
            && WRITE_EFFECT_DAG.contains("run_idempotency_write_effect_receipt")
            && WRITE_EFFECT_DAG.contains("run_test_claim(")
            && WRITE_EFFECT_DAG.contains("eval_mvp2_context"),
        "{WRITE_EFFECT_PATH}: T-38B subject + run_test_claim over eval_mvp2"
    );
    assert!(
        SUBJECT_ROSTER_DAG.contains("lens_idempotency_subject_rows")
            && SUBJECT_ROSTER_DAG.contains("subject_idempotency_write_effect_receipt"),
        "{SUBJECT_ROSTER_PATH}: subject roster"
    );
    assert!(
        FAMILY_RECEIPT_DAG.contains("lens_idempotency_runtime_value_rows")
            && FAMILY_RECEIPT_DAG.contains("run_idempotency_write_effect_receipt"),
        "{FAMILY_RECEIPT_PATH}: family receipt"
    );
    let roster = parse_module(SUBJECT_ROSTER_DAG, SUBJECT_ROSTER_PATH);
    assert_eq!(
        module_path(&roster),
        vec!["v4", "test", "claim", "lens_idempotency", "subject_roster"],
        "{SUBJECT_ROSTER_PATH}: module path"
    );
}
