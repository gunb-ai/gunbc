// AUTO-GENERATED from `src/v3/lenses/infer_helpers.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum TemplateArgumentLookup {
    MissingTemplateArgument,
    FoundTemplateArgument { _0: DeclarationId },
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
