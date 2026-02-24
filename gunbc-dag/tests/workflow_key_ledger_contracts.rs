//! Workflow key contracts (WF3).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, plan_workflow, required_input_contract,
    required_output_contract, PlanAction, PlannerInputs, PlannerWorkflowSpec, ProcessUnitRef,
    ProcessUnitRegistry, WorkflowId, WorkflowOp, WorkflowPlannerError, WorkflowUnit,
};
use gunbc_ir::{Dag, Node, NodeId};

fn temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-workflow-key-contracts-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[test]
fn keys_are_deterministic_for_fixed_inputs() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let inputs = PlannerInputs::new();

    let first = plan_workflow(&spec, &registry, &inputs, &root).expect("first plan");
    let second = plan_workflow(&spec, &registry, &inputs, &root).expect("second plan");

    let first_digests = first
        .nodes
        .iter()
        .map(|node| (node.node_id.clone(), node.key.digest.clone()))
        .collect::<Vec<_>>();
    let second_digests = second
        .nodes
        .iter()
        .map(|node| (node.node_id.clone(), node.key.digest.clone()))
        .collect::<Vec<_>>();
    assert_eq!(first_digests, second_digests);
}

#[test]
fn all_nodes_always_execute() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let inputs = PlannerInputs::new();

    let plan = plan_workflow(&spec, &registry, &inputs, &root).expect("plan");
    for node in &plan.nodes {
        assert!(
            matches!(node.action, PlanAction::Execute { .. }),
            "all nodes should be Execute"
        );
    }
}

#[test]
fn unknown_process_units_fail_planner_admission() {
    let root = temp_root();
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "wf.unknown",
        required_input_contract(),
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
            "wf",
            "wf.unknown",
        ))),
    ));
    let spec = PlannerWorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
    let registry = ProcessUnitRegistry::new();

    let err = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root)
        .expect_err("unknown process unit should fail planner");
    assert!(matches!(
        err,
        WorkflowPlannerError::UnknownProcessUnit { node_id, .. } if node_id == NodeId::from("wf.unknown")
    ));
}

#[test]
fn concurrent_planning_calls_remain_deterministic() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let baseline = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root)
        .expect("baseline plan should succeed")
        .nodes
        .into_iter()
        .map(|node| (node.node_id, node.key.digest))
        .collect::<Vec<_>>();

    let mut handles = Vec::new();
    for _ in 0..6 {
        let spec = spec.clone();
        let registry = registry.clone();
        let root = root.clone();
        handles.push(thread::spawn(move || {
            plan_workflow(&spec, &registry, &PlannerInputs::new(), &root)
                .expect("concurrent plan should succeed")
                .nodes
                .into_iter()
                .map(|node| (node.node_id, node.key.digest))
                .collect::<Vec<_>>()
        }));
    }

    for handle in handles {
        let digests = handle.join().expect("planning thread should not panic");
        assert_eq!(digests, baseline);
    }
}
