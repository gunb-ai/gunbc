//! **Layer:** integration
//!
//! Producer-first ratchets for `src/v3/lenses/dag_shape.dag`.

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{
    ArrowBody, Behavior, Dag, DeclarationId, TransformTarget, TypeConnective, ValueBody,
};

fn dag_shape_dag() -> Dag {
    cached_compile_to_dag(
        include_str!("../../../lenses/dag_shape.dag"),
        "src/v3/lenses/dag_shape.dag",
    )
}

#[test]
fn dag_shape_report_carrier_projects_reflected_dag_shape_lists() {
    let dag = dag_shape_dag();
    let report = dag
        .declaration_by_name("DagShapeReport")
        .expect("DagShapeReport carrier exists");
    let TypeConnective::Conj { children } = &report.connective else {
        panic!("DagShapeReport must be a record carrier");
    };
    let labels: Vec<&str> = children.iter().map(|field| field.label.as_str()).collect();
    assert_eq!(labels, ["declarations", "nodes", "ports", "clusters"]);

    for field in children {
        let TypeConnective::Instantiation { template, .. } = &dag.declaration(field.ty).connective
        else {
            panic!("DagShapeReport.{} must be a List<...>", field.label);
        };
        assert_eq!(
            dag.declaration(*template).name.as_deref(),
            Some("List"),
            "DagShapeReport.{} must be list-shaped",
            field.label
        );
    }
}

#[test]
fn dag_shape_report_public_producer_returns_shape_report() {
    let dag = dag_shape_dag();
    let report = dag
        .declaration_by_name("DagShapeReport")
        .expect("DagShapeReport carrier exists")
        .id;

    assert_arrow_output(&dag, "dag_shape_report", report);
}

#[test]
fn dag_shape_producer_helpers_are_lens_shaped_without_fake_data_instance() {
    let dag = dag_shape_dag();
    let report = dag
        .declaration_by_name("DagShapeReport")
        .expect("DagShapeReport carrier exists")
        .id;
    let witness = dag
        .declaration_by_name("Witness")
        .expect("Witness carrier exists")
        .id;
    let optional_diagnostic = dag
        .declaration_by_name("OptionalDiagnostic")
        .expect("OptionalDiagnostic carrier exists")
        .id;

    assert_arrow_output_instantiation(&dag, "dag_shape_read", witness, report);
    assert_dag_shape_read_consumes_whole_dag_report(&dag);
    assert_arrow_output(&dag, "combine_dag_shape_reports", report);
    assert_arrow_output(&dag, "dag_shape_branch", report);
    assert_arrow_output(&dag, "dag_shape_iterate", report);
    assert_arrow_output(&dag, "dag_shape_validate", optional_diagnostic);

    let empty = dag
        .declaration_by_name("empty_dag_shape_report")
        .expect("empty_dag_shape_report data exists");
    assert!(
        matches!(empty.value_body, Some(ValueBody::Structural { .. })),
        "empty report value should lower structurally"
    );
    assert!(
        dag.declaration_by_name("dag_shape_lens").is_none(),
        "do not author fake Lens<DagShapeReport> data until generic function-field validation lands"
    );
}

fn assert_dag_shape_read_consumes_whole_dag_report(dag: &Dag) {
    let read = dag
        .declaration_by_name("dag_shape_read")
        .expect("dag_shape_read declaration exists");
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = read.connective
    else {
        panic!("dag_shape_read must be a user-defined arrow");
    };
    let bind = dag
        .node(bind_id)
        .as_bind()
        .expect("dag_shape_read body is a Bind");
    let witness_transform = match dag.port(bind.value).produced_by {
        Some(node_id) => match dag.node(node_id) {
            Behavior::Transform(transform) => transform,
            other => panic!("dag_shape_read body should build a Witness, got {other:?}"),
        },
        None => panic!("dag_shape_read body must have a producer"),
    };
    let report_port = *witness_transform
        .inputs
        .first()
        .expect("Inhabits should receive the report value");
    let report_transform = match dag.port(report_port).produced_by {
        Some(node_id) => match dag.node(node_id) {
            Behavior::Transform(transform) => transform,
            other => panic!("dag_shape_read witness input should be a transform, got {other:?}"),
        },
        None => panic!("dag_shape_read witness input must have a producer"),
    };
    let TransformTarget::Callable(target) = report_transform.target else {
        panic!("dag_shape_read should call dag_shape_report");
    };
    assert_eq!(
        dag.declaration(target).name.as_deref(),
        Some("dag_shape_report"),
        "dag_shape_read must preserve the whole-Dag producer authority"
    );
}

fn assert_arrow_output(dag: &Dag, name: &str, expected: DeclarationId) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("{name} declaration exists"));
    let TypeConnective::Arrow { output, body, .. } = &decl.connective else {
        panic!("{name} must be an Arrow");
    };
    assert_eq!(*output, expected, "{name} output drifted");
    assert!(
        matches!(body, ArrowBody::UserDefined(_)),
        "{name} should lower to a user-defined body"
    );
}

fn assert_arrow_output_instantiation(
    dag: &Dag,
    name: &str,
    expected_template: DeclarationId,
    expected_arg: DeclarationId,
) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("{name} declaration exists"));
    let TypeConnective::Arrow { output, body, .. } = &decl.connective else {
        panic!("{name} must be an Arrow");
    };
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &dag.declaration(*output).connective
    else {
        panic!("{name} output must be an instantiation");
    };
    assert_eq!(
        *template, expected_template,
        "{name} output template drifted"
    );
    assert!(
        arguments.iter().any(|arg| arg.value == expected_arg),
        "{name} output must be instantiated with DagShapeReport"
    );
    assert!(
        matches!(body, ArrowBody::UserDefined(_)),
        "{name} should lower to a user-defined body"
    );
}
