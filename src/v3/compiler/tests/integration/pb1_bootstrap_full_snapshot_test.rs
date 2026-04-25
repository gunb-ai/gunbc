//! **Layer:** integration
//!
//! PB-1-e: in-tree DB-8 cross-check is structural — committed snapshots are
//! internally consistent; `Dag::new()` is diagnostic-clean and byte-stable across
//! clones; the std-only snapshot is a strict prefix-shape of the full snapshot.
//! The fresh-compile vs committed snapshot contract is enforced by CI
//! `regen_bootstrap --verify` (`--features bootstrap-regen-fresh`), not on every
//! `cargo test`. See `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md`.

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
fn generated_std_bootstrap_snapshot_has_no_diagnostics() {
    let dag = generated_std_bootstrap_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "expected clean std snapshot bootstrap, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn generated_std_bootstrap_snapshot_includes_bool() {
    let dag = generated_std_bootstrap_dag();
    assert!(
        dag.declaration_by_name("Bool").is_some(),
        "std snapshot should include kernel Bool"
    );
}

#[test]
fn generated_full_bootstrap_snapshots_have_no_diagnostics() {
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
    }
}

#[test]
fn generated_full_bootstrap_snapshots_include_parse_stage() {
    for (label, dag) in [
        ("full", generated_full_bootstrap_dag()),
        (
            "without_parse_surface",
            generated_full_bootstrap_without_parse_surface_dag(),
        ),
    ] {
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
