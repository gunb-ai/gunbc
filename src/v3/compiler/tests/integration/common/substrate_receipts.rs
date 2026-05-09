//! Structural receipts and small lenses for substrate integration tests.
//! Keeps `m1_substrate_test` focused on claims instead of repeating the same
//! `nodes().iter().filter_map` scaffolding.

use std::collections::HashMap;

use v3_compiler::dag::{
    Behavior, BindNode, Dag, DeclarationId, Field, PortState, TemplateArgument, TransformNode,
    TransformTarget, TypeConnective,
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

/// Receipt: `Int64` is a width refinement `Compose<Int, MachineWidth<Word64>>` (R3 gate #19),
/// not parallel `OrderedRing<Word64>` substrate. Abstract `Int` is
/// `AbelianGroup<GroupCompletion<Nat>>` (Slice 3); fixed-width names compose it with
/// the machine-width axis.
pub fn assert_bootstrap_int64_compose_int_machine_width(dag: &Dag) {
    let int64_id = find_named(dag, "Int64");
    let compose_id = find_named(dag, "Compose");
    let int_id = find_named(dag, "Int");
    let machine_width_id = find_named(dag, "MachineWidth");
    let word64_id = find_named(dag, "Word64");

    let connective = &dag.declaration(int64_id).connective;
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = connective
    else {
        panic!("Int64 must be a Compose instantiation, got {connective:?}");
    };
    assert_eq!(*template, compose_id);
    assert_eq!(
        arguments.len(),
        2,
        "Compose<Int, MachineWidth<…>> takes two arguments"
    );

    let mut saw_int = false;
    let mut saw_mw_word64 = false;
    for arg in arguments {
        if arg.value == int_id {
            saw_int = true;
            continue;
        }
        if let TypeConnective::Instantiation {
            template: mw_template,
            arguments: mw_args,
        } = &dag.declaration(arg.value).connective
        {
            if *mw_template == machine_width_id
                && mw_args.len() == 1
                && mw_args[0].value == word64_id
            {
                saw_mw_word64 = true;
            }
        }
    }
    assert!(
        saw_int,
        "Int64 Compose must instantiate abstract Int (construction-chain carrier)"
    );
    assert!(
        saw_mw_word64,
        "Int64 Compose must include MachineWidth<Word64>"
    );
}

/// Receipt: `Float64` refines opaque `Ieee754Float` with `MachineWidth<Word64>` (R3 gate #19).
pub fn assert_bootstrap_float64_compose_ieee_machine_width(dag: &Dag) {
    let float64_id = find_named(dag, "Float64");
    let compose_id = find_named(dag, "Compose");
    let ieee_id = find_named(dag, "Ieee754Float");
    let machine_width_id = find_named(dag, "MachineWidth");
    let word64_id = find_named(dag, "Word64");

    let connective = &dag.declaration(float64_id).connective;
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = connective
    else {
        panic!("Float64 must be a Compose instantiation, got {connective:?}");
    };
    assert_eq!(*template, compose_id);
    assert_eq!(arguments.len(), 2);

    let mut saw_ieee = false;
    let mut saw_mw_word64 = false;
    for arg in arguments {
        if arg.value == ieee_id {
            saw_ieee = true;
            continue;
        }
        if let TypeConnective::Instantiation {
            template: mw_template,
            arguments: mw_args,
        } = &dag.declaration(arg.value).connective
        {
            if *mw_template == machine_width_id
                && mw_args.len() == 1
                && mw_args[0].value == word64_id
            {
                saw_mw_word64 = true;
            }
        }
    }
    assert!(
        saw_ieee,
        "Float64 Compose must instantiate Ieee754Float axis"
    );
    assert!(
        saw_mw_word64,
        "Float64 Compose must include MachineWidth<Word64>"
    );
}
