//! **Layer:** integration
//!
//! §1.8 / R3 gate **`lens_application_carrier_landed` (#88)** — T-Lens-Application-Surface:
//! `EnforcedApplication<Output, Budget, Projected>` and `IntrospectApplication<Output>` template
//! declarations in `src/v3/std/lens_application.dag` stay structurally aligned with
//! `docs/design-lens-application-surface.md` §2 (`../INVARIANTS.md` P2 / practice-5 sibling).
//!
//! Companion substrate (gate #89 `SectionRef`, gate #90 `LensEnforcement` / `EnforceableLens`) shares
//! the same module; this harness pins only the two **top-level application** carriers for #88.

use std::collections::HashSet;

use v3_compiler::dag::{Dag, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn conj_field_labels(dag: &Dag, name: &str) -> HashSet<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

#[test]
fn r3_gate_88_enforced_application_carrier_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let enforced = dag
        .declaration_by_name("EnforcedApplication")
        .expect("EnforcedApplication missing from full bootstrap");
    assert_eq!(
        enforced.type_params.len(),
        3,
        "EnforcedApplication must carry Output, Budget, Projected parameters"
    );

    let labels = conj_field_labels(&dag, "EnforcedApplication");
    let expected: HashSet<&str> = [
        "enforceable_lens",
        "section",
        "budget",
        "diagnostic_severity",
        "span",
    ]
    .into_iter()
    .collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "EnforcedApplication field set drifted from T-LAS Slice A design doc §2"
    );
}

#[test]
fn r3_gate_88_introspect_application_carrier_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let intro = dag
        .declaration_by_name("IntrospectApplication")
        .expect("IntrospectApplication missing from full bootstrap");
    assert_eq!(
        intro.type_params.len(),
        1,
        "IntrospectApplication must carry a single Output parameter"
    );

    let labels = conj_field_labels(&dag, "IntrospectApplication");
    let expected: HashSet<&str> = ["lens", "section", "span"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "IntrospectApplication field set drifted from T-LAS Slice A design doc §2"
    );
}
