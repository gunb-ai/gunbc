// AUTO-GENERATED from `src/v3/lenses/cost.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct SymbolicCostEntry {
    pub port: PortId,
    pub cost: SymbolicCostLookup,
}
#[derive(Clone, Debug)]
pub enum SymbolicCostLookup {
    MissingCost,
    FoundCost { _0: SymbolicCost },
}
pub fn symbolic_cost_of(p0: &Dag, p1: &PortId) -> SymbolicCostLookup {
    lookup_cost(&(compute_symbolic_costs(p0)), p1)
}
pub fn compute_symbolic_costs(p0: &Dag) -> Vec<SymbolicCostEntry> {
    ((p0).nodes())
        .iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, entry_for(__fold_item));
            __list
        })
}
pub fn entry_for(p0: &Behavior) -> SymbolicCostEntry {
    match p0 {
        Behavior::Value(v) => SymbolicCostEntry {
            port: (v).result_port(),
            cost: SymbolicCostLookup::FoundCost {
                _0: SymbolicCost::ConstantCost { _0: 0 },
            },
        },
        Behavior::Transform(t) => SymbolicCostEntry {
            port: (t).result_port(),
            cost: SymbolicCostLookup::FoundCost {
                _0: SymbolicCost::ConstantCost { _0: 0 },
            },
        },
        Behavior::Branch(b) => SymbolicCostEntry {
            port: (b).result_port(),
            cost: SymbolicCostLookup::FoundCost {
                _0: SymbolicCost::ConstantCost { _0: 0 },
            },
        },
        Behavior::Loop(l) => SymbolicCostEntry {
            port: (l).result_port(),
            cost: SymbolicCostLookup::FoundCost {
                _0: SymbolicCost::ConstantCost { _0: 0 },
            },
        },
        Behavior::Bind(bind) => SymbolicCostEntry {
            port: (bind).result_port(),
            cost: SymbolicCostLookup::FoundCost {
                _0: SymbolicCost::ConstantCost { _0: 0 },
            },
        },
    }
}
pub fn lookup_cost(p0: &[SymbolicCostEntry], p1: &PortId) -> SymbolicCostLookup {
    match p0 {
        [] => SymbolicCostLookup::MissingCost,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).port == (*(p1))) {
                ((__list_head).cost).clone()
            } else {
                lookup_cost(__list_tail, p1)
            }
        }
    }
}
