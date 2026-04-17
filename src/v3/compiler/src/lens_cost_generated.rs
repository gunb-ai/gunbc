// AUTO-GENERATED from `src/v3/lenses/complexity.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct CostEntry {
    pub port: PortId,
    pub cost: CostLookup,
}
#[derive(Clone, Debug)]
pub enum CostLookup {
    MissingCost,
    FoundCost { _0: i64 },
}
pub fn cost_of(p0: &Dag, p1: &PortId) -> CostLookup {
    lookup_cost(&(compute_costs(p0)), p1)
}
pub fn compute_costs(p0: &Dag) -> Vec<CostEntry> {
    ((p0).nodes())
        .iter()
        .fold(seed_bind_params((p0).nodes()), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, entry_for(&__fold_acc, __fold_item));
            __list
        })
}
pub fn seed_bind_params(p0: &[Behavior]) -> Vec<CostEntry> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __left = params_of(__list_head);
            __left.extend(seed_bind_params(__list_tail));
            __left
        }
    }
}
pub fn params_of(p0: &Behavior) -> Vec<CostEntry> {
    match p0 {
        Behavior::Value(_) => Vec::new(),
        Behavior::Transform(_) => Vec::new(),
        Behavior::Branch(_) => Vec::new(),
        Behavior::Loop(_) => Vec::new(),
        Behavior::Bind(bind) => param_entries(&((bind).params)),
    }
}
pub fn param_entries(p0: &[PortId]) -> Vec<CostEntry> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __list = param_entries(__list_tail);
            __list.insert(
                0,
                CostEntry {
                    port: (*(__list_head)),
                    cost: CostLookup::FoundCost { _0: 0 },
                },
            );
            __list
        }
    }
}
pub fn entry_for(p0: &[CostEntry], p1: &Behavior) -> CostEntry {
    match p1 {
        Behavior::Value(v) => CostEntry {
            port: (v).result_port(),
            cost: CostLookup::FoundCost { _0: 0 },
        },
        Behavior::Transform(t) => CostEntry {
            port: (t).result_port(),
            cost: add_one(&(sum_costs(p0, &((t).inputs)))),
        },
        Behavior::Branch(b) => CostEntry {
            port: (b).result_port(),
            cost: add_one(
                &(add_cost(
                    &(lookup_cost(p0, &((b).input))),
                    &(max_path_cost(p0, &((b).paths))),
                )),
            ),
        },
        Behavior::Loop(l) => CostEntry {
            port: (l).result_port(),
            cost: add_one(
                &(add_cost(
                    &(lookup_cost(p0, &((l).source))),
                    &(lookup_cost(p0, &((l).init))),
                )),
            ),
        },
        Behavior::Bind(bind) => CostEntry {
            port: (bind).result_port(),
            cost: lookup_cost(p0, &((bind).result_port())),
        },
    }
}
pub fn sum_costs(p0: &[CostEntry], p1: &[PortId]) -> CostLookup {
    (p1).iter().fold(
        CostLookup::FoundCost { _0: 0 },
        |__fold_acc, __fold_item| add_cost(&__fold_acc, &(lookup_cost(p0, __fold_item))),
    )
}
pub fn max_path_cost(p0: &[CostEntry], p1: &[Path]) -> CostLookup {
    (p1).iter().fold(
        CostLookup::FoundCost { _0: 0 },
        |__fold_acc, __fold_item| {
            max_cost(
                &__fold_acc,
                &(lookup_cost(p0, &((__fold_item).result_port()))),
            )
        },
    )
}
pub fn lookup_cost(p0: &[CostEntry], p1: &PortId) -> CostLookup {
    match p0 {
        [] => CostLookup::MissingCost,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).port == (*(p1))) {
                ((__list_head).cost).clone()
            } else {
                lookup_cost(__list_tail, p1)
            }
        }
    }
}
pub fn add_one(p0: &CostLookup) -> CostLookup {
    match p0 {
        CostLookup::MissingCost => CostLookup::MissingCost,
        CostLookup::FoundCost { _0: n } => CostLookup::FoundCost { _0: (1 + (*(n))) },
    }
}
pub fn add_cost(p0: &CostLookup, p1: &CostLookup) -> CostLookup {
    match p0 {
        CostLookup::MissingCost => CostLookup::MissingCost,
        CostLookup::FoundCost { _0: x } => match p1 {
            CostLookup::MissingCost => CostLookup::MissingCost,
            CostLookup::FoundCost { _0: y } => CostLookup::FoundCost {
                _0: ((*(x)) + (*(y))),
            },
        },
    }
}
pub fn max_cost(p0: &CostLookup, p1: &CostLookup) -> CostLookup {
    match p0 {
        CostLookup::MissingCost => CostLookup::MissingCost,
        CostLookup::FoundCost { _0: x } => match p1 {
            CostLookup::MissingCost => CostLookup::MissingCost,
            CostLookup::FoundCost { _0: y } => CostLookup::FoundCost {
                _0: if ((*(x)) > (*(y))) { (*(x)) } else { (*(y)) },
            },
        },
    }
}
