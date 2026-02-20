//! Workflow key + ledger contracts (WF3).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeMap;
use std::fs;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    append_global_ledger_entry, ci_workflow_spec, default_process_unit_registry, plan_workflow,
    required_input_contract, required_output_contract, store_output_payload, LedgerStatus,
    MissReason, PlanAction, PlannerInputs, ProcessUnitRef, ProcessUnitRegistry, RunLedgerEntry,
    PlannerWorkflowSpec, WorkflowId, WorkflowOp, WorkflowPlannerError, WorkflowUnit,
};
use gunbc_ir::{Dag, Node, NodeId, PortName, Value};

fn temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-workflow-key-ledger-contracts-{}-{}",
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
fn key_drift_produces_typed_miss_reason() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();

    let mut first_inputs = PlannerInputs::new();
    first_inputs.insert(
        NodeId::from("ci.codegen"),
        BTreeMap::from([(PortName::from("after"), Value::Str("alpha".to_string()))]),
    );
    let first_plan = plan_workflow(&spec, &registry, &first_inputs, &root).expect("first plan");
    let node = first_plan
        .nodes
        .iter()
        .find(|entry| entry.node_id == NodeId::from("ci.codegen"))
        .expect("ci.codegen plan entry")
        .clone();

    let payload = Value::Str("cached".to_string());
    let hash = store_output_payload(&root, &payload).expect("store payload");
    append_global_ledger_entry(
        &root,
        RunLedgerEntry {
            exec_node_id: node.node_id.clone(),
            work_id: node.work_id.clone(),
            key: node.key.clone(),
            status: LedgerStatus::CachedHit {
                previous_run: "run-alpha".to_string(),
            },
            output_hashes: BTreeMap::from([(PortName::from("result"), hash)]),
            duration_ms: 1,
        },
    )
    .expect("append ledger entry");

    let mut second_inputs = PlannerInputs::new();
    second_inputs.insert(
        NodeId::from("ci.codegen"),
        BTreeMap::from([(PortName::from("after"), Value::Str("beta".to_string()))]),
    );
    let second_plan = plan_workflow(&spec, &registry, &second_inputs, &root).expect("second plan");
    let node = second_plan
        .nodes
        .iter()
        .find(|entry| entry.node_id == NodeId::from("ci.codegen"))
        .expect("ci.codegen plan entry");
    assert!(matches!(
        node.action,
        PlanAction::Execute {
            miss_reason: MissReason::InputChanged { .. }
        }
    ));
}

#[test]
fn cached_hits_rehydrate_result_outputs() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let inputs = PlannerInputs::new();

    let initial = plan_workflow(&spec, &registry, &inputs, &root).expect("initial plan");
    let node = initial
        .nodes
        .iter()
        .find(|entry| entry.node_id == NodeId::from("ci.codegen"))
        .expect("ci.codegen initial entry")
        .clone();

    let payload = Value::Map(BTreeMap::from([("ok".to_string(), Value::Bool(true))]));
    let hash = store_output_payload(&root, &payload).expect("store payload");
    append_global_ledger_entry(
        &root,
        RunLedgerEntry {
            exec_node_id: node.node_id.clone(),
            work_id: node.work_id.clone(),
            key: node.key.clone(),
            status: LedgerStatus::CachedHit {
                previous_run: "run-cached".to_string(),
            },
            output_hashes: BTreeMap::from([(PortName::from("result"), hash)]),
            duration_ms: 1,
        },
    )
    .expect("append cached ledger entry");

    let replanned = plan_workflow(&spec, &registry, &inputs, &root).expect("replanned");
    let node = replanned
        .nodes
        .iter()
        .find(|entry| entry.node_id == NodeId::from("ci.codegen"))
        .expect("ci.codegen replanned entry");
    let PlanAction::CachedHit {
        rehydrated_outputs, ..
    } = &node.action
    else {
        panic!("ci.codegen should be cache-hit with rehydrated outputs");
    };
    assert_eq!(
        rehydrated_outputs.get(&PortName::from("result")),
        Some(&payload)
    );
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
fn corrupted_ledger_fails_closed() {
    let root = temp_root();
    let ledger_root = root.join(".gunbc").join("workflow-ledger");
    fs::create_dir_all(&ledger_root).expect("create workflow ledger dir");
    fs::write(ledger_root.join("global.ndjson"), b"{not valid json\n")
        .expect("write corrupt global ledger");

    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let err = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root)
        .expect_err("corrupt ledger should fail planning");
    assert!(matches!(err, WorkflowPlannerError::Ledger(_)));
}

#[test]
fn corrupted_cached_payload_fails_with_ledger_error() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let initial = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root)
        .expect("initial plan should succeed");
    let codegen = initial
        .nodes
        .iter()
        .find(|node| node.node_id == NodeId::from("ci.codegen"))
        .expect("ci.codegen should exist")
        .clone();

    let ledger_root = root.join(".gunbc").join("workflow-ledger");
    fs::create_dir_all(ledger_root.join("cas")).expect("create cas dir");
    fs::write(ledger_root.join("cas").join("broken-payload.json"), b"not-json")
        .expect("write invalid cas payload");

    append_global_ledger_entry(
        &root,
        RunLedgerEntry {
            exec_node_id: codegen.node_id.clone(),
            work_id: codegen.work_id.clone(),
            key: codegen.key.clone(),
            status: LedgerStatus::CachedHit {
                previous_run: "run-bad-cas".to_string(),
            },
            output_hashes: BTreeMap::from([(PortName::from("result"), "broken-payload".into())]),
            duration_ms: 1,
        },
    )
    .expect("append corrupted cached entry");

    let err = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root)
        .expect_err("invalid cas payload should fail planner");
    assert!(matches!(err, WorkflowPlannerError::Ledger(_)));
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
