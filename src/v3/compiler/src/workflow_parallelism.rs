//! Lane 2 Stage 2e — parallel workflow composition safety (DB-20).
//!
//! Reads `lane2_workflow` from native `Value` / `Bind` (same authority as
//! Stage 2b). For [`crate::dag::WorkflowEffect::ParallelEffect`], checks that
//! linear branches can be scheduled concurrently without reorder hazards,
//! projecting through [`crate::dag::CompositionVerdict`] per DB-18 / PR #529.
//!
//! [`register_independent_bind_parallelism`] is the R3 free-consequence hook:
//! when every top-level `Bind` is pairwise dataflow-independent, the compiler
//! records a `ParallelEffect` of empty `LinearEffect` branches on the workflow
//! root (α: last `Bind`). Effect-commutativity and cost witnesses remain
//! future lenses; this pass is intentionally conservative on side-effecting
//! RHS (not modeled here).

use std::collections::HashSet;

use crate::dag::{
    Behavior, BindNode, CompositionVerdict, Dag, EffectShape, ElementRef, IdempotentShape, NodeId,
    NonSingletonList, OperationEffect, ParallelismUnsupportedDetail, ParallelismUnsupportedKind,
    PortId, WorkflowEffect, WorkflowParallelismReport,
};

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
) -> Option<ElementRef<OperationEffect>> {
    // Stage 2e reuses `CompositionVerdict`, so breaker identity stays an
    // index-shaped handle rather than a copied record. The canonical evidence
    // list here is the branch-order flattening of every linear branch's `ops`;
    // callers must resolve the handle against that same flattening, which
    // `WorkflowEffect::operation_at` reconstructs.
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

/// Analyze whether a `ParallelEffect` workflow can be scheduled concurrently.
///
/// This remains the native Rust bridge for `src/v3/lenses/parallelism.dag`
/// while that lens surface is still pending full `.dag` ownership.
pub fn analyze_parallelism(d: &Dag, workflow_root: NodeId) -> WorkflowParallelismReport {
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

fn expand_behavior_backward_ports(
    d: &Dag,
    node: NodeId,
    frontier: &mut Vec<PortId>,
    visited_nodes: &mut HashSet<NodeId>,
) {
    if !visited_nodes.insert(node) {
        return;
    }
    let Some(behavior) = d.node_opt(&node) else {
        return;
    };
    match behavior {
        Behavior::Value(_) => {}
        Behavior::Transform(transform) => {
            for input in &transform.inputs {
                frontier.push(*input);
            }
        }
        Behavior::Branch(branch) => {
            frontier.push(branch.input);
            for path in &branch.paths {
                expand_behavior_backward_ports(d, path.body, frontier, visited_nodes);
                frontier.push(path.output);
            }
        }
        Behavior::Loop(loop_node) => {
            frontier.push(loop_node.source);
            frontier.push(loop_node.init);
            if let Some(count) = loop_node.bound.count_port() {
                frontier.push(count);
            }
            expand_behavior_backward_ports(d, loop_node.body, frontier, visited_nodes);
        }
        Behavior::Bind(bind) => {
            for param in &bind.params {
                frontier.push(*param);
            }
            frontier.push(bind.value);
        }
    }
}

fn backward_reachable_ports_from_port(d: &Dag, start: PortId) -> HashSet<PortId> {
    let mut visited_ports: HashSet<PortId> = HashSet::new();
    let mut visited_nodes: HashSet<NodeId> = HashSet::new();
    let mut frontier: Vec<PortId> = vec![start];
    while let Some(port) = frontier.pop() {
        if !visited_ports.insert(port) {
            continue;
        }
        let Some(p) = d.port_opt(&port) else {
            continue;
        };
        let Some(producer) = p.produced_by else {
            continue;
        };
        expand_behavior_backward_ports(d, producer, &mut frontier, &mut visited_nodes);
    }
    visited_ports
}

/// `later` depends on `earlier` when the RHS port of `earlier` appears in the
/// backward port slice from `later`'s RHS — `SurfaceExpr::Var` reuses the bound
/// `PortId` (see `lower_expr` / `scope.get`), so the producer graph may never
/// mention [`BindNode::id`] even when data flows from an earlier `let`.
fn bind_rhs_depends_on_prior_bind(d: &Dag, later: &BindNode, earlier: &BindNode) -> bool {
    backward_reachable_ports_from_port(d, later.value).contains(&earlier.value)
}

/// When there are two or more top-level [`Behavior::Bind`] nodes in `nodes`
/// order and every later bind's RHS is dataflow-independent of every earlier
/// bind (no backward path from `bind.value` reaches a prior bind), installs
/// [`WorkflowEffect::ParallelEffect`] on the workflow root (last bind).
///
/// Skips when any bind already carries `lane2_workflow` or when the pairwise
/// check fails. Side-effect / commutativity certification is **not** performed
/// here — pair with future `Lens<Effect-Commutativity>` before widening beyond
/// pure-looking programs.
///
/// `user_source_file` must match [`BindNode::span`].`file` for user-authored
/// binds (the `file` argument to [`crate::compile_to_dag`]) so bootstrap/std
/// binds are excluded from the independence walk.
pub fn register_independent_bind_parallelism(dag: &mut Dag, user_source_file: &str) {
    let mut binds: Vec<&BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|b| b.span.file == user_source_file)
        .collect();
    if binds.len() < 2 {
        return;
    }
    binds.sort_by_key(|b| (b.span.byte_start, b.span.byte_end));
    if binds.iter().any(|b| b.lane2_workflow().is_some()) {
        return;
    }
    for (i, bi) in binds.iter().enumerate() {
        for bj in binds.iter().skip(i + 1) {
            if bind_rhs_depends_on_prior_bind(dag, bj, bi) {
                return;
            }
        }
    }
    let last_id = binds.last().expect("len >= 2").id;
    let branches: Vec<Box<WorkflowEffect>> = (0..binds.len())
        .map(|_| {
            Box::new(WorkflowEffect::LinearEffect {
                ops: Vec::new(),
            })
        })
        .collect();
    let Some(ns) = NonSingletonList::from_vec(branches) else {
        return;
    };
    let wf = WorkflowEffect::ParallelEffect { branches: ns };
    dag.try_register_lane2_workflow_effect(last_id, wf);
}

/// R3 T-Free-Consequences witness: `1` when [`register_independent_bind_parallelism`] installed
/// [`WorkflowEffect::ParallelEffect`] on the claim-file workflow root (last user `Bind` by source
/// order). Evaluated by `LensOutputEquals` in the test runner (M1(2.8) blocks `.dag` match bodies
/// for this fixture module).
pub(crate) fn r3_auto_parallelism_schedule_witness(dag: &Dag, claim_file: &str) -> i64 {
    let mut binds: Vec<&BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|b| b.span.file == claim_file)
        .collect();
    if binds.is_empty() {
        return 0;
    }
    binds.sort_by_key(|b| (b.span.byte_start, b.span.byte_end));
    let root = binds.last().expect("non-empty").id;
    match dag.lane2_workflow_effect_at(&root) {
        Some(WorkflowEffect::ParallelEffect { .. }) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod register_parallelism_tests {
    use super::*;
    use crate::compile_to_dag;

    #[test]
    fn independent_binds_register_parallel_on_last_bind() {
        let dag = compile_to_dag("let a: Int = 1\nlet b: Int = 2", "t.v3").expect("compile");
        let mut binds: Vec<_> = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .filter(|b| b.span.file == "t.v3")
            .collect();
        binds.sort_by_key(|b| (b.span.byte_start, b.span.byte_end));
        assert_eq!(binds.len(), 2);
        let last = binds.last().expect("two binds").id;
        let wf = dag
            .lane2_workflow_effect_at(&last)
            .expect("parallelism registered");
        assert!(matches!(wf, WorkflowEffect::ParallelEffect { .. }));
    }

    #[test]
    fn dependent_binds_do_not_register_parallel() {
        let dag = compile_to_dag("let a: Int = 1\nlet b: Int = a + 1", "t.v3").expect("compile");
        let mut binds: Vec<_> = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .filter(|b| b.span.file == "t.v3")
            .collect();
        binds.sort_by_key(|b| (b.span.byte_start, b.span.byte_end));
        let last = binds.last().expect("two binds").id;
        assert!(dag.lane2_workflow_effect_at(&last).is_none());
    }
}
