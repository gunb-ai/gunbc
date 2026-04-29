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

use std::collections::{BTreeSet, HashSet};
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

/// Algebra-template source — the actual authority for which method
/// names exist in the seven `dsl/std/algebra.dag` per-profile template
/// lists. The drift-detection test extracts the unique `name: "..."`
/// strings appearing inside those template-list bodies and asserts
/// each has a matching `<name>_method: MethodDeclaration` registry
/// binding. No hand-maintained mirror constant: if a new method name
/// lands in algebra.dag without a corresponding registry binding,
/// this test fails fail-closed at the same boundary the registry's
/// own SCAFFOLD comment promises.
const ALGEBRA_DAG_SOURCE: &str = include_str!("../../../../../dsl/std/algebra.dag");

/// Extracts unique method names from the algebra-template source by
/// scanning for the literal `name: "<id>"` pattern that appears inside
/// the seven per-profile template-list returns. Identifier characters
/// only (lowercase ASCII + underscore) so we don't accidentally match
/// other `name:` strings used elsewhere as record-field labels at the
/// type-definition layer (e.g., `NamedTemplate { name: "Int" }` —
/// those are TypeShape-side names that look the same lexically).
///
/// Lexical matching is intentional: the alternative (running the
/// per-profile fn bodies through the Dag's value-body walker) would
/// require the bootstrap to have lowered the function returns to
/// structural list values, which it doesn't yet. The source-file
/// scan keeps `dsl/std/algebra.dag` as the canonical authority for
/// the drift comparison without smuggling structural inhabitance
/// through a partial-lowering bridge.
fn algebra_template_method_names() -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    let bytes = ALGEBRA_DAG_SOURCE.as_bytes();
    let mut i = 0;
    while i + 5 <= bytes.len() {
        // Find the next `name:` substring; only count occurrences
        // followed by a `"<id>"` literal to ignore type-shape names
        // like `name: "Int"` (those satisfy the literal pattern but
        // their values uppercase-start, so further filtering below
        // catches them — algebra-template method names are all
        // lowercase identifiers).
        if &bytes[i..i + 5] == b"name:" {
            let mut j = i + 5;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                let start = j + 1;
                let mut end = start;
                while end < bytes.len() && bytes[end] != b'"' {
                    end += 1;
                }
                if end < bytes.len() {
                    let s = &ALGEBRA_DAG_SOURCE[start..end];
                    if !s.is_empty()
                        && s.chars()
                            .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
                    {
                        names.insert(s.to_string());
                    }
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    names
}

#[test]
fn method_registry_covers_all_algebra_template_names() {
    let dag = generated_full_bootstrap_dag();
    let method_decl_id = decl_id_by_name(&dag, "MethodDeclaration");

    let template_names = algebra_template_method_names();
    assert!(
        !template_names.is_empty(),
        "algebra_template_method_names() returned empty set — extraction \
         pattern broke against `dsl/std/algebra.dag`. The drift detector \
         is the authority over which method names need registry bindings; \
         a zero-name extraction would silently pass the loop below."
    );

    let mut errors: Vec<String> = Vec::new();
    for name in &template_names {
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
                errors.push(format!("`{binding_name}.name` = {s:?}, expected {name:?}"));
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
