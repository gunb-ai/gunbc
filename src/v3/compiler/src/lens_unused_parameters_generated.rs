// AUTO-GENERATED from `src/v3/lenses/unused_parameters.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct UnusedParameter {
    pub function: NodeId,
    pub parameter: PortId,
    pub parameter_index: i64,
    pub function_span: SourceSpan,
}
pub fn check(p0: &Dag) -> Vec<UnusedParameter> {
    ((p0).nodes())
        .iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            let mut __left = (__fold_acc).clone();
            __left.extend(check_behavior(p0, __fold_item));
            __left
        })
}
pub fn check_behavior(p0: &Dag, p1: &Behavior) -> Vec<UnusedParameter> {
    match p1 {
        Behavior::Value(_) => Vec::new(),
        Behavior::Transform(_) => Vec::new(),
        Behavior::Branch(_) => Vec::new(),
        Behavior::Loop(_) => Vec::new(),
        Behavior::Bind(bind) => {
            if ((bind).params).is_empty() {
                Vec::new()
            } else {
                check_bind(p0, bind)
            }
        }
    }
}
pub fn check_bind(p0: &Dag, p1: &BindNode) -> Vec<UnusedParameter> {
    collect_unused_params(
        &((p1).params),
        p1,
        &(referenced_ports(p0, (p1).result_port())),
        0,
    )
}
pub fn collect_unused_params(
    p0: &[PortId],
    p1: &BindNode,
    p2: &[PortId],
    p3: i64,
) -> Vec<UnusedParameter> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            if (p2).contains(__list_head) {
                collect_unused_params(__list_tail, p1, p2, (p3 + 1))
            } else {
                {
                    let mut __list = collect_unused_params(__list_tail, p1, p2, (p3 + 1));
                    __list.insert(
                        0,
                        UnusedParameter {
                            function: (p1).id,
                            parameter: (*(__list_head)),
                            parameter_index: p3,
                            function_span: ((p1).span).clone(),
                        },
                    );
                    __list
                }
            }
        }
    }
}
pub fn referenced_ports(p0: &Dag, p1: PortId) -> Vec<PortId> {
    walk_steps(&(((p0).ports()).len() as i64), p0, &[p1], Vec::new())
}
pub fn walk_steps(p0: &i64, p1: &Dag, p2: &[PortId], p3: Vec<PortId>) -> Vec<PortId> {
    if ((*(p0)) == 0) {
        p3
    } else {
        if (p2).is_empty() {
            p3
        } else {
            walk_steps(
                &((*(p0)) - 1),
                p1,
                &(expand_frontier_list(p2, p1, &p3)),
                expand_referenced_list(p2, (p3).clone()),
            )
        }
    }
}
pub fn expand_frontier_list(p0: &[PortId], p1: &Dag, p2: &[PortId]) -> Vec<PortId> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            if (p2).contains(__list_head) {
                expand_frontier_list(__list_tail, p1, p2)
            } else {
                {
                    let mut __left = inputs_for_port(p1, __list_head);
                    __left.extend(expand_frontier_list(__list_tail, p1, p2));
                    __left
                }
            }
        }
    }
}
pub fn expand_referenced_list(p0: &[PortId], p1: Vec<PortId>) -> Vec<PortId> {
    (p0).iter().fold((p1).clone(), |__fold_acc, __fold_item| {
        if (__fold_acc).contains(__fold_item) {
            __fold_acc
        } else {
            {
                let mut __list = (__fold_acc).clone();
                __list.insert(0, (*(__fold_item)));
                __list
            }
        }
    })
}
pub fn inputs_for_port(p0: &Dag, p1: &PortId) -> Vec<PortId> {
    match &((p0).port_opt(p1).cloned()) {
        None => Vec::new(),
        Some(p) => match &((p).produced_by) {
            None => Vec::new(),
            Some(node_id) => inputs_for_node(p0, node_id),
        },
    }
}
pub fn inputs_for_node(p0: &Dag, p1: &NodeId) -> Vec<PortId> {
    match &((p0).node_opt(p1).cloned()) {
        None => Vec::new(),
        Some(behavior) => inputs_for_behavior(p0, behavior),
    }
}
pub fn inputs_for_behavior(p0: &Dag, p1: &Behavior) -> Vec<PortId> {
    match p1 {
        Behavior::Value(_) => Vec::new(),
        Behavior::Transform(t) => ((t).inputs).to_vec(),
        Behavior::Branch(branch) => {
            let mut __list = branch_path_outputs(&((branch).paths));
            __list.insert(0, (branch).input);
            __list
        }
        Behavior::Loop(loop_node) => loop_inputs(p0, loop_node),
        Behavior::Bind(bind) => vec![(bind).result_port()],
    }
}
pub fn loop_inputs(p0: &Dag, p1: &LoopNode) -> Vec<PortId> {
    {
        let mut __left = {
            let mut __list = {
                let mut __list = vec![((p1).bound).count];
                __list.insert(0, (p1).init);
                __list
            };
            __list.insert(0, (p1).source);
            __list
        };
        __left.extend(match &((p0).node_opt(&((p1).body)).cloned()) {
            None => Vec::new(),
            Some(body_node) => vec![behavior_port(body_node)],
        });
        __left
    }
}
pub fn branch_path_outputs(p0: &[Path]) -> Vec<PortId> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __list = branch_path_outputs(__list_tail);
            __list.insert(0, (__list_head).result_port());
            __list
        }
    }
}
pub fn behavior_port(p0: &Behavior) -> PortId {
    match p0 {
        Behavior::Value(v) => (v).result_port(),
        Behavior::Transform(t) => (t).result_port(),
        Behavior::Branch(b) => (b).result_port(),
        Behavior::Loop(l) => (l).result_port(),
        Behavior::Bind(bind) => (bind).result_port(),
    }
}
