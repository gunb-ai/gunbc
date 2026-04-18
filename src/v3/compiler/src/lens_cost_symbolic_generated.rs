// AUTO-GENERATED from `src/v3/lenses/cost.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct SymbolicCostEntry {
    pub port: PortId,
    pub cost: SymbolicCost,
}
pub fn symbolic_cost_of(p0: &Dag, p1: &PortId) -> SymbolicCost {
    SymbolicCost::ConstantCost { _0: 0 }
}
