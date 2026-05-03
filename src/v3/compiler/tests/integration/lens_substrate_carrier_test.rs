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
use std::path::PathBuf;

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{ArrowBody, Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("src/v3/compiler has three parents")
        .to_path_buf()
}

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
fn e6_g1_stop_receipt_pins_no_bootstrap_lens_value_yet() {
    let dag = generated_full_bootstrap_dag();
    let lens = decl_id_by_name(&dag, "Lens");
    let lens_values: Vec<String> = dag
        .declarations()
        .iter()
        .filter(|decl| decl.value_body.is_some())
        .filter(|decl| {
            matches!(
                &decl.connective,
                TypeConnective::Instantiation { template, .. } if *template == lens
            )
        })
        .map(|decl| {
            decl.name
                .clone()
                .unwrap_or_else(|| format!("DeclarationId({})", decl.id.raw()))
        })
        .collect();

    assert!(
        lens_values.is_empty(),
        "E6-G1 must not consume placeholder Lens<C> data values before the \
         function-valued structural data surface lands; found {lens_values:?}"
    );
}

#[test]
fn e6_g1_stop_receipt_names_exact_lens_value_blockers() {
    let root = workspace_root();
    let receipt =
        std::fs::read_to_string(root.join("docs/briefs/r3-pr-e6-lens-value-authoring-stop.md"))
            .expect("read E6 lens value authoring STOP receipt");

    for required in [
        "Live for the non-generic pieces this shape needs",
        "blocked through instantiated generic Conj fields",
        "instantiated generic Conj substitution gap",
        "Live for the representative constructors",
        "Explicit typed lens-instance handle instead of full `data Lens<C>`",
        "No honest `Lens<C>` value can be authored or referenced on current `main`",
        "no Rust lens registry",
        "lens_value_generic_conj_field_substitution_lands",
    ] {
        assert!(
            receipt.contains(required),
            "E6-G1 STOP receipt must classify blocker `{required}`"
        );
    }
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
fn dag_shape_report_reuses_reflected_dag_authority() {
    let dag = dag_shape_dag();
    let dag_decl = dag.declaration_by_name("Dag").expect("Dag carrier exists");
    assert!(
        dag.declaration_by_name("DagShapeReport").is_none(),
        "raw Dag shape producer must not duplicate std.substrate.Dag as a second report record"
    );
    assert_arrow_output(&dag, "dag_shape_report", dag_decl.id);
}

#[test]
fn dag_shape_report_public_producer_returns_raw_dag() {
    let dag = dag_shape_dag();
    let dag_decl = dag.declaration_by_name("Dag").expect("Dag carrier exists");

    assert_arrow_output(&dag, "dag_shape_report", dag_decl.id);
    assert!(
        dag.declaration_by_name("dag_shape_lens").is_none(),
        "do not author fake Dag shape lens data until whole-Dag lens contract is honest"
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
