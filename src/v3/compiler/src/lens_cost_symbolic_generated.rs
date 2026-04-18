// AUTO-GENERATED from `src/v3/lenses/cost.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct SymbolicCostEntry {
    pub port: PortId,
    pub cost: SymbolicCost,
}
pub fn symbolic_cost_of(p0: &Dag, p1: &PortId) -> SymbolicCost {
    lookup_cost(&(compute_symbolic_costs(p0)), p1)
}
pub fn compute_symbolic_costs(p0: &Dag) -> Vec<SymbolicCostEntry> {
    ((p0).nodes())
        .iter()
        .fold(seed_bind_params((p0).nodes()), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, entry_for(__fold_item));
            __list
        })
}
pub fn seed_bind_params(p0: &[Behavior]) -> Vec<SymbolicCostEntry> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            concat_entries(&(params_of(__list_head)), seed_bind_params(__list_tail))
        }
    }
}
pub fn params_of(p0: &Behavior) -> Vec<SymbolicCostEntry> {
    match p0 {
        Behavior::Value(_) => Vec::new(),
        Behavior::Transform(_) => Vec::new(),
        Behavior::Branch(_) => Vec::new(),
        Behavior::Loop(_) => Vec::new(),
        Behavior::Bind(bind) => param_entries(&((bind).params)),
    }
}
pub fn param_entries(p0: &[PortId]) -> Vec<SymbolicCostEntry> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __list = param_entries(__list_tail);
            __list.insert(
                0,
                SymbolicCostEntry {
                    port: (*(__list_head)),
                    cost: SymbolicCost::ConstantCost { _0: 0 },
                },
            );
            __list
        }
    }
}
pub fn concat_entries(
    p0: &[SymbolicCostEntry],
    p1: Vec<SymbolicCostEntry>,
) -> Vec<SymbolicCostEntry> {
    match p0 {
        [] => p1,
        [__list_head, __list_tail @ ..] => {
            let mut __list = concat_entries(__list_tail, (p1).clone());
            __list.insert(0, (__list_head).clone());
            __list
        }
    }
}
pub fn entry_for(p0: &Behavior) -> SymbolicCostEntry {
    match p0 {
        Behavior::Value(v) => SymbolicCostEntry {
            port: (v).result_port(),
            cost: SymbolicCost::ConstantCost { _0: 0 },
        },
        Behavior::Transform(t) => SymbolicCostEntry {
            port: (t).result_port(),
            cost: SymbolicCost::ConstantCost { _0: 0 },
        },
        Behavior::Branch(b) => SymbolicCostEntry {
            port: (b).result_port(),
            cost: SymbolicCost::ConstantCost { _0: 0 },
        },
        Behavior::Loop(l) => SymbolicCostEntry {
            port: (l).result_port(),
            cost: SymbolicCost::ConstantCost { _0: 0 },
        },
        Behavior::Bind(bind) => SymbolicCostEntry {
            port: (bind).result_port(),
            cost: SymbolicCost::ConstantCost { _0: 0 },
        },
    }
}
pub fn lookup_cost(p0: &[SymbolicCostEntry], p1: &PortId) -> SymbolicCost {
    match p0 {
        [] => SymbolicCost::ConstantCost { _0: 0 },
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).port == (*(p1))) {
                ((__list_head).cost).clone()
            } else {
                lookup_cost(__list_tail, p1)
            }
        }
    }
}
