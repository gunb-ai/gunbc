//! Structural receipts and small lenses for substrate integration tests.
//! Keeps `m1_substrate_test` focused on claims instead of repeating the same
//! `nodes().iter().filter_map` scaffolding.

use std::collections::HashMap;

use v3_compiler::dag::{
    ArrowBody, Behavior, BindNode, Dag, DeclarationId, Field, PortState, TemplateArgument,
    TransformNode, TransformTarget, TypeConnective,
};

pub fn find_named(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

pub fn field<'a>(fields: &'a [Field], label: &str) -> &'a Field {
    fields
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("field `{label}` not found"))
}

pub fn bind_value_type_decl(dag: &Dag, name: &str) -> DeclarationId {
    let value_port = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Bind(bind) if bind.name == name => Some(bind.value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("bind `{name}` not found"));
    match dag.port(value_port).state() {
        PortState::Resolved(ty) => ty.declaration,
        other => panic!("bind `{name}` did not resolve, got {other:?}"),
    }
}

pub fn callable_instantiation_arguments(
    dag: &Dag,
    template: DeclarationId,
) -> Vec<&[TemplateArgument]> {
    dag.nodes()
        .iter()
        .filter_map(|node| {
            let Behavior::Transform(transform) = node else {
                return None;
            };
            let TransformTarget::Callable(target) = transform.target else {
                return None;
            };
            let TypeConnective::Instantiation {
                template: inst_template,
                arguments,
            } = &dag.declaration(target).connective
            else {
                return None;
            };
            (*inst_template == template).then_some(arguments.as_slice())
        })
        .collect()
}

pub fn walk_instantiation_chain(
    dag: &Dag,
    start: DeclarationId,
    subst: &mut HashMap<DeclarationId, DeclarationId>,
) -> DeclarationId {
    let mut current = start;
    for _ in 0..16 {
        match &dag.declaration(current).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                for arg in arguments {
                    subst.insert(arg.parameter, arg.value);
                }
                current = *template;
            }
            _ => return current,
        }
    }
    current
}

pub fn transforms_in_source_file<'a>(
    dag: &'a Dag,
    file: &'a str,
) -> impl Iterator<Item = &'a TransformNode> + 'a {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_transform)
        .filter(move |t| t.span.file == file)
}

pub fn bind_named<'a>(dag: &'a Dag, name: &str) -> &'a BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("Bind({name}) not found"))
}

/// Receipt: the fixed-width `Int64` row instantiates to `OrderedRing`; `.add` is binary
/// `(T,T)->T` with `NoBody`, and template substitution lines operands up with `Word64`.
///
/// **T-Numeric-Construction Slice 3 pivot.** Pre-Slice-3 this receipt walked the default
/// `Int` alias (which used to be `Int = Int64`). Slice 3 pivots the default alias to the
/// construction-chain shape `Int = AbelianGroup<GroupCompletion<Nat>>` (per
/// `docs/audit/t-numeric-construction-group-completion-6q.md`); the fixed-width
/// `Int64 = OrderedRing<Word64>` row stays intact at `dsl/std/integer.dag`. The
/// `OrderedRing<Word64>` chain is now reachable through the `Int64` name directly,
/// and that's what this receipt continues to pin. The default `Int` alias has its
/// own ratchet (`int_default_alias_resolves_to_abelian_group_over_group_completion_of_nat`).
pub fn assert_bootstrap_int_ordered_ring_add_arrow(dag: &Dag) {
    let int64_id = find_named(dag, "Int64");
    let word64_id = find_named(dag, "Word64");
    let ordered_ring_id = find_named(dag, "OrderedRing");

    let mut subst = HashMap::new();
    let algebra_id = walk_instantiation_chain(dag, int64_id, &mut subst);
    assert_eq!(
        algebra_id, ordered_ring_id,
        "Int64 fixed-width row must still terminate at OrderedRing (legacy storage chain stays intact in Slice 3)"
    );

    let ordered_ring_fields = match &dag.declaration(ordered_ring_id).connective {
        TypeConnective::Conj { children } => children,
        other => panic!("OrderedRing should be a Conj, got {other:?}"),
    };
    let add_field = field(ordered_ring_fields, "add");
    let (inputs, output, body) = match &dag.declaration(add_field.ty).connective {
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => (inputs, output, body),
        other => panic!("OrderedRing.add should be an Arrow, got {other:?}"),
    };
    assert!(
        matches!(body, ArrowBody::NoBody),
        "bootstrap algebra arrows must stay NoBody so Pending remains an R13 leak signal"
    );
    assert_eq!(inputs.len(), 2, "OrderedRing.add should stay binary");

    let substitute = |id: DeclarationId| -> DeclarationId { *subst.get(&id).unwrap_or(&id) };
    assert_eq!(substitute(inputs[0]), word64_id);
    assert_eq!(substitute(inputs[1]), word64_id);
    assert_eq!(substitute(*output), word64_id);
}
