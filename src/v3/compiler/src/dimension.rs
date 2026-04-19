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

/// Evidence partition — mirrors `Witness<Carrier>` in `std/dimensions.dag`.
#[derive(Debug, Clone)]
pub enum Witness<C> {
    Inhabits(C),
    Violates { reason: String, at: Behavior },
}

/// Report carrier — mirrors `DimensionReport<Carrier>` in `std/dimensions.dag`.
#[derive(Debug, Clone)]
pub struct DimensionReport<C> {
    pub dimension_name: String,
    pub composed: C,
    pub violations: Vec<crate::diagnostics::Diagnostic>,
    pub witnesses: Vec<Witness<C>>,
}

/// Same `Dag.nodes` order as `lenses/cost.dag::compute_symbolic_costs` /
/// `std/workflows.dag::behavior_spine`.
pub fn behavior_spine_in_node_order(d: &Dag) -> &[Behavior] {
    d.nodes()
}

/// Symbolic-cost dimension analysis — DB-3 dispatch over the cost lens algebra.
///
/// `composed` is the asymptotic bound at `workflow_root`'s result port. Witnesses
/// walk every [`Behavior`] in node order (see [`behavior_spine_in_node_order`]).
pub fn analyze_symbolic_cost_dimension(d: &Dag, workflow_root: NodeId) -> DimensionReport<SymbolicCost> {
    const DIMENSION_NAME: &str = "symbolic_cost";
    let mut witnesses = Vec::new();
    for behavior in d.nodes() {
        let port = behavior_result_port(behavior);
        match symbolic_cost_of(d, &port) {
            SymbolicCostLookup::MissingCost => witnesses.push(Witness::Violates {
                reason: "missing symbolic cost for behavior result port".into(),
                at: behavior.clone(),
            }),
            SymbolicCostLookup::FoundCost { _0: cost } => witnesses.push(Witness::Inhabits(cost)),
        }
    }

    let root = d.node(workflow_root);
    let composed = match symbolic_cost_of(d, &behavior_result_port(root)) {
        SymbolicCostLookup::FoundCost { _0: cost } => cost,
        SymbolicCostLookup::MissingCost => SymbolicCost::UnknownCost(
            "missing symbolic cost for workflow root result port".into(),
        ),
    };

    DimensionReport {
        dimension_name: DIMENSION_NAME.to_string(),
        composed,
        violations: Vec::new(),
        witnesses,
    }
}
