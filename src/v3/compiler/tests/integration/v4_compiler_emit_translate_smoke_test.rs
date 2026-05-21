//! **Layer:** integration
//!
//! **T-10 / Wave-3-B:** `06_translate.dag` + `05_emit.dag` + `mvp1_rust_add_translate.dag`
//! tokenize/parse cleanly (M1(2.7) single-file path; full `compile_to_dag` deferred until
//! multi-module v4 load lands). Peers: `v4_bin_main_dag_smoke_test.rs`, `v4_extdeps_file_system_dag_smoke_test.rs`.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-10 (`src/v4/compiler/06_translate.dag`, `05_emit.dag`).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS §SG-0 hand-authored integration test receipts row
//! land in the same PR.
//!
//! **Dissolution:** remove when translate/emit/MVP-1 surfaces are exercised only by `.dag`
//! `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const EMIT_DAG: &str = include_str!("../../../../v4/compiler/05_emit.dag");
const EMIT_PATH: &str = "src/v4/compiler/05_emit.dag";
const MVP1_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_rust_add_translate.dag");
const MVP1_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_rust_add_translate.dag";

fn parse_v4_dag(source: &str, path: &str) {
    let tokens = tokenize_for_test(source, path)
        .unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"));
}

#[test]
fn v4_translate_dag_tokenizes_and_parses() {
    parse_v4_dag(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        TRANSLATE_DAG.contains("coercion_fold"),
        "{TRANSLATE_PATH}: translate must call coercion_fold wrapper, not inline find_witness"
    );
    assert!(
        TRANSLATE_DAG.contains("fold_node"),
        "{TRANSLATE_PATH}: translate must traverse via fold_node"
    );
}

#[test]
fn v4_emit_dag_tokenizes_and_parses() {
    parse_v4_dag(EMIT_DAG, EMIT_PATH);
    assert!(
        EMIT_DAG.contains("translate"),
        "{EMIT_PATH}: emit must compose serialize after translate"
    );
    assert!(
        !EMIT_DAG.contains("find_witness"),
        "{EMIT_PATH}: emit must not inline find_witness (Practice 11)"
    );
}

#[test]
fn v4_mvp1_rust_add_claim_tokenizes_and_parses() {
    parse_v4_dag(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
    assert!(
        MVP1_CLAIM_DAG.contains("v4.compiler.translate"),
        "{MVP1_CLAIM_PATH}: claim must import translate stage"
    );
}
