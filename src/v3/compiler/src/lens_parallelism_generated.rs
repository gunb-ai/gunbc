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

fn pairwise_non_commute(
    left: Operation,
    right: Operation,
    reason: impl Into<String>,
) -> ParallelismUnsupportedDetail {
    ParallelismUnsupportedDetail {
        kind: ParallelismUnsupportedKind::PairwiseNonCommute { left, right },
        downstream_stage: DOWNSTREAM.to_string(),
        reason: reason.into(),
    }
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

fn operations_commute(
    dag: &Dag,
    a: &Operation,
    b: &Operation,
) -> Result<bool, EffectClassificationFailure> {
    Ok(match (classify_operation_effect(dag, a)?, classify_operation_effect(dag, b)?) {
        (EffectShape::IsIdempotent(ia), EffectShape::IsIdempotent(ib)) => {
            idempotent_shapes_commute(&ia, &ib)
        }
        _ => false,
    })
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
) -> Result<(), ParallelismUnsupportedDetail> {
    for i in 0..branch_ops.len() {
        for j in (i + 1)..branch_ops.len() {
            for oa in &branch_ops[i] {
                for ob in &branch_ops[j] {
                    match operations_commute(dag, oa, ob) {
                        Ok(true) => {}
                        Ok(false) => {
                            return Err(pairwise_non_commute(
                                oa.clone(),
                                ob.clone(),
                                "parallel branch operations do not commute under parallel scheduling",
                            ));
                        }
                        Err(EffectClassificationFailure::StdMethodAnchorResolutionFailed) => {
                            return Err(ParallelismUnsupportedDetail {
                                kind: ParallelismUnsupportedKind::EffectClassificationUnavailable,
                                downstream_stage: DOWNSTREAM.to_string(),
                                reason: "std.effects method anchors are missing or ambiguous; operation effect classification cannot safely prove parallelism"
                                    .to_string(),
                            });
                        }
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
        Ok(CompositionVerdict::BrokenBy { first_breaker }) => {
            return WorkflowParallelismReport::ParallelCompositionVerdict(
                CompositionVerdict::BrokenBy { first_breaker },
            );
        }
        Ok(CompositionVerdict::IdempotentComposition) => {}
        Err(_) => {
            return parallel_unsupported(
                ParallelismUnsupportedKind::EffectClassificationUnavailable,
                "std.effects method anchors are missing or ambiguous; operation effect classification cannot safely prove parallelism",
            );
        }
    }
    match pairwise_cross_branch_commutes(p0, &branch_ops) {
        Ok(()) => WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition,
        ),
        Err(detail) => WorkflowParallelismReport::ParallelismUnsupported(detail),
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
        match classify_operation_effect(p0, op) {
            Ok(EffectShape::IsIdempotent(IdempotentShape::ReadEffect)) => {}
            Ok(_) | Err(EffectClassificationFailure::StdMethodAnchorResolutionFailed) => return 0,
        }
    }
    1
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParallelismMode {
    OptInIndependent,
    Sequential,
}

fn behavior_parallelism_root(behavior: &Behavior) -> NodeId {
    match behavior {
        Behavior::Value(v) => v.id,
        Behavior::Transform(t) => t.id,
        Behavior::Branch(b) => b.id,
        Behavior::Loop(l) => l.id,
        Behavior::Bind(bind) => bind.id,
    }
}

pub(super) fn parallelism_iteration_observed_mode(p0: &Dag, p1: NodeId) -> ParallelismMode {
    if loop_iteration_parallel_emission_indicator(p0, p1) == 1 {
        ParallelismMode::OptInIndependent
    } else {
        ParallelismMode::Sequential
    }
}

pub(super) fn parallelism_lens_read(p0: &Dag, p1: &Behavior) -> Witness<ParallelismMode> {
    Witness::Inhabits(parallelism_iteration_observed_mode(p0, behavior_parallelism_root(p1)))
}

pub(super) fn parallelism_lens_iterate(
    body: ParallelismMode,
    _bound: &LoopBound,
) -> ParallelismMode {
    body
}

pub(super) fn parallelism_lens_validate(
    _p0: &Dag,
    _p1: &ParallelismMode,
) -> OptionalDiagnostic {
    OptionalDiagnostic::NoDiagnostic
}

pub(super) fn parallelism_enforcement_project(m: ParallelismMode) -> ParallelismMode {
    m
}

pub(super) fn parallelism_enforcement_violates(
    observed: &ParallelismMode,
    declared: &ParallelismMode,
) -> bool {
    match declared {
        ParallelismMode::OptInIndependent => match observed {
            ParallelismMode::Sequential => true,
            ParallelismMode::OptInIndependent => false,
        },
        ParallelismMode::Sequential => match observed {
            ParallelismMode::OptInIndependent => false,
            ParallelismMode::Sequential => false,
        },
    }
}

pub(super) fn parallelism_lens_combine(
    a: &ParallelismMode,
    b: &ParallelismMode,
) -> ParallelismMode {
    match a {
        ParallelismMode::Sequential => ParallelismMode::Sequential,
        ParallelismMode::OptInIndependent => match b {
            ParallelismMode::Sequential => ParallelismMode::Sequential,
            ParallelismMode::OptInIndependent => ParallelismMode::OptInIndependent,
        },
    }
}
