//! **Layer:** integration
//!
//! Structural acceptance for T-Workflow-As-Data Slice 2 timing-lens substrate
//! (gunbc#1955): `TimingMeasurement`, `TimingObservationSet`,
//! `WorkflowObservationAnchor`, `TimingBudget`, and nominal `TimingLens` (`Lens<TimingMeasurement>`)
//! in `src/v3/std/timing_lens.dag`.

use std::collections::HashSet;

use v3_compiler::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};
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

fn disj_variant_ty(dag: &Dag, name: &str, variant: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|v| v.label == variant)
                .unwrap_or_else(|| panic!("`{name}` missing `{variant}` variant"))
                .ty
        }
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

fn data_field<'a>(dag: &'a Dag, data_name: &str, field: &str) -> &'a FieldValue {
    let decl = dag
        .declaration_by_name(data_name)
        .unwrap_or_else(|| panic!("`{data_name}` missing from full bootstrap"));
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        panic!(
            "`{data_name}` must lower to a structural data body, got {:?}",
            decl.value_body
        );
    };
    fields
        .iter()
        .find(|(label, _)| label == field)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("`{data_name}` missing `{field}` field"))
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

/// P2 single-authority (openai-pro / PR #2360): field *names* are not enough —
/// `anchor` × `measurement` must be the structural product that prevents
/// provenance/payload drift (`TimingObservationEntry` in `timing_lens.dag`).
#[test]
fn timing_observation_entry_field_types_locked() {
    let dag = generated_full_bootstrap_dag();
    let anchor = dag
        .declaration_by_name("WorkflowObservationAnchor")
        .expect("`WorkflowObservationAnchor` missing from full bootstrap")
        .id;
    let measurement = dag
        .declaration_by_name("TimingMeasurement")
        .expect("`TimingMeasurement` missing from full bootstrap")
        .id;
    assert_eq!(
        conj_field_ty(&dag, "TimingObservationEntry", "anchor"),
        anchor,
        "`TimingObservationEntry.anchor` must be `WorkflowObservationAnchor`"
    );
    assert_eq!(
        conj_field_ty(&dag, "TimingObservationEntry", "measurement"),
        measurement,
        "`TimingObservationEntry.measurement` must be `TimingMeasurement`"
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

/// P2 / gate #55 inv.3–4: provenance roles and run key use distinct branded
/// `std.types` nominals, not bare `String`.
#[test]
fn workflow_observation_anchor_provenance_ids_are_branded() {
    let dag = generated_full_bootstrap_dag();
    let producer = dag
        .declaration_by_name("WorkflowProducerId")
        .expect("`WorkflowProducerId` missing from full bootstrap")
        .id;
    let observer = dag
        .declaration_by_name("WorkflowObserverId")
        .expect("`WorkflowObserverId` missing from full bootstrap")
        .id;
    let prover = dag
        .declaration_by_name("WorkflowProverId")
        .expect("`WorkflowProverId` missing from full bootstrap")
        .id;
    let run = dag
        .declaration_by_name("WorkflowRunId")
        .expect("`WorkflowRunId` missing from full bootstrap")
        .id;
    assert_eq!(
        conj_field_ty(&dag, "WorkflowObservationAnchor", "producer_id"),
        producer
    );
    assert_eq!(
        conj_field_ty(&dag, "WorkflowObservationAnchor", "observer_id"),
        observer
    );
    assert_eq!(
        conj_field_ty(&dag, "WorkflowObservationAnchor", "prover_id"),
        prover
    );
    assert_eq!(
        conj_field_ty(&dag, "WorkflowObservationAnchor", "workflow_run_id"),
        run
    );
}

/// P2 / gate #55 inv.2: artifact digest must be `ContentHash`, not bare `String`
/// (M9 content-addressed nominal vs free-form text).
#[test]
fn workflow_observation_anchor_artifact_digest_is_content_hash() {
    let dag = generated_full_bootstrap_dag();
    let content_hash = dag
        .declaration_by_name("ContentHash")
        .expect("`ContentHash` missing from full bootstrap (std.types authority)")
        .id;
    let ty = conj_field_ty(&dag, "WorkflowObservationAnchor", "artifact_digest");
    assert_eq!(
        ty, content_hash,
        "`WorkflowObservationAnchor.artifact_digest` must be `ContentHash`, not `String`"
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

/// Gate #55 inv.4 / Practice 4: attachment instant shares the `Nanoseconds` nominal
/// used for duration magnitudes (openai-pro — label-only anchor tests miss this).
#[test]
fn workflow_observation_anchor_attached_at_ns_is_nanoseconds() {
    let dag = generated_full_bootstrap_dag();
    let nanoseconds = dag
        .declaration_by_name("Nanoseconds")
        .expect("`Nanoseconds` missing from full bootstrap")
        .id;
    let ty = conj_field_ty(&dag, "WorkflowObservationAnchor", "attached_at_ns");
    assert_eq!(
        ty, nanoseconds,
        "`WorkflowObservationAnchor.attached_at_ns` must be `Nanoseconds` (SI-ns scaffold coordinate)"
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

/// P2 / M9: budget ceiling uses the same `Nanoseconds` nominal as timing reports.
#[test]
fn timing_budget_max_field_is_nanoseconds() {
    let dag = generated_full_bootstrap_dag();
    let nanoseconds = dag
        .declaration_by_name("Nanoseconds")
        .expect("`Nanoseconds` missing from full bootstrap")
        .id;
    let ty = conj_field_ty(&dag, "TimingBudget", "max");
    assert_eq!(
        ty, nanoseconds,
        "`TimingBudget.max` must be `Nanoseconds` (aligned timing magnitude carrier)"
    );
}

/// Gate #54 acceptance: the carrier is not only the `TimingMeasurement` sum; the
/// std module must publish the concrete `Lens<TimingMeasurement>` data row that
/// downstream workflow-as-data/self-application gates can reference.
#[test]
fn timing_lens_data_row_lowers_to_lens_timing_measurement_hooks() {
    let dag = generated_full_bootstrap_dag();
    let expected_refs = [
        ("read", "timing_lens_read"),
        ("branch", "timing_branch_op"),
        ("iterate", "timing_measurement_iterate"),
        ("validate", "timing_lens_validate"),
    ];
    for (field, target) in expected_refs {
        let target_id = dag
            .declaration_by_name(target)
            .unwrap_or_else(|| panic!("`{target}` missing from full bootstrap"))
            .id;
        assert_eq!(
            data_field(&dag, "timing_lens", field),
            &FieldValue::Reference(target_id),
            "`timing_lens.{}` must reference `{}`",
            field,
            target
        );
    }

    let lens_name = data_field(&dag, "timing_lens", "name");
    assert_eq!(
        lens_name,
        &FieldValue::Literal(v3_compiler::dag::LiteralBits::String("timing".to_string())),
        "`timing_lens.name` must stay the canonical timing dimension label"
    );

    let monoid_id = dag
        .declaration_by_name("timing_lens_monoid")
        .expect("`timing_lens_monoid` missing from full bootstrap")
        .id;
    assert_eq!(
        data_field(&dag, "timing_lens", "sequential"),
        &FieldValue::Reference(monoid_id),
        "`timing_lens.sequential` must use the timing monoid data row"
    );
}

#[test]
fn timing_lens_monoid_data_row_locks_observed_zero_identity() {
    let dag = generated_full_bootstrap_dag();
    let op_id = dag
        .declaration_by_name("timing_sequential_op")
        .expect("`timing_sequential_op` missing from full bootstrap")
        .id;
    assert_eq!(
        data_field(&dag, "timing_lens_monoid", "op"),
        &FieldValue::Reference(op_id),
        "`timing_lens_monoid.op` must reference timing_sequential_op"
    );

    let identity = data_field(&dag, "timing_lens_monoid", "identity");
    let FieldValue::Variant {
        constructor,
        payload,
    } = identity
    else {
        panic!("`timing_lens_monoid.identity` must be Observed payload, got {identity:?}");
    };
    assert_eq!(
        *constructor,
        disj_variant_ty(&dag, "TimingMeasurement", "Observed"),
        "`timing_lens_monoid.identity` must use the TimingMeasurement.Observed constructor"
    );
    assert_eq!(
        payload.len(),
        1,
        "`Observed` identity payload arity drifted"
    );
    let FieldValue::Record(observed_fields) = &payload[0] else {
        panic!("`Observed` identity payload must be a record, got {payload:?}");
    };
    let duration_fields = observed_fields
        .iter()
        .find(|(field, _)| field == "duration")
        .map(|(_, value)| value)
        .map(|duration| {
            let FieldValue::Record(duration_fields) = duration else {
                panic!("`Observed.duration` identity must be a record, got {duration:?}");
            };
            duration_fields
        })
        .unwrap_or(observed_fields);
    let count_field = duration_fields.iter().find(|(field, _)| field == "count");
    assert!(
        count_field.is_some(),
        "`Observed` identity must carry zero ns as either `duration.count` or lowered `count`; got {observed_fields:?}"
    );
    let Some((_, count)) = count_field else {
        unreachable!("asserted count field presence above");
    };
    assert_eq!(
        count,
        &FieldValue::Literal(v3_compiler::dag::literal_bits_int(0)),
        "`timing_lens_monoid.identity` must be Observed zero ns"
    );
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

/// Inline review (briansrls / PR #2360): outer `Stale` must not return `Stale`
/// without inspecting `b` — `(Stale, Unobserved)` must stay **`Unobserved`**
/// (strict missing-evidence on the RHS; P2 facts-forward / no silencing).
#[test]
fn timing_measurement_lens_combine_stale_arm_inspects_rhs() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../std/timing_lens.dag");
    let src = std::fs::read_to_string(&path).expect("read src/v3/std/timing_lens.dag");
    let start = src
        .find("fn timing_measurement_lens_combine")
        .expect("timing_measurement_lens_combine");
    let end = src[start..]
        .find("// Sequential monoid")
        .map(|i| start + i)
        .expect("comment sentinel before timing_sequential_op");
    let body = &src[start..end];
    let stale_arm = body
        .find("Stale =>")
        .expect("outer `Stale =>` arm in timing_measurement_lens_combine");
    let head = &body[stale_arm..(stale_arm + 120).min(body.len())];
    assert!(
        head.contains("match b") && head.contains("Unobserved => Unobserved"),
        "outer `Stale` arm must `match b` and map `Unobserved` to `Unobserved`; got:\n{head}"
    );
    assert!(
        !head.contains("Stale =>\n      Stale"),
        "outer `Stale` arm must not short-circuit to `Stale` without inspecting `b`; got:\n{head}"
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
    assert!(
        !src.contains("Empty => NoDiagnostic"),
        "timing_lens_validate_non_observed must not fail-open non-evidence on empty behavior_spine (P3); remove Empty => NoDiagnostic"
    );
    assert!(
        src.contains("d.declarations")
            && (src.contains("timing_lens_degenerate_empty_dag_span")
                || src.contains("dd.head.span")),
        "empty behavior_spine path must borrow a real `SourceSpan` (first declaration span) or the degenerate placeholder helper; got missing wiring"
    );
}

/// openai-pro / PR #2360 (REQUEST_CHANGES): `timing_measurement_iterate` must not
/// return unchanged `Observed` while dropping `LoopBound` (P3 fail-closed).
#[test]
fn timing_measurement_iterate_fail_closed_on_observed_with_loop_bound() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../std/timing_lens.dag");
    let src = std::fs::read_to_string(&path).expect("read src/v3/std/timing_lens.dag");
    let start = src
        .find("fn timing_measurement_iterate(body: TimingMeasurement, bound: LoopBound)")
        .expect("timing_measurement_iterate");
    let end = src[start..]
        .find("fn timing_lens_read")
        .map(|i| start + i)
        .expect("timing_lens_read follows iterate");
    let body = &src[start..end];
    assert!(
        body.contains("Observed { duration: _ } =>")
            && body.contains("Cardinality(_) => timing_measurement_unobserved()")
            && body.contains("Descent(_) => timing_measurement_unobserved()"),
        "`Observed` × any `LoopBound` must map to `timing_measurement_unobserved()` until lowering lands; got:\n{body}"
    );
    assert!(
        body.contains("Unobserved => Unobserved")
            && body.contains("Ambiguous => Ambiguous")
            && body.contains("Stale => Stale"),
        "non-`Observed` arms must pass through unchanged; got:\n{body}"
    );
    assert!(
        !body.contains("Cardinality(_) => body") && !body.contains("Descent(_) => body"),
        "`timing_measurement_iterate` must not ignore `LoopBound` by returning `body` unchanged on `Observed`; got:\n{body}"
    );
}

/// §1.8 gate #54 `timing_lens_carrier_landed`: `type TimingLens = Lens<TimingMeasurement>` is the
/// substrate nominal for the parameterized lens surface (`docs/design-timing-lens.md` §1).
#[test]
fn timing_lens_carrier_landed_timing_lens_is_lens_instantiation() {
    let dag = generated_full_bootstrap_dag();
    let lens_template = dag.declaration_by_name("Lens").expect("Lens template").id;
    let timing_m = dag
        .declaration_by_name("TimingMeasurement")
        .expect("TimingMeasurement")
        .id;
    let timing_lens_ty = dag
        .declaration_by_name("TimingLens")
        .expect("TimingLens nominal")
        .id;
    let decl = dag.declaration(timing_lens_ty);
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &decl.connective
    else {
        panic!(
            "TimingLens must be Lens<TimingMeasurement> instantiation, got {:?}",
            decl.connective
        );
    };
    assert_eq!(*template, lens_template, "TimingLens template");
    assert_eq!(arguments.len(), 1, "Lens has one type parameter");
    assert_eq!(
        arguments[0].value, timing_m,
        "TimingLens must instantiate Lens at TimingMeasurement"
    );
}
