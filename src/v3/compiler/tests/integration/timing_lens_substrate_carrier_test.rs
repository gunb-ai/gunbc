//! **Layer:** integration
//!
//! Structural acceptance for T-Workflow-As-Data Slice 2 timing-lens substrate
//! (gunbc#1955): `TimingMeasurement`, `TimingObservationSet`,
//! `WorkflowObservationAnchor`, and `TimingBudget` in `src/v3/std/timing_lens.dag`.

use std::collections::HashSet;

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
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

fn conj_field_ty(dag: &Dag, name: &str, field: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => {
            children
                .iter()
                .find(|f| f.label == field)
                .unwrap_or_else(|| panic!("`{name}` missing `{field}` field"))
                .ty
        }
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

#[test]
fn timing_measurement_variants_locked() {
    let dag = generated_full_bootstrap_dag();
    let variants: HashSet<String> = disj_variant_labels(&dag, "TimingMeasurement")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["Observed", "Unobserved", "Ambiguous", "Stale"]
        .into_iter()
        .collect();
    let actual: HashSet<&str> = variants.iter().map(String::as_str).collect();
    assert_eq!(
        actual, expected,
        "TimingMeasurement coproduct drifted from T-WAD timing-lens substrate receipt"
    );
}

/// P3 / bootstrap-cleanliness receipt: a `Missing` **variant label** on this
/// coproduct collides with other std `Missing`-shaped names and has produced
/// `ResolveError` at `Inhabits(Missing)` witness sites. The substrate uses
/// `Unobserved` instead (see `timing_lens.dag` banner + PR #2360 discussion).
#[test]
fn timing_measurement_excludes_missing_variant_label() {
    let dag = generated_full_bootstrap_dag();
    let variants: HashSet<String> = disj_variant_labels(&dag, "TimingMeasurement")
        .into_iter()
        .collect();
    assert!(
        !variants.contains("Missing"),
        "`TimingMeasurement` must not declare a `Missing` variant label (witness/bootstrap collision); got {variants:?}"
    );
}

#[test]
fn timing_observation_set_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "TimingObservationSet")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["observations"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(actual, expected, "TimingObservationSet field set drifted");
}

#[test]
fn timing_observation_entry_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels: Vec<String> = conj_field_labels(&dag, "TimingObservationEntry");
    labels.sort();
    assert_eq!(
        labels,
        vec!["anchor".to_string(), "measurement".to_string()],
        "TimingObservationEntry field set drifted"
    );
}

#[test]
fn workflow_observation_anchor_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels: Vec<String> = conj_field_labels(&dag, "WorkflowObservationAnchor");
    labels.sort();
    assert_eq!(
        labels,
        vec![
            "artifact_digest".to_string(),
            "attached_at_ns".to_string(),
            "observer_id".to_string(),
            "producer_id".to_string(),
            "prover_id".to_string(),
            "subject_stable_id".to_string(),
            "workflow_run_id".to_string(),
        ],
        "WorkflowObservationAnchor field set drifted"
    );
}

#[test]
fn nanoseconds_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "Nanoseconds").into_iter().collect();
    let expected: HashSet<&str> = ["count"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(actual, expected, "Nanoseconds field set drifted");
}

/// P2 / M9: timing magnitudes must not use bare `Int` (negative / unitless-by-construction).
#[test]
fn nanoseconds_count_field_is_nat() {
    let dag = generated_full_bootstrap_dag();
    let nat = dag
        .declaration_by_name("Nat")
        .expect("`Nat` missing from full bootstrap (std.nat authority)")
        .id;
    let count_ty = conj_field_ty(&dag, "Nanoseconds", "count");
    assert_eq!(
        count_ty, nat,
        "`Nanoseconds.count` must refine to `Nat`, matching `PerfBaselineMeasurement` ns fields in substrate"
    );
}

#[test]
fn timing_budget_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "TimingBudget")
        .into_iter()
        .collect();
    let expected: HashSet<&str> = ["max"].into_iter().collect();
    let actual: HashSet<&str> = labels.iter().map(String::as_str).collect();
    assert_eq!(actual, expected, "TimingBudget field set drifted");
}
