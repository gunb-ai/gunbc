//! PR-A.1 runtime Value coproduct carrier.
//!
//! This pins the observable runtime `Value` declaration from
//! `src/v3/std/runtime.dag` against the PB-Runtime / PR-A.0 locked shape.
//! Evaluator-internal state carriers are intentionally out of scope.

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn decl_id(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"))
        .id
}

fn variant_payload(dag: &Dag, variant: &str) -> DeclarationId {
    let value = dag
        .declaration_by_name("Value")
        .expect("runtime Value missing from full bootstrap");
    match &value.connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|field| field.label == variant)
                .unwrap_or_else(|| panic!("Value missing `{variant}` variant"))
                .ty
        }
        other => panic!("runtime Value is not a Disj: {other:?}"),
    }
}

fn conj_field(dag: &Dag, name: &str, field_name: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    conj_field_by_id(dag, decl.id, field_name)
}

fn conj_field_by_id(dag: &Dag, id: DeclarationId, field_name: &str) -> DeclarationId {
    let decl = dag.declaration(id);
    match &decl.connective {
        TypeConnective::Conj { children } => {
            children
                .iter()
                .find(|field| field.label == field_name)
                .unwrap_or_else(|| panic!("declaration {id:?} missing `{field_name}` field"))
                .ty
        }
        other => panic!("declaration {id:?} is not a Conj: {other:?}"),
    }
}

fn positional_payload(dag: &Dag, id: DeclarationId) -> DeclarationId {
    conj_field_by_id(dag, id, "_0")
}

fn assert_instantiation(dag: &Dag, actual: DeclarationId, template: &str, argument: &str) {
    let expected_template = decl_id(dag, template);
    let expected_argument = decl_id(dag, argument);
    match &dag.declaration(actual).connective {
        TypeConnective::Instantiation {
            template: actual_template,
            arguments,
        } => {
            assert_eq!(
                *actual_template, expected_template,
                "expected instantiation template `{template}`"
            );
            assert_eq!(arguments.len(), 1, "expected one template argument");
            assert_eq!(
                arguments[0].value, expected_argument,
                "expected template argument `{argument}`"
            );
        }
        other => panic!("expected Instantiation, got {other:?}"),
    }
}

#[test]
fn runtime_value_has_locked_pb_runtime_coproduct_shape() {
    let dag = generated_full_bootstrap_dag();

    let value = dag
        .declaration_by_name("Value")
        .expect("runtime Value missing from full bootstrap");
    assert_eq!(
        value.span.file, "src/v3/std/runtime.dag",
        "bare Value must be the runtime carrier, not an L1 behavior marker"
    );

    let variants = match &value.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("runtime Value is not a Disj: {other:?}"),
    };
    let labels: Vec<&str> = variants.iter().map(|field| field.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "LiteralValue",
            "RecordValue",
            "VariantValue",
            "NodeRef",
            "CardinalityValue"
        ],
        "runtime Value coproduct drifted from PB-Runtime section 3.2"
    );

    assert_eq!(
        positional_payload(&dag, variant_payload(&dag, "LiteralValue")),
        decl_id(&dag, "LiteralBits")
    );
    assert_instantiation(
        &dag,
        positional_payload(&dag, variant_payload(&dag, "RecordValue")),
        "List",
        "NamedField",
    );
    assert_eq!(
        positional_payload(&dag, variant_payload(&dag, "NodeRef")),
        decl_id(&dag, "NodeId")
    );
    assert_eq!(
        positional_payload(&dag, variant_payload(&dag, "CardinalityValue")),
        decl_id(&dag, "LoopBound")
    );

    let variant_value = variant_payload(&dag, "VariantValue");
    assert_eq!(
        conj_field_by_id(&dag, variant_value, "tag"),
        decl_id(&dag, "DeclarationId")
    );
    assert_eq!(conj_field_by_id(&dag, variant_value, "payload"), value.id);
}

#[test]
fn named_field_uses_runtime_value_payload() {
    let dag = generated_full_bootstrap_dag();
    assert_eq!(
        conj_field(&dag, "NamedField", "label"),
        decl_id(&dag, "String")
    );
    assert_eq!(
        conj_field(&dag, "NamedField", "value"),
        decl_id(&dag, "Value")
    );
}

#[test]
fn value_behavior_marker_remains_distinct_from_runtime_value() {
    let dag = generated_full_bootstrap_dag();
    let runtime_value = decl_id(&dag, "Value");
    let value_behavior = decl_id(&dag, "ValueBehavior");
    assert_ne!(
        runtime_value, value_behavior,
        "runtime Value must not alias the L1 ValueBehavior marker"
    );
    assert_eq!(
        dag.value_marker(),
        Some(value_behavior),
        "Dag::value_marker() must keep returning the L1 behavior marker"
    );
}
