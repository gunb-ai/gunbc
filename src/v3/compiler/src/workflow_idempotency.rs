//! Lane 2 Stage 2b — workflow idempotency analysis (`std.effects` mirror).
//!
//! Authority for the algebra lives in `src/v3/std/effects.dag`; these helpers
//! are the compiler-side projection used by tests and native consumers until
//! the emitted lens module is the sole entry point.

use crate::dag::{
    CompositionVerdict, Dag, EffectShape, IdempotencyUnsupportedDetail, NodeId, OperationEffect,
    WorkflowEffect, WorkflowIdempotencyReport,
};

pub fn operation_to_breaker(op: &OperationEffect) -> Option<crate::dag::BreakingOperation> {
    match &op.shape {
        EffectShape::IsIdempotent(_) => None,
        EffectShape::IsBreaking(shape) => Some(crate::dag::BreakingOperation {
            operation_name: op.operation_name.clone(),
            shape: shape.clone(),
        }),
    }
}

pub fn compose_operation_effects(effects: &[OperationEffect]) -> CompositionVerdict {
    for effect in effects {
        if let Some(b) = operation_to_breaker(effect) {
            return CompositionVerdict::BrokenBy { first_breaker: b };
        }
    }
    CompositionVerdict::IdempotentComposition
}

pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    let Some(workflow) = d.lane2_workflow_effect_at(workflow_root) else {
        return WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "Lane2WorkflowRoot".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "no WorkflowEffect facts on the Dag for this NodeId — analysis reads only Dag-local carriers (try_register_lane2_workflow_effect until lowering attaches them)"
                    .to_string(),
            },
        );
    };
    match workflow {
        WorkflowEffect::LinearEffect { ops } => WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            compose_operation_effects(ops.to_vec().as_slice()),
        ),
        WorkflowEffect::BranchEffect { .. } => WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "BranchEffect".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "non-linear workflow; branch-wise idempotency composition is not in the Stage 2b algebra"
                    .to_string(),
            },
        ),
        WorkflowEffect::LoopEffect { .. } => WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "LoopEffect".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "non-linear workflow; loop-carried idempotency composition is not in the Stage 2b algebra"
                    .to_string(),
            },
        ),
        WorkflowEffect::ParallelEffect { .. } => WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "ParallelEffect".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "non-linear workflow; parallel idempotency composition is not in the Stage 2b algebra"
                    .to_string(),
            },
        ),
    }
}
