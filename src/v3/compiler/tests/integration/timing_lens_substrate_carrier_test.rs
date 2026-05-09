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
            "subject_node".to_string(),
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

/// P2 / gate #55 inv.1: subject key must be structural (`NodeId`), not free-form
/// `String` (span/path blobs must not type-check as the stable subject slot).
#[test]
fn workflow_observation_anchor_subject_node_is_node_id() {
    let dag = generated_full_bootstrap_dag();
    let node_id = dag
        .declaration_by_name("NodeId")
        .expect("`NodeId` missing from full bootstrap (substrate_minimal authority)")
        .id;
    let ty = conj_field_ty(&dag, "WorkflowObservationAnchor", "subject_node");
    assert_eq!(
        ty, node_id,
        "`WorkflowObservationAnchor.subject_node` must be `NodeId` (structural subject key), not `String`"
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

/// openai-pro / PR #2360: sequential and branch lens hooks must share one join
/// (`timing_measurement_lens_combine`); branch must not re-implement a stale
/// short-circuit that makes `(Stale, Unobserved)` order-sensitive vs sequential.
#[test]
fn timing_lens_sequential_and_branch_delegate_to_shared_combine() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../std/timing_lens.dag");
    let src = std::fs::read_to_string(&path).expect("read src/v3/std/timing_lens.dag");
    let seq = src
        .find("fn timing_sequential_op")
        .expect("timing_sequential_op");
    let br = src.find("fn timing_branch_op").expect("timing_branch_op");
    assert!(
        seq < br,
        "expected sequential_op before branch_op in source order"
    );
    let seq_window = &src[seq..(seq + 220).min(src.len())];
    assert!(
        seq_window.contains("timing_measurement_lens_combine(a, b, true)"),
        "timing_sequential_op must delegate to timing_measurement_lens_combine(..., true); got: {seq_window:?}"
    );
    let br_window = &src[br..(br + 220).min(src.len())];
    assert!(
        br_window.contains("timing_measurement_lens_combine(a, b, false)"),
        "timing_branch_op must delegate to timing_measurement_lens_combine(..., false); got: {br_window:?}"
    );
}

/// openai-pro / PR #2360: `timing_lens_validate` must not all-pass non-`Observed`
/// report states at the substrate hook (P3 / design §2.6).
#[test]
fn timing_lens_validate_surfaces_diagnostic_for_non_observed_states() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../std/timing_lens.dag");
    let src = std::fs::read_to_string(&path).expect("read src/v3/std/timing_lens.dag");
    let start = src
        .find("fn timing_lens_validate(d: Dag, composed: TimingMeasurement)")
        .expect("timing_lens_validate");
    let end = src[start..]
        .find("fn timing_measurement_iterate")
        .map(|i| start + i)
        .unwrap_or(src.len());
    let body = &src[start..end];
    assert!(
        body.contains("Unobserved =>")
            && body.contains("timing_lens_validate_non_observed")
            && body.contains("Ambiguous =>")
            && body.contains("Stale =>"),
        "non-observed arms must delegate to timing_lens_validate_non_observed; got:\n{body}"
    );
    assert!(
        body.contains("Observed { duration: _ } =>") && body.contains("NoDiagnostic"),
        "Observed arm should retain NoDiagnostic at this scaffold; got:\n{body}"
    );
    let helper_start = src
        .find("fn timing_lens_validate_non_observed(d: Dag, text: String)")
        .expect("timing_lens_validate_non_observed");
    let helper_window = &src[helper_start..(helper_start + 320).min(src.len())];
    assert!(
        helper_window.contains("SomeDiagnostic"),
        "timing_lens_validate_non_observed must construct SomeDiagnostic; got: {helper_window:?}"
    );
}
