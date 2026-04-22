//! **Layer:** integration

use v3_compiler::{
    compile_full_bootstrap_dag, compile_full_bootstrap_without_parse_surface_dag,
    generated_full_bootstrap_dag, generated_full_bootstrap_without_parse_surface_dag,
    generated_std_bootstrap_dag, serialize::first_difference,
};

#[test]
fn generated_full_bootstrap_snapshot_matches_runtime_full_bootstrap() {
    let runtime = compile_full_bootstrap_dag();
    let generated = generated_full_bootstrap_dag();
    assert!(
        first_difference(&runtime, &generated).is_none(),
        "generated full bootstrap drifted from runtime full bootstrap"
    );
}

#[test]
fn generated_full_bootstrap_without_parse_surface_matches_runtime_variant() {
    let runtime = compile_full_bootstrap_without_parse_surface_dag();
    let generated = generated_full_bootstrap_without_parse_surface_dag();
    assert!(
        first_difference(&runtime, &generated).is_none(),
        "generated no-runtime-mirrors bootstrap drifted from runtime variant"
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
