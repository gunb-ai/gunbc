//! **Layer:** integration
//!
//! Structural acceptance for T-Workflow-As-Data Slice 2 timing-lens substrate
//! (gunbc#1955): `TimingMeasurement`, `TimingObservationSet`,
//! `WorkflowObservationAnchor`, and `TimingBudget` in `src/v3/std/timing_lens.dag`.

use std::collections::HashSet;

use v3_compiler::dag::{Dag, TypeConnective};
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
        vec!["measurement".to_string(), "subject_stable_id".to_string()],
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
