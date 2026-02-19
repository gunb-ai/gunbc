//! Workflow schema contract tests (WF1).

use std::collections::BTreeSet;

use gunbc_dag::{ci_workflow_spec, has_required_unit_contract, test_all_workflow_spec, WorkflowOp};

#[test]
fn ci_workflow_spec_has_expected_units_and_contracts() {
    let spec = ci_workflow_spec().expect("ci workflow spec should build");

    let ids = spec
        .dag
        .nodes
        .iter()
        .map(|node| node.id.0.clone())
        .collect::<BTreeSet<_>>();
    let expected = [
        "ci.lint_upsert",
        "ci.codegen",
        "ci.bootstrap",
        "ci.pragma",
        "ci.testgen",
        "ci.build_compile",
        "ci.test_run",
        "ci.clippy_run",
        "ci.guardrails",
        "ci.verify",
        "ci.report",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(ids, expected, "ci workflow unit topology drifted");

    for node in &spec.dag.nodes {
        assert!(
            has_required_unit_contract(&node.inputs, &node.outputs),
            "node '{}' missing required workflow unit contract",
            node.id.0
        );
    }
}

#[test]
fn test_all_workflow_spec_has_expected_units_and_contracts() {
    let spec = test_all_workflow_spec().expect("test-all workflow spec should build");

    let ids = spec
        .dag
        .nodes
        .iter()
        .map(|node| node.id.0.clone())
        .collect::<BTreeSet<_>>();
    let expected = [
        "test_all.lint_upsert",
        "test_all.codegen",
        "test_all.testgen",
        "test_all.build_compile",
        "test_all.verify_fix",
        "test_all.cargo_test_xl",
        "test_all.report",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(ids, expected, "test-all workflow unit topology drifted");

    for node in &spec.dag.nodes {
        assert!(
            has_required_unit_contract(&node.inputs, &node.outputs),
            "node '{}' missing required workflow unit contract",
            node.id.0
        );
    }
}

#[test]
fn planner_schema_uses_typed_ops_not_shell_fallback() {
    let ci = ci_workflow_spec().expect("ci workflow spec should build");
    for node in &ci.dag.nodes {
        let gunbc_ir::NodeBody::Opaque(unit) = &node.body else {
            panic!("workflow unit '{}' must be opaque typed unit", node.id.0);
        };
        match &unit.op {
            WorkflowOp::InvokeProcessUnit(_) | WorkflowOp::Aggregate(_) | WorkflowOp::Report(_) => {
            }
        }
    }
}
