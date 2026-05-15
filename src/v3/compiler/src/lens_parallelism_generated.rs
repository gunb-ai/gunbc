// AUTO-GENERATED from `src/v3/lenses/parallelism.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum LinearBranchesLookup {
    LinearBranchesFound { branches: Vec<Vec<Operation>> },
    NonLinearBranchFound,
}
#[derive(Clone, Debug)]
pub enum NonCommutingPairLookup {
    NonCommutingPairFound { left: Operation, right: Operation },
    AllPairsCommute,
}
#[derive(Clone, Debug)]
pub enum ParallelismMode {
    OptInIndependent,
    Sequential,
}
pub fn report_pairwise_non_commute(p0: Operation, p1: Operation) -> WorkflowParallelismReport {
    WorkflowParallelismReport::ParallelismUnsupported {
        _0: ParallelismUnsupportedDetail {
            kind: ParallelismUnsupportedKind::PairwiseNonCommute {
                left: (p0).clone(),
                right: (p1).clone(),
            },
            downstream_stage: String::from("lane2_stage2e_parallelism_lens"),
            reason: String::from(
                "parallel branch operations do not commute under parallel scheduling",
            ),
        },
    }
}
pub fn nsl_to_workflow_list(p0: &NonSingletonList<WorkflowEffect>) -> Vec<WorkflowEffect> {
    {
        let mut __list = {
            let mut __list = ((p0).rest).to_vec();
            __list.insert(0, ((p0).second).clone());
            __list
        };
        __list.insert(0, ((p0).first).clone());
        __list
    }
}
pub fn idempotent_shapes_commute(p0: &IdempotentShape, p1: &IdempotentShape) -> bool {
    match p0 {
        IdempotentShape::ReadEffect => match p1 {
            IdempotentShape::ReadEffect => true,
            IdempotentShape::UpsertEffect { key_source: _ } => false,
            IdempotentShape::DeleteEffect { key_source: _ } => false,
        },
        IdempotentShape::UpsertEffect { key_source: _ } => false,
        IdempotentShape::DeleteEffect { key_source: ka } => match p1 {
            IdempotentShape::ReadEffect => false,
            IdempotentShape::UpsertEffect { key_source: _ } => false,
            IdempotentShape::DeleteEffect { key_source: kb } => key_sources_equal(ka, kb),
        },
    }
}
pub fn key_sources_equal(p0: &KeySource, p1: &KeySource) -> bool {
    match p0 {
        KeySource::PathParam { param: pa } => match p1 {
            KeySource::PathParam { param: pb } => ((*(pa)) == (*(pb))),
            KeySource::InputField { field: _ } => false,
        },
        KeySource::InputField { field: fa } => match p1 {
            KeySource::PathParam { param: _ } => false,
            KeySource::InputField { field: fb } => ((*(fa)) == (*(fb))),
        },
    }
}
pub fn operations_commute(p0: &Operation, p1: &Operation) -> bool {
    match &(operation_effect_shape(p0)) {
        EffectShape::IsIdempotent(ia) => match &(operation_effect_shape(p1)) {
            EffectShape::IsIdempotent(ib) => idempotent_shapes_commute(ia, ib),
            EffectShape::IsBreaking(_) => false,
        },
        EffectShape::IsBreaking(_) => false,
    }
}
pub fn extract_linear_branches_from_list(p0: &[WorkflowEffect]) -> LinearBranchesLookup {
    match p0 {
        [] => LinearBranchesLookup::LinearBranchesFound {
            branches: Vec::new(),
        },
        [__list_head, __list_tail @ ..] => match __list_head {
            WorkflowEffect::LinearEffect { ops: ops } => {
                match &(extract_linear_branches_from_list(__list_tail)) {
                    LinearBranchesLookup::LinearBranchesFound {
                        branches: tail_branches,
                    } => LinearBranchesLookup::LinearBranchesFound {
                        branches: {
                            let mut __list = (tail_branches).to_vec();
                            __list.insert(0, (ops).to_vec());
                            __list
                        },
                    },
                    LinearBranchesLookup::NonLinearBranchFound => {
                        LinearBranchesLookup::NonLinearBranchFound
                    }
                }
            }
            WorkflowEffect::BranchEffect { arms: _ } => LinearBranchesLookup::NonLinearBranchFound,
            WorkflowEffect::LoopEffect { body: _ } => LinearBranchesLookup::NonLinearBranchFound,
            WorkflowEffect::ParallelEffect { branches: _ } => {
                LinearBranchesLookup::NonLinearBranchFound
            }
        },
    }
}
pub fn extract_linear_branches(p0: &NonSingletonList<WorkflowEffect>) -> LinearBranchesLookup {
    extract_linear_branches_from_list(&(nsl_to_workflow_list(p0)))
}
pub fn append_ops(p0: &[Operation], p1: Vec<Operation>) -> Vec<Operation> {
    match p0 {
        [] => p1,
        [__list_head, __list_tail @ ..] => {
            let mut __list = append_ops(__list_tail, (p1).clone());
            __list.insert(0, (__list_head).clone());
            __list
        }
    }
}
pub fn flatten_branch_ops(p0: &[Vec<Operation>]) -> Vec<Operation> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => append_ops(__list_head, flatten_branch_ops(__list_tail)),
    }
}
pub fn first_non_commuting_op_in_branch(p0: &[Operation], p1: Operation) -> NonCommutingPairLookup {
    match p0 {
        [] => NonCommutingPairLookup::AllPairsCommute,
        [__list_head, __list_tail @ ..] => {
            if operations_commute(&p1, __list_head) {
                first_non_commuting_op_in_branch(__list_tail, (p1).clone())
            } else {
                NonCommutingPairLookup::NonCommutingPairFound {
                    left: (p1).clone(),
                    right: (__list_head).clone(),
                }
            }
        }
    }
}
pub fn first_non_commuting_branch_pair(
    p0: &[Operation],
    p1: &[Operation],
) -> NonCommutingPairLookup {
    match p0 {
        [] => NonCommutingPairLookup::AllPairsCommute,
        [__list_head, __list_tail @ ..] => {
            match &(first_non_commuting_op_in_branch(p1, (__list_head).clone())) {
                NonCommutingPairLookup::AllPairsCommute => {
                    first_non_commuting_branch_pair(__list_tail, p1)
                }
                NonCommutingPairLookup::NonCommutingPairFound {
                    left: ____payload_6134_6184_left,
                    right: ____payload_6134_6184_right,
                } => NonCommutingPairLookup::NonCommutingPairFound {
                    left: (____payload_6134_6184_left).clone(),
                    right: (____payload_6134_6184_right).clone(),
                },
            }
        }
    }
}
pub fn first_non_commuting_against_tail(
    p0: &[Vec<Operation>],
    p1: &[Operation],
) -> NonCommutingPairLookup {
    match p0 {
        [] => NonCommutingPairLookup::AllPairsCommute,
        [__list_head, __list_tail @ ..] => {
            match &(first_non_commuting_branch_pair(p1, __list_head)) {
                NonCommutingPairLookup::AllPairsCommute => {
                    first_non_commuting_against_tail(__list_tail, p1)
                }
                NonCommutingPairLookup::NonCommutingPairFound {
                    left: ____payload_6642_6692_left,
                    right: ____payload_6642_6692_right,
                } => NonCommutingPairLookup::NonCommutingPairFound {
                    left: (____payload_6642_6692_left).clone(),
                    right: (____payload_6642_6692_right).clone(),
                },
            }
        }
    }
}
pub fn pairwise_cross_branch_commutes(p0: &[Vec<Operation>]) -> NonCommutingPairLookup {
    match p0 {
        [] => NonCommutingPairLookup::AllPairsCommute,
        [__list_head, __list_tail @ ..] => {
            match &(first_non_commuting_against_tail(__list_tail, __list_head)) {
                NonCommutingPairLookup::AllPairsCommute => {
                    pairwise_cross_branch_commutes(__list_tail)
                }
                NonCommutingPairLookup::NonCommutingPairFound {
                    left: ____payload_7097_7147_left,
                    right: ____payload_7097_7147_right,
                } => NonCommutingPairLookup::NonCommutingPairFound {
                    left: (____payload_7097_7147_left).clone(),
                    right: (____payload_7097_7147_right).clone(),
                },
            }
        }
    }
}
pub fn parallelism_report_for_workflow(p0: &WorkflowEffect) -> WorkflowParallelismReport {
    match p0 {
        WorkflowEffect::ParallelEffect { branches: branches } => {
            match &(extract_linear_branches(branches)) {
                LinearBranchesLookup::NonLinearBranchFound => report_parallelism_unsupported(
                    &(ParallelismUnsupportedKind::NonLinearParallelBranch),
                    &(String::from(
                        "Stage 2e v1 requires every parallel branch to be `LinearEffect`",
                    )),
                ),
                LinearBranchesLookup::LinearBranchesFound {
                    branches: branch_ops,
                } => match &(compose_effects(&(flatten_branch_ops(branch_ops)))) {
                    CompositionVerdict::BrokenBy { first_breaker: b } => {
                        WorkflowParallelismReport::ParallelCompositionVerdict {
                            _0: CompositionVerdict::BrokenBy {
                                first_breaker: (b).clone(),
                            },
                        }
                    }
                    CompositionVerdict::IdempotentComposition => {
                        match &(pairwise_cross_branch_commutes(branch_ops)) {
                            NonCommutingPairLookup::AllPairsCommute => {
                                WorkflowParallelismReport::ParallelCompositionVerdict {
                                    _0: CompositionVerdict::IdempotentComposition,
                                }
                            }
                            NonCommutingPairLookup::NonCommutingPairFound {
                                left: ____payload_8046_8102_left,
                                right: ____payload_8046_8102_right,
                            } => report_pairwise_non_commute(
                                (____payload_8046_8102_left).clone(),
                                (____payload_8046_8102_right).clone(),
                            ),
                        }
                    }
                },
            }
        }
        WorkflowEffect::LinearEffect { ops: _ } => report_parallelism_unsupported(
            &(ParallelismUnsupportedKind::NotParallelEffectRoot),
            &(String::from("parallelism lens analyzes `ParallelEffect` roots only")),
        ),
        WorkflowEffect::BranchEffect { arms: _ } => report_parallelism_unsupported(
            &(ParallelismUnsupportedKind::NotParallelEffectRoot),
            &(String::from("parallelism lens analyzes `ParallelEffect` roots only")),
        ),
        WorkflowEffect::LoopEffect { body: _ } => report_parallelism_unsupported(
            &(ParallelismUnsupportedKind::NotParallelEffectRoot),
            &(String::from("parallelism lens analyzes `ParallelEffect` roots only")),
        ),
    }
}
pub fn analyze_parallelism(p0: &Dag, p1: &NodeId) -> WorkflowParallelismReport {
    match &((p0).lane2_workflow_effect_at(p1).cloned()) { None => report_parallelism_unsupported(&(ParallelismUnsupportedKind::NoWorkflowProjection), &(String::from("no WorkflowEffect projection on this substrate node - analysis reads only `Value`/`Bind` fields set by lowering or `try_register_lane2_workflow_effect`"))), Some(wf) => parallelism_report_for_workflow(wf), }
}
pub fn read_only_ops(p0: &[Operation]) -> bool {
    match p0 {
        [] => true,
        [__list_head, __list_tail @ ..] => match &(operation_effect_shape(__list_head)) {
            EffectShape::IsIdempotent(shape) => match shape {
                IdempotentShape::ReadEffect => read_only_ops(__list_tail),
                IdempotentShape::UpsertEffect { key_source: _ } => false,
                IdempotentShape::DeleteEffect { key_source: _ } => false,
            },
            EffectShape::IsBreaking(_) => false,
        },
    }
}
pub fn loop_iteration_parallel_emission_indicator(p0: &Dag, p1: &NodeId) -> i64 {
    match &((p0).lane2_workflow_effect_at(p1).cloned()) {
        None => 0,
        Some(wf) => match wf {
            WorkflowEffect::LoopEffect { body: body } => match body {
                WorkflowEffect::LinearEffect { ops: ops } => match ops {
                    [] => 0,
                    [__list_head, __list_tail @ ..] => {
                        if read_only_ops(ops) {
                            1
                        } else {
                            0
                        }
                    }
                },
                WorkflowEffect::BranchEffect { arms: _ } => 0,
                WorkflowEffect::LoopEffect { body: _ } => 0,
                WorkflowEffect::ParallelEffect { branches: _ } => 0,
            },
            WorkflowEffect::LinearEffect { ops: _ } => 0,
            WorkflowEffect::BranchEffect { arms: _ } => 0,
            WorkflowEffect::ParallelEffect { branches: _ } => 0,
        },
    }
}
pub fn parallelism_enforcement_project(p0: ParallelismMode) -> ParallelismMode {
    p0
}
pub fn parallelism_enforcement_violates(p0: &ParallelismMode, p1: &ParallelismMode) -> bool {
    match p1 {
        ParallelismMode::OptInIndependent => match p0 {
            ParallelismMode::Sequential => true,
            ParallelismMode::OptInIndependent => false,
        },
        ParallelismMode::Sequential => match p0 {
            ParallelismMode::OptInIndependent => false,
            ParallelismMode::Sequential => false,
        },
    }
}
pub fn parallelism_lens_combine(p0: &ParallelismMode, p1: &ParallelismMode) -> ParallelismMode {
    match p0 {
        ParallelismMode::Sequential => ParallelismMode::Sequential,
        ParallelismMode::OptInIndependent => match p1 {
            ParallelismMode::Sequential => ParallelismMode::Sequential,
            ParallelismMode::OptInIndependent => ParallelismMode::OptInIndependent,
        },
    }
}
pub fn behavior_node_id(p0: &Behavior) -> NodeId {
    match p0 {
        Behavior::Value(v) => (v).id,
        Behavior::Transform(t) => (t).id,
        Behavior::Branch(b) => (b).id,
        Behavior::Loop(l) => (l).id,
        Behavior::Bind(bind) => (bind).id,
    }
}
pub fn parallelism_iteration_observed_mode(p0: &Dag, p1: &NodeId) -> ParallelismMode {
    if (loop_iteration_parallel_emission_indicator(p0, p1) == 1) {
        ParallelismMode::OptInIndependent
    } else {
        ParallelismMode::Sequential
    }
}
pub fn parallelism_lens_read(p0: &Dag, p1: &Behavior) -> Witness<ParallelismMode> {
    Witness::Inhabits((parallelism_iteration_observed_mode(p0, &(behavior_node_id(p1)))).clone())
}
pub fn parallelism_lens_iterate(p0: ParallelismMode, p1: &LoopBound) -> ParallelismMode {
    p0
}
pub fn parallelism_lens_validate(p0: &Dag, p1: &ParallelismMode) -> OptionalDiagnostic {
    OptionalDiagnostic::NoDiagnostic
}
