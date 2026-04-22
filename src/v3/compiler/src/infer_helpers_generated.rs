// AUTO-GENERATED from `src/v3/lenses/infer_helpers.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum TemplateArgumentLookup {
    MissingTemplateArgument,
    FoundTemplateArgument { _0: DeclarationId },
}
#[derive(Clone, Debug)]
pub enum TemplateArgumentBinding {
    TemplateArgumentBindingConflict,
    TemplateArgumentsBound { _0: Vec<TemplateArgument> },
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
) -> TemplateArgumentLookup {
    match p0 {
        [] => TemplateArgumentLookup::MissingTemplateArgument,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).parameter == (*(p1))) {
                TemplateArgumentLookup::FoundTemplateArgument {
                    _0: (__list_head).value,
                }
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
            TemplateArgumentLookup::MissingTemplateArgument => p2,
            TemplateArgumentLookup::FoundTemplateArgument { _0: next } => {
                if ((*(next)) == p2) {
                    p2
                } else {
                    resolve_template_argument_value(&((*(p0)) - 1), p1, (*(next)))
                }
            }
        }
    }
}
pub fn template_arguments_match(p0: &[TemplateArgument], p1: &[TemplateArgument]) -> bool {
    match p0 {
        [] => match p1 {
            [] => true,
            [__list_head, __list_tail @ ..] => false,
        },
        [__list_head, __list_tail @ ..] => match p1 {
            [] => false,
            [__list_head, __list_tail @ ..] => {
                if ((__list_head).parameter == (__list_head).parameter) {
                    if ((__list_head).value == (__list_head).value) {
                        template_arguments_match(__list_tail, __list_tail)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        },
    }
}
pub fn push_template_argument_binding(
    p0: Vec<TemplateArgument>,
    p1: DeclarationId,
    p2: DeclarationId,
) -> TemplateArgumentBinding {
    match &p0 {
        [] => TemplateArgumentBinding::TemplateArgumentsBound {
            _0: {
                let mut __list = Vec::new();
                __list.insert(
                    0,
                    TemplateArgument {
                        parameter: p1,
                        value: p2,
                    },
                );
                __list
            },
        },
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).parameter == p1) {
                if ((__list_head).value == p1) {
                    TemplateArgumentBinding::TemplateArgumentsBound {
                        _0: {
                            let mut __list = (__list_tail).to_vec();
                            __list.insert(
                                0,
                                TemplateArgument {
                                    parameter: (__list_head).parameter,
                                    value: p2,
                                },
                            );
                            __list
                        },
                    }
                } else {
                    if ((__list_head).value == p2) {
                        TemplateArgumentBinding::TemplateArgumentsBound { _0: (p0).clone() }
                    } else {
                        TemplateArgumentBinding::TemplateArgumentBindingConflict
                    }
                }
            } else {
                match &(push_template_argument_binding((__list_tail).to_vec(), p1, p2)) {
                    TemplateArgumentBinding::TemplateArgumentBindingConflict => {
                        TemplateArgumentBinding::TemplateArgumentBindingConflict
                    }
                    TemplateArgumentBinding::TemplateArgumentsBound { _0: updated_tail } => {
                        TemplateArgumentBinding::TemplateArgumentsBound {
                            _0: {
                                let mut __list = (updated_tail).to_vec();
                                __list.insert(0, (__list_head).clone());
                                __list
                            },
                        }
                    }
                }
            }
        }
    }
}
