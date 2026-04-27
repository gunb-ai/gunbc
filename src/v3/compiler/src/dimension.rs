//! DB-3 dimension analysis — Lane 2 Stage 2f.
//!
//! Type authority: `src/v3/std/dimensions.dag`, spine order:
//! [`behavior_spine_in_node_order`] (`src/v3/std/workflows.dag`).
//!
//! Symbolic-cost dimension is the first migrated lens: it reuses
//! [`crate::lens_cost_symbolic::symbolic_cost_of`] for per-Behavior
//! witnesses and reports the composed carrier at `workflow_root`'s result
//! port (same query the cost lens answers today).
//!
//! Witnesses are collected only for behaviors **reachable backward** from
//! `workflow_root`'s result port (plus `workflow_root` itself). The symbolic
//! cost lens fold only resolves costs along producer chains in `Dag.nodes`
//! construction order; bootstrap declarations outside the active workflow
//! slice routinely carry `Lookup::Miss` on their result ports even though the
//! user program is well-costed — whole-DAG iteration would false-fail.

use std::collections::HashSet;

use crate::dag::{Behavior, Dag, NodeId, PortId, SymbolicCost};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

fn behavior_result_port(b: &Behavior) -> PortId {
    match b {
        Behavior::Value(v) => v.result_port(),
        Behavior::Transform(t) => t.result_port(),
        Behavior::Branch(br) => br.result_port(),
        Behavior::Loop(l) => l.result_port(),
        Behavior::Bind(bind) => bind.result_port(),
    }
}

fn behavior_span(at: &Behavior) -> SourceSpan {
    match at {
        Behavior::Value(v) => v.span.clone(),
        Behavior::Transform(t) => t.span.clone(),
        Behavior::Branch(b) => b.span.clone(),
        Behavior::Loop(l) => l.span.clone(),
        Behavior::Bind(bind) => bind.span.clone(),
    }
}

/// Evidence partition — mirrors `Witness<Carrier>` in `std/dimensions.dag`.
#[derive(Debug, Clone)]
pub enum Witness<C> {
    Inhabits(C),
    Violates { reason: String, at: Behavior },
}

/// Report carrier — mirrors `DimensionReport<Carrier>` in `std/dimensions.dag`.
#[derive(Debug, Clone)]
pub enum DimensionReport<C> {
    DimensionOk {
        dimension_name: String,
        composed: C,
        witnesses: Vec<Witness<C>>,
    },
    DimensionFail {
        dimension_name: String,
        violations: Vec<Diagnostic>,
        witnesses: Vec<Witness<C>>,
    },
}

/// Same `Dag.nodes` order as `lenses/cost.dag::compute_symbolic_costs` /
/// `std/workflows.dag::behavior_spine`.
pub fn behavior_spine_in_node_order(d: &Dag) -> &[Behavior] {
    d.nodes()
}

/// Behaviors on the backward dataflow slice from `workflow_root`'s result
/// port, including `workflow_root` itself (the bind / root may not appear as
/// the `produced_by` target of its own result port).
fn workflow_reachable_behavior_ids(d: &Dag, workflow_root: NodeId) -> HashSet<NodeId> {
    let mut visited_nodes = HashSet::new();
    let mut visited_ports: HashSet<PortId> = HashSet::new();
    let mut frontier: Vec<PortId> = Vec::new();

    let root_behavior = d.node(workflow_root);
    frontier.push(behavior_result_port(root_behavior));

    while let Some(port) = frontier.pop() {
        if !visited_ports.insert(port) {
            continue;
        }
        let Some(producer) = d.port_opt(&port).and_then(|p| p.produced_by) else {
            continue;
        };
        expand_behavior_backward(d, producer, &mut frontier, &mut visited_nodes);
    }

    // Always include the workflow root itself for dimension witnesses (e.g. a
    // `Bind` is never the `produced_by` of its own `value` port). Do **not**
    // seed `workflow_root` into `visited_nodes` before the walk: when the root
    // is a `Transform`, its result port's producer is that same node — an early
    // insert would make `expand_behavior_backward` return without enqueueing the
    // transform's inputs, shrinking the slice to `{root}` only.
    visited_nodes.insert(workflow_root);
    visited_nodes
}

fn expand_behavior_backward(
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
                expand_behavior_backward(d, path.body, frontier, visited_nodes);
                frontier.push(path.output);
            }
        }
        Behavior::Loop(loop_node) => {
            frontier.push(loop_node.source);
            frontier.push(loop_node.init);
            if let Some(count) = loop_node.bound.count_port() {
                frontier.push(count);
            }
            expand_behavior_backward(d, loop_node.body, frontier, visited_nodes);
        }
        Behavior::Bind(bind) => {
            for param in &bind.params {
                frontier.push(*param);
            }
            frontier.push(bind.value);
        }
    }
}

/// Symbolic-cost dimension analysis — DB-3 dispatch over the cost lens algebra.
///
/// On success ([`DimensionReport::DimensionOk`]), `composed` is the asymptotic
/// bound at `workflow_root`'s result port. Witnesses walk every reachable
/// [`Behavior`] in [`behavior_spine_in_node_order`] (construction order,
/// filtered to the backward slice from the workflow root). Failure is only
/// [`DimensionReport::DimensionFail`]: there is no fabricated carrier.
pub fn analyze_symbolic_cost_dimension(
    d: &Dag,
    workflow_root: NodeId,
) -> DimensionReport<SymbolicCost> {
    const DIMENSION_NAME: &str = "symbolic_cost";
    let scope = workflow_reachable_behavior_ids(d, workflow_root);
    let mut witnesses = Vec::new();
    for behavior in d.nodes() {
        if !scope.contains(&behavior.id()) {
            continue;
        }
        let port = behavior_result_port(behavior);
        match symbolic_cost_of(d, &port) {
            SymbolicCostLookup::Miss => witnesses.push(Witness::Violates {
                reason: "missing symbolic cost for behavior result port".into(),
                at: behavior.clone(),
            }),
            SymbolicCostLookup::Hit(cost) => witnesses.push(Witness::Inhabits(cost)),
        }
    }

    let root = d.node(workflow_root);
    let root_lookup = symbolic_cost_of(d, &behavior_result_port(root));

    let witness_failure = witnesses
        .iter()
        .any(|w| matches!(w, Witness::Violates { .. }));

    if !witness_failure {
        if let SymbolicCostLookup::Hit(composed) = root_lookup {
            return DimensionReport::DimensionOk {
                dimension_name: DIMENSION_NAME.to_string(),
                composed,
                witnesses,
            };
        }
    }

    let mut violations: Vec<Diagnostic> = witnesses
        .iter()
        .filter_map(|w| {
            let Witness::Violates { reason, at } = w else {
                return None;
            };
            Some(Diagnostic::ParseError {
                message: format!("symbolic_cost dimension: {reason}"),
                span: behavior_span(at),
                fixes: vec![],
            })
        })
        .collect();

    // Root `Miss` normally duplicates a `Witness::Violates` on the workflow root
    // (same port as `behavior_result_port(root)`). Only synthesize a diagnostic
    // here if the witness spine and root lookup ever disagree (fail-closed).
    if matches!(root_lookup, SymbolicCostLookup::Miss) && violations.is_empty() {
        violations.push(Diagnostic::ParseError {
            message: "symbolic_cost dimension: missing symbolic cost for workflow root result port"
                .into(),
            span: behavior_span(root),
            fixes: vec![],
        });
    }

    DimensionReport::DimensionFail {
        dimension_name: DIMENSION_NAME.to_string(),
        violations,
        witnesses,
    }
}

#[cfg(test)]
mod fail_closed_tests {
    use super::*;
    use crate::dag::{Dag, LiteralBits, TransformTarget};
    use crate::operators::{ArithmeticOp, OperatorKind};

    #[test]
    fn missing_symbolic_cost_surfaces_as_dimension_fail_with_violates_witnesses() {
        let mut dag = Dag::new();
        let span = SourceSpan::new("dimension_fail_closed_test", 0, 0);
        let int_shape = dag.int_shape().expect("bootstrap Int");
        let lhs = dag.push_value(LiteralBits::Int(1), span.clone());
        let ghost_input = dag.alloc_port_with_shape(int_shape);
        let bad_add = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, ghost_input],
            span.clone(),
        );
        let bind_id = dag.push_bind("x", bad_add, vec![], span);

        let report = analyze_symbolic_cost_dimension(&dag, bind_id);
        let DimensionReport::DimensionFail {
            violations,
            witnesses,
            ..
        } = report
        else {
            panic!("expected DimensionFail when cost lens misses an input port, got {report:?}");
        };

        assert!(
            violations
                .iter()
                .any(|v| matches!(v, Diagnostic::ParseError { .. })),
            "expected ParseError diagnostics from dimension proof failure, got {violations:?}"
        );
        assert!(
            witnesses
                .iter()
                .any(|w| matches!(w, Witness::Violates { .. })),
            "expected at least one Violates witness, got {witnesses:?}"
        );
        assert!(
            witnesses
                .iter()
                .all(|w| !matches!(w, Witness::Inhabits(SymbolicCost::UnknownCost { .. }))),
            "dimension witnesses must not fabricate UnknownCost carriers"
        );
    }

    #[test]
    fn transform_workflow_root_still_backward_reaches_operand_ports() {
        let mut dag = Dag::new();
        let span = SourceSpan::new("transform_root_reach_test", 0, 0);
        let lhs = dag.push_value(LiteralBits::Int(1), span.clone());
        let rhs = dag.push_value(LiteralBits::Int(2), span.clone());
        let add_out = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, rhs],
            span,
        );
        let transform_root = dag
            .port(add_out)
            .produced_by
            .expect("transform output port wired to its node");

        let report = analyze_symbolic_cost_dimension(&dag, transform_root);
        let DimensionReport::DimensionOk { witnesses, .. } = report else {
            panic!("expected DimensionOk for well-wired Int add, got {report:?}");
        };
        assert!(
            witnesses.len() >= 3,
            "expected transform + two literal witnesses, got {}",
            witnesses.len()
        );
    }
}
