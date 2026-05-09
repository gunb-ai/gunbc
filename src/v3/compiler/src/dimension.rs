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
///
/// 🟢 TERMINAL (aggregate from DB-3 dimension evaluation; see `dimensions.dag`).
/// Pass/fail partition: success carries `composed`; failure carries `violations`
/// and must not fabricate a carrier when witnesses violate or root composition
/// misses (R2 fail-closed).
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

/// PR-E E7 symbolic-cost-only first executable slice: public
/// complexity-analysis entrypoint per
/// `docs/briefs/r3-evaluator-e7-symbolic-cost-only-follow-on-readiness.md`.
///
/// **Single authority.** Thin wrapper that delegates to
/// [`analyze_symbolic_cost_dimension`] — the lens-spine path that walks
/// reachable behaviors from `workflow_root` via
/// `workflow_reachable_behavior_ids` and consumes
/// [`crate::lens_cost_symbolic::symbolic_cost_of`] for each behavior's
/// result port. The wrapper exists so the E7 public surface is named
/// the way the dispatch brief locks it (`analyze_complexity` /
/// `analyze_tenant_flow` / `analyze_ifc`) without introducing a
/// parallel analyzer.
///
/// **Lens-spine, not body-evaluator-driven.** This entrypoint does
/// **not** depend on `eval_node` / `evaluate_body` at all — the
/// symbolic-cost lens (`lens_cost_symbolic_generated.rs`) walks the
/// program DAG structurally and produces `SymbolicCostLookup::Hit(cost)`
/// for every reachable behavior including `Loop`. Even now that
/// `eval_node` dispatches `Behavior::Loop` (E5 landed), this wrapper
/// remains the single-authority complexity entrypoint until
/// `analyze_with_evaluator` (the body-evaluator-driven form) lands;
/// at that point this lens-spine wrapper either dissolves or stays as
/// the not-evaluator-driven path for cost specifically.
///
/// **Diagnostics.** `DimensionFail.violations` carries typed
/// [`Diagnostic`] entries; tests must assert by typed pattern match,
/// never by parsing `Witness::Violates.reason` strings.
pub fn analyze_complexity(dag: &Dag, workflow_root: NodeId) -> DimensionReport<SymbolicCost> {
    analyze_symbolic_cost_dimension(dag, workflow_root)
}

#[cfg(test)]
mod analyze_complexity_tests {
    use super::*;
    use crate::dag::{literal_bits_int, Dag, LiteralBits, TransformTarget};
    use crate::operators::{ArithmeticOp, OperatorKind};

    fn span() -> SourceSpan {
        SourceSpan::new("analyze_complexity_test", 0, 0)
    }

    fn int_add_dag() -> (Dag, NodeId) {
        let mut dag = Dag::new();
        let lhs = dag.push_value(literal_bits_int(1), span());
        let rhs = dag.push_value(literal_bits_int(2), span());
        let add_out = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, rhs],
            span(),
        );
        let root = dag.push_bind("sum", add_out, vec![], span());
        (dag, root)
    }

    // E7 §test 1 — single-authority: analyze_complexity must produce
    // the same DimensionReport as the live analyze_symbolic_cost_dimension
    // for the same (dag, workflow_root). Pins that the wrapper does not
    // introduce a parallel analyzer.
    #[test]
    fn analyze_complexity_matches_analyze_symbolic_cost_dimension_for_known_workflow() {
        let (dag, root) = int_add_dag();

        let wrapped = analyze_complexity(&dag, root);
        let direct = analyze_symbolic_cost_dimension(&dag, root);

        match (&wrapped, &direct) {
            (
                DimensionReport::DimensionOk {
                    dimension_name: w_name,
                    composed: w_composed,
                    witnesses: w_witnesses,
                },
                DimensionReport::DimensionOk {
                    dimension_name: d_name,
                    composed: d_composed,
                    witnesses: d_witnesses,
                },
            ) => {
                assert_eq!(w_name, d_name);
                assert_eq!(w_composed, d_composed);
                assert_eq!(w_witnesses.len(), d_witnesses.len());
            }
            other => {
                panic!("expected both branches DimensionOk with matching content, got {other:?}")
            }
        }
    }

    // E7 §test 1 (returns_ok shape check) — DimensionOk arm carries the
    // expected dimension_name and a SymbolicCost composed value.
    #[test]
    fn analyze_complexity_returns_ok_for_bounded_int_add_workflow() {
        let (dag, root) = int_add_dag();

        let report = analyze_complexity(&dag, root);

        let DimensionReport::DimensionOk {
            dimension_name,
            composed: _,
            witnesses,
        } = report
        else {
            panic!("expected DimensionOk for bounded int-add, got {report:?}");
        };
        assert_eq!(dimension_name, "symbolic_cost");
        assert!(
            witnesses
                .iter()
                .all(|w| !matches!(w, Witness::Violates { .. })),
            "no Violates witness expected on the happy path",
        );
    }

    // E7 §test 2 — fail-closed on missing cost: program with a
    // ghost-input transform produces DimensionFail with at least one
    // typed Diagnostic::ParseError in violations. Asserted by typed
    // pattern match on the Diagnostic enum, never by string parsing.
    #[test]
    fn analyze_complexity_fails_closed_with_typed_diagnostic_on_missing_cost() {
        let mut dag = Dag::new();
        let int_shape = dag.int_shape().expect("bootstrap Int");
        let lhs = dag.push_value(literal_bits_int(1), span());
        let ghost_input = dag.alloc_port_with_shape(int_shape);
        let bad_add = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, ghost_input],
            span(),
        );
        let root = dag.push_bind("x", bad_add, vec![], span());

        let report = analyze_complexity(&dag, root);

        let DimensionReport::DimensionFail {
            dimension_name,
            violations,
            witnesses: _,
        } = report
        else {
            panic!("expected DimensionFail when the cost lens misses a port, got {report:?}");
        };
        assert_eq!(dimension_name, "symbolic_cost");
        assert!(
            !violations.is_empty(),
            "DimensionFail must carry at least one typed diagnostic",
        );
        assert!(
            violations
                .iter()
                .all(|d| matches!(d, Diagnostic::ParseError { .. })),
            "every violation must be a typed Diagnostic enum variant, got {violations:?}",
        );
    }

    // E7 §test 3 — DimensionFail does not fabricate a `composed`
    // carrier. Pattern-matching the `DimensionFail` arm is enough to
    // assert this: the field is unreachable on that arm by
    // construction. The wrapper preserves the partition. Reuses the
    // ghost-input transform pattern from the existing
    // `missing_symbolic_cost_surfaces_as_dimension_fail_with_violates_witnesses`
    // test so the workflow is well-formed enough for the lens walk.
    #[test]
    fn analyze_complexity_fail_arm_has_no_composed_field() {
        let mut dag = Dag::new();
        let int_shape = dag.int_shape().expect("bootstrap Int");
        let lhs = dag.push_value(literal_bits_int(1), span());
        let ghost_input = dag.alloc_port_with_shape(int_shape);
        let bad_add = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, ghost_input],
            span(),
        );
        let root = dag.push_bind("y", bad_add, vec![], span());

        let report = analyze_complexity(&dag, root);

        match report {
            DimensionReport::DimensionFail { .. } => {
                // Exhaustive match on this arm: the only fields are
                // dimension_name / violations / witnesses. There is no
                // `composed` field to fabricate. Pattern coverage IS
                // the assertion.
            }
            DimensionReport::DimensionOk { .. } => {
                panic!("expected DimensionFail on a ghost-input workflow, not DimensionOk")
            }
        }
    }

    // E7 §test 6 — typed-diagnostic discipline: every entry in
    // DimensionFail.violations must be inspectable as a typed
    // Diagnostic enum variant. No to_string / format / regex on the
    // human-facing message field beyond non-empty checks. (Test #5
    // from the brief is a discipline rule enforced by reviewer
    // convention, not a runtime assertion.)
    #[test]
    fn analyze_complexity_violation_entries_are_typed_diagnostic_enums() {
        let mut dag = Dag::new();
        let int_shape = dag.int_shape().expect("bootstrap Int");
        let lhs = dag.push_value(literal_bits_int(1), span());
        let ghost_input = dag.alloc_port_with_shape(int_shape);
        let bad_add = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, ghost_input],
            span(),
        );
        let root = dag.push_bind("z", bad_add, vec![], span());

        let report = analyze_complexity(&dag, root);
        let DimensionReport::DimensionFail { violations, .. } = report else {
            panic!("expected DimensionFail for ghost-port workflow");
        };

        for diagnostic in &violations {
            // Exhaustive enum match: every variant of Diagnostic is a
            // structural type the test can pattern-match on without
            // parsing message content. Non-`ParseError` variants are
            // also acceptable typed inhabitants.
            let typed = match diagnostic {
                Diagnostic::ParseError { message, .. } => !message.is_empty(),
                _ => true,
            };
            assert!(
                typed,
                "Diagnostic must carry a non-empty human-facing message; got {diagnostic:?}",
            );
        }
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
        let lhs = dag.push_value(literal_bits_int(1), span.clone());
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
        let lhs = dag.push_value(literal_bits_int(1), span.clone());
        let rhs = dag.push_value(literal_bits_int(2), span.clone());
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
