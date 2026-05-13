//! **Layer:** integration
//!
//! Structural acceptance for R3 §1.8 gate #62 `substrate_gap_file_ingestion_closed`:
//! `FileAttachment` in `src/v3/std/timing_lens.dag` (Refined-B-1 — 5-of-7 subset of
//! `WorkflowObservationAnchor`; ratification `docs/briefs/r3-substrate-gate-62-file-attachment-carrier-worker.md`).

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
fn file_attachment_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels: Vec<String> = conj_field_labels(&dag, "FileAttachment");
    labels.sort();
    assert_eq!(
        labels,
        vec![
            "attached_at_ns".to_string(),
            "content_digest".to_string(),
            "producer_id".to_string(),
            "subject_node".to_string(),
            "workflow_run_id".to_string(),
        ],
        "FileAttachment field set drifted from Refined-B-1 ratification (gate #62)"
    );
}

/// P2 / M9: each field resolves to the same cross-module nominal authority as gate #55 anchor subset.
#[test]
fn file_attachment_field_types_locked() {
    let dag = generated_full_bootstrap_dag();
    let node_id = dag
        .declaration_by_name("NodeId")
        .expect("`NodeId` missing from full bootstrap (substrate authority)")
        .id;
    let content_hash = dag
        .declaration_by_name("ContentHash")
        .expect("`ContentHash` missing from full bootstrap (std.types authority)")
        .id;
    let producer = dag
        .declaration_by_name("WorkflowProducerId")
        .expect("`WorkflowProducerId` missing from full bootstrap")
        .id;
    let run = dag
        .declaration_by_name("WorkflowRunId")
        .expect("`WorkflowRunId` missing from full bootstrap")
        .id;
    let nanoseconds = dag
        .declaration_by_name("Nanoseconds")
        .expect("`Nanoseconds` missing from full bootstrap (timing_lens authority)")
        .id;
    assert_eq!(
        conj_field_ty(&dag, "FileAttachment", "subject_node"),
        node_id,
        "`FileAttachment.subject_node` must be `NodeId`"
    );
    assert_eq!(
        conj_field_ty(&dag, "FileAttachment", "content_digest"),
        content_hash,
        "`FileAttachment.content_digest` must be `ContentHash`"
    );
    assert_eq!(
        conj_field_ty(&dag, "FileAttachment", "producer_id"),
        producer,
        "`FileAttachment.producer_id` must be `WorkflowProducerId`"
    );
    assert_eq!(
        conj_field_ty(&dag, "FileAttachment", "workflow_run_id"),
        run,
        "`FileAttachment.workflow_run_id` must be `WorkflowRunId`"
    );
    assert_eq!(
        conj_field_ty(&dag, "FileAttachment", "attached_at_ns"),
        nanoseconds,
        "`FileAttachment.attached_at_ns` must be `Nanoseconds`"
    );
}

#[test]
fn file_attachment_field_count_is_five() {
    let dag = generated_full_bootstrap_dag();
    let labels = conj_field_labels(&dag, "FileAttachment");
    let unique: HashSet<String> = labels.into_iter().collect();
    assert_eq!(
        unique.len(),
        5,
        "FileAttachment must declare exactly 5 fields (Refined-B-1)"
    );
}
