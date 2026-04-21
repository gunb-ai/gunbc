// AUTO-GENERATED from `src/v3/lenses/infer_helpers.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub struct CallableTemplateInfo {
    pub template: DeclarationId,
    pub arguments: Vec<TemplateArgument>,
}
#[derive(Clone, Debug)]
pub enum OptionalCardinalityDeclLookup {
    MissingOptionalCardinality,
    FoundOptionalCardinality { _0: DeclarationId },
}
#[derive(Clone, Debug)]
pub enum TemplateArgumentLookup {
    MissingTemplateArgument,
    FoundTemplateArgument { _0: DeclarationId },
}
#[derive(Clone, Debug)]
pub enum DeclarationLookup {
    MissingDeclaration,
    FoundDeclaration { _0: Declaration },
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
pub fn find_declaration(p0: &[Declaration], p1: &DeclarationId) -> DeclarationLookup {
    match p0 {
        [] => DeclarationLookup::MissingDeclaration,
        [__list_head, __list_tail @ ..] => {
            if ((__list_head).id == (*(p1))) {
                DeclarationLookup::FoundDeclaration {
                    _0: (__list_head).clone(),
                }
            } else {
                find_declaration(__list_tail, p1)
            }
        }
    }
}
pub fn walk_to_optional_cardinality_decl(
    p0: &i64,
    p1: &Dag,
    p2: DeclarationId,
) -> OptionalCardinalityDeclLookup {
    if ((*(p0)) <= 0) {
        OptionalCardinalityDeclLookup::MissingOptionalCardinality
    } else {
        match &(find_declaration((p1).declarations(), &p2)) {
            DeclarationLookup::MissingDeclaration => {
                OptionalCardinalityDeclLookup::MissingOptionalCardinality
            }
            DeclarationLookup::FoundDeclaration { _0: decl } => match &((decl).connective) {
                TypeConnective::Cardinality {
                    element: __c_element,
                    bound: __c_bound,
                } => match __c_bound {
                    CardinalityBound::AtMostOne => {
                        OptionalCardinalityDeclLookup::FoundOptionalCardinality { _0: p2 }
                    }
                    CardinalityBound::Exact(_) => {
                        OptionalCardinalityDeclLookup::MissingOptionalCardinality
                    }
                    CardinalityBound::Unbounded => {
                        OptionalCardinalityDeclLookup::MissingOptionalCardinality
                    }
                },
                TypeConnective::Instantiation {
                    template: __i_template,
                    arguments: __i_arguments,
                } => walk_to_optional_cardinality_decl(&((*(p0)) - 1), p1, (*(__i_template))),
                TypeConnective::Atom(payload) => match payload {
                    AtomPayload::ResolvedByStructure(next) => {
                        walk_to_optional_cardinality_decl(&((*(p0)) - 1), p1, (*(next)))
                    }
                    AtomPayload::ResolvedByName(next) => {
                        walk_to_optional_cardinality_decl(&((*(p0)) - 1), p1, (*(next)))
                    }
                    AtomPayload::UnresolvedIdentifier(_) => {
                        OptionalCardinalityDeclLookup::MissingOptionalCardinality
                    }
                    AtomPayload::TypeParam(_) => {
                        OptionalCardinalityDeclLookup::MissingOptionalCardinality
                    }
                    AtomPayload::Literal(_) => {
                        OptionalCardinalityDeclLookup::MissingOptionalCardinality
                    }
                },
                TypeConnective::Conj { children: _ } => {
                    OptionalCardinalityDeclLookup::MissingOptionalCardinality
                }
                TypeConnective::Disj { variants: _ } => {
                    OptionalCardinalityDeclLookup::MissingOptionalCardinality
                }
                TypeConnective::Arrow {
                    inputs: _,
                    output: _,
                    body: _,
                } => OptionalCardinalityDeclLookup::MissingOptionalCardinality,
            },
        }
    }
}
pub fn callable_template_arguments(p0: &Dag, p1: DeclarationId) -> CallableTemplateInfo {
    match &(find_declaration((p0).declarations(), &p1)) {
        DeclarationLookup::MissingDeclaration => CallableTemplateInfo {
            template: p1,
            arguments: Vec::new(),
        },
        DeclarationLookup::FoundDeclaration { _0: decl } => match &((decl).connective) {
            TypeConnective::Instantiation {
                template: __i_template,
                arguments: __i_arguments,
            } => CallableTemplateInfo {
                template: (*(__i_template)),
                arguments: (__i_arguments).to_vec(),
            },
            TypeConnective::Atom(_) => CallableTemplateInfo {
                template: p1,
                arguments: Vec::new(),
            },
            TypeConnective::Conj { children: _ } => CallableTemplateInfo {
                template: p1,
                arguments: Vec::new(),
            },
            TypeConnective::Disj { variants: _ } => CallableTemplateInfo {
                template: p1,
                arguments: Vec::new(),
            },
            TypeConnective::Arrow {
                inputs: _,
                output: _,
                body: _,
            } => CallableTemplateInfo {
                template: p1,
                arguments: Vec::new(),
            },
            TypeConnective::Cardinality {
                element: _,
                bound: _,
            } => CallableTemplateInfo {
                template: p1,
                arguments: Vec::new(),
            },
        },
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
