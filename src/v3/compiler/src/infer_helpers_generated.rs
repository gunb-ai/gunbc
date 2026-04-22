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
    TemplateArgumentBindingNoOp,
    TemplateArgumentBindingAppend,
    TemplateArgumentBindingReplaceAt { _0: i64, _1: DeclarationId },
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
        [] => TemplateArgumentBinding::TemplateArgumentBindingAppend,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).parameter == (*(p1))) {
                if ((__list_head).value == (*(p1))) {
                    TemplateArgumentBinding::TemplateArgumentBindingReplaceAt { _0: p3, _1: p2 }
                } else {
                    if ((__list_head).value == p2) {
                        TemplateArgumentBinding::TemplateArgumentBindingNoOp
                    } else {
                        TemplateArgumentBinding::TemplateArgumentBindingConflict
                    }
                }
            } else {
                push_template_argument_binding_at(__list_tail, p1, p2, (p3 + 1))
            }
        }
    }
}
