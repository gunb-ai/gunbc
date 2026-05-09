// AUTO-GENERATED from `src/v3/lenses/cost.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum CostBasisKind {
    PerWrite,
    PerCall,
    PeakMemory,
}
#[derive(Clone, Debug)]
pub struct CostBasisDeclaration {
    pub subject: DeclarationId,
    pub kind: CostBasisKind,
    pub cost: SymbolicCost,
    pub span: SourceSpan,
}
#[derive(Clone, Debug)]
pub struct SymbolicCostEntry {
    pub port: PortId,
    pub cost: Lookup<SymbolicCost>,
}
pub fn symbolic_cost_of(p0: &Dag, p1: &PortId) -> Lookup<SymbolicCost> {
    lookup_cost(&(compute_symbolic_costs(p0)), p1)
}
pub fn method_contract_cost_shape(p0: &MethodContract) -> Option<CostShape> {
    ((p0).cost_shape).clone()
}
pub fn compute_symbolic_costs(p0: &Dag) -> Vec<SymbolicCostEntry> {
    ((p0).nodes())
        .iter()
        .fold(seed_bind_params((p0).nodes()), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, entry_for(p0, &__fold_acc, __fold_item));
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
                    cost: Lookup::Hit(SymbolicCost::ConstantCost { _0: 0 }),
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
pub fn entry_for(p0: &Dag, p1: &[SymbolicCostEntry], p2: &Behavior) -> SymbolicCostEntry {
    match p2 {
        Behavior::Value(v) => SymbolicCostEntry {
            port: (v).result_port(),
            cost: Lookup::Hit(SymbolicCost::ConstantCost { _0: 0 }),
        },
        Behavior::Transform(t) => SymbolicCostEntry {
            port: (t).result_port(),
            cost: transform_cost(p1, &((t).inputs)),
        },
        Behavior::Branch(b) => SymbolicCostEntry {
            port: (b).result_port(),
            cost: branch_cost(p1, &((b).input), &((b).paths)),
        },
        Behavior::Loop(l) => SymbolicCostEntry {
            port: (l).result_port(),
            cost: loop_cost(p0, p1, l),
        },
        Behavior::Bind(bind) => SymbolicCostEntry {
            port: (bind).result_port(),
            cost: lookup_cost(p1, &((bind).result_port())),
        },
    }
}
pub fn loop_cost(p0: &Dag, p1: &[SymbolicCostEntry], p2: &LoopNode) -> Lookup<SymbolicCost> {
    combine_iterate(
        &(linear_at(&((p2).source))),
        &(body_cost(p0, p1, &((p2).body))),
    )
}
pub fn body_cost(p0: &Dag, p1: &[SymbolicCostEntry], p2: &NodeId) -> Lookup<SymbolicCost> {
    match &((p0).node_opt(p2).cloned()) {
        None => Lookup::Miss,
        Some(body_behavior) => lookup_cost(p1, &(behavior_result_port(body_behavior))),
    }
}
pub fn behavior_result_port(p0: &Behavior) -> PortId {
    match p0 {
        Behavior::Value(v) => (v).result_port(),
        Behavior::Transform(t) => (t).result_port(),
        Behavior::Branch(br) => (br).result_port(),
        Behavior::Loop(lp) => (lp).result_port(),
        Behavior::Bind(bind) => (bind).result_port(),
    }
}
pub fn linear_at(p0: &PortId) -> Lookup<SymbolicCost> {
    Lookup::Hit(SymbolicCost::LinearCost {
        _0: SizeVariable {
            source_port: *p0,
            display_name: None,
        },
    })
}
pub fn combine_iterate(
    p0: &Lookup<SymbolicCost>,
    p1: &Lookup<SymbolicCost>,
) -> Lookup<SymbolicCost> {
    match p0 {
        Lookup::Miss => Lookup::Miss,
        Lookup::Hit(b) => match p1 {
            Lookup::Miss => Lookup::Miss,
            Lookup::Hit(y) => Lookup::Hit(iterate((b).clone(), (y).clone())),
        },
    }
}
pub fn branch_cost(p0: &[SymbolicCostEntry], p1: &PortId, p2: &[Path]) -> Lookup<SymbolicCost> {
    combine_sequential(
        &(Lookup::Hit(SymbolicCost::ConstantCost { _0: 1 })),
        &(combine_sequential(&(lookup_cost(p0, p1)), &(max_of_paths(p0, p2)))),
    )
}
pub fn max_of_paths(p0: &[SymbolicCostEntry], p1: &[Path]) -> Lookup<SymbolicCost> {
    (p1).iter().fold(
        Lookup::Hit(SymbolicCost::ConstantCost { _0: 0 }),
        |__fold_acc, __fold_item| {
            combine_max(
                &__fold_acc,
                &(lookup_cost(p0, &((__fold_item).result_port()))),
            )
        },
    )
}
pub fn combine_max(p0: &Lookup<SymbolicCost>, p1: &Lookup<SymbolicCost>) -> Lookup<SymbolicCost> {
    match p0 {
        Lookup::Miss => Lookup::Miss,
        Lookup::Hit(ax) => match p1 {
            Lookup::Miss => Lookup::Miss,
            Lookup::Hit(bx) => Lookup::Hit(dominant((ax).clone(), (bx).clone())),
        },
    }
}
pub fn dominant(p0: SymbolicCost, p1: SymbolicCost) -> SymbolicCost {
    max_path(
        &({
            let mut __list = {
                let mut __list = Vec::new();
                __list.insert(0, (p1).clone());
                __list
            };
            __list.insert(0, (p0).clone());
            __list
        }),
    )
}
pub fn transform_cost(p0: &[SymbolicCostEntry], p1: &[PortId]) -> Lookup<SymbolicCost> {
    combine_sequential(
        &(Lookup::Hit(SymbolicCost::ConstantCost { _0: 1 })),
        &(sum_costs(p0, p1)),
    )
}
pub fn sum_costs(p0: &[SymbolicCostEntry], p1: &[PortId]) -> Lookup<SymbolicCost> {
    (p1).iter().fold(
        Lookup::Hit(SymbolicCost::ConstantCost { _0: 0 }),
        |__fold_acc, __fold_item| combine_sequential(&__fold_acc, &(lookup_cost(p0, __fold_item))),
    )
}
pub fn combine_sequential(
    p0: &Lookup<SymbolicCost>,
    p1: &Lookup<SymbolicCost>,
) -> Lookup<SymbolicCost> {
    match p0 {
        Lookup::Miss => Lookup::Miss,
        Lookup::Hit(ax) => match p1 {
            Lookup::Miss => Lookup::Miss,
            Lookup::Hit(bx) => Lookup::Hit(sequential((ax).clone(), (bx).clone())),
        },
    }
}
pub fn lookup_cost(p0: &[SymbolicCostEntry], p1: &PortId) -> Lookup<SymbolicCost> {
    match p0 {
        [] => Lookup::Miss,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).port == (*(p1))) {
                ((__list_head).cost).clone()
            } else {
                lookup_cost(__list_tail, p1)
            }
        }
    }
}
