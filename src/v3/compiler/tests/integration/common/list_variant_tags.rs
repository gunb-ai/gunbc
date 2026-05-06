//! Locate existing `List<elem>.Empty` / `List<elem>.Cons` `Instantiation` rows emitted by lowering.
//!
//! Integration tests assemble substrate-shaped [`v3_compiler::evaluator::Value`] carriers
//! without mutating the [`v3_compiler::dag::Dag`] (no `push_declaration`). Any `List<τ>.Empty`
//! / `Cons` tag must therefore reuse a declaration id that already exists in the compiled graph.

use v3_compiler::dag::{AtomPayload, Dag, DeclarationId, TypeConnective};

fn peel_alias_equal(dag: &Dag, mut lhs: DeclarationId, mut rhs: DeclarationId) -> bool {
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        if lhs == rhs {
            return true;
        }
        let lhs_next = match &dag.declaration(lhs).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => Some(*template),
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => Some(*next),
            _ => None,
        };
        let rhs_next = match &dag.declaration(rhs).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => Some(*template),
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => Some(*next),
            _ => None,
        };
        match (lhs_next, rhs_next) {
            (Some(a), Some(b)) => {
                lhs = a;
                rhs = b;
            }
            (Some(a), None) => lhs = a,
            (None, Some(b)) => rhs = b,
            (None, None) => return false,
        }
    }
    false
}

/// Returns the `DeclarationId` for `List<elem>.Cons` matching `list_ty` (`List<elem>`).
///
/// Mirrors [`find_list_empty_constructor_tag`]: the lowered graph must already contain an
/// `Instantiation` row for this `Cons` at the same element type (the fixture seeds one via
/// `Cons { head: …, tail: … }` in expression position).
pub fn find_list_cons_constructor_tag(dag: &Dag, list_ty: DeclarationId) -> DeclarationId {
    let list_decl = dag
        .declaration_by_name("List")
        .expect("bootstrap must define `List`");
    let list_id = list_decl.id;
    let cons_arm_ty = match &list_decl.connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|v| v.label == "Cons")
                .expect("List.Cons arm")
                .ty
        }
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
            if *template != cons_arm_ty {
                return None;
            }
            let elem_ok = |arg: DeclarationId| peel_alias_equal(dag, arg, elem_value);
            let matches = match arguments.as_slice() {
                [a] => elem_ok(a.value),
                [a, b] => {
                    (a.value == elem_value && b.value == list_ty)
                        || (elem_ok(a.value) && b.value == list_ty)
                        || (a.value == list_ty && elem_ok(b.value))
                }
                _ => false,
            };
            matches.then_some(decl.id)
        })
        .unwrap_or_else(|| {
            let mut any_cons: Vec<String> = Vec::new();
            for decl in dag.declarations() {
                if let TypeConnective::Instantiation {
                    template,
                    arguments,
                } = &decl.connective
                {
                    if *template == cons_arm_ty {
                        any_cons.push(format!(
                            "args={:?}",
                            arguments.iter().map(|a| a.value).collect::<Vec<_>>()
                        ));
                    }
                }
            }
            panic!(
                "no existing `Instantiation` for List<..>.Cons with element type {elem_value:?}; \
                 cons_arm_ty={cons_arm_ty:?} list_ty={list_ty:?}. Any Cons-template rows: {any_cons:?}"
            )
        })
}

/// Returns the `DeclarationId` for `List<elem>.Empty` matching `list_ty` (`List<elem>`).
pub fn find_list_empty_constructor_tag(dag: &Dag, list_ty: DeclarationId) -> DeclarationId {
    let list_decl = dag
        .declaration_by_name("List")
        .expect("bootstrap must define `List`");
    let list_id = list_decl.id;
    let empty_arm_ty = match &list_decl.connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|v| v.label == "Empty")
                .expect("List.Empty arm")
                .ty
        }
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
