//! **Layer:** boundary — emit-coupling exercise for v4 leaf-model R3-internal.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`; claim wiring in `rust_r3_internal.dag`.
//! Host runner: `scripts/v4-leaf-model-rust-r3-internal-verify.sh`.
//!
//! **P5 receipt:** `EXPECTED_HAND_AUTHORED_TEST` in `sg0_census_test.rs`.
//! **Dissolution:** retire when T-22 eval exercises `RustEmitProjectionEqualityExpectation`
//! without this hand-Rust bridge.

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const FIXTURE_PATH: &str = "src/v4/lens/leaf_model_verification.dag";
const CLAIM_DAG: &str = include_str!("../../../../v4/test/claim/language_model/rust_r3_internal.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/language_model/rust_r3_internal.dag";

fn parse_module(source: &str, path: &str) {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"));
}

fn extract_symbol_rhs(dag_text: &str, data_name: &str) -> Option<String> {
    let needle = format!("data {data_name}: Symbol = ");
    let line = dag_text.lines().find(|l| l.starts_with(&needle))?;
    Some(line.strip_prefix(&needle)?.trim().to_string())
}

#[test]
fn v4_leaf_model_rust_r3_internal_lens_and_claim_parse() {
    parse_module(FIXTURE_DAG, FIXTURE_PATH);
    parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_leaf_model_rust_r3_internal_mutated_row_differs_from_baseline() {
    assert!(
        FIXTURE_DAG.contains("value_form: target_value_template_for_kind(kind: ValueSymbolToOwnedString)")
            || FIXTURE_DAG.contains("rust_target_atom_realization_symbol"),
        "baseline row authority must reference production Symbol realization"
    );
    assert!(
        FIXTURE_DAG.contains("value_form: target_value_template_for_kind(kind: ValueSymbolIdentityPassthrough)"),
        "mutated row must change value_form template kind"
    );
    assert!(
        FIXTURE_DAG.contains("type_form: rust_target_atom_type_form(spelling: rust_r3_internal_mutated_type_spelling)"),
        "mutated row must change type_form spelling"
    );
}

#[test]
fn v4_leaf_model_rust_r3_internal_emit_kind_labels_differ() {
    let baseline = extract_symbol_rhs(FIXTURE_DAG, "rust_r3_internal_baseline_value_emit_kind")
        .expect("rust_r3_internal_baseline_value_emit_kind");
    let mutated = extract_symbol_rhs(FIXTURE_DAG, "rust_r3_internal_mutated_value_emit_kind")
        .expect("rust_r3_internal_mutated_value_emit_kind");
    assert_ne!(
        baseline, mutated,
        "value projection kind label bindings must differ after row mutation"
    );
    assert!(
        baseline.contains("rust_target_atom_realization_symbol"),
        "baseline value projection must consult production Symbol row"
    );
    assert!(
        mutated.contains("rust_r3_internal_mutated_row"),
        "mutated value projection must consult mutated Symbol row"
    );
}

#[test]
fn v4_leaf_model_rust_r3_internal_type_emit_bindings_reference_distinct_rows() {
    let baseline = extract_symbol_rhs(FIXTURE_DAG, "rust_r3_internal_baseline_type_emit")
        .expect("rust_r3_internal_baseline_type_emit");
    let mutated = extract_symbol_rhs(FIXTURE_DAG, "rust_r3_internal_mutated_type_emit")
        .expect("rust_r3_internal_mutated_type_emit");
    assert_ne!(baseline, mutated, "type emit bindings must reference distinct rows");
    assert!(baseline.contains("rust_target_atom_realization_symbol"));
    assert!(mutated.contains("rust_r3_internal_mutated_row"));
}

#[test]
fn v4_leaf_model_rust_r3_internal_claim_wires_coupling_oracle() {
    assert!(CLAIM_DAG.contains("claim_rust_r3_internal_emit_coupling_wired"));
    assert!(CLAIM_DAG.contains("rust_r3_internal_emit_coupling_both_projections_changed"));
    assert!(CLAIM_DAG.contains("leaf_model_claim_rust_r3_internal_symbol_coupling"));
}
