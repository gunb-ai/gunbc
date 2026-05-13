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

fn operations_commute(dag: &Dag, a: &Operation, b: &Operation) -> bool {
    match (
        &operation_effect_shape(dag, a),
        &operation_effect_shape(dag, b),
    ) {
        (EffectShape::IsIdempotent(ia), EffectShape::IsIdempotent(ib)) => {
            idempotent_shapes_commute(&ia, &ib)
        }
        _ => false,
    }
}

fn operation_locator(dag: &Dag, op: &Operation) -> String {
    dag.declaration_opt(&op.callable.decl)
        .and_then(|decl| decl.name.as_deref())
        .map(|name| format!("{}:{:?}", name, op.callable.decl))
        .unwrap_or_else(|| format!("{:?}", op.callable.decl))
}

fn extract_linear_branches(
    branches: &NonSingletonList<Box<WorkflowEffect>>,
) -> Option<Vec<Vec<Operation>>> {
    let mut out = Vec::new();
    for br in branches.iter() {
        match br.as_ref() {
            WorkflowEffect::LinearEffect { ops } => out.push(ops.to_vec()),
            _ => return None,
        }
    }
    Some(out)
}

fn flatten_branch_ops(branch_ops: &[Vec<Operation>]) -> Vec<Operation> {
    branch_ops
        .iter()
        .flat_map(|ops| ops.iter().cloned())
        .collect()
}

fn pairwise_cross_branch_commutes(
    dag: &Dag,
    branch_ops: &[Vec<Operation>],
) -> Result<(), (Operation, Operation)> {
    for i in 0..branch_ops.len() {
        for j in (i + 1)..branch_ops.len() {
            for oa in &branch_ops[i] {
                for ob in &branch_ops[j] {
                    if !operations_commute(dag, oa, ob) {
                        return Err((oa.clone(), ob.clone()));
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
    match compose_operation_effects(p0, flatten_branch_ops(&branch_ops).as_slice()) {
        CompositionVerdict::BrokenBy { first_breaker } => {
            return WorkflowParallelismReport::ParallelCompositionVerdict(
                CompositionVerdict::BrokenBy { first_breaker },
            );
        }
        CompositionVerdict::IdempotentComposition => {}
    }
    match pairwise_cross_branch_commutes(p0, &branch_ops) {
        Ok(()) => WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition,
        ),
        Err((left, right)) => parallel_unsupported(
            ParallelismUnsupportedKind::PairwiseNonCommute,
            format!(
                "parallel branch operations do not commute under parallel scheduling: left={}, right={}",
                operation_locator(p0, &left),
                operation_locator(p0, &right)
            ),
        ),
    }
}

pub(super) fn loop_iteration_parallel_emission_indicator(p0: &Dag, p1: NodeId) -> i64 {
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
            operation_effect_shape(p0, op),
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect)
        ) {
            return 0;
        }
    }
    1
}
