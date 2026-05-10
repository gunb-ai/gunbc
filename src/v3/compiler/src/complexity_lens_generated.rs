// AUTO-GENERATED from `src/v3/lenses/complexity.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum Certainty {
    Proven,
    Conservative,
}
#[derive(Clone, Debug)]
pub struct ComplexitySummary {
    pub work: SymbolicCost,
    pub span: SymbolicCost,
    pub asymptotic_class: AsymptoticClass,
    pub work_certainty: Certainty,
    pub span_certainty: Certainty,
}
#[derive(Clone, Debug)]
pub struct ComplexityEntry {
    pub port: PortId,
    pub summary: Lookup<ComplexitySummary>,
}
#[derive(Clone, Debug)]
pub enum DominanceOutcome {
    BothSurvive,
    OuterDominates,
    BodyDominates,
}
pub fn miss_complexity_summary_lookup() -> Lookup<ComplexitySummary> {
    Lookup::Miss
}
pub fn hit_complexity_summary_lookup(p0: ComplexitySummary) -> Lookup<ComplexitySummary> {
    Lookup::Hit(p0).clone()
}
pub fn complexity_of(p0: &Dag, p1: &PortId) -> Lookup<ComplexitySummary> {
    lookup_summary(&(compute_summaries(p0)), p1)
}
pub fn compute_summaries(p0: &Dag) -> Vec<ComplexityEntry> {
    ((p0).nodes())
        .iter()
        .fold(seed_bind_params((p0).nodes()), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, entry_for(p0, &__fold_acc, __fold_item));
            __list
        })
}
pub fn seed_bind_params(p0: &[Behavior]) -> Vec<ComplexityEntry> {
    (p0).iter().fold(Vec::new(), |__fold_acc, __fold_item| {
        let mut __left = params_of(__fold_item);
        __left.extend((__fold_acc).clone());
        __left
    })
}
pub fn params_of(p0: &Behavior) -> Vec<ComplexityEntry> {
    match p0 {
        Behavior::Value(_) => Vec::new(),
        Behavior::Transform(_) => Vec::new(),
        Behavior::Branch(_) => Vec::new(),
        Behavior::Loop(_) => Vec::new(),
        Behavior::Bind(bind) => param_entries(&((bind).params)),
    }
}
pub fn param_entries(p0: &[PortId]) -> Vec<ComplexityEntry> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __list = param_entries(__list_tail);
            __list.insert(
                0,
                ComplexityEntry {
                    port: (*(__list_head)),
                    summary: hit_complexity_summary_lookup(zero_summary()),
                },
            );
            __list
        }
    }
}
pub fn zero_summary() -> ComplexitySummary {
    summary_from_costs(
        SymbolicCost::ConstantCost { _0: 0 },
        SymbolicCost::ConstantCost { _0: 0 },
        Certainty::Proven,
        Certainty::Proven,
    )
}
pub fn unit_summary() -> ComplexitySummary {
    summary_from_costs(
        SymbolicCost::ConstantCost { _0: 1 },
        SymbolicCost::ConstantCost { _0: 1 },
        Certainty::Proven,
        Certainty::Proven,
    )
}
pub fn conservative_unknown_summary(p0: String) -> ComplexitySummary {
    summary_from_costs(
        SymbolicCost::UnknownCost { _0: (p0).clone() },
        SymbolicCost::UnknownCost { _0: (p0).clone() },
        Certainty::Conservative,
        Certainty::Conservative,
    )
}
pub fn summary_from_costs(
    p0: SymbolicCost,
    p1: SymbolicCost,
    p2: Certainty,
    p3: Certainty,
) -> ComplexitySummary {
    ComplexitySummary {
        work: (p0).clone(),
        span: (p1).clone(),
        asymptotic_class: classify_symbolic_cost(&p0),
        work_certainty: (p2).clone(),
        span_certainty: (p3).clone(),
    }
}
pub fn entry_for(p0: &Dag, p1: &[ComplexityEntry], p2: &Behavior) -> ComplexityEntry {
    match p2 {
        Behavior::Value(v) => ComplexityEntry {
            port: (v).result_port(),
            summary: hit_complexity_summary_lookup(zero_summary()),
        },
        Behavior::Transform(t) => ComplexityEntry {
            port: (t).result_port(),
            summary: transform_summary(p0, p1, &((t).id), &((t).inputs)),
        },
        Behavior::Branch(b) => ComplexityEntry {
            port: (b).result_port(),
            summary: branch_summary(p1, &((b).input), &((b).paths)),
        },
        Behavior::Loop(l) => ComplexityEntry {
            port: (l).result_port(),
            summary: loop_summary(p0, p1, l),
        },
        Behavior::Bind(bind) => ComplexityEntry {
            port: (bind).result_port(),
            summary: lookup_summary(p1, &((bind).result_port())),
        },
    }
}
pub fn transform_summary(
    p0: &Dag,
    p1: &[ComplexityEntry],
    p2: &NodeId,
    p3: &[PortId],
) -> Lookup<ComplexitySummary> {
    // BEGIN HOST FOLD MIRROR
    if let Some(summary) = std_list_fold_transform_summary(p0, p1, p2, p3) {
        return summary;
    }
    // END HOST FOLD MIRROR
    match &(per_call_pattern_at(p0, *p2)) {
        None => compose_many_inputs(p1, p3),
        Some(pattern) => recursive_transform_summary(p1, pattern, p3),
    }
}
// BEGIN HOST FOLD MIRROR
// The `.dag` authority models iteration with `LoopNode`, but surface `fold`
// calls lower as `TransformTarget::Callable(std.list.fold<...>)` today. This
// host mirror keeps `complexity_of` authoritative for those calls until the
// reflected substrate exposes the fold step as a direct behavior edge.
pub fn std_list_fold_transform_summary(
    p0: &Dag,
    p1: &[ComplexityEntry],
    p2: &NodeId,
    p3: &[PortId],
) -> Option<Lookup<ComplexitySummary>> {
    let behavior = (p0).node_opt(p2)?;
    let Behavior::Transform(transform) = behavior else {
        return None;
    };
    let TransformTarget::Callable(callee) = &transform.target else {
        return None;
    };
    if !is_std_list_fold_callable(p0, *callee) {
        return None;
    }
    let source = *(p3).first()?;
    let init = (p3).get(1).copied();
    let base = match init {
        None => lookup_summary(p1, &source),
        Some(init_port) => {
            combine_sequential(&(lookup_summary(p1, &source)), &(lookup_summary(p1, &init_port)))
        }
    };
    let step_body = fold_step_body_summary(p0, p1, *callee);
    let folded_body = if fold_step_body_nesting(p0, *callee, 0) > 0 {
        hit_complexity_summary_lookup(summary_from_costs(
            SymbolicCost::LinearCost {
                _0: SizeVariable {
                    source_port: source,
                    display_name: None,
                },
            },
            SymbolicCost::LinearCost {
                _0: SizeVariable {
                    source_port: source,
                    display_name: None,
                },
            },
            Certainty::Proven,
            Certainty::Proven,
        ))
    } else {
        step_body
    };
    Some(combine_sequential(
        &base,
        &(combine_iterate(
            &(hit_complexity_summary_lookup(summary_from_costs(
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: source,
                        display_name: None,
                    },
                },
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: source,
                        display_name: None,
                    },
                },
                Certainty::Proven,
                Certainty::Proven,
            ))),
            &folded_body,
        )),
    ))
}
pub fn is_std_list_fold_callable(p0: &Dag, mut p1: DeclarationId) -> bool {
    for _ in 0..16 {
        let decl = (p0).declaration(p1);
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => {
                if (p0).std_list_fold_decl() == Some(*template) {
                    return true;
                }
                p1 = *template;
            }
            _ => return false,
        }
    }
    false
}
pub fn fold_step_body_summary(
    p0: &Dag,
    p1: &[ComplexityEntry],
    mut p2: DeclarationId,
) -> Lookup<ComplexitySummary> {
    for _ in 0..16 {
        let decl = (p0).declaration(p2);
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                if (p0).std_list_fold_decl() == Some(*template) {
                    return (arguments).iter().fold(
                        hit_complexity_summary_lookup(zero_summary()),
                        |running, arg| {
                            combine_branch(
                                &running,
                                &(bind_summary_for_declaration(p0, p1, arg.value)),
                            )
                        },
                    );
                }
                p2 = *template;
            }
            _ => return hit_complexity_summary_lookup(zero_summary()),
        }
    }
    hit_complexity_summary_lookup(zero_summary())
}
pub fn bind_summary_for_declaration(
    p0: &Dag,
    p1: &[ComplexityEntry],
    mut p2: DeclarationId,
) -> Lookup<ComplexitySummary> {
    for _ in 0..16 {
        let decl = (p0).declaration(p2);
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => p2 = *template,
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(root),
                ..
            } => return lookup_summary(p1, &(root.bind(p0).result_port())),
            _ => return hit_complexity_summary_lookup(zero_summary()),
        }
    }
    hit_complexity_summary_lookup(zero_summary())
}
pub fn fold_step_body_nesting(p0: &Dag, mut p1: DeclarationId, p2: usize) -> usize {
    for _ in 0..16 {
        let decl = (p0).declaration(p1);
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                if (p0).std_list_fold_decl() == Some(*template) {
                    return (arguments)
                        .iter()
                        .filter_map(|arg| bind_result_port_for_declaration(p0, arg.value))
                        .map(|port| fold_nesting_at_port(p0, port, p2 + 1))
                        .max()
                        .unwrap_or(0);
                }
                p1 = *template;
            }
            _ => return 0,
        }
    }
    0
}
pub fn fold_nesting_at_port(p0: &Dag, p1: PortId, p2: usize) -> usize {
    if p2 > 32 {
        return 0;
    }
    let producer = match (p0).resolve_producer_lookup(&p1) {
        ProducerLookup::Found(producer) => producer,
        ProducerLookup::NoProducer
        | ProducerLookup::MissingPort { .. }
        | ProducerLookup::MissingNode { .. }
        | ProducerLookup::BindCycle { .. } => return 0,
    };
    match producer {
        Behavior::Value(_) => 0,
        Behavior::Branch(branch) => (branch)
            .paths
            .iter()
            .map(|path| fold_nesting_at_port(p0, path.output, p2 + 1))
            .max()
            .unwrap_or(0),
        Behavior::Loop(loop_node) => 1 + fold_nesting_at_node(p0, loop_node.body, p2 + 1),
        Behavior::Bind(bind) => fold_nesting_at_port(p0, bind.value, p2 + 1),
        Behavior::Transform(transform) => {
            let input_depth = (transform)
                .inputs
                .iter()
                .map(|input| fold_nesting_at_port(p0, *input, p2 + 1))
                .max()
                .unwrap_or(0);
            let callable_depth = match &transform.target {
                TransformTarget::Callable(callee) if is_std_list_fold_callable(p0, *callee) => {
                    1 + fold_step_body_nesting(p0, *callee, p2 + 1)
                }
                _ => 0,
            };
            input_depth.max(callable_depth)
        }
    }
}
pub fn fold_nesting_at_node(p0: &Dag, p1: NodeId, p2: usize) -> usize {
    let Some(behavior) = (p0).node_opt(&p1) else {
        return 0;
    };
    fold_nesting_at_port(p0, behavior_result_port(behavior), p2 + 1)
}
pub fn bind_result_port_for_declaration(p0: &Dag, mut p1: DeclarationId) -> Option<PortId> {
    for _ in 0..16 {
        let decl = (p0).declaration(p1);
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => p1 = *template,
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(root),
                ..
            } => return Some(root.bind(p0).result_port()),
            _ => return None,
        }
    }
    None
}
// END HOST FOLD MIRROR
pub fn compose_many_inputs(p0: &[ComplexityEntry], p1: &[PortId]) -> Lookup<ComplexitySummary> {
    (p1).iter().fold(
        hit_complexity_summary_lookup(unit_summary()),
        |__fold_acc, __fold_item| {
            combine_sequential(&__fold_acc, &(lookup_summary(p0, __fold_item)))
        },
    )
}
pub fn recursive_transform_summary(
    p0: &[ComplexityEntry],
    p1: &CallPattern,
    p2: &[PortId],
) -> Lookup<ComplexitySummary> {
    match p2 {
        [] => miss_complexity_summary_lookup(),
        [__list_head, __list_tail @ ..] => combine_iterate(
            &(summary_from_iter_bound(pattern_to_iter_bound(p1, __list_head))),
            &(compose_many_inputs(p0, p2)),
        ),
    }
}
pub fn pattern_to_iter_bound(p0: &CallPattern, p1: &PortId) -> SymbolicCost {
    match p0 {
        CallPattern::ArithmeticSubtractCall {
            steps: _,
            ring_param: _,
        } => SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::ArithmeticDivideCall {
            divisor: _,
            ring_param: _,
        } => SymbolicCost::LogCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::ChildAccessorCall { accessor: _ } => SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::CollectionShrinkCall {
            amount: _,
            collection: _,
        } => SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::FoldBodyCall {
            outer_collection: _,
        } => SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::ParserAdvanceCall { witness: _ } => SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::WorklistDrainCall { element: _ } => SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: *p1,
                display_name: None,
            },
        },
        CallPattern::SameArgumentCall => SymbolicCost::UnknownCost {
            _0: String::from("same-argument recursive call has no descent"),
        },
    }
}
pub fn summary_from_iter_bound(p0: SymbolicCost) -> Lookup<ComplexitySummary> {
    match &p0 {
        SymbolicCost::UnknownCost { _0: reason } => {
            hit_complexity_summary_lookup(conservative_unknown_summary((reason).clone()))
        }
        SymbolicCost::ConstantCost { _0: _ } => hit_complexity_summary_lookup(summary_from_costs(
            (p0).clone(),
            (p0).clone(),
            Certainty::Proven,
            Certainty::Proven,
        )),
        SymbolicCost::LinearCost { _0: _ } => hit_complexity_summary_lookup(summary_from_costs(
            (p0).clone(),
            (p0).clone(),
            Certainty::Proven,
            Certainty::Proven,
        )),
        SymbolicCost::PolynomialCost { var: _, degree: _ } => {
            hit_complexity_summary_lookup(summary_from_costs(
                (p0).clone(),
                (p0).clone(),
                Certainty::Proven,
                Certainty::Proven,
            ))
        }
        SymbolicCost::ProductCost { _0: _ } => hit_complexity_summary_lookup(summary_from_costs(
            (p0).clone(),
            (p0).clone(),
            Certainty::Proven,
            Certainty::Proven,
        )),
        SymbolicCost::SumCost { _0: _ } => hit_complexity_summary_lookup(summary_from_costs(
            (p0).clone(),
            (p0).clone(),
            Certainty::Proven,
            Certainty::Proven,
        )),
        SymbolicCost::LogCost { _0: _ } => hit_complexity_summary_lookup(summary_from_costs(
            (p0).clone(),
            (p0).clone(),
            Certainty::Proven,
            Certainty::Proven,
        )),
    }
}
pub fn loop_summary(p0: &Dag, p1: &[ComplexityEntry], p2: &LoopNode) -> Lookup<ComplexitySummary> {
    combine_sequential(
        &(combine_sequential(
            &(lookup_summary(p1, &((p2).source))),
            &(lookup_summary(p1, &((p2).init))),
        )),
        &(combine_iterate(
            &(hit_complexity_summary_lookup(summary_from_costs(
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: ((p2).source),
                        display_name: None,
                    },
                },
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: ((p2).source),
                        display_name: None,
                    },
                },
                Certainty::Proven,
                Certainty::Proven,
            ))),
            &(body_summary(p0, p1, &((p2).body))),
        )),
    )
}
pub fn body_summary(p0: &Dag, p1: &[ComplexityEntry], p2: &NodeId) -> Lookup<ComplexitySummary> {
    match &((p0).node_opt(p2).cloned()) {
        None => miss_complexity_summary_lookup(),
        Some(body_behavior) => lookup_summary(p1, &(behavior_result_port(body_behavior))),
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
pub fn branch_summary(
    p0: &[ComplexityEntry],
    p1: &PortId,
    p2: &[Path],
) -> Lookup<ComplexitySummary> {
    combine_sequential(&(lookup_summary(p0, p1)), &(max_path_summaries(p0, p2)))
}
pub fn max_path_summaries(p0: &[ComplexityEntry], p1: &[Path]) -> Lookup<ComplexitySummary> {
    (p1).iter().fold(
        hit_complexity_summary_lookup(zero_summary()),
        |__fold_acc, __fold_item| {
            combine_branch(
                &__fold_acc,
                &(lookup_summary(p0, &((__fold_item).result_port()))),
            )
        },
    )
}
pub fn lookup_summary(p0: &[ComplexityEntry], p1: &PortId) -> Lookup<ComplexitySummary> {
    (p0).iter().fold(
        miss_complexity_summary_lookup(),
        |__fold_acc, __fold_item| match &__fold_acc {
            Lookup::Hit(_) => __fold_acc,
            Lookup::Miss => {
                if ((__fold_item).port == (*(p1))) {
                    ((__fold_item).summary).clone()
                } else {
                    miss_complexity_summary_lookup()
                }
            }
        },
    )
}
pub fn combine_sequential(
    p0: &Lookup<ComplexitySummary>,
    p1: &Lookup<ComplexitySummary>,
) -> Lookup<ComplexitySummary> {
    match p0 {
        Lookup::Miss => miss_complexity_summary_lookup(),
        Lookup::Hit(ax) => match p1 {
            Lookup::Miss => miss_complexity_summary_lookup(),
            Lookup::Hit(bx) => hit_complexity_summary_lookup(compose_summary_sequential(ax, bx)),
        },
    }
}
pub fn combine_iterate(
    p0: &Lookup<ComplexitySummary>,
    p1: &Lookup<ComplexitySummary>,
) -> Lookup<ComplexitySummary> {
    match p0 {
        Lookup::Miss => miss_complexity_summary_lookup(),
        Lookup::Hit(outer_summary) => match p1 {
            Lookup::Miss => miss_complexity_summary_lookup(),
            Lookup::Hit(body_summary_value) => hit_complexity_summary_lookup(
                compose_summary_iterate(outer_summary, body_summary_value),
            ),
        },
    }
}
pub fn combine_branch(
    p0: &Lookup<ComplexitySummary>,
    p1: &Lookup<ComplexitySummary>,
) -> Lookup<ComplexitySummary> {
    match p0 {
        Lookup::Miss => miss_complexity_summary_lookup(),
        Lookup::Hit(ax) => match p1 {
            Lookup::Miss => miss_complexity_summary_lookup(),
            Lookup::Hit(bx) => hit_complexity_summary_lookup(compose_summary_branch_pair(ax, bx)),
        },
    }
}
pub fn compose_summary_sequential(
    p0: &ComplexitySummary,
    p1: &ComplexitySummary,
) -> ComplexitySummary {
    summary_from_costs(
        sequential(((p0).work).clone(), ((p1).work).clone()),
        sequential(((p0).span).clone(), ((p1).span).clone()),
        certainty_of_surviving_per_dim(
            ((p0).work).clone(),
            ((p1).work).clone(),
            ((p0).work_certainty).clone(),
            ((p1).work_certainty).clone(),
        ),
        certainty_of_surviving_per_dim(
            ((p0).span).clone(),
            ((p1).span).clone(),
            ((p0).span_certainty).clone(),
            ((p1).span_certainty).clone(),
        ),
    )
}
pub fn compose_summary_iterate(
    p0: &ComplexitySummary,
    p1: &ComplexitySummary,
) -> ComplexitySummary {
    summary_from_costs(
        iterate(((p0).work).clone(), ((p1).work).clone()),
        iterate(((p0).work).clone(), ((p1).span).clone()),
        certainty_of_surviving_per_dim(
            ((p0).work).clone(),
            ((p1).work).clone(),
            ((p0).work_certainty).clone(),
            ((p1).work_certainty).clone(),
        ),
        certainty_of_surviving_per_dim(
            ((p0).work).clone(),
            ((p1).span).clone(),
            ((p0).work_certainty).clone(),
            ((p1).span_certainty).clone(),
        ),
    )
}
pub fn compose_summary_branch_pair(
    p0: &ComplexitySummary,
    p1: &ComplexitySummary,
) -> ComplexitySummary {
    summary_from_costs(
        max_path(
            &({
                let mut __list = {
                    let mut __list = Vec::new();
                    __list.insert(0, ((p1).work).clone());
                    __list
                };
                __list.insert(0, ((p0).work).clone());
                __list
            }),
        ),
        max_path(
            &({
                let mut __list = {
                    let mut __list = Vec::new();
                    __list.insert(0, ((p1).span).clone());
                    __list
                };
                __list.insert(0, ((p0).span).clone());
                __list
            }),
        ),
        certainty_of_surviving_per_dim(
            ((p0).work).clone(),
            ((p1).work).clone(),
            ((p0).work_certainty).clone(),
            ((p1).work_certainty).clone(),
        ),
        certainty_of_surviving_per_dim(
            ((p0).span).clone(),
            ((p1).span).clone(),
            ((p0).span_certainty).clone(),
            ((p1).span_certainty).clone(),
        ),
    )
}
pub fn certainty_of_surviving_per_dim(
    p0: SymbolicCost,
    p1: SymbolicCost,
    p2: Certainty,
    p3: Certainty,
) -> Certainty {
    match &(dominance_outcome((p0).clone(), (p1).clone())) {
        DominanceOutcome::BothSurvive => meet_pair(&p2, (p3).clone()),
        DominanceOutcome::OuterDominates => p2,
        DominanceOutcome::BodyDominates => p3,
    }
}
pub fn dominance_outcome(p0: SymbolicCost, p1: SymbolicCost) -> DominanceOutcome {
    if dominates(&p0, (p1).clone()) {
        if dominates(&p1, (p0).clone()) {
            DominanceOutcome::BothSurvive
        } else {
            DominanceOutcome::OuterDominates
        }
    } else {
        if dominates(&p1, (p0).clone()) {
            DominanceOutcome::BodyDominates
        } else {
            DominanceOutcome::BothSurvive
        }
    }
}
pub fn meet_pair(p0: &Certainty, p1: Certainty) -> Certainty {
    match p0 {
        Certainty::Proven => p1,
        Certainty::Conservative => Certainty::Conservative,
    }
}
pub fn complexity_enforcement_project(p0: &ComplexitySummary) -> AsymptoticClass {
    ((p0).asymptotic_class).clone()
}
pub fn complexity_enforcement_violates(p0: &AsymptoticClass, p1: &AsymptoticClass) -> bool {
    if asymptotic_dominates(p1, p0) {
        if asymptotic_dominates(p0, p1) {
            (0 == 1)
        } else {
            (0 == 0)
        }
    } else {
        (0 == 1)
    }
}
pub fn witness_from_complexity_lookup(
    p0: &Lookup<ComplexitySummary>,
    p1: Behavior,
) -> Witness<ComplexitySummary> {
    match p0 {
        Lookup::Hit(summary) => Witness::Inhabits((summary).clone()),
        Lookup::Miss => Witness::Violates {
            reason: String::from(
                "complexity_of: missing ComplexitySummary for behavior result port",
            ),
            at: (p1).clone(),
        },
    }
}
pub fn complexity_lens_read(p0: &Dag, p1: Behavior) -> Witness<ComplexitySummary> {
    witness_from_complexity_lookup(
        &(complexity_of(p0, &(behavior_result_port(&p1)))),
        (p1).clone(),
    )
}
pub fn complexity_lens_sequential_op(
    p0: &ComplexitySummary,
    p1: &ComplexitySummary,
) -> ComplexitySummary {
    compose_summary_sequential(p0, p1)
}
pub fn complexity_lens_branch_op(
    p0: &ComplexitySummary,
    p1: &ComplexitySummary,
) -> ComplexitySummary {
    compose_summary_branch_pair(p0, p1)
}
pub fn complexity_lens_iterate_op(p0: &ComplexitySummary, p1: &LoopBound) -> ComplexitySummary {
    match p1 {
        LoopBound::Cardinality { count: payload } => compose_summary_iterate(
            &(summary_from_costs(
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: *payload,
                        display_name: None,
                    },
                },
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: *payload,
                        display_name: None,
                    },
                },
                Certainty::Proven,
                Certainty::Proven,
            )),
            p0,
        ),
        LoopBound::Descent {
            cluster: __payload_cluster,
            measure: __payload_measure,
        } => compose_summary_iterate(
            &(summary_from_costs(
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: *__payload_measure,
                        display_name: None,
                    },
                },
                SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: *__payload_measure,
                        display_name: None,
                    },
                },
                Certainty::Proven,
                Certainty::Proven,
            )),
            p0,
        ),
    }
}
pub fn complexity_summary_work_class_consistent(p0: &ComplexitySummary) -> bool {
    asymptotic_dominates(
        &((p0).asymptotic_class),
        &(classify_symbolic_cost(&((p0).work))),
    )
}
pub fn complexity_lens_validate(p0: &Dag, p1: &ComplexitySummary) -> OptionalDiagnostic {
    OptionalDiagnostic::NoDiagnostic
}
