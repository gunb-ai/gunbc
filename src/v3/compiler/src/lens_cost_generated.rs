// AUTO-GENERATED from `src/v3/lenses/complexity.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct CostEntry {
    pub port: PortId,
    pub cost: i64,
}
#[derive(Clone, Debug)]
pub enum CostLookup {
    MissingCost,
    FoundCost { _0: i64 },
}
pub fn cost_of(p0: &Dag, p1: &PortId) -> i64 {
    lookup_int(&(compute_costs(p0)), p1)
}
pub fn compute_costs(p0: &Dag) -> Vec<CostEntry> {
    ((p0).nodes())
        .iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, entry_for(&__fold_acc, __fold_item));
            __list
        })
}
pub fn entry_for(p0: &[CostEntry], p1: &Behavior) -> CostEntry {
    match p1 {
        Behavior::Value(v) => CostEntry {
            port: (v).result_port(),
            cost: 0,
        },
        Behavior::Transform(t) => CostEntry {
            port: (t).result_port(),
            cost: (1 + sum_costs(p0, &((t).inputs))),
        },
        Behavior::Branch(b) => CostEntry {
            port: (b).result_port(),
            cost: ((1 + lookup_int(p0, &((b).input))) + max_path_cost(p0, &((b).paths))),
        },
        Behavior::Loop(l) => CostEntry {
            port: (l).result_port(),
            cost: ((1 + lookup_int(p0, &((l).source))) + lookup_int(p0, &((l).init))),
        },
        Behavior::Bind(bind) => CostEntry {
            port: (bind).result_port(),
            cost: lookup_int(p0, &((bind).result_port())),
        },
    }
}
pub fn sum_costs(p0: &[CostEntry], p1: &[PortId]) -> i64 {
    (p1).iter().fold(0, |__fold_acc, __fold_item| {
        (__fold_acc + lookup_int(p0, __fold_item))
    })
}
pub fn max_path_cost(p0: &[CostEntry], p1: &[Path]) -> i64 {
    (p1).iter().fold(0, |__fold_acc, __fold_item| {
        max_int(
            &__fold_acc,
            &(lookup_int(p0, &((__fold_item).result_port()))),
        )
    })
}
pub fn lookup_int(p0: &[CostEntry], p1: &PortId) -> i64 {
    match &(lookup_cost(p0, p1)) {
        CostLookup::MissingCost => 0,
        CostLookup::FoundCost { _0: c } => (*(c)),
    }
}
pub fn lookup_cost(p0: &[CostEntry], p1: &PortId) -> CostLookup {
    match p0 {
        [] => CostLookup::MissingCost,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).port == (*(p1))) {
                CostLookup::FoundCost {
                    _0: (__list_head).cost,
                }
            } else {
                lookup_cost(__list_tail, p1)
            }
        }
    }
}
pub fn max_int(p0: &i64, p1: &i64) -> i64 {
    if ((*(p0)) > (*(p1))) {
        (*(p0))
    } else {
        (*(p1))
    }
}
