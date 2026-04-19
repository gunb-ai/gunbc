//! Lane 2 Stage 2b — workflow idempotency analysis (`std.effects` mirror).
//!
//! Authority for the algebra lives in `src/v3/std/effects.dag`; the core
//! projection is shared with [`crate::lens_idempotency`] and with
//! [`report_unsupported_workflow_variant`] / [`lane2_workflow_idempotency_report`],
//! which `emit_rust_module` consumers import for rustc round-trips. Workflow structure
//! for analysis is read from **native** `Value` / `Bind` fields on the [`Dag`]
//! (`lane2_workflow`), mirrored on the reflected substrate (`substrate.dag`).

use crate::dag::{
    CompositionVerdict, Dag, EffectShape, ElementRef, IdempotencyUnsupportedDetail, NodeId,
    OperationEffect, WorkflowEffect, WorkflowIdempotencyReport,
};

pub(crate) fn compose_operation_effects(effects: &[OperationEffect]) -> CompositionVerdict {
    for (index, effect) in effects.iter().enumerate() {
        if matches!(effect.shape, EffectShape::IsBreaking(_)) {
            // `ElementRef` preserves the validated in-bounds position of the
            // breaker without copying a second breaker record; the breaking
            // subset fact and the owner-list identity still come from this
            // partition check plus callers resolving against the same slice.
            let first_breaker = ElementRef::from_slice(effects, index)
                .expect("enumerated workflow effect index must stay in-bounds");
            return CompositionVerdict::BrokenBy { first_breaker };
        }
    }
    CompositionVerdict::IdempotentComposition
}

/// Pure projection used by Stage 2b — kept aligned with
/// `std.effects::lane2_workflow_idempotency_report`.
/// Mirrors `std.effects::report_unsupported_workflow_variant` — exported for
/// `emit_rust_module` output from `src/v3/lenses/idempotency.dag` (rustc
/// round-trip in `m2_lens_idempotency_migration_test`).
pub fn report_unsupported_workflow_variant(
    variant_name: &str,
    downstream_stage: &str,
    reason: &str,
) -> WorkflowIdempotencyReport {
    WorkflowIdempotencyReport::IdempotencyUnsupported(IdempotencyUnsupportedDetail {
        variant_name: variant_name.to_string(),
        downstream_stage: downstream_stage.to_string(),
        reason: reason.to_string(),
    })
}

/// Mirrors `std.effects::lane2_workflow_idempotency_report`.
pub fn lane2_workflow_idempotency_report(workflow: &WorkflowEffect) -> WorkflowIdempotencyReport {
    project_workflow_idempotency_report(workflow)
}

pub(crate) fn project_workflow_idempotency_report(
    workflow: &WorkflowEffect,
) -> WorkflowIdempotencyReport {
    match workflow {
        WorkflowEffect::LinearEffect { ops } => WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            compose_operation_effects(ops.as_slice()),
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

pub(crate) fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    let Some(workflow) = d.lane2_workflow_effect_at(&workflow_root) else {
        return WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "Lane2WorkflowRoot".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "no WorkflowEffect at this substrate root - populate `lane2_workflow` on `Value`/`Bind` via lowering or `try_register_lane2_workflow_effect`"
                    .to_string(),
            },
        );
    };
    project_workflow_idempotency_report(workflow)
}
