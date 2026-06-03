//! **Layer:** integration
//!
//! G3.1–G3.4 spine claim modules (`src/v4/test/claim/claim_pipeline/*`) — M1(2.7) tokenize/parse gate.
//! Full `compile_to_dag` import merge deferred per TESTING.md; M1 v4 emit exercises cross-module closure.
//!
//! **Authority:** docs/planning/v4-grounding-spine-rr-g-worksheet-2026-06-02.md §2 G3.1–G3.4.

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

const NORMALIZE_CLAIM: &str =
    include_str!("../../../../v4/test/claim/claim_pipeline/normalize.dag");
const NORMALIZE_PATH: &str = "src/v4/test/claim/claim_pipeline/normalize.dag";
const RESOLVE_CLAIM: &str = include_str!("../../../../v4/test/claim/claim_pipeline/resolve.dag");
const RESOLVE_PATH: &str = "src/v4/test/claim/claim_pipeline/resolve.dag";
const INFER_CLAIM: &str = include_str!("../../../../v4/test/claim/claim_pipeline/infer.dag");
const INFER_PATH: &str = "src/v4/test/claim/claim_pipeline/infer.dag";
const TRANSLATE_CLAIM: &str =
    include_str!("../../../../v4/test/claim/claim_pipeline/translate.dag");
const TRANSLATE_PATH: &str = "src/v4/test/claim/claim_pipeline/translate.dag";

fn assert_parses(source: &str, path: &str) {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"));
}

#[test]
fn v4_claim_pipeline_spine_normalize_tokenizes_and_parses() {
    assert_parses(NORMALIZE_CLAIM, NORMALIZE_PATH);
}

#[test]
fn v4_claim_pipeline_spine_resolve_tokenizes_and_parses() {
    assert_parses(RESOLVE_CLAIM, RESOLVE_PATH);
}

#[test]
fn v4_claim_pipeline_spine_infer_tokenizes_and_parses() {
    assert_parses(INFER_CLAIM, INFER_PATH);
}

#[test]
fn v4_claim_pipeline_spine_translate_tokenizes_and_parses() {
    assert_parses(TRANSLATE_CLAIM, TRANSLATE_PATH);
}
