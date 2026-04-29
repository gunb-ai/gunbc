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
//!   (pending until Grounding lands target row-list authorities after the
//!   Substrate method registry)
//! - `method_template_contract_does_not_carry_cost_data`

use std::collections::HashSet;
use v3_compiler::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};
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

/// Walk a per-target `List<MethodTemplateContract>` declaration's value body
/// and assert that every row's `dag_method: DeclarationRef` is unique within
/// the list. Empty lists vacuously pass (no rows to compare); once Substrate's
/// method-decl registry lands and rows reference real method declarations,
/// this check becomes load-bearing.
fn assert_per_target_list_dag_method_unique(dag: &Dag, list_name: &str) {
    let decl = dag
        .declaration_by_name(list_name)
        .unwrap_or_else(|| panic!("`{list_name}` missing from full bootstrap"));
    let body = decl.value_body.as_ref().unwrap_or_else(|| {
        panic!("`{list_name}` has no value body — must be a `data` declaration")
    });
    let ValueBody::List(rows) = body else {
        panic!(
            "`{list_name}` value body must be `ValueBody::List` \
             (declared as `List<MethodTemplateContract>`); got {body:?}"
        );
    };

    let mut seen: HashSet<DeclarationId> = HashSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let FieldValue::Record(fields) = row else {
            panic!(
                "row {idx} in `{list_name}` is not a `FieldValue::Record` — \
                 every `MethodTemplateContract` row must be a record literal"
            );
        };
        let (_, dag_method) = fields
            .iter()
            .find(|(label, _)| label == "dag_method")
            .unwrap_or_else(|| panic!("row {idx} in `{list_name}` missing `dag_method` field"));
        let FieldValue::Reference(decl_id) = dag_method else {
            panic!(
                "row {idx} in `{list_name}`: `dag_method` must be a \
                 `FieldValue::Reference(DeclarationId)`; got {dag_method:?}"
            );
        };
        assert!(
            seen.insert(*decl_id),
            "duplicate `dag_method` in `{list_name}` at row {idx} — per-target \
             MethodTemplateContract rows must be unique by `dag_method`"
        );
    }
}

#[test]
fn method_template_contract_per_target_dag_method_unique() {
    // The uniqueness walker is intentionally present, but no target row-list
    // authorities are loaded yet. Empty `dsl/extdeps/.../method_template_contracts.dag`
    // scaffolds imported `v3.std.emit_model` from the shared extdeps tree and
    // polluted the v2 loader; Grounding row-list authorities land after the
    // Substrate method-declaration registry gives `dag_method` real targets.
    // Until then this loop is a zero-authority placeholder, not a fake empty
    // fixture.
    let dag = generated_full_bootstrap_dag();
    for list_name in EXPECTED_PER_TARGET_LISTS {
        assert_per_target_list_dag_method_unique(&dag, list_name);
    }
}

const EXPECTED_PER_TARGET_LISTS: &[&str] = &[];
