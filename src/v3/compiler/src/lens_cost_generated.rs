// AUTO-GENERATED from `src/v3/lenses/complexity.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum PortLookup {
    MissingPort,
    FoundPort { _0: Port },
}
#[derive(Clone, Debug)]
pub enum BehaviorLookup {
    MissingBehavior,
    FoundBehavior { _0: Behavior },
}
#[derive(Clone, Debug)]
pub enum CostMode {
    Port,
    SumPorts,
    MaxPaths,
}
pub fn cost_of(p0: &Dag, p1: &PortId) -> i64 {
    eval_cost(
        &(((p0).ports())
            .iter()
            .fold(0, |__fold_acc, __fold_item| (__fold_acc + 1))),
        p0,
        &(CostMode::Port),
        p1,
        &[],
        &[],
    )
}
pub fn eval_cost(
    p0: &i64,
    p1: &Dag,
    p2: &CostMode,
    p3: &PortId,
    p4: &[PortId],
    p5: &[Path],
) -> i64 {
    if ((*(p0)) == 0) {
        0
    } else {
        match p2 {
            CostMode::Port => match &(find_port(&((p1).ports()), p3)) {
                PortLookup::MissingPort => 0,
                PortLookup::FoundPort { _0: port } => match &((port).produced_by) {
                    None => 0,
                    Some(node_id) => match &(find_behavior((p1).nodes(), node_id)) {
                        BehaviorLookup::MissingBehavior => 0,
                        BehaviorLookup::FoundBehavior { _0: behavior } => match behavior {
                            Behavior::Value(v) => 0,
                            Behavior::Transform(t) => {
                                (1 + eval_cost(
                                    &((*(p0)) - 1),
                                    p1,
                                    &(CostMode::SumPorts),
                                    p3,
                                    &((t).inputs),
                                    &[],
                                ))
                            }
                            Behavior::Branch(branch) => {
                                ((1 + eval_cost(
                                    &((*(p0)) - 1),
                                    p1,
                                    &(CostMode::Port),
                                    &((branch).input),
                                    &[],
                                    &[],
                                )) + eval_cost(
                                    &((*(p0)) - 1),
                                    p1,
                                    &(CostMode::MaxPaths),
                                    p3,
                                    &[],
                                    &((branch).paths),
                                ))
                            }
                            Behavior::Loop(loop_node) => {
                                ((1 + eval_cost(
                                    &((*(p0)) - 1),
                                    p1,
                                    &(CostMode::Port),
                                    &((loop_node).source),
                                    &[],
                                    &[],
                                )) + eval_cost(
                                    &((*(p0)) - 1),
                                    p1,
                                    &(CostMode::Port),
                                    &((loop_node).init),
                                    &[],
                                    &[],
                                ))
                            }
                            Behavior::Bind(bind) => eval_cost(
                                &((*(p0)) - 1),
                                p1,
                                &(CostMode::Port),
                                &((bind).result_port()),
                                &[],
                                &[],
                            ),
                        },
                    },
                },
            },
            CostMode::SumPorts => match p4 {
                [] => 0,
                [__list_head, __list_tail @ ..] => sum_int(
                    &(eval_cost(&((*(p0)) - 1), p1, &(CostMode::Port), __list_head, &[], &[])),
                    &(eval_cost(
                        &((*(p0)) - 1),
                        p1,
                        &(CostMode::SumPorts),
                        p3,
                        __list_tail,
                        &[],
                    )),
                ),
            },
            CostMode::MaxPaths => match p5 {
                [] => 0,
                [__list_head, __list_tail @ ..] => max_int(
                    &(eval_cost(
                        &((*(p0)) - 1),
                        p1,
                        &(CostMode::Port),
                        &((__list_head).result_port()),
                        &[],
                        &[],
                    )),
                    &(eval_cost(
                        &((*(p0)) - 1),
                        p1,
                        &(CostMode::MaxPaths),
                        p3,
                        &[],
                        __list_tail,
                    )),
                ),
            },
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
pub fn sum_int(p0: &i64, p1: &i64) -> i64 {
    ((*(p0)) + (*(p1)))
}
pub fn find_port(p0: &[Port], p1: &PortId) -> PortLookup {
    match p0 {
        [] => PortLookup::MissingPort,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).id() == (*(p1))) {
                PortLookup::FoundPort {
                    _0: (__list_head).clone(),
                }
            } else {
                find_port(__list_tail, p1)
            }
        }
    }
}
pub fn find_behavior(p0: &[Behavior], p1: &NodeId) -> BehaviorLookup {
    match p0 {
        [] => BehaviorLookup::MissingBehavior,
        [__list_head, __list_tail @ ..] => {
            if (behavior_id(__list_head) == (*(p1))) {
                BehaviorLookup::FoundBehavior {
                    _0: (__list_head).clone(),
                }
            } else {
                find_behavior(__list_tail, p1)
            }
        }
    }
}
pub fn behavior_id(p0: &Behavior) -> NodeId {
    match p0 {
        Behavior::Value(v) => (v).id,
        Behavior::Transform(t) => (t).id,
        Behavior::Branch(b) => (b).id,
        Behavior::Loop(l) => (l).id,
        Behavior::Bind(bind) => (bind).id,
    }
}
