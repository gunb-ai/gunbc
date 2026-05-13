// AUTO-GENERATED from `src/v3/lenses/idempotency.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub fn analyze_workflow(p0: &Dag, p1: &NodeId) -> WorkflowIdempotencyReport {
    match &((p0).lane2_workflow_effect_at(p1).cloned()) { None => report_unsupported_workflow_variant(&(String::from("Lane2WorkflowRoot")), &(String::from("lane2_stage2b_idempotency_lens")), &(String::from("no WorkflowEffect at this substrate root - populate `lane2_workflow` on `Value`/`Bind` via lowering or `try_register_lane2_workflow_effect`"))), Some(wf) => lane2_workflow_idempotency_report(wf), }
}
