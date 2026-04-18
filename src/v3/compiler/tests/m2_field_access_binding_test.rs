use std::collections::HashMap;

use v3_compiler::dag::{FieldValue, TypeConnective, ValueBody};
use v3_compiler::Dag;

fn find_named(dag: &Dag, name: &str) -> v3_compiler::dag::DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn type_realization_fields_for(
    dag: &Dag,
    target_name: &str,
) -> HashMap<String, (String, String, bool)> {
    let type_realization_meta = find_named(dag, "TypeRealization");
    let target = find_named(dag, target_name);
    let realization = dag
        .declarations()
        .iter()
        .find(|decl| {
            decl.name
                .as_deref()
                .is_some_and(|name| name.starts_with("rust_"))
                && decl.meta_tag == Some(type_realization_meta)
                && matches!(
                    &decl.value_body,
                    Some(ValueBody::Structural { fields })
                        if matches!(
                            fields.iter().find(|(label, _)| label == "target").map(|(_, value)| value),
                            Some(FieldValue::Reference(id)) if *id == target
                        )
                )
        })
        .unwrap_or_else(|| panic!("TypeRealization for `{target_name}` not found"));
    let ValueBody::Structural { fields } = realization.value_body.as_ref().expect("value body")
    else {
        unreachable!()
    };
    let list = fields
        .iter()
        .find(|(label, _)| label == "fields")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("TypeRealization `{target_name}` missing `fields` entry"));
    let FieldValue::List(entries) = list else {
        panic!("TypeRealization `{target_name}` `fields` entry must be a list");
    };

    let mut out = HashMap::new();
    for entry in entries {
        let FieldValue::Record(fields) = entry else {
            panic!("FieldBinding entry must be a record");
        };
        let dag_name = fields
            .iter()
            .find(|(label, _)| label == "dag_name")
            .and_then(|(_, value)| match value {
                FieldValue::Literal(v3_compiler::dag::LiteralBits::String(s)) => Some(s.clone()),
                _ => None,
            })
            .expect("FieldBinding.dag_name string");
        let access = fields
            .iter()
            .find(|(label, _)| label == "access")
            .map(|(_, value)| value)
            .expect("FieldBinding.access");
        let FieldValue::Variant {
            constructor,
            payload,
        } = access
        else {
            panic!("FieldBinding.access must be a FieldAccess variant");
        };
        let borrowed_read = fields
            .iter()
            .find(|(label, _)| label == "borrowed_read")
            .and_then(|(_, value)| match value {
                FieldValue::Literal(v3_compiler::dag::LiteralBits::Bool(value)) => Some(*value),
                _ => None,
            })
            .unwrap_or(false);
        let [FieldValue::Literal(v3_compiler::dag::LiteralBits::String(name))] = &payload[..]
        else {
            panic!("FieldAccess payload must be a single String literal");
        };
        out.insert(
            dag_name,
            (
                variant_label(dag, *constructor),
                name.clone(),
                borrowed_read,
            ),
        );
    }
    out
}

fn variant_label(dag: &Dag, variant_id: v3_compiler::dag::DeclarationId) -> String {
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == variant_id)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "variant declaration {:?} not found under any reflected sum",
                variant_id
            )
        })
}

fn declared_record_fields(dag: &Dag, type_name: &str) -> Vec<String> {
    let id = find_named(dag, type_name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|field| field.label.clone()).collect()
        }
        other => panic!("expected `{type_name}` to be a Conj, got {other:?}"),
    }
}

#[test]
fn every_realized_reflection_record_field_has_a_binding() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load rust.dag cleanly: {:?}",
        dag.diagnostics()
    );

    for type_name in [
        "FieldEntry",
        "TypeShape",
        "DagPort",
        "SourceSpan",
        "Dag",
        "Declaration",
        "TemplateArgument",
        "PayloadBinding",
        "BranchPath",
        "NonEmptyList",
        "NonSingletonList",
        "ParamRef",
        "TransformRef",
        "MemberDescent",
        "IntraClusterCall",
        "Cluster",
        "ValueNode",
        "TransformNode",
        "BranchNode",
        "LoopNode",
        "BindNode",
    ] {
        let declared = declared_record_fields(&dag, type_name);
        let realized = type_realization_fields_for(&dag, type_name);
        for field in declared {
            assert!(
                realized.contains_key(&field),
                "TypeRealization `{type_name}` is missing a FieldBinding for `{field}`"
            );
        }
    }
}

#[test]
fn alias_bindings_cover_method_backed_fields() {
    let dag = Dag::new();
    let dag_fields = type_realization_fields_for(&dag, "Dag");
    assert_eq!(
        dag_fields.get("nodes"),
        Some(&(String::from("AccessorMethod"), String::from("nodes"), true))
    );
    assert_eq!(
        dag_fields.get("declarations"),
        Some(&(
            String::from("AccessorMethod"),
            String::from("declarations"),
            true
        ))
    );
    assert_eq!(
        dag_fields.get("ports"),
        Some(&(String::from("AccessorMethod"), String::from("ports"), false))
    );
    assert_eq!(
        dag_fields.get("clusters"),
        Some(&(String::from("AccessorMethod"), String::from("clusters"), true))
    );

    let entry_fields = type_realization_fields_for(&dag, "FieldEntry");
    assert_eq!(
        entry_fields.get("label"),
        Some(&(String::from("DirectField"), String::from("0"), false))
    );
    assert_eq!(
        entry_fields.get("value"),
        Some(&(String::from("DirectField"), String::from("1"), false))
    );

    let bind_fields = type_realization_fields_for(&dag, "BindNode");
    assert_eq!(
        bind_fields.get("result_port"),
        Some(&(
            String::from("AccessorMethod"),
            String::from("result_port"),
            false
        ))
    );

    let value_fields = type_realization_fields_for(&dag, "ValueNode");
    assert_eq!(
        value_fields.get("payload"),
        Some(&(String::from("DirectField"), String::from("data"), false))
    );

    let span_fields = type_realization_fields_for(&dag, "SourceSpan");
    assert_eq!(
        span_fields.get("start"),
        Some(&(
            String::from("DirectField"),
            String::from("byte_start"),
            false
        ))
    );
    assert_eq!(
        span_fields.get("end"),
        Some(&(String::from("DirectField"), String::from("byte_end"), false))
    );

    let port_fields = type_realization_fields_for(&dag, "DagPort");
    assert_eq!(
        port_fields.get("id"),
        Some(&(String::from("AccessorMethod"), String::from("id"), false))
    );
    assert_eq!(
        port_fields.get("state"),
        Some(&(
            String::from("AccessorMethod"),
            String::from("state_value"),
            false
        ))
    );
}
