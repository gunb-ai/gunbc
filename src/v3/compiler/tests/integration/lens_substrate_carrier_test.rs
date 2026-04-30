//! **Layer:** integration
//!
//! Acceptance for the T-Substrate-Lens-Primitive first slice
//! (`docs/design-lens-framework.md`, `docs/briefs/r2-substrate-manager.md`).
//!
//! Director-locked option (c) per parent inbox #1130 dispatch:
//! - `Lens<C>` lands in `src/v3/std/lens.dag` with the locked 6-field
//!   shape (`name`, `read`, `sequential: Monoid<C>`, `branch`, `iterate`,
//!   `validate`).
//! - `Diagnostic.kind` widens from `CompilerDiagnosticKind` to
//!   `AnyDiagnosticKind`; the Layer-1 closed sum stays unchanged.
//! - `LensInstanceKindWitness` is decl-only (no payload value field)
//!   until refinement/dependent typing lands; this is the explicit
//!   substrate gap receipt.

use std::collections::HashSet;

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{ArrowBody, Dag, DeclarationId, TypeConnective};
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

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj: {other:?}"),
    }
}

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

fn dag_shape_dag() -> Dag {
    cached_compile_to_dag(
        include_str!("../../../lenses/dag_shape.dag"),
        "src/v3/lenses/dag_shape.dag",
    )
}

#[test]
fn lens_carrier_has_locked_six_field_shape() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "Lens").into_iter().collect();
    let expected: HashSet<&str> = [
        "name",
        "read",
        "sequential",
        "branch",
        "iterate",
        "validate",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "Lens<C> field set diverged from Director-locked 6-field shape"
    );
}

#[test]
fn diagnostic_kind_widened_to_any_diagnostic_kind() {
    let dag = generated_full_bootstrap_dag();
    let any_id = decl_id_by_name(&dag, "AnyDiagnosticKind");

    let diag_decl = dag
        .declaration_by_name("Diagnostic")
        .expect("Diagnostic missing from full bootstrap");
    let kind_field = match &diag_decl.connective {
        TypeConnective::Conj { children } => children
            .iter()
            .find(|f| f.label == "kind")
            .expect("Diagnostic missing `kind` field"),
        other => panic!("Diagnostic is not a Conj: {other:?}"),
    };
    assert_eq!(
        kind_field.ty, any_id,
        "Diagnostic.kind must point at AnyDiagnosticKind, not the Layer-1 closed sum"
    );
}

#[test]
fn compiler_diagnostic_kind_closed_sum_unchanged() {
    let dag = generated_full_bootstrap_dag();
    let variants: HashSet<String> = disj_variant_labels(&dag, "CompilerDiagnosticKind")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = [
        "TokenizerError",
        "ParseError",
        "TypeMismatch",
        "UnitMismatch",
        "ArityMismatch",
        "ResolveError",
        "NominalOpacityViolation",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = variants.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "CompilerDiagnosticKind closed sum changed — Layer-1 must stay locked; \
         lens-instance kinds enter via Layer-2 LensInstanceKindWitness, NOT \
         by extending this sum (anti-bridge invariant per Q6.5)"
    );
}

#[test]
fn any_diagnostic_kind_has_two_layer_constructors() {
    let dag = generated_full_bootstrap_dag();
    let variants: HashSet<String> = disj_variant_labels(&dag, "AnyDiagnosticKind")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["CompilerKind", "LensInstanceKind"].into_iter().collect();
    let actual: HashSet<&str> = variants.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "AnyDiagnosticKind must have exactly two constructors — Layer-1 \
         CompilerKind(CompilerDiagnosticKind) and Layer-2 \
         LensInstanceKind(LensInstanceKindWitness)"
    );
}

#[test]
fn lens_instance_kind_witness_payload_intentionally_absent() {
    // Layer-2 substrate gap receipt (per parent #1130 dispatch + Q6.5
    // §State-space discipline). Today's .dag grammar cannot express
    // `payload: <inhabits kind_decl.payload>` — a refinement-type-on-
    // sibling-field shape. The flat alternative (free `TypeShape`
    // payload coordinate) ratifies the very illegal state Q6.5 rejects:
    // `(Lens<TenantFlow>, "WrongName", payload-of-different-shape)`.
    //
    // Director-approved option (c): land Layer-2 kind identity and
    // namespace authority via `LensInstanceKindWitness { kind_decl }`
    // alone; the structured payload value is intentionally NOT carried
    // through the substrate until dependent-field typing or substrate
    // inhabitance witnesses lower without hand-Rust scaffolding.
    //
    // This test pins the gap. When that grammar feature lands and
    // `LensInstanceKindWitness` grows a checked `payload` field, this
    // test fails loudly and forces the dissolution-trigger comment in
    // `diagnostics.dag` to retire in lock-step.
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "LensInstanceKindWitness")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["kind_decl"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "LensInstanceKindWitness field set drifted. Adding a `payload` \
         field is a substantive change — verify the dependent-field \
         typing trigger in `diagnostics.dag` has actually closed before \
         landing it (do not pretend payload is enforced via a free \
         TypeShape coordinate; that is the Q6.5-rejected illegal state)."
    );
}

#[test]
fn dag_shape_report_carrier_projects_reflected_dag_shape_lists() {
    let dag = dag_shape_dag();
    let report = dag
        .declaration_by_name("DagShapeReport")
        .expect("DagShapeReport carrier exists");
    let TypeConnective::Conj { children } = &report.connective else {
        panic!("DagShapeReport must be a record carrier");
    };
    let labels: Vec<&str> = children.iter().map(|field| field.label.as_str()).collect();
    assert_eq!(labels, ["declarations", "nodes", "ports", "clusters"]);

    for field in children {
        let TypeConnective::Instantiation { template, .. } = &dag.declaration(field.ty).connective
        else {
            panic!("DagShapeReport.{} must be a List<...>", field.label);
        };
        assert_eq!(
            dag.declaration(*template).name.as_deref(),
            Some("List"),
            "DagShapeReport.{} must be list-shaped",
            field.label
        );
    }
}

#[test]
fn dag_shape_report_public_producer_returns_shape_report() {
    let dag = dag_shape_dag();
    let report = dag
        .declaration_by_name("DagShapeReport")
        .expect("DagShapeReport carrier exists")
        .id;

    assert_arrow_output(&dag, "dag_shape_report", report);
    assert!(
        dag.declaration_by_name("dag_shape_lens").is_none(),
        "do not author fake Lens<DagShapeReport> data until whole-Dag lens contract is honest"
    );
}

fn assert_arrow_output(dag: &Dag, name: &str, expected: DeclarationId) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("{name} declaration exists"));
    let TypeConnective::Arrow { output, body, .. } = &decl.connective else {
        panic!("{name} must be an Arrow");
    };
    assert_eq!(*output, expected, "{name} output drifted");
    assert!(
        matches!(body, ArrowBody::UserDefined(_)),
        "{name} should lower to a user-defined body"
    );
}
