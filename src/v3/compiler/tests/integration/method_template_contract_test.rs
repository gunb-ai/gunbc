//! **Layer:** integration
//!
//! Acceptance for `MethodTemplateContract` substrate carrier in
//! `src/v3/std/emit_model.dag`. Sibling type to §6a `MethodContract` in
//! `src/v3/std/algebra.dag`; this PR lands the type only — row population
//! and `MethodTranslation` / `SimpleMethodSpec` retirement are
//! Grounding-owned follow-ups.
//!
//! Three claims per Director dispatch:
//! - `method_template_contract_distinct_from_method_contract`
//! - `method_template_contract_per_target_dag_method_unique`
//!   (vacuous today over zero rows; load-bearing once Grounding populates)
//! - `method_template_contract_does_not_carry_cost_data`

use std::collections::HashSet;
use v3_compiler::dag::{Dag, DeclarationId, Field, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

#[test]
fn method_template_contract_distinct_from_method_contract() {
    let dag = generated_full_bootstrap_dag();

    let template_id = decl_id_by_name(&dag, "MethodTemplateContract");
    let metadata_id = decl_id_by_name(&dag, "MethodContract");
    assert_ne!(
        template_id, metadata_id,
        "MethodTemplateContract and §6a MethodContract must be distinct \
         declarations (P2 single-authority)"
    );

    let template_fields: HashSet<String> = conj_field_labels(&dag, "MethodTemplateContract")
        .into_iter()
        .collect();
    let metadata_fields: HashSet<String> = conj_field_labels(&dag, "MethodContract")
        .into_iter()
        .collect();
    assert!(
        template_fields.is_disjoint(&metadata_fields),
        "MethodTemplateContract and §6a MethodContract field sets must be \
         disjoint — they are orthogonal sibling facts attached to method \
         declarations (P1 step 1). template={template_fields:?} \
         metadata={metadata_fields:?}"
    );
}

#[test]
fn method_template_contract_does_not_carry_cost_data() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "MethodTemplateContract")
        .into_iter()
        .collect();

    for forbidden in ["cost_shape", "size_effect", "callback_element_position"] {
        assert!(
            !labels.contains(forbidden),
            "MethodTemplateContract carries `{forbidden}` — that field \
             belongs on §6a MethodContract (target-agnostic cost/complexity \
             metadata). Template-contract carrier holds only render-template \
             facts (P1 step 2). actual fields={labels:?}"
        );
    }

    let expected: HashSet<&str> = [
        "dag_method",
        "runtime_template",
        "emit_template",
        "wraps_result",
        "placeholder_convention",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "MethodTemplateContract field set diverged from Director-locked shape"
    );
}

/// Helper: extract `dag_method` decl ids from a list of MethodTemplateContract
/// rows represented as Conj field-binding lists. Used by the uniqueness check.
/// Today no such lists exist (Grounding owns row population); the check runs
/// vacuously over zero rows and becomes load-bearing once rows land.
fn assert_dag_method_unique(rows: &[Vec<Field>], list_name: &str) {
    let mut seen: HashSet<DeclarationId> = HashSet::new();
    for row in rows {
        let dag_method_field = row
            .iter()
            .find(|f| f.label == "dag_method")
            .unwrap_or_else(|| panic!("row in `{list_name}` missing `dag_method` field"));
        assert!(
            seen.insert(dag_method_field.ty),
            "duplicate `dag_method` in `{list_name}` — per-target \
             MethodTemplateContract rows must be unique by `dag_method`"
        );
    }
}

#[test]
fn method_template_contract_per_target_dag_method_unique() {
    // Substrate-only PR: zero `List<MethodTemplateContract>` data lists exist
    // today (Grounding owns Rust/Python/Go row population). The uniqueness
    // check is wired here and runs vacuously over zero synthetic rows.
    let zero_rows: Vec<Vec<Field>> = Vec::new();
    for list_name in EXPECTED_PER_TARGET_LISTS {
        assert_dag_method_unique(&zero_rows, list_name);
    }

    // Fail-loud trigger: when Grounding lands ANY of the per-target row lists,
    // this assertion fires and forces this test to grow real row-walking
    // logic in lock-step. Without it, the vacuous pass above could silently
    // outlive its trigger and leave per-target uniqueness unchecked once rows
    // exist. The list authority is `data <target>_method_template_contracts`
    // in the per-target extdeps file (Grounding-owned).
    let dag = generated_full_bootstrap_dag();
    let landed: Vec<&str> = EXPECTED_PER_TARGET_LISTS
        .iter()
        .copied()
        .filter(|name| dag.declaration_by_name(name).is_some())
        .collect();
    assert!(
        landed.is_empty(),
        "per-target `MethodTemplateContract` row list(s) have landed: {landed:?}. \
         Grow this test to enumerate each list's rows and hand them to \
         `assert_dag_method_unique` instead of relying on the synthetic \
         zero-row vacuous pass above."
    );
}

const EXPECTED_PER_TARGET_LISTS: &[&str] = &[
    "rust_method_template_contracts",
    "python_method_template_contracts",
    "go_method_template_contracts",
];
