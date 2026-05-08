//! T-CostLens-Composition Slice 1a.1 consumer-proof tests.
//!
//! Exercises `v3_compiler::lens_cost_target_realization::*` — the `.dag`-tier
//! consumer of the `declaration_by_name` substrate accessor introduced by
//! Slice 1a.0 (PR #2194 merged at commit 633f83854; Director ratification
//! at gunbc#828 #issuecomment-4402899692).
//!
//! Closes the same-PR-consumer-evidence gap per INVARIANTS P2 raised by
//! codex BLOCKING on PR #2194 sha 633f8385 (resolved post-merge by this
//! Slice 1a.1 landing).

use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::lens_cost_target_realization::{
    behavior_realization_meta, callable_realization_meta, operator_realization_meta,
    type_realization_meta,
};

#[test]
fn type_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = type_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "type_realization_meta should resolve `TypeRealization` declaration in bootstrap dag"
    );
    let decl = meta.unwrap();
    assert_eq!(
        decl.name.as_deref(),
        Some("TypeRealization"),
        "resolved declaration's name should be `TypeRealization`"
    );
}

#[test]
fn callable_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = callable_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "callable_realization_meta should resolve `CallableRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("CallableRealization"));
}

#[test]
fn operator_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = operator_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "operator_realization_meta should resolve `OperatorRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("OperatorRealization"));
}

#[test]
fn behavior_realization_meta_resolves_against_bootstrap() {
    let dag = generated_full_bootstrap_dag();
    let meta = behavior_realization_meta(&dag);
    assert!(
        meta.is_some(),
        "behavior_realization_meta should resolve `BehaviorRealization` in bootstrap"
    );
    assert_eq!(meta.unwrap().name.as_deref(), Some("BehaviorRealization"));
}
