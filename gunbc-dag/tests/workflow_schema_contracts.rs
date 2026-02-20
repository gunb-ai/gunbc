//! Workflow schema contract tests (WF1).

use std::collections::BTreeSet;

use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, has_required_unit_contract,
    test_all_workflow_spec, ProcessUnitRegistry, WorkflowOp,
};

fn assert_workflow_schema_contracts(
    spec: &gunbc_dag::PlannerWorkflowSpec,
    expected_node_prefix: &str,
    expected_process_namespace: &str,
    registry: &ProcessUnitRegistry,
) {
    assert!(
        !spec.dag.nodes.is_empty(),
        "workflow '{}' should contain at least one node",
        spec.id.0
    );

    let ids = spec
        .dag
        .nodes
        .iter()
        .map(|node| node.id.0.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids.len(),
        spec.dag.nodes.len(),
        "workflow '{}' contains duplicate node IDs",
        spec.id.0
    );

    let mut has_report_node = false;
    let mut invoke_count = 0usize;
    for node in &spec.dag.nodes {
        assert!(
            node.id.0.starts_with(expected_node_prefix),
            "node '{}' drifted outside expected workflow prefix '{}'",
            node.id.0,
            expected_node_prefix
        );
        assert!(
            has_required_unit_contract(&node.inputs, &node.outputs),
            "node '{}' missing required workflow unit contract",
            node.id.0
        );

        let gunbc_ir::NodeBody::Opaque(unit) = &node.body else {
            panic!("workflow node '{}' must use opaque typed unit", node.id.0);
        };
        match &unit.op {
            WorkflowOp::InvokeProcessUnit(process_ref) => {
                invoke_count += 1;
                assert_eq!(
                    process_ref.process_id.0, expected_process_namespace,
                    "invoke node '{}' should stay in '{}' process namespace",
                    node.id.0, expected_process_namespace
                );
                assert!(
                    registry.contains(process_ref),
                    "invoke node '{}' references non-registered process unit '{}::{}'",
                    node.id.0,
                    process_ref.process_id.0,
                    process_ref.unit_id.0
                );
            }
            WorkflowOp::Aggregate(_) => {}
            WorkflowOp::Report(_) => has_report_node = true,
        }
    }

    assert!(has_report_node, "workflow '{}' must include a report node", spec.id.0);
    assert!(
        invoke_count > 0,
        "workflow '{}' must include at least one invoke node",
        spec.id.0
    );
}

#[test]
fn ci_workflow_spec_satisfies_schema_contracts() {
    let spec = ci_workflow_spec().expect("ci workflow spec should build");
    let registry = default_process_unit_registry();
    assert_workflow_schema_contracts(&spec, "ci.", "ci", &registry);
}

#[test]
fn test_all_workflow_spec_satisfies_schema_contracts() {
    let spec = test_all_workflow_spec().expect("test-all workflow spec should build");
    let registry = default_process_unit_registry();
    assert_workflow_schema_contracts(&spec, "test_all.", "test_all", &registry);
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
