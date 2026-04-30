//! **Layer:** integration
//!
//! Producer-first ratchets for `src/v3/lenses/dag_shape.dag`.

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{FieldValue, TypeConnective, ValueBody};

fn dag_shape_dag() -> v3_compiler::dag::Dag {
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
fn dag_shape_lens_data_instance_lowers_structurally() {
    let dag = dag_shape_dag();
    let lens = dag
        .declaration_by_name("dag_shape_lens")
        .expect("dag_shape_lens data instance exists");
    let Some(ValueBody::Structural { fields }) = &lens.value_body else {
        panic!("dag_shape_lens must lower to a structural data body");
    };
    let labels: Vec<&str> = fields.iter().map(|(label, _)| label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "name",
            "read",
            "sequential",
            "branch",
            "iterate",
            "validate"
        ]
    );

    for expected in [
        "dag_shape_read",
        "dag_shape_report_monoid",
        "dag_shape_branch",
        "dag_shape_iterate",
        "dag_shape_validate",
    ] {
        let decl = dag
            .declaration_by_name(expected)
            .unwrap_or_else(|| panic!("{expected} declaration exists"));
        assert!(
            fields.iter().any(|(_, value)| matches!(
                value,
                FieldValue::Reference(id) if *id == decl.id
            )),
            "dag_shape_lens must reference `{expected}` structurally"
        );
    }
}
