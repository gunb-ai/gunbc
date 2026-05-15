//! **Layer:** integration
//!
//! R3 gate #82 (`effect_enumeration_lens_behaviorally_complete`) Band-C receipt.
//!
//! The generated effect-enumeration lens must consume the canonical
//! `v3.std.services::Operation` / `std.effects::operation_effect_shape` authority
//! for operation rows, not only infer effects from raw `Dag.nodes` callable shape.
//! These tests pin the public generated adapter until the same carrier assertions
//! can move into `.dag` `TestClaim` data.

use std::collections::BTreeMap;

use v3_compiler::dag::{
    operation_effect_shape, BreakingShape, CallableRef, CreateCause, EffectShape, HttpMethodScalar,
    IdempotentShape, InputField, Operation, PathTemplate, RestEndpointBinding, UrlPathToken,
};
use v3_compiler::lens_effect_enumeration::{
    operation_structural_effect_shape, StructuralEffectShape,
};
use v3_compiler::Dag;

fn op(
    dag: &Dag,
    callable_name: &str,
    method: HttpMethodScalar,
    tokens: Vec<UrlPathToken>,
) -> Operation {
    let callable = dag
        .declaration_by_name(callable_name)
        .unwrap_or_else(|| panic!("missing callable declaration `{callable_name}`"))
        .id;
    let inputs = tokens
        .iter()
        .filter_map(|token| match token {
            UrlPathToken::ParamToken { name } => Some((name.clone(), InputField {})),
            UrlPathToken::LiteralToken { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    Operation {
        callable: CallableRef { decl: callable },
        inputs,
        endpoint: RestEndpointBinding {
            method,
            path: PathTemplate { tokens },
        },
    }
}

fn op_with_inputs(
    dag: &Dag,
    callable_name: &str,
    method: HttpMethodScalar,
    tokens: Vec<UrlPathToken>,
    input_names: &[&str],
) -> Operation {
    let callable = dag
        .declaration_by_name(callable_name)
        .unwrap_or_else(|| panic!("missing callable declaration `{callable_name}`"))
        .id;
    let inputs = input_names
        .iter()
        .map(|name| ((*name).to_string(), InputField {}))
        .collect::<BTreeMap<_, _>>();
    Operation {
        callable: CallableRef { decl: callable },
        inputs,
        endpoint: RestEndpointBinding {
            method,
            path: PathTemplate { tokens },
        },
    }
}

#[test]
fn effect_enumeration_lens_behaviorally_complete_classifies_operation_reads() {
    let dag = Dag::new();
    let read = op(&dag, "get_method", HttpMethodScalar::Get, vec![]);

    assert!(matches!(
        operation_structural_effect_shape(&read),
        StructuralEffectShape::ReadShaped
    ));
}

#[test]
fn effect_enumeration_lens_behaviorally_complete_classifies_operation_writes() {
    let dag = Dag::new();
    let write = op(
        &dag,
        "map_insert_method",
        HttpMethodScalar::Put,
        vec![UrlPathToken::ParamToken {
            name: "id".to_string(),
        }],
    );

    assert!(matches!(
        operation_structural_effect_shape(&write),
        StructuralEffectShape::WriteShaped
    ));
}

#[test]
fn effect_enumeration_lens_behaviorally_complete_classifies_breaking_operations_as_writes() {
    let dag = Dag::new();
    let append = op(&dag, "append_method", HttpMethodScalar::Post, vec![]);
    let create = op(&dag, "concat_method", HttpMethodScalar::Post, vec![]);

    assert!(matches!(
        operation_structural_effect_shape(&append),
        StructuralEffectShape::WriteShaped
    ));
    assert!(matches!(
        operation_structural_effect_shape(&create),
        StructuralEffectShape::WriteShaped
    ));
}

#[test]
fn effect_enumeration_lens_behaviorally_complete_uses_callable_authority_over_transport() {
    let dag = Dag::new();
    let read_over_post = op(&dag, "get_method", HttpMethodScalar::Post, vec![]);
    let write_over_get = op(
        &dag,
        "map_insert_method",
        HttpMethodScalar::Get,
        vec![UrlPathToken::ParamToken {
            name: "id".to_string(),
        }],
    );
    let breaking_over_get = op(&dag, "append_method", HttpMethodScalar::Get, vec![]);

    assert!(matches!(
        operation_structural_effect_shape(&read_over_post),
        StructuralEffectShape::ReadShaped
    ));
    assert!(matches!(
        operation_structural_effect_shape(&write_over_get),
        StructuralEffectShape::WriteShaped
    ));
    assert!(matches!(
        operation_structural_effect_shape(&breaking_over_get),
        StructuralEffectShape::WriteShaped
    ));
}

#[test]
fn effect_enumeration_lens_behaviorally_complete_fails_closed_for_unknown_callable() {
    let dag = Dag::new();
    let unknown = op(&dag, "Int", HttpMethodScalar::Get, vec![]);

    assert!(matches!(
        operation_structural_effect_shape(&unknown),
        StructuralEffectShape::UnknownEffect { .. }
    ));
    assert!(operation_effect_shape(&dag, &unknown).is_none());
}

#[test]
fn effect_enumeration_lens_behaviorally_complete_does_not_use_path_without_input_authority() {
    let dag = Dag::new();
    let path_only_key = op_with_inputs(
        &dag,
        "map_insert_method",
        HttpMethodScalar::Put,
        vec![UrlPathToken::ParamToken {
            name: "id".to_string(),
        }],
        &[],
    );
    let input_only_key = op_with_inputs(
        &dag,
        "map_insert_method",
        HttpMethodScalar::Put,
        vec![],
        &["id"],
    );

    assert!(matches!(
        operation_effect_shape(&dag, &path_only_key),
        Some(EffectShape::IsBreaking(BreakingShape::CreateEffect {
            cause: CreateCause::KeylessFallback {
                method: HttpMethodScalar::Put
            }
        }))
    ));
    assert!(matches!(
        operation_effect_shape(&dag, &input_only_key),
        Some(EffectShape::IsBreaking(BreakingShape::CreateEffect {
            cause: CreateCause::KeylessFallback {
                method: HttpMethodScalar::Put
            }
        }))
    ));

    let coupled_key = op(
        &dag,
        "map_insert_method",
        HttpMethodScalar::Put,
        vec![UrlPathToken::ParamToken {
            name: "id".to_string(),
        }],
    );
    assert!(matches!(
        operation_effect_shape(&dag, &coupled_key),
        Some(EffectShape::IsIdempotent(
            IdempotentShape::UpsertEffect { .. }
        ))
    ));
}
