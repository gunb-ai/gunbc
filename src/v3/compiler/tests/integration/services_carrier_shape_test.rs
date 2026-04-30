//! **Layer:** integration
//!
//! PR-α declaration-shape ratchet for `src/v3/std/services.dag`. Locks the
//! carrier types' Conj field shape so PR-β fixture authors and downstream
//! lens consumers depend on a stable surface.
//!
//! Asserts:
//! - `Operation`, `RestEndpointBinding`, and `InputField` resolve in the full
//!   bootstrap.
//! - `InputField` is the empty record reserved for PR-β extension; the
//!   canonical name is the enclosing map key, never duplicated as a field.
//! - `RestEndpointBinding` carries `method` + `path` only.
//! - `Operation` carries `name`, `inputs`, `endpoint` only.
//! - `Operation.inputs` is `Map<String, InputField>` — by-construction
//!   uniqueness on input-field names; `ParamToken.name` resolves into the
//!   key set in PR-β.
//! - `RestEndpointBinding.path` points at the existing `PathTemplate`
//!   authority from `std.effects` rather than a parallel path model.
//! - PR-α adds no `bootstrap_fixture_authority` row and no `data` row over
//!   any of the carrier types.

use std::collections::HashSet;
use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
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

fn conj_field_ty(dag: &Dag, owner: &str, label: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(owner)
        .unwrap_or_else(|| panic!("`{owner}` missing from full bootstrap"));
    let children = match &decl.connective {
        TypeConnective::Conj { children } => children,
        other => panic!("`{owner}` is not a Conj: {other:?}"),
    };
    children
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("`{owner}.{label}` missing"))
        .ty
}

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

#[test]
fn input_field_is_empty_record_reserved_for_pr_beta() {
    let dag = generated_full_bootstrap_dag();
    let labels = conj_field_labels(&dag, "InputField");
    assert!(
        labels.is_empty(),
        "InputField is the empty record reserved for PR-β per-input-field \
         metadata extension; the canonical input-field name is the enclosing \
         `Operation.inputs: Map<String, InputField>` key. Adding a `name: \
         String` field here would duplicate the map key and break the \
         single-authority story `services.dag` promises. Got: {labels:?}"
    );
}

#[test]
fn rest_endpoint_binding_carries_only_method_and_path() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "RestEndpointBinding")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["method", "path"].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        labels, expected,
        "RestEndpointBinding is the minimal PR-α REST-realization carrier — \
         method + path only, body-shape deferred per the file comment. \
         Adding a field here is a scope change requiring an updated \
         dispatch."
    );
}

#[test]
fn operation_carries_only_name_inputs_endpoint() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "Operation")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["name", "inputs", "endpoint"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        labels, expected,
        "Operation is the minimal PR-α product: name + inputs + endpoint. \
         Adding outputs / body / response shape here is PR-β..ω scope, not PR-α."
    );
}

#[test]
fn operation_inputs_field_is_map_string_to_input_field() {
    // Walks the anonymous Map instantiation: template ≡ `Map`, two
    // arguments resolving to (String, InputField). Encoding a parallel
    // input-field-name authority (e.g., `List<InputField>` with a duplicate
    // `name: String` field) would fail this shape check.
    let dag = generated_full_bootstrap_dag();
    let inputs_ty = conj_field_ty(&dag, "Operation", "inputs");
    let inputs_decl = dag.declaration(inputs_ty);
    let (template, args) = match &inputs_decl.connective {
        TypeConnective::Instantiation {
            template, arguments, ..
        } => (*template, arguments),
        other => panic!("`Operation.inputs` is not an Instantiation: {other:?}"),
    };
    let map_id = decl_id_by_name(&dag, "Map");
    assert_eq!(
        template, map_id,
        "`Operation.inputs` must instantiate `Map`; got DeclarationId({:?})",
        template
    );
    assert_eq!(
        args.len(),
        2,
        "`Operation.inputs` Map instantiation must carry exactly two \
         template arguments (key, value); got {}",
        args.len()
    );
    let string_id = decl_id_by_name(&dag, "String");
    let input_field_id = decl_id_by_name(&dag, "InputField");
    assert_eq!(
        args[0].value, string_id,
        "`Operation.inputs` Map key must be `String` (canonical input-field \
         name authority); got DeclarationId({:?})",
        args[0].value
    );
    assert_eq!(
        args[1].value, input_field_id,
        "`Operation.inputs` Map value must be `InputField` (per-field \
         metadata slot, empty in PR-α); got DeclarationId({:?})",
        args[1].value
    );
}

#[test]
fn rest_endpoint_binding_path_field_resolves_to_std_effects_path_template() {
    // Single-authority discipline: PR-α must not declare a parallel
    // `PathTemplate` shape. The `path` field's referenced declaration must
    // be the `PathTemplate` declared inside `src/v3/std/effects.dag` (which
    // mirrors `dsl/std/http_path.dag`'s `UrlPathToken` flat-token shape).
    // If PR-α grew its own `PathTemplate` type, this id would point at a
    // declaration whose span is `src/v3/std/services.dag`, failing the
    // assertion below.
    let dag = generated_full_bootstrap_dag();
    let path_ty = conj_field_ty(&dag, "RestEndpointBinding", "path");
    let path_decl = dag.declaration(path_ty);
    assert_eq!(
        path_decl.name.as_deref(),
        Some("PathTemplate"),
        "`RestEndpointBinding.path` must point at a declaration named \
         `PathTemplate`; got {:?}",
        path_decl.name
    );
    assert_eq!(
        path_decl.span.file, "src/v3/std/effects.dag",
        "`RestEndpointBinding.path` must point at the `PathTemplate` \
         authority in `src/v3/std/effects.dag` (which mirrors `std.http_path`'s \
         flat-token shape). Found in: {}. A `services.dag`-local path model \
         would be the parallel-authority drift this PR explicitly avoids.",
        path_decl.span.file
    );
}

#[test]
fn services_dag_authors_no_data_rows_in_pr_alpha() {
    // PR-α is type authority only. The fixture-load authority
    // `bootstrap_fixture_authority` is unchanged in this PR; no
    // `data <provider>_operations: List<Operation>` row lands here.
    // Any declaration whose span file is `src/v3/std/services.dag` and
    // which carries a `value_body` would be such a fixture row leaking
    // into PR-α scope.
    let dag = generated_full_bootstrap_dag();
    let leaks: Vec<String> = dag
        .declarations()
        .iter()
        .filter(|d| d.span.file == "src/v3/std/services.dag" && d.value_body.is_some())
        .map(|d| {
            d.name
                .clone()
                .unwrap_or_else(|| format!("DeclarationId({:?})", d.id))
        })
        .collect();
    assert!(
        leaks.is_empty(),
        "PR-α is type authority only — no `data` rows or fixture-row leaks \
         allowed in `src/v3/std/services.dag`. Found populated value_body \
         on: {leaks:?}. PR-β..ω authors `data <provider>_operations: \
         List<Operation>` rows in sibling `src/v3/std/*_operations.dag` \
         files, not here."
    );
}
