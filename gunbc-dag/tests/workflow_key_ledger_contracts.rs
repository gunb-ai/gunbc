//! Workflow key + ledger contracts (WF3).

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    append_global_ledger_entry, ci_workflow_spec, default_process_unit_registry, plan_workflow,
    store_output_payload, LedgerStatus, MissReason, PlanAction, PlannerInputs, RunLedgerEntry,
};
use gunbc_ir::{NodeId, PortName, Value};

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
