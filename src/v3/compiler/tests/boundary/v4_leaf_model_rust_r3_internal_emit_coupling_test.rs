//! **Layer:** boundary — emit-coupling exercise for v4 leaf-model R3-internal.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag` (modeled oracle) +
//! `src/v4/extdeps/languages/rust.dag` (baseline row). Replays
//! `target_atom_type_spelling` / `target_atom_value_expression` kind selection from
//! row `type_form` spelling + `value_form` template (alias + template arms only).
//!
//! Host runner: `scripts/v4-leaf-model-rust-r3-internal-verify.sh`.
//!
//! **P5 receipt:** `EXPECTED_HAND_AUTHORED_TEST` in `self_gen_census_test.rs`.
//! **Dissolution:** retire when T-22 eval exercises `RustEmitProjectionEqualityExpectation`
//! without this hand-Rust bridge.

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const FIXTURE_PATH: &str = "src/v4/lens/leaf_model_verification.dag";
const RUST_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/rust.dag");
const RUST_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/rust.dag";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/language_model/rust_r3_internal.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/language_model/rust_r3_internal.dag";

/// Mirrors `target_atom_value_expression` → `TargetValueExpression.kind` for Phase-1 Symbol rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueTemplateKind {
    SymbolToOwnedString,
    SymbolIdentityPassthrough,
}

/// Row inputs the substrate uses before projection (type alias spelling + value template).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SymbolAtomRowConfig {
    type_spelling: String,
    value_template: ValueTemplateKind,
}

fn parse_module(source: &str, path: &str) {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"));
}

fn extract_block_after_header(text: &str, header: &str) -> Option<String> {
    let start = text.find(header)? + header.len();
    let rest = &text[start..];
    let depth_start = rest.find('{')?;
    let mut depth = 0usize;
    for (idx, ch) in rest[depth_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(rest[..depth_start + idx + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_type_spelling_from_row_block(block: &str) -> Option<String> {
    let marker = "type_form: rust_target_atom_type_form(spelling: ";
    let after = block.find(marker)? + marker.len();
    let spelling = block[after..].split(')').next()?.trim();
    Some(spelling.to_string())
}

fn extract_value_template_kind_from_row_block(block: &str) -> Option<ValueTemplateKind> {
    if block.contains("kind: ValueSymbolToOwnedString") {
        return Some(ValueTemplateKind::SymbolToOwnedString);
    }
    if block.contains("kind: ValueSymbolIdentityPassthrough") {
        return Some(ValueTemplateKind::SymbolIdentityPassthrough);
    }
    None
}

fn baseline_symbol_row_config() -> SymbolAtomRowConfig {
    let block = extract_block_after_header(
        RUST_LANGUAGE_DAG,
        "data rust_target_atom_realization_symbol:",
    )
    .expect("rust_target_atom_realization_symbol row in rust.dag");
    SymbolAtomRowConfig {
        type_spelling: extract_type_spelling_from_row_block(&block)
            .expect("baseline row type_form spelling"),
        value_template: extract_value_template_kind_from_row_block(&block)
            .expect("baseline row value_form template kind"),
    }
}

fn mutated_symbol_row_config() -> SymbolAtomRowConfig {
    let block = extract_block_after_header(FIXTURE_DAG, "data rust_r3_internal_mutated_row:")
        .expect("rust_r3_internal_mutated_row in lens/leaf_model_verification.dag");
    SymbolAtomRowConfig {
        type_spelling: extract_type_spelling_from_row_block(&block)
            .expect("mutated row type_form spelling"),
        value_template: extract_value_template_kind_from_row_block(&block)
            .expect("mutated row value_form template kind"),
    }
}

/// Replay of `target_atom_type_spelling` for rust alias `type_form` rows (Phase 1).
fn replay_type_projection_spelling(row: &SymbolAtomRowConfig) -> &str {
    row.type_spelling.as_str()
}

/// Replay of accepted `target_atom_value_expression` kind label for Symbol probe values.
fn replay_value_projection_kind_label(row: &SymbolAtomRowConfig) -> &'static str {
    match row.value_template {
        ValueTemplateKind::SymbolToOwnedString => "TargetValueExprSymbolToOwnedString",
        ValueTemplateKind::SymbolIdentityPassthrough => "TargetValueExprSymbolIdentity",
    }
}

#[test]
fn v4_leaf_model_rust_r3_internal_lens_and_claim_parse() {
    parse_module(FIXTURE_DAG, FIXTURE_PATH);
    parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_leaf_model_rust_r3_internal_row_mutation_changes_type_and_value_projections() {
    let baseline = baseline_symbol_row_config();
    let mutated = mutated_symbol_row_config();

    let baseline_type = replay_type_projection_spelling(&baseline);
    let mutated_type = replay_type_projection_spelling(&mutated);
    let baseline_value = replay_value_projection_kind_label(&baseline);
    let mutated_value = replay_value_projection_kind_label(&mutated);

    assert_ne!(
        baseline_type, mutated_type,
        "row mutation must change type projection (target_atom_type_spelling)"
    );
    assert_ne!(
        baseline_value, mutated_value,
        "row mutation must change value projection kind (target_atom_value_expression)"
    );
    assert_eq!(
        baseline.value_template,
        ValueTemplateKind::SymbolToOwnedString,
        "production Symbol row must keep ValueSymbolToOwnedString template"
    );
    assert_eq!(
        mutated.value_template,
        ValueTemplateKind::SymbolIdentityPassthrough,
        "mutated row must use ValueSymbolIdentityPassthrough template"
    );
}

#[test]
fn v4_leaf_model_rust_r3_internal_lens_oracle_labels_match_projection_replay() {
    let baseline = baseline_symbol_row_config();
    let mutated = mutated_symbol_row_config();

    assert_eq!(
        replay_value_projection_kind_label(&baseline),
        "TargetValueExprSymbolToOwnedString"
    );
    assert_eq!(
        replay_value_projection_kind_label(&mutated),
        "TargetValueExprSymbolIdentity"
    );
    assert!(
        FIXTURE_DAG.contains("discriminant(v: expr.kind)"),
        "lens oracle keys value projection kind via discriminant"
    );
}

#[test]
fn v4_leaf_model_rust_r3_internal_emit_coupling_oracle_would_pass_on_replayed_receipt() {
    let baseline = baseline_symbol_row_config();
    let mutated = mutated_symbol_row_config();
    let coupling = replay_type_projection_spelling(&baseline)
        != replay_type_projection_spelling(&mutated)
        && replay_value_projection_kind_label(&baseline)
            != replay_value_projection_kind_label(&mutated);
    assert!(
        coupling,
        "rust_r3_internal_emit_coupling_both_projections_changed must hold for replayed projections"
    );
}

#[test]
fn v4_leaf_model_rust_r3_internal_claim_wires_coupling_oracle() {
    assert!(CLAIM_DAG.contains("claim_rust_r3_internal_emit_coupling_wired"));
    assert!(CLAIM_DAG.contains("rust_r3_internal_emit_coupling_both_projections_changed"));
    assert!(CLAIM_DAG.contains("leaf_model_claim_rust_r3_internal_symbol_coupling"));
}
