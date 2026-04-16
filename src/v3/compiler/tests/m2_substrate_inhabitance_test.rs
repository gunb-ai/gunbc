use v3_compiler::dag::{FieldValue, TypeConnective, ValueBody};
use v3_compiler::Dag;

fn find_named(dag: &Dag, name: &str) -> v3_compiler::dag::DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn record_fields(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => children.iter().map(|field| field.label.clone()).collect(),
        other => panic!("expected `{name}` to lower to a Conj, got {other:?}"),
    }
}

#[test]
fn substrate_declares_expected_reflection_surface() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load substrate reflection files cleanly: {:?}",
        dag.diagnostics()
    );

    assert_eq!(
        record_fields(&dag, "Declaration"),
        vec![
            "id",
            "name",
            "connective",
            "type_params",
            "meta_tag",
            "inhabits",
            "value_body",
            "span",
        ]
    );
    assert_eq!(record_fields(&dag, "PayloadBinding"), vec!["binding_name", "payload_port"]);
    assert_eq!(
        record_fields(&dag, "BranchPath"),
        vec!["body", "result_port", "pattern", "binding"]
    );
    assert_eq!(record_fields(&dag, "LoopBound"), vec!["count"]);
    assert_eq!(
        record_fields(&dag, "ValueNode"),
        vec!["id", "payload", "result_port", "span"]
    );
    assert_eq!(
        record_fields(&dag, "TransformNode"),
        vec!["id", "target", "inputs", "result_port", "span"]
    );
    assert_eq!(
        record_fields(&dag, "BranchNode"),
        vec!["id", "input", "paths", "result_port", "span"]
    );
    assert_eq!(
        record_fields(&dag, "LoopNode"),
        vec!["id", "source", "init", "body", "bound", "result_port", "span"]
    );
    assert_eq!(
        record_fields(&dag, "BindNode"),
        vec!["id", "name", "result_port", "params", "span"]
    );
    assert_eq!(record_fields(&dag, "Dag"), vec!["declarations", "nodes"]);
}

#[test]
fn rust_dag_realizes_reflected_substrate_types() {
    let dag = Dag::new();
    let type_realization_meta = find_named(&dag, "TypeRealization");
    for name in [
        "Dag",
        "Declaration",
        "TemplateArgument",
        "PayloadBinding",
        "BranchPath",
        "LoopBound",
        "ValueNode",
        "TransformNode",
        "BranchNode",
        "LoopNode",
        "BindNode",
    ] {
        let target = find_named(&dag, name);
        let realized = dag.declarations().iter().find(|decl| {
            decl.meta_tag == Some(type_realization_meta)
                && matches!(
                    &decl.value_body,
                    Some(ValueBody::Structural { fields })
                        if matches!(
                            fields.iter().find(|(label, _)| label == "target").map(|(_, value)| value),
                            Some(FieldValue::Reference(id)) if *id == target
                        )
                )
        });
        assert!(
            realized.is_some(),
            "expected a TypeRealization entry targeting `{name}`"
        );
    }
}
