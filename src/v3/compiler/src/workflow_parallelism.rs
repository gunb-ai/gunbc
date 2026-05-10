//! Lane 2 Stage 2e — parallel workflow composition safety (DB-20).
//!
//! Reads `lane2_workflow` from native `Value` / `Bind` (same authority as
//! Stage 2b). For [`crate::dag::WorkflowEffect::ParallelEffect`], checks that
//! linear branches can be scheduled concurrently without reorder hazards,
//! projecting through [`crate::dag::CompositionVerdict`] per DB-18 / PR #529.
//!
//! R3 free-consequences gate #43 uses [`r3_auto_parallelism_schedule_witness`]
//! (test-runner host mirror) to observe **pairwise dataflow independence** among
//! module-item `let` binds, and to stay **0** when any bind RHS carries
//! [`Behavior::Branch`] / [`Behavior::Loop`] (branch arms are not parallel-safe
//! in this witness). Installing [`WorkflowEffect::ParallelEffect`] on
//! `lane2_workflow` remains **deferred** until `Lens<Effect-Commutativity>` and
//! `Lens<Cost>` certify safe parallel scheduling per `docs/design-free-consequences.md`
//! — DB-20 fail-closed: do not fabricate a parallel workflow fact without those
//! witnesses.

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

/// True when the backward slice from [`BindNode::value`] reaches [`Behavior::Branch`] or
/// [`Behavior::Loop`]. Uses the same port walk driver as [`backward_reachable_ports_from_port`]
/// ([`expand_behavior_backward_ports`]) so subgraph shape stays consistent; exits early on
/// branch/loop without descending further. The schedule witness stays **0** when any module `let`
/// RHS carries this nonlinear control.
fn bind_rhs_subgraph_contains_branch_or_loop(d: &Dag, bind: &BindNode) -> bool {
    let mut visited_ports: HashSet<PortId> = HashSet::new();
    let mut visited_nodes: HashSet<NodeId> = HashSet::new();
    let mut frontier: Vec<PortId> = vec![bind.value];
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
        if visited_nodes.contains(&producer) {
            continue;
        }
        let Some(behavior) = d.node_opt(&producer) else {
            continue;
        };
        if matches!(behavior, Behavior::Branch(_) | Behavior::Loop(_)) {
            return true;
        }
        expand_behavior_backward_ports(d, producer, &mut frontier, &mut visited_nodes);
    }
    false
}

/// `later` depends on `earlier` when the RHS port of `earlier` appears in the
/// backward port slice from `later`'s RHS — `SurfaceExpr::Var` reuses the bound
/// `PortId` (see `lower_expr` / `scope.get`), so the producer graph may never
/// mention [`BindNode::id`] even when data flows from an earlier `let`.
fn bind_rhs_depends_on_prior_bind(d: &Dag, later: &BindNode, earlier: &BindNode) -> bool {
    backward_reachable_ports_from_port(d, later.value).contains(&earlier.value)
}

/// `SurfaceItem::Let` at module scope (`lower_item`) — empty `params`, no
/// [`BindEmitParticipation::UserCallable`], not a `where`-refinement wrapper.
/// Contrasts with function/lambda binds (`emit_participation: UserCallable`) and
/// refinement Binds (non-empty `params` / `<refinement:…>` names).
fn is_module_item_value_let_bind(b: &BindNode) -> bool {
    b.emit_participation().is_none() && b.params.is_empty() && !b.name.starts_with("<refinement:")
}

fn module_item_value_lets_in_file<'a>(dag: &'a Dag, user_source_file: &str) -> Vec<&'a BindNode> {
    let mut binds: Vec<&BindNode> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .filter(|b| b.span.file == user_source_file && is_module_item_value_let_bind(b))
        .collect();
    binds.sort_by_key(|b| (b.span.byte_start, b.span.byte_end));
    binds
}

/// Returns `false` when there are fewer than two binds (nothing to compare pairwise) **or** when
/// any later bind's RHS is dataflow-dependent on an earlier bind ([`bind_rhs_depends_on_prior_bind`]).
fn module_lets_pairwise_rhs_independent(dag: &Dag, binds: &[&BindNode]) -> bool {
    if binds.len() < 2 {
        return false;
    }
    for (i, bi) in binds.iter().enumerate() {
        for bj in binds.iter().skip(i + 1) {
            if bind_rhs_depends_on_prior_bind(dag, bj, bi) {
                return false;
            }
        }
    }
    true
}

/// R3 T-Free-Consequences witness: `1` when the claim file has two or more
/// module-item value [`Behavior::Bind`] nodes ([`SurfaceItem::Let`]), every
/// later bind's RHS is dataflow-independent of every earlier bind, and **no**
/// such bind's RHS producer subgraph contains [`Behavior::Branch`] or
/// [`Behavior::Loop`] (branch arms / loop bodies are not treated as parallel-safe
/// here). Does **not** read `lane2_workflow` — full parallel workflow facts stay
/// gated on commutativity + cost per DB-20.
pub(crate) fn r3_auto_parallelism_schedule_witness(dag: &Dag, claim_file: &str) -> i64 {
    let binds = module_item_value_lets_in_file(dag, claim_file);
    if !module_lets_pairwise_rhs_independent(dag, &binds) {
        return 0;
    }
    if binds
        .iter()
        .any(|b| bind_rhs_subgraph_contains_branch_or_loop(dag, b))
    {
        return 0;
    }
    i64::from(binds.len() >= 2)
}

#[cfg(test)]
mod register_parallelism_tests {
    use super::*;
    use crate::compile_to_dag;

    #[test]
    fn independent_module_lets_witness_parallel_schedule() {
        let dag = compile_to_dag("let a: Int = 1\nlet b: Int = 2", "t.v3").expect("compile");
        assert_eq!(r3_auto_parallelism_schedule_witness(&dag, "t.v3"), 1);
    }

    #[test]
    fn interleaved_fn_item_does_not_mix_into_module_let_witness() {
        let dag = compile_to_dag(
            "let a: Int = 1\nfn f() -> Int = 1\nlet b: Int = 2\n",
            "t.v3",
        )
        .expect("compile");
        assert_eq!(r3_auto_parallelism_schedule_witness(&dag, "t.v3"), 1);
    }

    #[test]
    fn dependent_module_lets_witness_sequential_schedule() {
        let dag = compile_to_dag("let a: Int = 1\nlet b: Int = a + 1", "t.v3").expect("compile");
        assert_eq!(r3_auto_parallelism_schedule_witness(&dag, "t.v3"), 0);
    }

    #[test]
    fn independent_module_lets_with_branch_in_rhs_stay_sequential() {
        let dag = compile_to_dag(
            "let a: Int = 1\nlet r: Int = if 1 > 0 then 10 else 20\n",
            "t.v3",
        )
        .expect("compile");
        assert_eq!(r3_auto_parallelism_schedule_witness(&dag, "t.v3"), 0);
    }
}
