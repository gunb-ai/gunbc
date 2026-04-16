// AUTO-GENERATED from `src/v3/lenses/unused_parameters.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct UnusedParameter {
    pub function: NodeId,
    pub parameter: PortId,
    pub parameter_index: i64,
    pub function_span: SourceSpan,
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
#[derive(Clone, Debug)]
pub enum ResultPortLookup {
    MissingResultPort,
    FoundResultPort { _0: PortId },
}
pub fn check(p0: Dag) -> Vec<UnusedParameter> {
    ((p0).clone().nodes_owned())
        .clone()
        .into_iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            let mut __left = (__fold_acc).clone();
            __left.extend(check_behavior(
                ((p0).clone()).clone(),
                (__fold_item).clone(),
            ));
            __left
        })
}
pub fn check_behavior(p0: Dag, p1: Behavior) -> Vec<UnusedParameter> {
    match (p1).clone() {
        Behavior::Value(v) => Vec::new(),
        Behavior::Transform(t) => Vec::new(),
        Behavior::Branch(b) => Vec::new(),
        Behavior::Loop(l) => Vec::new(),
        Behavior::Bind(bind) => {
            if ((bind).clone().params).is_empty() {
                Vec::new()
            } else {
                check_bind((p0).clone(), (bind).clone())
            }
        }
    }
}
pub fn check_bind(p0: Dag, p1: BindNode) -> Vec<UnusedParameter> {
    collect_unused_params(
        (p1).clone().params,
        (p1).clone(),
        referenced_ports((p0).clone(), (p1).clone().result_port()),
        0,
    )
}
pub fn collect_unused_params(
    p0: Vec<PortId>,
    p1: BindNode,
    p2: Vec<PortId>,
    p3: i64,
) -> Vec<UnusedParameter> {
    match (p0).clone().as_slice() {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            if ((p2).clone()).contains(&__list_head.clone()) {
                collect_unused_params(
                    __list_tail.to_vec(),
                    (p1).clone(),
                    (p2).clone(),
                    ((p3).clone() + 1),
                )
            } else {
                {
                    let mut __list = collect_unused_params(
                        __list_tail.to_vec(),
                        (p1).clone(),
                        (p2).clone(),
                        ((p3).clone() + 1),
                    );
                    __list.insert(
                        0,
                        UnusedParameter {
                            function: (p1).clone().id,
                            parameter: __list_head.clone(),
                            parameter_index: (p3).clone(),
                            function_span: (p1).clone().span,
                        },
                    );
                    __list
                }
            }
        }
    }
}
pub fn referenced_ports(p0: Dag, p1: PortId) -> Vec<PortId> {
    walk_steps(
        ((p0).clone().ports()).len() as i64,
        (p0).clone(),
        vec![(p1).clone()],
        Vec::new(),
    )
}
pub fn walk_steps(p0: i64, p1: Dag, p2: Vec<PortId>, p3: Vec<PortId>) -> Vec<PortId> {
    if ((p0).clone() == 0) {
        (p3).clone()
    } else {
        if ((p2).clone()).is_empty() {
            (p3).clone()
        } else {
            walk_steps(
                ((p0).clone() - 1),
                (p1).clone(),
                expand_frontier_list((p2).clone(), (p1).clone(), (p3).clone()),
                expand_referenced_list((p2).clone(), (p3).clone()),
            )
        }
    }
}
pub fn expand_frontier_list(p0: Vec<PortId>, p1: Dag, p2: Vec<PortId>) -> Vec<PortId> {
    match (p0).clone().as_slice() {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            if ((p2).clone()).contains(&__list_head.clone()) {
                expand_frontier_list(__list_tail.to_vec(), (p1).clone(), (p2).clone())
            } else {
                {
                    let mut __left = inputs_for_port((p1).clone(), __list_head.clone());
                    __left.extend(expand_frontier_list(
                        __list_tail.to_vec(),
                        (p1).clone(),
                        (p2).clone(),
                    ));
                    __left
                }
            }
        }
    }
}
pub fn expand_referenced_list(p0: Vec<PortId>, p1: Vec<PortId>) -> Vec<PortId> {
    ((p0).clone())
        .clone()
        .into_iter()
        .fold((p1).clone(), |__fold_acc, __fold_item| {
            if ((__fold_acc).clone()).contains(&(__fold_item).clone()) {
                (__fold_acc).clone()
            } else {
                {
                    let mut __list = (__fold_acc).clone();
                    __list.insert(0, (__fold_item).clone());
                    __list
                }
            }
        })
}
pub fn inputs_for_port(p0: Dag, p1: PortId) -> Vec<PortId> {
    match find_producer((p0).clone().nodes_owned(), (p1).clone()) {
        BehaviorLookup::MissingBehavior => Vec::new(),
        BehaviorLookup::FoundBehavior { _0: behavior } => {
            inputs_for_behavior((p0).clone(), (behavior).clone())
        }
    }
}
pub fn inputs_for_node(p0: Dag, p1: NodeId) -> Vec<PortId> {
    match find_behavior((p0).clone().nodes_owned(), (p1).clone()) {
        BehaviorLookup::MissingBehavior => Vec::new(),
        BehaviorLookup::FoundBehavior { _0: behavior } => {
            inputs_for_behavior((p0).clone(), (behavior).clone())
        }
    }
}
pub fn inputs_for_behavior(p0: Dag, p1: Behavior) -> Vec<PortId> {
    match (p1).clone() {
        Behavior::Value(v) => Vec::new(),
        Behavior::Transform(t) => (t).clone().inputs,
        Behavior::Branch(branch) => {
            let mut __list = branch_path_outputs((branch).clone().paths);
            __list.insert(0, (branch).clone().input);
            __list
        }
        Behavior::Loop(loop_node) => loop_inputs((p0).clone(), (loop_node).clone()),
        Behavior::Bind(bind) => vec![(bind).clone().result_port()],
    }
}
pub fn loop_inputs(p0: Dag, p1: LoopNode) -> Vec<PortId> {
    {
        let mut __left = {
            let mut __list = {
                let mut __list = vec![(p1).clone().bound.count];
                __list.insert(0, (p1).clone().init);
                __list
            };
            __list.insert(0, (p1).clone().source);
            __list
        };
        __left.extend(
            match behavior_result_port((p0).clone().nodes_owned(), (p1).clone().body) {
                ResultPortLookup::MissingResultPort => Vec::new(),
                ResultPortLookup::FoundResultPort { _0: port } => vec![(port).clone()],
            },
        );
        __left
    }
}
pub fn branch_path_outputs(p0: Vec<Path>) -> Vec<PortId> {
    match (p0).clone().as_slice() {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __list = branch_path_outputs(__list_tail.to_vec());
            __list.insert(0, __list_head.clone().result_port());
            __list
        }
    }
}
pub fn find_port(p0: Vec<Port>, p1: PortId) -> PortLookup {
    match (p0).clone().as_slice() {
        [] => PortLookup::MissingPort,
        [__list_head, __list_tail @ ..] => {
            if (__list_head.clone().id() == (p1).clone()) {
                PortLookup::FoundPort {
                    _0: __list_head.clone(),
                }
            } else {
                find_port(__list_tail.to_vec(), (p1).clone())
            }
        }
    }
}
pub fn find_behavior(p0: Vec<Behavior>, p1: NodeId) -> BehaviorLookup {
    match (p0).clone().as_slice() {
        [] => BehaviorLookup::MissingBehavior,
        [__list_head, __list_tail @ ..] => {
            if (behavior_id(__list_head.clone()) == (p1).clone()) {
                BehaviorLookup::FoundBehavior {
                    _0: __list_head.clone(),
                }
            } else {
                find_behavior(__list_tail.to_vec(), (p1).clone())
            }
        }
    }
}
pub fn find_producer(p0: Vec<Behavior>, p1: PortId) -> BehaviorLookup {
    match (p0).clone().as_slice() {
        [] => BehaviorLookup::MissingBehavior,
        [__list_head, __list_tail @ ..] => {
            if (behavior_port(__list_head.clone()) == (p1).clone()) {
                BehaviorLookup::FoundBehavior {
                    _0: __list_head.clone(),
                }
            } else {
                find_producer(__list_tail.to_vec(), (p1).clone())
            }
        }
    }
}
pub fn behavior_result_port(p0: Vec<Behavior>, p1: NodeId) -> ResultPortLookup {
    match find_behavior((p0).clone(), (p1).clone()) {
        BehaviorLookup::MissingBehavior => ResultPortLookup::MissingResultPort,
        BehaviorLookup::FoundBehavior { _0: behavior } => ResultPortLookup::FoundResultPort {
            _0: behavior_port((behavior).clone()),
        },
    }
}
pub fn behavior_id(p0: Behavior) -> NodeId {
    match (p0).clone() {
        Behavior::Value(v) => (v).clone().id,
        Behavior::Transform(t) => (t).clone().id,
        Behavior::Branch(b) => (b).clone().id,
        Behavior::Loop(l) => (l).clone().id,
        Behavior::Bind(bind) => (bind).clone().id,
    }
}
pub fn behavior_port(p0: Behavior) -> PortId {
    match (p0).clone() {
        Behavior::Value(v) => (v).clone().result_port(),
        Behavior::Transform(t) => (t).clone().result_port(),
        Behavior::Branch(b) => (b).clone().result_port(),
        Behavior::Loop(l) => (l).clone().result_port(),
        Behavior::Bind(bind) => (bind).clone().result_port(),
    }
}
