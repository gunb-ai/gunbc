//! **Layer:** integration
//!
//! Consumer receipt for R3 §1.8 gate #106 `show_correct_code_diagnostic_coverage`
//! (`docs/r3-program-plan.md` §1.8 row 106): substrate carriers at
//! `src/v3/std/diagnostics.dag` (`Correction`, `CorrectionWitness`, `RetirementPlan`,
//! mandatory `Diagnostic.correction`) plus a compile-time live-correction roundtrip
//! anchor (break → diagnose → apply → recompile with zero diagnostics).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::diagnostics::{
    apply_correction_and_reparse, Correction, Diagnostic as HostDiagnostic,
};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::CompileError;

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
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

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj: {other:?}"),
    }
}

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

fn variant_payload_field_labels_sorted(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
) -> Vec<String> {
    let sum_decl = dag
        .declaration_by_name(sum_name)
        .unwrap_or_else(|| panic!("`{sum_name}` missing from full bootstrap"));
    let variants = match &sum_decl.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("`{sum_name}` is not a Disj: {other:?}"),
    };
    let variant = variants
        .iter()
        .find(|v| v.label == variant_label)
        .unwrap_or_else(|| panic!("`{sum_name}` missing `{variant_label}` variant"));
    let payload_decl = dag.declaration(variant.ty);
    let mut labels: Vec<String> = match &payload_decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        conn => panic!("`{sum_name}::{variant_label}` payload is not a Conj: {conn:?}"),
    };
    labels.sort();
    labels
}

fn variant_payload_field_ty(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
    field_label: &str,
) -> DeclarationId {
    let sum_decl = dag
        .declaration_by_name(sum_name)
        .unwrap_or_else(|| panic!("`{sum_name}` missing from full bootstrap"));
    let variants = match &sum_decl.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("`{sum_name}` is not a Disj: {other:?}"),
    };
    let variant = variants
        .iter()
        .find(|v| v.label == variant_label)
        .unwrap_or_else(|| panic!("`{sum_name}` missing `{variant_label}` variant"));
    let payload_decl = dag.declaration(variant.ty);
    let fields = match &payload_decl.connective {
        TypeConnective::Conj { children } => children,
        conn => panic!("`{sum_name}::{variant_label}` payload is not a Conj: {conn:?}",),
    };
    fields
        .iter()
        .find(|f| f.label == field_label)
        .unwrap_or_else(|| panic!("`{sum_name}::{variant_label}` missing `{field_label}` field"))
        .ty
}

#[test]
fn gate_106_correction_sum_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    assert_eq!(
        disj_variant_labels(&dag, "Correction"),
        vec![
            "LiveCorrection".to_string(),
            "DeferredCorrection".to_string()
        ],
        "Correction coproduct drifted from row #106 ratified shape (`diagnostics.dag`)"
    );
}

#[test]
fn gate_106_correction_witness_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels = conj_field_labels(&dag, "CorrectionWitness");
    labels.sort();
    assert_eq!(
        labels,
        vec![
            "description".to_string(),
            "new_source".to_string(),
            "span".to_string(),
        ],
        "CorrectionWitness field set drifted from row #106 ratification"
    );
    let string_id = decl_id_by_name(&dag, "String");
    let span_id = decl_id_by_name(&dag, "SourceSpan");
    assert_eq!(
        conj_field_ty(&dag, "CorrectionWitness", "description"),
        string_id,
        "`CorrectionWitness.description` must be `String`"
    );
    assert_eq!(
        conj_field_ty(&dag, "CorrectionWitness", "span"),
        span_id,
        "`CorrectionWitness.span` must be `SourceSpan`"
    );
    assert_eq!(
        conj_field_ty(&dag, "CorrectionWitness", "new_source"),
        string_id,
        "`CorrectionWitness.new_source` must be `String`"
    );
}

#[test]
fn gate_106_retirement_plan_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels = conj_field_labels(&dag, "RetirementPlan");
    labels.sort();
    assert_eq!(
        labels,
        vec!["exit_condition".to_string(), "owner".to_string()],
        "RetirementPlan field set drifted from row #106 ratification"
    );
    let string_id = decl_id_by_name(&dag, "String");
    assert_eq!(
        conj_field_ty(&dag, "RetirementPlan", "owner"),
        string_id,
        "`RetirementPlan.owner` must be `String`"
    );
    assert_eq!(
        conj_field_ty(&dag, "RetirementPlan", "exit_condition"),
        string_id,
        "`RetirementPlan.exit_condition` must be `String`"
    );
}

#[test]
fn gate_106_diagnostic_record_carries_mandatory_correction_field() {
    let dag = generated_full_bootstrap_dag();
    let correction_id = decl_id_by_name(&dag, "Correction");
    let any_kind_id = decl_id_by_name(&dag, "AnyDiagnosticKind");
    let span_id = decl_id_by_name(&dag, "SourceSpan");
    let string_id = decl_id_by_name(&dag, "String");

    let mut labels = conj_field_labels(&dag, "Diagnostic");
    labels.sort();
    assert_eq!(
        labels,
        vec![
            "correction".to_string(),
            "kind".to_string(),
            "message".to_string(),
            "span".to_string(),
        ],
        "substrate Diagnostic record drifted — mandatory `correction` authority lives in diagnostics.dag"
    );
    assert_eq!(
        conj_field_ty(&dag, "Diagnostic", "kind"),
        any_kind_id,
        "`Diagnostic.kind` must remain `AnyDiagnosticKind` (Q6.5 widening)"
    );
    assert_eq!(
        conj_field_ty(&dag, "Diagnostic", "span"),
        span_id,
        "`Diagnostic.span` must be `SourceSpan`"
    );
    assert_eq!(
        conj_field_ty(&dag, "Diagnostic", "message"),
        string_id,
        "`Diagnostic.message` must be `String`"
    );
    assert_eq!(
        conj_field_ty(&dag, "Diagnostic", "correction"),
        correction_id,
        "`Diagnostic.correction` must point at substrate `Correction` (no nullable carrier)"
    );
}

#[test]
fn gate_106_live_correction_variant_payload_locked() {
    let dag = generated_full_bootstrap_dag();
    assert_eq!(
        variant_payload_field_labels_sorted(&dag, "Correction", "LiveCorrection"),
        vec!["witness".to_string()],
        "`LiveCorrection` payload field-set must match diagnostics.dag (no extras / omissions)"
    );
    let witness_id = decl_id_by_name(&dag, "CorrectionWitness");
    assert_eq!(
        variant_payload_field_ty(&dag, "Correction", "LiveCorrection", "witness"),
        witness_id,
        "`LiveCorrection` payload must carry `witness: CorrectionWitness`"
    );
}

#[test]
fn gate_106_deferred_correction_variant_payload_locked() {
    let dag = generated_full_bootstrap_dag();
    assert_eq!(
        variant_payload_field_labels_sorted(&dag, "Correction", "DeferredCorrection"),
        vec!["reason".to_string(), "retirement_plan".to_string(),],
        "`DeferredCorrection` payload field-set must match diagnostics.dag (no extras / omissions)"
    );
    let string_id = decl_id_by_name(&dag, "String");
    let plan_id = decl_id_by_name(&dag, "RetirementPlan");
    assert_eq!(
        variant_payload_field_ty(&dag, "Correction", "DeferredCorrection", "reason"),
        string_id,
        "`DeferredCorrection.reason` must be `String`"
    );
    assert_eq!(
        variant_payload_field_ty(&dag, "Correction", "DeferredCorrection", "retirement_plan",),
        plan_id,
        "`DeferredCorrection.retirement_plan` must be `RetirementPlan`"
    );
}

#[test]
fn gate_106_type_mismatch_live_correction_roundtrip_recompiles_cleanly() {
    let source = "let x: Bool = 1\n";
    let file = "gate_106_type_mismatch_roundtrip.v3";
    let dag = match compile_to_dag(source, file) {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected semantic failure for {file}, got {other:?}"),
    };
    let diagnostic = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diagnostic)| match diagnostic {
            HostDiagnostic::TypeMismatch { .. } => Some(diagnostic),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected TypeMismatch diagnostic, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        matches!(diagnostic.correction(), Correction::LiveCorrection { .. }),
        "gate #106 anchor expects LiveCorrection for Bool/Int mismatch; got {:?}",
        diagnostic.correction()
    );
    let fix = diagnostic.correction();
    let repaired = apply_correction_and_reparse(source, file, fix).unwrap_or_else(|error| {
        panic!("correction should apply for {file}: {fix:?}\nerror: {error:?}")
    });
    match compile_to_dag(&repaired, file) {
        Ok(_) => {}
        Err(other) => panic!(
            "expected clean compile after applying correction: {other:?}\nrepaired:\n{repaired}"
        ),
    }
}
