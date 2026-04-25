//! **Layer:** integration
//!
//! PB-1-e: runtime `compile_full_bootstrap_*` drift tests retired; the
//! fresh-compile vs committed snapshot contract is enforced by
//! `regen_bootstrap --verify`. These tests pin structural relationships between
//! the committed full and std-only snapshots and `Dag::new()` stability.

use v3_compiler::{
    generated_full_bootstrap_dag, generated_full_bootstrap_without_parse_surface_dag,
    generated_std_bootstrap_dag,
    serialize::{first_difference, serialize_dag},
    Dag,
};

#[test]
fn full_bootstrap_extends_std_snapshot() {
    let std_only = generated_std_bootstrap_dag();
    let full = generated_full_bootstrap_dag();
    assert!(
        first_difference(&std_only, &full).is_some(),
        "full bootstrap unexpectedly identical to std-only snapshot"
    );
}

#[test]
fn generated_full_bootstrap_snapshots_are_clean() {
    for (label, dag) in [
        ("full", generated_full_bootstrap_dag()),
        (
            "without_parse_surface",
            generated_full_bootstrap_without_parse_surface_dag(),
        ),
    ] {
        assert!(
            dag.diagnostics().is_empty(),
            "{label}: expected clean generated bootstrap, got {:?}",
            dag.diagnostics()
        );
        assert!(
            dag.declaration_by_name("parse").is_some(),
            "{label}: expected pipeline `parse` stage in bootstrap Dag"
        );
    }
}

#[test]
fn dag_new_bootstrap_is_clean_and_byte_stable() {
    let first = Dag::new();
    assert!(
        first.diagnostics().is_empty(),
        "Dag::new() bootstrap should be clean, got {:?}",
        first.diagnostics()
    );

    let second = Dag::new();
    assert!(
        first_difference(&first, &second).is_none(),
        "Dag::new() should clone a stable committed bootstrap snapshot"
    );
    assert_eq!(
        serialize_dag(&first),
        serialize_dag(&second),
        "Dag::new() serialized bootstrap bytes should be stable across clones"
    );
}
