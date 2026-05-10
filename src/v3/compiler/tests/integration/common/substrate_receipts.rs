//! Structural receipts and small lenses for substrate integration tests.
//! Keeps `m1_substrate_test` focused on claims instead of repeating the same
//! `nodes().iter().filter_map` scaffolding.

use std::collections::HashMap;

use v3_compiler::dag::{
    AtomPayload, Behavior, BindNode, Dag, DeclarationId, Field, PortState, TemplateArgument,
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
    panic!("zero-arg alias chain exceeded receipt peel bound at {current:?}")
}

fn resolve_atom_alias(dag: &Dag, mut current: DeclarationId) -> DeclarationId {
    for _ in 0..16 {
        match &dag.declaration(current).connective {
            TypeConnective::Atom(AtomPayload::ResolvedByName(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => current = *next,
            _ => return current,
        }
    }
    panic!("atom alias chain exceeded receipt peel bound at {current:?}")
}

fn assert_single_arg_instantiation(
    dag: &Dag,
    alias_name: &str,
    template_name: &str,
    argument: DeclarationId,
    message: &str,
) {
    let alias_id = resolve_atom_alias(dag, find_named(dag, alias_name));
    let template_id = find_named(dag, template_name);
    let connective = &dag.declaration(alias_id).connective;
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = connective
    else {
        panic!("{alias_name} must resolve to {template_name}<…>; got {connective:?}");
    };
    assert_eq!(*template, template_id, "{message}");
    assert_eq!(
        arguments.len(),
        1,
        "{template_name}<…> takes one carrier argument in this construction receipt"
    );
    assert_eq!(arguments[0].value, argument, "{message}");
}

/// Receipt: `Nat` is the natural-number carrier `CommutativeSemiring<Magnitude>`.
pub fn assert_bootstrap_nat_is_commutative_semiring_magnitude(dag: &Dag) {
    let magnitude_id = find_named(dag, "Magnitude");
    assert_single_arg_instantiation(
        dag,
        "Nat",
        "CommutativeSemiring",
        magnitude_id,
        "Nat must be CommutativeSemiring<Magnitude>",
    );
}

/// Receipt: abstract `Int` is `AbelianGroup<GroupCompletion<Nat>>`.
pub fn assert_bootstrap_int_is_group_completion_of_nat(dag: &Dag) {
    let nat_id = find_named(dag, "Nat");
    let group_completion_id = find_named(dag, "GroupCompletion");
    let int_id = resolve_atom_alias(dag, find_named(dag, "Int"));
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &dag.declaration(int_id).connective
    else {
        panic!("Int must resolve to AbelianGroup<GroupCompletion<Nat>>");
    };
    assert_eq!(
        *template,
        find_named(dag, "AbelianGroup"),
        "Int must be AbelianGroup<GroupCompletion<Nat>>"
    );
    assert_eq!(arguments.len(), 1, "AbelianGroup<T> takes one carrier");
    match &dag.declaration(arguments[0].value).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, group_completion_id,
                "Int must complete Nat via GroupCompletion"
            );
            assert_eq!(arguments.len(), 1, "GroupCompletion<M> takes one carrier");
            assert_eq!(
                arguments[0].value, nat_id,
                "Int must use Nat as the completed carrier"
            );
        }
        other => panic!("Int carrier must be GroupCompletion<Nat>; got {other:?}"),
    }
}

/// Receipt: `Rational` is exact `Field<FieldOfFractions<Int>>`.
pub fn assert_bootstrap_rational_is_field_of_fractions_int(dag: &Dag) {
    let field_of_fractions_id = find_named(dag, "FieldOfFractions");
    let rational_id = resolve_atom_alias(dag, find_named(dag, "Rational"));
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &dag.declaration(rational_id).connective
    else {
        panic!("Rational must resolve to Field<FieldOfFractions<Int>>");
    };
    assert_eq!(
        *template,
        find_named(dag, "Field"),
        "Rational must be Field<FieldOfFractions<Int>>"
    );
    assert_eq!(arguments.len(), 1, "Field<F> takes one carrier");
    match &dag.declaration(arguments[0].value).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, field_of_fractions_id,
                "Rational must use FieldOfFractions as its carrier"
            );
            assert_eq!(arguments.len(), 1, "FieldOfFractions<R> takes one carrier");
            assert_eq!(
                arguments[0].value,
                find_named(dag, "Int"),
                "Rational must be over Int"
            );
        }
        other => panic!("Rational carrier must be FieldOfFractions<Int>; got {other:?}"),
    }
}

/// Receipt: `Real` is approximate `ApproximateField<FieldOfFractions<Int>>`.
pub fn assert_bootstrap_real_is_approximate_field_of_fractions_int(dag: &Dag) {
    let field_of_fractions_id = find_named(dag, "FieldOfFractions");
    let real_id = resolve_atom_alias(dag, find_named(dag, "Real"));
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &dag.declaration(real_id).connective
    else {
        panic!("Real must resolve to ApproximateField<FieldOfFractions<Int>>");
    };
    assert_eq!(
        resolve_atom_alias(dag, *template),
        find_named(dag, "ApproximateField"),
        "Real must be ApproximateField<FieldOfFractions<Int>>"
    );
    assert_eq!(arguments.len(), 1, "ApproximateField<F> takes one carrier");
    match &dag.declaration(arguments[0].value).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(
                *template, field_of_fractions_id,
                "Real must approximate the same FieldOfFractions<Int> carrier"
            );
            assert_eq!(arguments.len(), 1, "FieldOfFractions<R> takes one carrier");
            assert_eq!(arguments[0].value, find_named(dag, "Int"));
        }
        other => panic!("Real carrier must be FieldOfFractions<Int>; got {other:?}"),
    }
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

/// Receipt: `Int32` is a width refinement `Compose<Int, MachineWidth<Word32>>`.
pub fn assert_bootstrap_int32_compose_int_machine_width(dag: &Dag) {
    assert_compose_with_machine_width(dag, "Int32", "Int", "Word32");
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
