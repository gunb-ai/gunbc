// AUTO-GENERATED from `src/v3/lenses/parallelism.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

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

fn key_sources_equal(a: &KeySource, b: &KeySource) -> bool {
    match (a, b) {
        (KeySource::PathParam { param: pa }, KeySource::PathParam { param: pb }) => pa == pb,
        (KeySource::InputField { field: fa }, KeySource::InputField { field: fb }) => fa == fb,
        (KeySource::CompositeKey { fields: fa }, KeySource::CompositeKey { fields: fb }) => {
            fa == fb
        }
        _ => false,
    }
}

fn idempotent_shapes_commute(a: &IdempotentShape, b: &IdempotentShape) -> bool {
    match (a, b) {
        (IdempotentShape::ReadEffect, IdempotentShape::ReadEffect) => true,
        (
            IdempotentShape::DeleteEffect { key_source: ka },
            IdempotentShape::DeleteEffect { key_source: kb },
        ) => key_sources_equal(ka, kb),
        _ => false,
    }
}

fn operations_commute(a: &OperationEffect, b: &OperationEffect) -> bool {
    match (&a.shape, &b.shape) {
        (EffectShape::IsIdempotent(ia), EffectShape::IsIdempotent(ib)) => {
            idempotent_shapes_commute(ia, ib)
        }
        _ => false,
    }
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

fn first_breaking_across_branches(
    branch_ops: &[Vec<OperationEffect>],
) -> Option<ElementRef<OperationEffect>> {
    let flattened: Vec<OperationEffect> = branch_ops
        .iter()
        .flat_map(|ops| ops.iter().cloned())
        .collect();
    for (index, op) in flattened.iter().enumerate() {
        if matches!(op.shape, EffectShape::IsBreaking(_)) {
            return ElementRef::from_slice(flattened.as_slice(), index);
        }
    }
    None
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

pub fn analyze_parallelism(p0: &Dag, p1: NodeId) -> WorkflowParallelismReport {
    let Some(workflow) = p0.lane2_workflow_effect_at(&p1) else {
        return parallel_unsupported(
            ParallelismUnsupportedKind::NoWorkflowProjection,
            "no WorkflowEffect projection on this substrate node - analysis reads only `Value`/`Bind` fields set by lowering or `try_register_lane2_workflow_effect`",
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
        return WorkflowParallelismReport::ParallelCompositionVerdict(CompositionVerdict::BrokenBy {
            first_breaker: b,
        });
    }
    match pairwise_cross_branch_commutes(&branch_ops) {
        Ok(()) => WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition,
        ),
        Err((_a, _b)) => parallel_unsupported(
            ParallelismUnsupportedKind::PairwiseNonCommute,
            "parallel branch operations do not commute under parallel scheduling",
        ),
    }
}

pub fn loop_iteration_parallel_emission_indicator(p0: &Dag, p1: NodeId) -> i64 {
    let Some(workflow) = p0.lane2_workflow_effect_at(&p1) else {
        return 0;
    };
    let WorkflowEffect::LoopEffect { body } = workflow else {
        return 0;
    };
    let WorkflowEffect::LinearEffect { ops } = body.as_ref() else {
        return 0;
    };
    if ops.is_empty() {
        return 0;
    }
    for op in ops {
        if !matches!(
            op.shape,
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect)
        ) {
            return 0;
        }
    }
    1
}
