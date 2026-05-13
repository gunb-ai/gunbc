// AUTO-GENERATED from `src/v3/lenses/idempotency.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub(super) fn analyze_workflow(p0: &Dag, p1: &NodeId) -> WorkflowIdempotencyReport {
    let Some(workflow) = p0.lane2_workflow_effect_at(p1) else {
        return report_unsupported_workflow_variant(
            "Lane2WorkflowRoot",
            "lane2_stage2b_idempotency_lens",
            "no WorkflowEffect at this substrate root - populate `lane2_workflow` on `Value`/`Bind` via lowering or `try_register_lane2_workflow_effect`",
        );
    };
    lane2_workflow_idempotency_report(workflow)
}
