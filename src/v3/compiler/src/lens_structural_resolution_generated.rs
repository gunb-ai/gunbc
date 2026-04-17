// AUTO-GENERATED from `src/v3/lenses/structural_resolution.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct UnresolvedArrowBody {
    pub declaration: DeclarationId,
    pub name: String,
    pub span: SourceSpan,
}
pub fn check(p0: &Dag) -> Vec<UnresolvedArrowBody> {
    ((p0).declarations())
        .iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            prepend_all(&(check_declaration(__fold_item)), (__fold_acc).clone())
        })
}
pub fn check_declaration(p0: &Declaration) -> Vec<UnresolvedArrowBody> {
    match &((p0).name) {
        None => Vec::new(),
        Some(name_str) => check_named_connective(p0, (name_str).clone()),
    }
}
pub fn check_named_connective(p0: &Declaration, p1: String) -> Vec<UnresolvedArrowBody> {
    match &((p0).connective) {
        TypeConnective::Atom(payload) => Vec::new(),
        TypeConnective::Conj { children: c } => Vec::new(),
        TypeConnective::Disj { variants: d } => Vec::new(),
        a @ TypeConnective::Arrow {
            inputs: __a_inputs,
            output: __a_output,
            body: __a_body,
        } => check_arrow_body(p0, (p1).clone(), __a_body),
        c @ TypeConnective::Cardinality {
            element: __c_element,
            bound: __c_bound,
        } => Vec::new(),
        i @ TypeConnective::Instantiation {
            template: __i_template,
            arguments: __i_arguments,
        } => Vec::new(),
    }
}
pub fn check_arrow_body(p0: &Declaration, p1: String, p2: &ArrowBody) -> Vec<UnresolvedArrowBody> {
    match p2 {
        ArrowBody::UserDefined(node_id) => Vec::new(),
        ArrowBody::ExternalRealization(decl_id) => Vec::new(),
        ArrowBody::Pending => singleton_violation(p0, (p1).clone()),
        ArrowBody::Unparsed(span) => Vec::new(),
    }
}
pub fn singleton_violation(p0: &Declaration, p1: String) -> Vec<UnresolvedArrowBody> {
    {
        let mut __list = Vec::new();
        __list.insert(
            0,
            UnresolvedArrowBody {
                declaration: (p0).id,
                name: (p1).clone(),
                span: ((p0).span).clone(),
            },
        );
        __list
    }
}
pub fn prepend_all(
    p0: &[UnresolvedArrowBody],
    p1: Vec<UnresolvedArrowBody>,
) -> Vec<UnresolvedArrowBody> {
    (p0).iter().fold((p1).clone(), |__fold_acc, __fold_item| {
        let mut __list = (__fold_acc).clone();
        __list.insert(0, (__fold_item).clone());
        __list
    })
}
