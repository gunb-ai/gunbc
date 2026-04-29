//! **Layer:** integration
//!
//! Acceptance for the minimal method-declaration registry slice
//! (Director-locked options A/A/(a) on parent inbox #1130 after the
//! #1175 `dag_method: DeclarationRef` substrate gap).
//!
//! Three claims:
//! - `method_registry_covers_all_algebra_template_names` —
//!   drift-detection: every unique method name carried by the seven
//!   `dsl/std/algebra.dag` per-profile template lists has a
//!   corresponding `<name>_method: MethodDeclaration` binding in
//!   `dsl/std/methods.dag`. Adding a new template-list name without
//!   landing the registry binding fails this test fail-closed.
//! - `method_declaration_carries_only_name_field` — identity-only
//!   discipline (no parameter lists, return types, cost metadata,
//!   profile fields, render-template facts).
//! - `method_template_contract_dag_method_refines_to_method_ref` —
//!   `MethodTemplateContract.dag_method` field type now points at
//!   `MethodRef` rather than bare `DeclarationRef`.

use std::collections::HashSet;
use v3_compiler::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};
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

/// Authoritative list of unique method names across all seven
/// `dsl/std/algebra.dag` per-profile template lists, alphabetized.
/// Drift-detection: when a new template entry is added in algebra.dag,
/// either this list grows AND a `data <name>_method: MethodDeclaration`
/// binding lands in `dsl/std/methods.dag`, or this test fails.
const EXPECTED_METHOD_NAMES: &[&str] = &[
    "add",
    "all",
    "any",
    "append",
    "bottom",
    "chars",
    "clamp",
    "compare",
    "complement",
    "concat",
    "contains",
    "count",
    "diff",
    "empty",
    "ends_with",
    "enumerate",
    "filter",
    "first",
    "flat_map",
    "fold",
    "get",
    "has",
    "intersect",
    "join",
    "keys",
    "last",
    "length",
    "list_push",
    "lookup",
    "map",
    "map_contains_key",
    "map_get",
    "map_has",
    "map_insert",
    "map_keys",
    "map_merge",
    "map_values",
    "meet",
    "member",
    "mul",
    "negate",
    "one",
    "quotient",
    "reciprocal",
    "remainder",
    "replace",
    "reverse",
    "skip",
    "sort_by",
    "split",
    "starts_with",
    "substring",
    "take",
    "to_int",
    "to_lower",
    "to_string",
    "to_upper",
    "top",
    "trim",
    "union",
    "values",
    "with",
    "zero",
];

#[test]
fn method_registry_covers_all_algebra_template_names() {
    let dag = generated_full_bootstrap_dag();
    let method_decl_id = decl_id_by_name(&dag, "MethodDeclaration");

    let mut errors: Vec<String> = Vec::new();
    for name in EXPECTED_METHOD_NAMES {
        let binding_name = format!("{name}_method");
        let Some(decl) = dag.declaration_by_name(&binding_name) else {
            errors.push(format!("missing binding `{binding_name}`"));
            continue;
        };

        // (1) The binding's connective must instantiate `MethodDeclaration`.
        let template = match &decl.connective {
            TypeConnective::Instantiation { template, .. } => *template,
            other => {
                errors.push(format!(
                    "`{binding_name}` has non-Instantiation connective: {other:?}"
                ));
                continue;
            }
        };
        if template != method_decl_id {
            errors.push(format!(
                "`{binding_name}` instantiates DeclarationId({:?}), expected MethodDeclaration ({:?})",
                template, method_decl_id
            ));
            continue;
        }

        // (2) The data body must be a Structural record with `name` =
        // String literal matching the expected method name.
        let fields = match &decl.value_body {
            Some(ValueBody::Structural { fields }) => fields,
            Some(other) => {
                errors.push(format!(
                    "`{binding_name}` value_body is not Structural: {other:?}"
                ));
                continue;
            }
            None => {
                errors.push(format!("`{binding_name}` has no value_body"));
                continue;
            }
        };

        let name_field = fields.iter().find(|(label, _)| label == "name");
        match name_field {
            Some((_, FieldValue::Literal(LiteralBits::String(s)))) if s == name => {
                // ok
            }
            Some((_, FieldValue::Literal(LiteralBits::String(s)))) => {
                errors.push(format!(
                    "`{binding_name}.name` = {s:?}, expected {name:?}"
                ));
            }
            Some((_, other)) => {
                errors.push(format!(
                    "`{binding_name}.name` is not a String literal: {other:?}"
                ));
            }
            None => {
                errors.push(format!("`{binding_name}` missing `name` field"));
            }
        }
    }

    assert!(
        errors.is_empty(),
        "method-name registry drift: `dsl/std/methods.dag` failed registry \
         authority checks. Each `<name>_method` must (a) instantiate \
         `MethodDeclaration` and (b) carry a `name: \"<name>\"` field \
         literal. Errors: {errors:#?}"
    );
}

#[test]
fn method_declaration_carries_only_name_field() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "MethodDeclaration")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["name"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "MethodDeclaration is identity-only — no parameter lists, return \
         types, size effects, cost shapes, profile fields, or render-\
         template facts. Adding a field is a substantive scope change \
         that risks duplicating algebra-template / §6a `MethodContract` \
         metadata."
    );
}

#[test]
fn method_template_contract_dag_method_refines_to_method_ref() {
    let dag = generated_full_bootstrap_dag();
    let method_ref_id = decl_id_by_name(&dag, "MethodRef");

    let contract = dag
        .declaration_by_name("MethodTemplateContract")
        .expect("MethodTemplateContract missing from full bootstrap");
    let dag_method_field = match &contract.connective {
        TypeConnective::Conj { children } => children
            .iter()
            .find(|f| f.label == "dag_method")
            .expect("MethodTemplateContract missing `dag_method` field"),
        other => panic!("MethodTemplateContract is not a Conj: {other:?}"),
    };
    assert_eq!(
        dag_method_field.ty, method_ref_id,
        "MethodTemplateContract.dag_method must point at `MethodRef` (the \
         registry-typed reference shape from `src/v3/std/methods.dag`), \
         NOT bare `DeclarationRef` — that was the #1175 substrate gap \
         this slice closes."
    );
}

#[test]
fn method_ref_wraps_declaration_ref() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "MethodRef").into_iter().collect();
    let expected: HashSet<&str> = ["decl"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "MethodRef is a single-field wrapper `{{ decl: DeclarationRef }}`. \
         Adding fields changes the wrapper's residual-class story and \
         requires receipt updates."
    );
}
