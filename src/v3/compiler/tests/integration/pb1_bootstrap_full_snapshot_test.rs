//! **Layer:** integration

use v3_compiler::{
    compile_full_bootstrap_dag, compile_full_bootstrap_without_parse_surface_dag,
    generated_full_bootstrap_dag, generated_full_bootstrap_without_parse_surface_dag,
    generated_std_bootstrap_dag,
    serialize::{first_difference, serialize_dag},
    Dag,
};

fn assert_no_bootstrap_drift(label: &str, runtime: &Dag, generated: &Dag) {
    if let Some(diff) = first_difference(runtime, generated) {
        panic!("{label} drifted from runtime bootstrap: {}", diff.detail);
    }
}

#[test]
fn generated_full_bootstrap_snapshot_matches_runtime_full_bootstrap() {
    let runtime = compile_full_bootstrap_dag();
    let generated = generated_full_bootstrap_dag();
    assert_no_bootstrap_drift("generated full bootstrap", &runtime, &generated);
}

#[test]
fn generated_full_bootstrap_without_parse_surface_matches_runtime_variant() {
    let runtime = compile_full_bootstrap_without_parse_surface_dag();
    let generated = generated_full_bootstrap_without_parse_surface_dag();
    assert_no_bootstrap_drift(
        "generated no-runtime-mirrors bootstrap",
        &runtime,
        &generated,
    );
}

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
