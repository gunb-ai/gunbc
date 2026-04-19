//! Lane 2 Stage 2e — parallel workflow composition safety (DB-20).
//!
//! Reads `lane2_workflow` from native `Value` / `Bind` (same authority as
//! Stage 2b). For [`crate::dag::WorkflowEffect::ParallelEffect`], checks that
//! linear branches can be scheduled concurrently without reorder hazards,
//! projecting through [`crate::dag::CompositionVerdict`] per DB-18 / PR #529.

use crate::dag::{
    CompositionVerdict, Dag, EffectShape, IdempotentShape, NodeId, NonSingletonList,
    OperationEffect, ParallelismUnsupportedDetail, ParallelismUnsupportedKind, WorkflowEffect,
    WorkflowParallelismReport,
};
use crate::workflow_idempotency::operation_to_breaker;

const DOWNSTREAM: &str = "lane2_stage2e_parallelism_lens";

fn parallel_unsupported(
    kind: ParallelismUnsupportedKind,
    reason: impl Into<String>,
) -> WorkflowParallelismReport {
    WorkflowParallelismReport::ParallelismUnsupported(ParallelismUnsupportedDetail {
        kind,
        downstream_stage: DOWNSTREAM.to_string(),
        reason: reason.into(),
    })
}

/// Pairwise commutativity for idempotent [`OperationEffect`] pairs.
///
/// **Upsert×Upsert:** always **not** commute in v1. `UpsertEffect` in
/// `std.effects` carries only [`crate::dag::KeySource`] — no value body or
/// merge-law witness — so the lens cannot certify **concurrent** write
/// commutativity (distinct payloads / last-write semantics). Fail-closed until
/// the algebra exposes an explicit witness (Codex / #543).
///
/// **Delete×Delete:** same structural key commutes (same cell removed).
///
/// **Read×Read:** always commutes.
///
/// Distinct [`KeySource`] values do not prove runtime disjointness (e.g. different
/// `PathParam` names may alias) — inequality on deletes → not commute.
fn idempotent_operations_commute(a: &OperationEffect, b: &OperationEffect) -> bool {
    match (&a.shape, &b.shape) {
        (EffectShape::IsIdempotent(ia), EffectShape::IsIdempotent(ib)) => match (ia, ib) {
            (IdempotentShape::ReadEffect, IdempotentShape::ReadEffect) => true,
            (IdempotentShape::UpsertEffect { .. }, IdempotentShape::UpsertEffect { .. }) => false,
            (
                IdempotentShape::DeleteEffect { key_source: ka },
                IdempotentShape::DeleteEffect { key_source: kb },
            ) => ka == kb,
            _ => false,
        },
        _ => false,
    }
}

fn operations_commute(a: &OperationEffect, b: &OperationEffect) -> bool {
    idempotent_operations_commute(a, b)
}

fn first_breaking_across_branches(
    branch_ops: &[Vec<OperationEffect>],
) -> Option<crate::dag::BreakingOperation> {
    for ops in branch_ops {
        for op in ops {
            if let Some(b) = operation_to_breaker(op) {
                return Some(b);
            }
        }
    }
    None
}

fn extract_linear_branches(
    branches: &NonSingletonList<Box<WorkflowEffect>>,
) -> Option<Vec<Vec<OperationEffect>>> {
    let mut out = Vec::new();
    for br in branches.iter() {
        match br.as_ref() {
            WorkflowEffect::LinearEffect { ops } => out.push(ops.to_vec()),
            _ => return None,
        }
    }
    Some(out)
}

fn pairwise_cross_branch_commutes(
    branch_ops: &[Vec<OperationEffect>],
) -> Result<(), (String, String)> {
    for i in 0..branch_ops.len() {
        for j in (i + 1)..branch_ops.len() {
            for oa in &branch_ops[i] {
                for ob in &branch_ops[j] {
                    if !operations_commute(oa, ob) {
                        return Err((oa.operation_name.clone(), ob.operation_name.clone()));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn analyze_parallelism(d: &Dag, workflow_root: NodeId) -> WorkflowParallelismReport {
    let Some(workflow) = d.lane2_workflow_effect_at(&workflow_root) else {
        return parallel_unsupported(
            ParallelismUnsupportedKind::NoWorkflowProjection,
            "no WorkflowEffect projection on this substrate node — analysis reads only `Value`/`Bind` fields set by lowering or `try_register_lane2_workflow_effect`",
        );
    };
    let WorkflowEffect::ParallelEffect { branches } = workflow else {
        return parallel_unsupported(
            ParallelismUnsupportedKind::NotParallelEffectRoot,
            "parallelism lens analyzes `ParallelEffect` roots only",
        );
    };

    let Some(branch_ops) = extract_linear_branches(branches) else {
        return parallel_unsupported(
            ParallelismUnsupportedKind::NonLinearParallelBranch,
            "Stage 2e v1 requires every parallel branch to be `LinearEffect`",
        );
    };

    if let Some(b) = first_breaking_across_branches(&branch_ops) {
        return WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::BrokenBy { first_breaker: b },
        );
    }

    match pairwise_cross_branch_commutes(&branch_ops) {
        Ok(()) => WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition,
        ),
        Err((a, b)) => parallel_unsupported(
            ParallelismUnsupportedKind::PairwiseNonCommute,
            format!("operations `{a}` and `{b}` do not commute under parallel scheduling"),
        ),
    }
}
