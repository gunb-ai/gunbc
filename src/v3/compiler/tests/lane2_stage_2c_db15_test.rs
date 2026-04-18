//! DB-15 — `requires` on `TestClaim` + obligation materialization entry (Stage 2c).

use std::collections::HashSet;

use v3_compiler::dag::{Dag, TypeConnective};

#[test]
fn test_claim_carries_requires_field() {
    let dag = Dag::new();
    let decl = dag
        .declaration_by_name("TestClaim")
        .expect("TestClaim from std.verification");
    let TypeConnective::Conj { children } = &decl.connective else {
        panic!("TestClaim not Conj");
    };
    let labels: Vec<_> = children.iter().map(|c| c.label.as_str()).collect();
    assert!(labels.contains(&"requires"), "{labels:?}");
}

#[test]
fn db15_obligation_surface_is_declared() {
    let dag = Dag::new();
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
    dag.declaration_by_name("TestObligation")
        .expect("TestObligation type");
    dag.declaration_by_name("materialize_test_obligations")
        .expect("materialize_test_obligations");
}

#[test]
fn resource_handle_matches_dsl_authority_including_cap() {
    let dag = Dag::new();
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
    let decl = dag
        .declaration_by_name("ResourceHandle")
        .expect("ResourceHandle from v3.std.resources");
    let TypeConnective::Conj { children } = &decl.connective else {
        panic!("ResourceHandle not a record");
    };
    let labels: HashSet<_> = children.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(
        labels,
        HashSet::from(["cap", "key", "resource_id", "type"]),
        "ResourceHandle field names must match dsl/std/resources.dag exactly"
    );
    let secret_decl = dag
        .declaration_by_name("Secret")
        .expect("Secret from std.types");
    let cap_field = children
        .iter()
        .find(|c| c.label == "cap")
        .expect("cap field");
    assert_eq!(
        cap_field.ty, secret_decl.id,
        "cap field must resolve to std.types.Secret — the dsl/std/resources.dag forgery proof"
    );
}
