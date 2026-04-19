// AUTO-GENERATED from `src/v3/lenses/structural_resolution.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct UnresolvedArrowBody {
    pub declaration: DeclarationId,
    pub name: String,
    pub span: SourceSpan,
}
#[derive(Clone, Debug)]
pub struct NameKeyedReference {
    pub declaration: DeclarationId,
    pub resolved_to: DeclarationId,
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
        TypeConnective::Atom(_) => Vec::new(),
        TypeConnective::Conj { children: _ } => Vec::new(),
        TypeConnective::Disj { variants: _ } => Vec::new(),
        TypeConnective::Arrow {
            inputs: __a_inputs,
            output: __a_output,
            body: __a_body,
        } => check_arrow_body(p0, (p1).clone(), __a_body),
        TypeConnective::Cardinality {
            element: _,
            bound: _,
        } => Vec::new(),
        TypeConnective::Instantiation {
            template: _,
            arguments: _,
        } => Vec::new(),
    }
}
pub fn check_arrow_body(p0: &Declaration, p1: String, p2: &ArrowBody) -> Vec<UnresolvedArrowBody> {
    match p2 {
        ArrowBody::UserDefined(_) => Vec::new(),
        ArrowBody::ExternalRealization(_) => Vec::new(),
        ArrowBody::Pending => singleton_violation(p0, (p1).clone()),
        ArrowBody::NoBody => Vec::new(),
        ArrowBody::Unparsed(_) => Vec::new(),
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
pub fn name_keyed_references(p0: &Dag) -> Vec<NameKeyedReference> {
    ((p0).declarations())
        .iter()
        .fold(Vec::new(), |__fold_acc, __fold_item| {
            prepend_name_keyed(
                &(check_name_keyed_declaration(__fold_item)),
                (__fold_acc).clone(),
            )
        })
}
pub fn check_name_keyed_declaration(p0: &Declaration) -> Vec<NameKeyedReference> {
    match &((p0).connective) {
        TypeConnective::Atom(payload) => check_name_keyed_payload(p0, payload),
        TypeConnective::Conj { children: _ } => Vec::new(),
        TypeConnective::Disj { variants: _ } => Vec::new(),
        TypeConnective::Arrow {
            inputs: _,
            output: _,
            body: _,
        } => Vec::new(),
        TypeConnective::Cardinality {
            element: _,
            bound: _,
        } => Vec::new(),
        TypeConnective::Instantiation {
            template: _,
            arguments: _,
        } => Vec::new(),
    }
}
pub fn check_name_keyed_payload(p0: &Declaration, p1: &AtomPayload) -> Vec<NameKeyedReference> {
    match p1 {
        AtomPayload::Literal(_) => Vec::new(),
        AtomPayload::UnresolvedIdentifier(_) => Vec::new(),
        AtomPayload::ResolvedByStructure(_) => Vec::new(),
        AtomPayload::ResolvedByName(id) => {
            let mut __list = Vec::new();
            __list.insert(
                0,
                NameKeyedReference {
                    declaration: (p0).id,
                    resolved_to: (*(id)),
                    span: ((p0).span).clone(),
                },
            );
            __list
        }
        AtomPayload::TypeParam(_) => Vec::new(),
    }
}
pub fn prepend_name_keyed(
    p0: &[NameKeyedReference],
    p1: Vec<NameKeyedReference>,
) -> Vec<NameKeyedReference> {
    (p0).iter().fold((p1).clone(), |__fold_acc, __fold_item| {
        let mut __list = (__fold_acc).clone();
        __list.insert(0, (__fold_item).clone());
        __list
    })
}
