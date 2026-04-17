// AUTO-GENERATED from `src/v3/lenses/provenance.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum Origin {
    NoProducer,
    MissingPort,
    MissingBehavior,
    Source { _0: NodeId },
    Computed { _0: NodeId },
    Selected { _0: NodeId },
    Accumulated { _0: NodeId },
}
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
pub fn origin_of(p0: &Dag, p1: &PortId) -> Origin {
    match &(find_port(&((p0).ports()), p1)) {
        PortLookup::MissingPort => Origin::MissingPort,
        PortLookup::FoundPort { _0: port } => match &((port).produced_by) {
            None => Origin::NoProducer,
            Some(producer_id) => match &(find_behavior((p0).nodes(), producer_id)) {
                BehaviorLookup::MissingBehavior => Origin::MissingBehavior,
                BehaviorLookup::FoundBehavior { _0: behavior } => origin_for_behavior(behavior),
            },
        },
    }
}
pub fn origin_for_behavior(p0: &Behavior) -> Origin {
    match p0 {
        Behavior::Value(v) => Origin::Source { _0: (v).id },
        Behavior::Transform(t) => Origin::Computed { _0: (t).id },
        Behavior::Branch(b) => Origin::Selected { _0: (b).id },
        Behavior::Loop(l) => Origin::Accumulated { _0: (l).id },
        Behavior::Bind(bind) => Origin::Source { _0: (bind).id },
    }
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
