//! **Layer:** integration
//!
//! PB-1-e — in-tree DB-8 cross-check is now "the committed bootstrap snapshot is
//! internally consistent": `Dag::new()` is diagnostic-clean and byte-stable across
//! clones, and the std-only snapshot is a strict prefix-shape of the full snapshot.
//! The fresh-parse-vs-snapshot acid test runs at regen time (`regen_bootstrap` +
//! CI's `git diff --exit-code` on the committed `bootstrap_*_generated.rs`),
//! not on every `cargo test`. See `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md`.

use v3_compiler::{
    generated_full_bootstrap_dag, generated_std_bootstrap_dag,
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
