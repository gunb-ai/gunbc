// AUTO-GENERATED from `src/v3/lenses/variant_payload.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

#[derive(Clone, Debug)]
pub enum VariantPayloadShape {
    Empty,
    PositionalSingle,
    NamedFields { _0: Vec<String> },
}
#[derive(Clone, Debug)]
pub enum VariantPayloadShapeLookup {
    DeclarationMissing,
    NotPayloadProduct,
    Found { _0: VariantPayloadShape },
}
pub fn conj_field_label(p0: &Field) -> String {
    ((p0).label).clone()
}
pub fn conj_payload_shape(p0: &[Field]) -> VariantPayloadShape {
    match p0 {
        [] => VariantPayloadShape::Empty,
        [__list_head, __list_tail @ ..] => match __list_tail {
            [] => {
                if ((__list_head).label == String::from("_0")) {
                    VariantPayloadShape::PositionalSingle
                } else {
                    VariantPayloadShape::NamedFields {
                        _0: {
                            let mut __list = Vec::new();
                            __list.insert(0, ((__list_head).label).clone());
                            __list
                        },
                    }
                }
            }
            [__list_head, __list_tail @ ..] => VariantPayloadShape::NamedFields {
                _0: (p0)
                    .iter()
                    .map(|__map_item| ((__map_item).label).clone())
                    .collect::<Vec<_>>(),
            },
        },
    }
}
pub fn variant_payload_shape(p0: &Dag, p1: &DeclarationId) -> VariantPayloadShapeLookup {
    match &((p0).declaration_opt(p1).cloned()) {
        None => VariantPayloadShapeLookup::DeclarationMissing,
        Some(decl) => match &((decl).connective) {
            TypeConnective::Conj { children: c } => VariantPayloadShapeLookup::Found {
                _0: conj_payload_shape(c),
            },
            TypeConnective::Atom(_) => VariantPayloadShapeLookup::NotPayloadProduct,
            TypeConnective::Disj { variants: _ } => VariantPayloadShapeLookup::NotPayloadProduct,
            TypeConnective::Arrow {
                inputs: _,
                output: _,
                body: _,
            } => VariantPayloadShapeLookup::NotPayloadProduct,
            TypeConnective::Cardinality {
                element: _,
                bound: _,
            } => VariantPayloadShapeLookup::NotPayloadProduct,
            TypeConnective::Instantiation {
                template: _,
                arguments: _,
            } => VariantPayloadShapeLookup::NotPayloadProduct,
        },
    }
}
