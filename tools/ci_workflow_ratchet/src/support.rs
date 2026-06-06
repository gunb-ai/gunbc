//! Thin re-exports for integration-style ratchet tests.

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};

pub use v3_compiler::{parse_for_test, tokenize_for_test};

/// Locate existing `List<elem>.Empty` `Instantiation` rows emitted by lowering.
pub fn find_list_empty_constructor_tag(dag: &Dag, list_ty: DeclarationId) -> DeclarationId {
    let list_decl = dag
        .declaration_by_name("List")
        .expect("bootstrap must define `List`");
    let list_id = list_decl.id;
    let empty_arm_ty = match &list_decl.connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|v| v.label == "Empty")
            .expect("List.Empty arm")
            .ty,
        other => panic!("`List` must be a Disj, got {other:?}"),
    };
    let elem_value = match &dag.declaration(list_ty).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if *template == list_id && arguments.len() == 1 => arguments[0].value,
        other => {
            panic!("expected `List<elem>` instantiation for list_ty={list_ty:?}, got {other:?}")
        }
    };
    dag.declarations()
        .iter()
        .find_map(|decl| {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                return None;
            };
            (*template == empty_arm_ty && arguments.len() == 1 && arguments[0].value == elem_value)
                .then_some(decl.id)
        })
        .unwrap_or_else(|| {
            panic!(
                "no existing `Instantiation` for List<..>.Empty with element type {elem_value:?}; \
                 lower at least one `Empty` expression at that list type in the same compile unit"
            )
        })
}
