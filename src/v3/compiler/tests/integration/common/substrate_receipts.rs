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

fn peel_zero_arg_alias(dag: &Dag, mut current: DeclarationId) -> DeclarationId {
    for _ in 0..16 {
        match &dag.declaration(current).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => current = *template,
            _ => return current,
        }
    }
    current
}

fn assert_compose_with_machine_width(
    dag: &Dag,
    alias_name: &str,
    algebra_name: &str,
    width_name: &str,
) {
    let alias_id = peel_zero_arg_alias(dag, find_named(dag, alias_name));
    let compose_id = find_named(dag, "Compose");
    let algebra_id = find_named(dag, algebra_name);
    let machine_width_id = find_named(dag, "MachineWidth");
    let width_id = find_named(dag, width_name);

    let connective = &dag.declaration(alias_id).connective;
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = connective
    else {
        panic!("{alias_name} must resolve to a Compose instantiation, got {connective:?}");
    };
    assert_eq!(*template, compose_id);
    assert_eq!(
        arguments.len(),
        2,
        "Compose<{algebra_name}, MachineWidth<…>> takes two arguments"
    );

    let mut saw_algebra = false;
    let mut saw_machine_width = false;
    for arg in arguments {
        if arg.value == algebra_id {
            saw_algebra = true;
            continue;
        }
        if let TypeConnective::Instantiation {
            template: mw_template,
            arguments: mw_args,
        } = &dag.declaration(arg.value).connective
        {
            if *mw_template == machine_width_id
                && mw_args.len() == 1
                && mw_args[0].value == width_id
            {
                saw_machine_width = true;
            }
        }
    }
    assert!(
        saw_algebra,
        "{alias_name} Compose must instantiate {algebra_name}"
    );
    assert!(
        saw_machine_width,
        "{alias_name} Compose must include MachineWidth<{width_name}>"
    );
}

/// Receipt: `Int64` is a width refinement `Compose<Int, MachineWidth<Word64>>` (R3 gate #19),
/// not parallel `OrderedRing<Word64>` substrate. Abstract `Int` is
/// `AbelianGroup<GroupCompletion<Nat>>` (Slice 3); fixed-width names compose it with
/// the machine-width axis.
pub fn assert_bootstrap_int64_compose_int_machine_width(dag: &Dag) {
    assert_compose_with_machine_width(dag, "Int64", "Int", "Word64");
}

/// Receipt: `Real64` refines abstract `Real` with `MachineWidth<Word64>` (R3 gate #67).
pub fn assert_bootstrap_real64_compose_real_machine_width(dag: &Dag) {
    assert_compose_with_machine_width(dag, "Real64", "Real", "Word64");
}

/// Receipt: compatibility `Float64` names the same fixed-width `Real64` construction.
pub fn assert_bootstrap_float64_aliases_real64(dag: &Dag) {
    let float64_id = peel_zero_arg_alias(dag, find_named(dag, "Float64"));
    let real64_id = peel_zero_arg_alias(dag, find_named(dag, "Real64"));
    assert_eq!(
        float64_id, real64_id,
        "Float64 should alias the canonical Real64 construction entry"
    );
}

/// Gate #67 demonstration receipt: both construction-chain endpoints are present.
pub fn assert_numeric_construction_demonstration_gate_67(dag: &Dag) {
    assert_compose_with_machine_width(dag, "Int32", "Int", "Word32");
    assert_bootstrap_real64_compose_real_machine_width(dag);
}
