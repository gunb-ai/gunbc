// AUTO-GENERATED from `src/v3/lenses/effect_enumeration.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum StructuralEffectShape {
    NoEffect,
    ReadShaped,
    WriteShaped,
    UnknownEffect { reason: String },
}
#[derive(Clone, Debug)]
pub struct EffectFact {
    pub port: PortId,
    pub shape: StructuralEffectShape,
}
#[derive(Clone, Debug)]
pub struct CoverageGap {
    pub node: NodeId,
    pub reason: String,
}
#[derive(Clone, Debug)]
pub struct RedundantReadError {
    pub first_read: PortId,
    pub redundant_read: PortId,
    pub reason: String,
}
#[derive(Clone, Debug)]
pub enum TransactionalPattern {
    NoTransaction,
    BeginModifyCommit { root: NodeId },
}
#[derive(Clone, Debug)]
pub struct EffectEnumerationReport {
    pub facts: Vec<EffectFact>,
    pub coverage_gaps: Vec<CoverageGap>,
    pub redundant_reads: Vec<RedundantReadError>,
    pub transaction: TransactionalPattern,
}
pub fn enumerate_effects(p0: &Dag) -> EffectEnumerationReport {
    EffectEnumerationReport {
        facts: compute_effect_facts(p0),
        coverage_gaps: compute_coverage_gaps(p0),
        redundant_reads: Vec::new(),
        transaction: transaction_pattern(p0),
    }
}
pub fn compute_effect_facts(p0: &Dag) -> Vec<EffectFact> {
    ((p0).nodes())
        .iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            let mut __list = (__fold_acc).clone();
            __list.insert(0, effect_fact_for(p0, &__fold_acc, __fold_item));
            __list
        })
}
pub fn effect_fact_for(p0: &Dag, p1: &[EffectFact], p2: &Behavior) -> EffectFact {
    match p2 {
        Behavior::Value(v) => EffectFact {
            port: (v).result_port(),
            shape: StructuralEffectShape::NoEffect,
        },
        Behavior::Transform(t) => EffectFact {
            port: (t).result_port(),
            shape: transform_effect(p0, t),
        },
        Behavior::Branch(b) => EffectFact {
            port: (b).result_port(),
            shape: branch_effect(p1, &((b).paths)),
        },
        Behavior::Loop(l) => EffectFact {
            port: (l).result_port(),
            shape: loop_effect(p0, p1, &((l).body)),
        },
        Behavior::Bind(bind) => EffectFact {
            port: (bind).result_port(),
            shape: bind_effect(p0, p1, bind),
        },
    }
}
pub fn bind_effect(p0: &Dag, p1: &[EffectFact], p2: &BindNode) -> StructuralEffectShape {
    match &((p0).resolve_producer_opt(&((p2).result_port())).cloned()) {
        None => StructuralEffectShape::UnknownEffect {
            reason: String::from("bind result producer missing from substrate"),
        },
        Some(producer) => {
            if (behavior_node_id(producer) == (p2).id) {
                StructuralEffectShape::UnknownEffect {
                    reason: String::from("bind result producer self-shadows bind output"),
                }
            } else {
                effect_at(p1, &(behavior_result_port(producer)))
            }
        }
    }
}
pub fn transform_effect(p0: &Dag, p1: &TransformNode) -> StructuralEffectShape {
    match &((p1).target) {
        TransformTarget::Callable(id) => callable_signature_effect(p0, id),
        TransformTarget::UnresolvedFieldProject { field_label: _ } => {
            StructuralEffectShape::NoEffect
        }
        TransformTarget::ResolvedFieldProject { field_label: _ } => StructuralEffectShape::NoEffect,
        TransformTarget::Operator(_) => StructuralEffectShape::NoEffect,
    }
}
pub fn callable_signature_effect(p0: &Dag, p1: &DeclarationId) -> StructuralEffectShape {
    match &((p0).declaration_opt(p1).cloned()) {
        None => StructuralEffectShape::UnknownEffect {
            reason: String::from("callable declaration missing from substrate"),
        },
        Some(decl) => match &((decl).connective) {
            TypeConnective::Arrow {
                inputs: __a_inputs,
                output: __a_output,
                body: __a_body,
            } => callable_arrow_effect(__a_inputs, __a_output, __a_body),
            TypeConnective::Atom(_) => StructuralEffectShape::UnknownEffect {
                reason: String::from("transform target is not an arrow declaration"),
            },
            TypeConnective::Conj { children: _ } => StructuralEffectShape::UnknownEffect {
                reason: String::from("transform target is not an arrow declaration"),
            },
            TypeConnective::Disj { variants: _ } => StructuralEffectShape::UnknownEffect {
                reason: String::from("transform target is not an arrow declaration"),
            },
            TypeConnective::Cardinality(_) => StructuralEffectShape::UnknownEffect {
                reason: String::from("transform target is not an arrow declaration"),
            },
            TypeConnective::Instantiation {
                template: _,
                arguments: _,
            } => StructuralEffectShape::UnknownEffect {
                reason: String::from("transform target is not an arrow declaration"),
            },
        },
    }
}
pub fn callable_arrow_effect(
    p0: &[DeclarationId],
    p1: &DeclarationId,
    p2: &ArrowBody,
) -> StructuralEffectShape {
    match p0 {
        [] => body_default_effect(p2),
        [__list_head, __list_tail @ ..] => {
            if ((*(__list_head)) == (*(p1))) {
                StructuralEffectShape::WriteShaped
            } else {
                callable_arrow_effect(__list_tail, p1, p2)
            }
        }
    }
}
pub fn body_default_effect(p0: &ArrowBody) -> StructuralEffectShape {
    match p0 {
        ArrowBody::ExternalRealization(_) => StructuralEffectShape::ReadShaped,
        ArrowBody::UserDefined(_) => StructuralEffectShape::ReadShaped,
        ArrowBody::NoBody => StructuralEffectShape::UnknownEffect {
            reason: String::from("callable has no body and no external realization"),
        },
        ArrowBody::Pending => StructuralEffectShape::UnknownEffect {
            reason: String::from("callable body pending; signature coverage incomplete"),
        },
        ArrowBody::Unparsed(_) => StructuralEffectShape::UnknownEffect {
            reason: String::from("callable body unparsed; signature coverage incomplete"),
        },
    }
}
pub fn branch_effect(p0: &[EffectFact], p1: &[Path]) -> StructuralEffectShape {
    (p1).iter().fold(
        StructuralEffectShape::NoEffect,
        |__fold_acc, __fold_item| {
            combine_effects(
                (__fold_acc).clone(),
                effect_at(p0, &((__fold_item).result_port())),
            )
        },
    )
}
pub fn loop_effect(p0: &Dag, p1: &[EffectFact], p2: &NodeId) -> StructuralEffectShape {
    match &((p0).node_opt(p2).cloned()) {
        None => StructuralEffectShape::UnknownEffect {
            reason: String::from("loop body node missing from substrate"),
        },
        Some(behavior) => effect_at(p1, &(behavior_result_port(behavior))),
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
pub fn combine_effects(
    p0: StructuralEffectShape,
    p1: StructuralEffectShape,
) -> StructuralEffectShape {
    match &p0 {
        StructuralEffectShape::UnknownEffect { reason: _ } => p0,
        StructuralEffectShape::WriteShaped => StructuralEffectShape::WriteShaped,
        StructuralEffectShape::ReadShaped => match &p1 {
            StructuralEffectShape::UnknownEffect { reason: _ } => p1,
            StructuralEffectShape::WriteShaped => StructuralEffectShape::WriteShaped,
            StructuralEffectShape::ReadShaped => StructuralEffectShape::ReadShaped,
            StructuralEffectShape::NoEffect => StructuralEffectShape::ReadShaped,
        },
        StructuralEffectShape::NoEffect => p1,
    }
}
pub fn effect_at(p0: &[EffectFact], p1: &PortId) -> StructuralEffectShape {
    match p0 {
        [] => StructuralEffectShape::UnknownEffect {
            reason: String::from("effect fact missing for port"),
        },
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).port == (*(p1))) {
                ((__list_head).shape).clone()
            } else {
                effect_at(__list_tail, p1)
            }
        }
    }
}
pub fn compute_coverage_gaps(p0: &Dag) -> Vec<CoverageGap> {
    compute_coverage_gaps_from_facts((p0).nodes(), p0, &(compute_effect_facts(p0)))
}
pub fn compute_coverage_gaps_from_facts(
    p0: &[Behavior],
    p1: &Dag,
    p2: &[EffectFact],
) -> Vec<CoverageGap> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => prepend_gap(
            &(coverage_gap_for(p1, p2, __list_head)),
            compute_coverage_gaps_from_facts(__list_tail, p1, p2),
        ),
    }
}
pub fn prepend_gap(p0: &[CoverageGap], p1: Vec<CoverageGap>) -> Vec<CoverageGap> {
    match p0 {
        [] => p1,
        [__list_head, __list_tail @ ..] => {
            let mut __list = prepend_gap(__list_tail, (p1).clone());
            __list.insert(0, (__list_head).clone());
            __list
        }
    }
}
pub fn coverage_gap_for(p0: &Dag, p1: &[EffectFact], p2: &Behavior) -> Vec<CoverageGap> {
    match p2 {
        Behavior::Transform(t) => coverage_gap_for_effect((t).id, &(transform_effect(p0, t))),
        Behavior::Branch(br) => {
            coverage_gap_for_effect((br).id, &(branch_effect(p1, &((br).paths))))
        }
        Behavior::Loop(l) => coverage_gap_for_effect((l).id, &(loop_effect(p0, p1, &((l).body)))),
        Behavior::Bind(bind) => {
            coverage_gap_for_effect((bind).id, &(effect_at(p1, &((bind).result_port()))))
        }
        Behavior::Value(_) => Vec::new(),
    }
}
pub fn coverage_gap_for_effect(p0: NodeId, p1: &StructuralEffectShape) -> Vec<CoverageGap> {
    match p1 {
        StructuralEffectShape::UnknownEffect { reason: u } => {
            let mut __list = Vec::new();
            __list.insert(
                0,
                CoverageGap {
                    node: p0,
                    reason: (u).clone(),
                },
            );
            __list
        }
        StructuralEffectShape::NoEffect => Vec::new(),
        StructuralEffectShape::ReadShaped => Vec::new(),
        StructuralEffectShape::WriteShaped => Vec::new(),
    }
}
pub fn transaction_pattern(p0: &Dag) -> TransactionalPattern {
    TransactionalPattern::NoTransaction
}
pub fn behavior_node_id(p0: &Behavior) -> NodeId {
    match p0 {
        Behavior::Value(v) => (v).id,
        Behavior::Transform(t) => (t).id,
        Behavior::Branch(b) => (b).id,
        Behavior::Loop(l) => (l).id,
        Behavior::Bind(bind) => (bind).id,
    }
}
