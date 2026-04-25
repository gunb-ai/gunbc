// AUTO-GENERATED from `src/v3/lenses/infer_helpers.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum IntegerAlgebra {
    OrderedRingAlgebra,
    SemiringAlgebra,
}
#[derive(Clone, Debug)]
pub enum NonIntegerAlgebra {
    BooleanAlgebraAlgebra,
    TerminalAlgebra,
}
#[derive(Clone, Debug)]
pub enum TargetCarrier {
    BitCarrier,
    ByteCarrier,
    Word16Carrier,
    Word32Carrier,
    Word64Carrier,
    TerminalCarrier,
}
#[derive(Clone, Debug)]
pub enum IntegerOverflow {
    TwoComplementWrap,
    Saturating,
    Trap,
}
#[derive(Clone, Debug)]
pub enum RustPrimitive {
    IntegerPrimitive {
        target_name: String,
        algebra: IntegerAlgebra,
        carrier: TargetCarrier,
        is_copy: bool,
        overflow: IntegerOverflow,
    },
    NonIntegerPrimitive {
        target_name: String,
        algebra: NonIntegerAlgebra,
        carrier: TargetCarrier,
        is_copy: bool,
    },
}
#[derive(Clone, Debug)]
pub enum TemplateArgumentBinding {
    Conflict,
    NoOp,
    Append,
    ReplaceAt { _0: i64, _1: DeclarationId },
}
#[derive(Clone, Debug)]
pub enum TemplateArgumentsMatch {
    Match,
    Mismatch,
}
#[derive(Clone, Debug)]
pub enum TemplateArgumentCursor {
    CursorEnd,
    CursorHead {
        head: TemplateArgument,
        tail: Vec<TemplateArgument>,
    },
}
#[derive(Clone, Debug)]
pub enum NormalizedInstantiationArgs {
    NotInstantiation,
    Normalized {
        template: DeclarationId,
        args: Vec<TemplateArgument>,
    },
}
pub fn behavior_output_port(p0: &Behavior) -> PortId {
    match p0 {
        Behavior::Value(v) => (v).result_port(),
        Behavior::Transform(t) => (t).result_port(),
        Behavior::Branch(b) => (b).result_port(),
        Behavior::Loop(l) => (l).result_port(),
        Behavior::Bind(bind) => (bind).result_port(),
    }
}
pub fn behavior_span(p0: &Behavior) -> SourceSpan {
    match p0 {
        Behavior::Value(v) => ((v).span).clone(),
        Behavior::Transform(t) => ((t).span).clone(),
        Behavior::Branch(b) => ((b).span).clone(),
        Behavior::Loop(l) => ((l).span).clone(),
        Behavior::Bind(bind) => ((bind).span).clone(),
    }
}
pub fn payload_binding_span(p0: &Path, p1: SourceSpan) -> SourceSpan {
    match &((p0).pattern) {
        BranchPattern::UnresolvedVariant {
            name: __u_name,
            span: __u_span,
        } => (__u_span).clone(),
        BranchPattern::ResolvedVariant(_) => p1,
    }
}
pub fn template_argument_value(
    p0: &[TemplateArgument],
    p1: &DeclarationId,
) -> Lookup<DeclarationId> {
    match p0 {
        [] => Lookup::Miss,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).parameter == (*(p1))) {
                Lookup::Hit((__list_head).value)
            } else {
                template_argument_value(__list_tail, p1)
            }
        }
    }
}
pub fn resolve_template_argument_value(
    p0: &i64,
    p1: &[TemplateArgument],
    p2: DeclarationId,
) -> DeclarationId {
    if ((*(p0)) <= 0) {
        p2
    } else {
        match &(template_argument_value(p1, &p2)) {
            Lookup::Miss => p2,
            Lookup::Hit(next) => {
                if ((*(next)) == p2) {
                    p2
                } else {
                    resolve_template_argument_value(&((*(p0)) - 1), p1, (*(next)))
                }
            }
        }
    }
}
pub fn push_template_argument_binding(
    p0: &[TemplateArgument],
    p1: &DeclarationId,
    p2: DeclarationId,
) -> TemplateArgumentBinding {
    push_template_argument_binding_at(p0, p1, p2, 0)
}
pub fn push_template_argument_binding_at(
    p0: &[TemplateArgument],
    p1: &DeclarationId,
    p2: DeclarationId,
    p3: i64,
) -> TemplateArgumentBinding {
    match p0 {
        [] => TemplateArgumentBinding::Append,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).parameter == (*(p1))) {
                if ((__list_head).value == (*(p1))) {
                    TemplateArgumentBinding::ReplaceAt { _0: p3, _1: p2 }
                } else {
                    if ((__list_head).value == p2) {
                        TemplateArgumentBinding::NoOp
                    } else {
                        TemplateArgumentBinding::Conflict
                    }
                }
            } else {
                push_template_argument_binding_at(__list_tail, p1, p2, (p3 + 1))
            }
        }
    }
}
pub fn template_arguments_match(
    p0: &[TemplateArgument],
    p1: &[TemplateArgument],
) -> TemplateArgumentsMatch {
    match p0 {
        [] => template_arguments_rhs_empty(p1),
        [__list_head, __list_tail @ ..] => match &(template_argument_cursor(p1)) {
            TemplateArgumentCursor::CursorEnd => TemplateArgumentsMatch::Mismatch,
            TemplateArgumentCursor::CursorHead {
                head: __cur_head,
                tail: __cur_tail,
            } => {
                if ((__list_head).parameter == (__cur_head).parameter) {
                    if ((__list_head).value == (__cur_head).value) {
                        template_arguments_match(__list_tail, __cur_tail)
                    } else {
                        TemplateArgumentsMatch::Mismatch
                    }
                } else {
                    TemplateArgumentsMatch::Mismatch
                }
            }
        },
    }
}
pub fn template_arguments_rhs_empty(p0: &[TemplateArgument]) -> TemplateArgumentsMatch {
    match p0 {
        [] => TemplateArgumentsMatch::Match,
        [__list_head, __list_tail @ ..] => TemplateArgumentsMatch::Mismatch,
    }
}
pub fn template_argument_cursor(p0: &[TemplateArgument]) -> TemplateArgumentCursor {
    match p0 {
        [] => TemplateArgumentCursor::CursorEnd,
        [__list_head, __list_tail @ ..] => TemplateArgumentCursor::CursorHead {
            head: (__list_head).clone(),
            tail: (__list_tail).to_vec(),
        },
    }
}
pub fn filter_non_self_template_arguments(p0: &[TemplateArgument]) -> Vec<TemplateArgument> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).parameter == (__list_head).value) {
                filter_non_self_template_arguments(__list_tail)
            } else {
                {
                    let mut __list = filter_non_self_template_arguments(__list_tail);
                    __list.insert(0, (__list_head).clone());
                    __list
                }
            }
        }
    }
}
pub fn normalize_instantiation_arguments(p0: &TypeConnective) -> NormalizedInstantiationArgs {
    match p0 {
        TypeConnective::Atom(_) => NormalizedInstantiationArgs::NotInstantiation,
        TypeConnective::Conj { children: _ } => NormalizedInstantiationArgs::NotInstantiation,
        TypeConnective::Disj { variants: _ } => NormalizedInstantiationArgs::NotInstantiation,
        TypeConnective::Arrow {
            inputs: _,
            output: _,
            body: _,
        } => NormalizedInstantiationArgs::NotInstantiation,
        TypeConnective::Cardinality {
            element: _,
            bound: _,
        } => NormalizedInstantiationArgs::NotInstantiation,
        TypeConnective::Instantiation {
            template: __payload_template,
            arguments: __payload_arguments,
        } => NormalizedInstantiationArgs::Normalized {
            template: (*(__payload_template)),
            args: filter_non_self_template_arguments(__payload_arguments),
        },
    }
}
