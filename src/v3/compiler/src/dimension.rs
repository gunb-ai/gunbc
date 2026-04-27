//! DB-3 dimension analysis — Lane 2 Stage 2f.
//!
//! Type authority: `src/v3/std/dimensions.dag`, spine order:
//! [`behavior_spine_in_node_order`] (`src/v3/std/workflows.dag`).
//!
//! Symbolic-cost dimension is the first migrated lens: it reuses
//! [`crate::lens_cost_symbolic::symbolic_cost_of`] for per-Behavior
//! witnesses and reports the composed carrier at `workflow_root`'s result
//! port (same query the cost lens answers today).

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

/// Symbolic-cost dimension analysis — DB-3 dispatch over the cost lens algebra.
///
/// On success ([`DimensionReport::DimensionOk`]), `composed` is the asymptotic
/// bound at `workflow_root`'s result port. Witnesses walk every [`Behavior`]
/// in node order (see [`behavior_spine_in_node_order`]). Failure is only
/// [`DimensionReport::DimensionFail`]: there is no fabricated carrier.
pub fn analyze_symbolic_cost_dimension(
    d: &Dag,
    workflow_root: NodeId,
) -> DimensionReport<SymbolicCost> {
    const DIMENSION_NAME: &str = "symbolic_cost";
    let mut witnesses = Vec::new();
    for behavior in d.nodes() {
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

    if !witness_failure && matches!(root_lookup, SymbolicCostLookup::Hit(_)) {
        let SymbolicCostLookup::Hit(composed) = root_lookup else {
            unreachable!("root_lookup checked Hit above");
        };
        return DimensionReport::DimensionOk {
            dimension_name: DIMENSION_NAME.to_string(),
            composed,
            witnesses,
        };
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
            violations.iter().any(|v| {
                matches!(
                    v,
                    Diagnostic::ParseError { message, .. }
                        if message.contains("symbolic_cost dimension:")
                )
            }),
            "expected structured dimension diagnostics, got {violations:?}"
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
}
