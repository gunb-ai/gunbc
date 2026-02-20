//! Workflow planner key/ledger integration (WF3).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use gunbc_exec::topo_sort;
use gunbc_ir::{canonical_edge_order, NodeBody, NodeId, PortName, Value};

use super::coordination::{coordination_status, BlockedReason, CoordinationStatus};
use super::key::{
    derive_miss_reason, CanonicalKeyPayload, MaterializationDigest, MaterializationKey, MissReason,
    WorkIdentity,
};
use super::ledger::{
    load_global_ledger, rehydrate_outputs_for_entry, LedgerStatus, RunLedgerEntry,
    WorkflowLedgerError,
};
use super::process_registry::{ProcessId, ProcessUnitRegistry};
use super::schema::{WorkflowOp, WorkflowSpec, WorkflowUnit, PORT_AFTER, PORT_RESULT};

/// Per-node planner action.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanAction {
    CachedHit {
        previous_run: String,
        rehydrated_outputs: BTreeMap<PortName, Value>,
    },
    Execute {
        miss_reason: MissReason,
    },
}

/// Node-level plan entry.
#[derive(Debug, Clone, PartialEq)]
pub struct NodePlan {
    pub node_id: NodeId,
    pub work_id: WorkIdentity,
    pub key: MaterializationKey,
    pub action: PlanAction,
}

/// Deterministic planner output for a workflow.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowPlan {
    pub nodes: Vec<NodePlan>,
    pub coordination: CoordinationStatus,
}

/// Planner dry-run strictness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryRunMode {
    Lenient,
    Strict,
}

/// Explainability projection for `--plan` output.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanExplain {
    pub execute_set: Vec<NodeId>,
    pub cache_hit_set: Vec<NodeId>,
    pub miss_reasons: BTreeMap<NodeId, MissReason>,
    pub blocked: BTreeMap<NodeId, Vec<BlockedReason>>,
    pub ready: Vec<NodeId>,
    pub critical_path: Vec<NodeId>,
}

/// Planner errors for WF3 key/ledger path.
#[derive(Debug)]
pub enum WorkflowPlannerError {
    Key(String),
    Ledger(WorkflowLedgerError),
    UnknownNode(NodeId),
    UnknownProcessUnit {
        node_id: NodeId,
        process_unit: super::process_registry::ProcessUnitRef,
    },
    StrictDryRunMissingInput {
        node_id: NodeId,
        port: PortName,
        trace: String,
    },
}

impl std::fmt::Display for WorkflowPlannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowPlannerError::Key(error) => write!(f, "workflow planner key error: {error}"),
            WorkflowPlannerError::Ledger(error) => {
                write!(f, "workflow planner ledger error: {error}")
            }
            WorkflowPlannerError::UnknownNode(node_id) => {
                write!(f, "workflow planner: unknown node '{}'", node_id.0)
            }
            WorkflowPlannerError::UnknownProcessUnit {
                node_id,
                process_unit,
            } => write!(
                f,
                "workflow planner: node '{}' references unknown process unit '{}::{}'",
                node_id.0, process_unit.process_id.0, process_unit.unit_id.0
            ),
            WorkflowPlannerError::StrictDryRunMissingInput {
                node_id,
                port,
                trace,
            } => write!(
                f,
                "strict dry-run missing required input: node='{}' port='{}' trace='{}'",
                node_id.0, port.0, trace
            ),
        }
    }
}

impl std::error::Error for WorkflowPlannerError {}

impl From<WorkflowLedgerError> for WorkflowPlannerError {
    fn from(error: WorkflowLedgerError) -> Self {
        Self::Ledger(error)
    }
}

/// Explicit planner input map (node_id -> port -> value).
pub type PlannerInputs = BTreeMap<NodeId, BTreeMap<PortName, Value>>;

/// Plan workflow nodes with deterministic keys and typed miss reasons.
///
/// No mtime-only freshness logic is used here.
pub fn plan_workflow(
    spec: &WorkflowSpec,
    registry: &ProcessUnitRegistry,
    planner_inputs: &PlannerInputs,
    workspace_root: &Path,
) -> Result<WorkflowPlan, WorkflowPlannerError> {
    plan_workflow_with_mode(
        spec,
        registry,
        planner_inputs,
        workspace_root,
        DryRunMode::Lenient,
    )
}

/// Plan workflow nodes with explicit dry-run strictness mode.
pub fn plan_workflow_with_mode(
    spec: &WorkflowSpec,
    registry: &ProcessUnitRegistry,
    planner_inputs: &PlannerInputs,
    workspace_root: &Path,
    dry_run_mode: DryRunMode,
) -> Result<WorkflowPlan, WorkflowPlannerError> {
    if matches!(dry_run_mode, DryRunMode::Strict) {
        validate_strict_dry_run_inputs(spec, planner_inputs)?;
    }

    let previous_entries = load_global_ledger(workspace_root)?;
    let mut previous_by_work_id: HashMap<WorkIdentity, RunLedgerEntry> = HashMap::new();
    for entry in previous_entries {
        previous_by_work_id.insert(entry.work_id.clone(), entry);
    }

    let mut keys_by_node: BTreeMap<NodeId, MaterializationKey> = BTreeMap::new();
    let mut plans = Vec::new();
    let order = topo_sort(&spec.dag);

    let ordered_edges = canonical_edge_order(&spec.dag.edges);
    for node_id in order {
        let node = spec
            .dag
            .get_node(&node_id)
            .ok_or_else(|| WorkflowPlannerError::UnknownNode(node_id.clone()))?;

        let NodeBody::Opaque(WorkflowUnit { op }) = &node.body else {
            return Err(WorkflowPlannerError::UnknownNode(node_id.clone()));
        };

        let (work_id, op_version) = match op {
            WorkflowOp::InvokeProcessUnit(process_ref) => {
                let process_spec = registry.get(process_ref).ok_or_else(|| {
                    WorkflowPlannerError::UnknownProcessUnit {
                        node_id: node_id.clone(),
                        process_unit: process_ref.clone(),
                    }
                })?;
                let (canonical_process, canonical_unit) = process_spec.canonical_work_identity();
                (
                    WorkIdentity::new(canonical_process, canonical_unit),
                    process_spec.op_version,
                )
            }
            WorkflowOp::Aggregate(_) => (
                WorkIdentity::new(
                    ProcessId::new(format!("workflow:{}", spec.id.0)),
                    node_id.clone(),
                ),
                1,
            ),
            WorkflowOp::Report(_) => (
                WorkIdentity::new(
                    ProcessId::new(format!("workflow:{}", spec.id.0)),
                    node_id.clone(),
                ),
                1,
            ),
        };

        let node_inputs = planner_inputs.get(&node_id).cloned().unwrap_or_default();
        let input_hashes = hash_input_map(&node_inputs)?;
        let upstream_keys = collect_upstream_keys(&node_id, &keys_by_node, &ordered_edges);

        let key = MaterializationKey::new(
            work_id.clone(),
            CanonicalKeyPayload {
                key_format_version: 1,
                op_version,
                input_hashes,
                upstream_keys,
                policy_version: spec.policy_version,
            },
        )
        .map_err(WorkflowPlannerError::Key)?;

        let action = match previous_by_work_id.get(&work_id) {
            None => PlanAction::Execute {
                miss_reason: MissReason::NoPriorRun,
            },
            Some(previous) if previous.key.digest == key.digest => {
                let previous_run = previous_run_id(previous);
                match rehydrate_outputs_for_entry(workspace_root, previous) {
                    Ok(outputs) => PlanAction::CachedHit {
                        previous_run,
                        rehydrated_outputs: outputs,
                    },
                    Err(WorkflowLedgerError::MissingOutputPayload { .. }) => PlanAction::Execute {
                        miss_reason: MissReason::OutputMissing {
                            port: PortName::from(PORT_RESULT),
                        },
                    },
                    Err(error) => return Err(WorkflowPlannerError::Ledger(error)),
                }
            }
            Some(previous) => {
                let reason =
                    derive_miss_reason(&previous.key, &key).unwrap_or(MissReason::NoPriorRun);
                PlanAction::Execute {
                    miss_reason: reason,
                }
            }
        };

        keys_by_node.insert(node_id.clone(), key.clone());
        plans.push(NodePlan {
            node_id,
            work_id,
            key,
            action,
        });
    }

    let provided_inputs = planner_inputs
        .iter()
        .map(|(node, ports)| (node.clone(), ports.keys().cloned().collect()))
        .collect::<BTreeMap<NodeId, std::collections::BTreeSet<PortName>>>();
    let coordination =
        coordination_status(spec, &std::collections::HashSet::new(), &provided_inputs);

    Ok(WorkflowPlan {
        nodes: plans,
        coordination,
    })
}

fn validate_strict_dry_run_inputs(
    spec: &WorkflowSpec,
    planner_inputs: &PlannerInputs,
) -> Result<(), WorkflowPlannerError> {
    for node in &spec.dag.nodes {
        let incoming_data_ports = spec
            .dag
            .edges
            .iter()
            .filter(|edge| edge.to_node == node.id && edge.kind.carries_data())
            .map(|edge| edge.to_port.clone())
            .collect::<BTreeSet<_>>();

        let provided = planner_inputs
            .get(&node.id)
            .map(|ports| ports.keys().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();

        for input in &node.inputs {
            if input.name.0 == PORT_AFTER
                || input.name.0.starts_with(gunbc_ir::RESOURCE_PORT_PREFIX)
            {
                continue;
            }
            if !input.cardinality.requires_one() {
                continue;
            }

            if !incoming_data_ports.contains(&input.name) && !provided.contains(&input.name) {
                return Err(WorkflowPlannerError::StrictDryRunMissingInput {
                    node_id: node.id.clone(),
                    port: input.name.clone(),
                    trace: format!(
                        "no incoming data edge and no planner input for '{}.{}'",
                        node.id.0, input.name.0
                    ),
                });
            }
        }
    }
    Ok(())
}

fn previous_run_id(entry: &RunLedgerEntry) -> String {
    match &entry.status {
        LedgerStatus::CachedHit { previous_run } => previous_run.clone(),
        _ => format!("{}:{}", entry.work_id.process_id.0, entry.work_id.unit_id.0),
    }
}

fn hash_input_map(
    inputs: &BTreeMap<PortName, Value>,
) -> Result<BTreeMap<PortName, Vec<String>>, WorkflowPlannerError> {
    let mut hashed = BTreeMap::new();
    for (port, value) in inputs {
        hashed.insert(port.clone(), value_hashes(value)?);
    }
    Ok(hashed)
}

fn value_hashes(value: &Value) -> Result<Vec<String>, WorkflowPlannerError> {
    match value {
        Value::List(items) | Value::Set(items) => {
            let mut hashes = Vec::new();
            for item in items {
                hashes.push(hash_value(item)?);
            }
            hashes.sort();
            Ok(hashes)
        }
        _ => Ok(vec![hash_value(value)?]),
    }
}

fn hash_value(value: &Value) -> Result<String, WorkflowPlannerError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        WorkflowPlannerError::Key(format!("failed to serialize value for hashing: {error}"))
    })?;
    Ok(gunbc_infra::hash::ContentHash::from_bytes(&bytes)
        .as_str()
        .to_string())
}

fn collect_upstream_keys(
    node_id: &NodeId,
    keys_by_node: &BTreeMap<NodeId, MaterializationKey>,
    ordered_edges: &[&gunbc_ir::Edge],
) -> BTreeMap<PortName, Vec<MaterializationDigest>> {
    let mut upstream: BTreeMap<PortName, Vec<MaterializationDigest>> = BTreeMap::new();
    for edge in ordered_edges {
        if &edge.to_node != node_id {
            continue;
        }
        let Some(upstream_key) = keys_by_node.get(&edge.from_node) else {
            continue;
        };
        upstream
            .entry(edge.to_port.clone())
            .or_default()
            .push(upstream_key.digest.clone());
    }
    for digests in upstream.values_mut() {
        digests.sort();
    }
    upstream
}

/// Produce deterministic explainability sets from a computed workflow plan.
pub fn explain_plan(spec: &WorkflowSpec, plan: &WorkflowPlan) -> PlanExplain {
    let mut execute_set = Vec::new();
    let mut cache_hit_set = Vec::new();
    let mut miss_reasons = BTreeMap::new();
    for node in &plan.nodes {
        match &node.action {
            PlanAction::CachedHit { .. } => cache_hit_set.push(node.node_id.clone()),
            PlanAction::Execute { miss_reason } => {
                execute_set.push(node.node_id.clone());
                miss_reasons.insert(node.node_id.clone(), miss_reason.clone());
            }
        }
    }
    execute_set.sort();
    cache_hit_set.sort();
    let critical_path = critical_path(spec);

    PlanExplain {
        execute_set,
        cache_hit_set,
        miss_reasons,
        blocked: plan.coordination.blocked.clone(),
        ready: plan.coordination.ready.clone(),
        critical_path,
    }
}

fn critical_path(spec: &WorkflowSpec) -> Vec<NodeId> {
    let order = topo_sort(&spec.dag);
    let mut parents: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in &spec.dag.nodes {
        parents.insert(node.id.clone(), Vec::new());
    }
    for edge in &spec.dag.edges {
        parents
            .entry(edge.to_node.clone())
            .or_default()
            .push(edge.from_node.clone());
    }

    let mut distance: HashMap<NodeId, usize> = HashMap::new();
    let mut predecessor: HashMap<NodeId, Option<NodeId>> = HashMap::new();
    for node in &order {
        let mut best_dist = 0usize;
        let mut best_pred: Option<NodeId> = None;
        if let Some(node_parents) = parents.get(node) {
            for parent in node_parents {
                let parent_dist = distance.get(parent).copied().unwrap_or(0) + 1;
                let replace = match best_pred.as_ref() {
                    None => true,
                    Some(current_pred) => {
                        parent_dist > best_dist
                            || (parent_dist == best_dist && parent < current_pred)
                    }
                };
                if replace {
                    best_dist = parent_dist;
                    best_pred = Some(parent.clone());
                }
            }
        }
        distance.insert(node.clone(), best_dist);
        predecessor.insert(node.clone(), best_pred);
    }

    let Some(end) = order.iter().cloned().max_by(|left, right| {
        let left_dist = distance.get(left).copied().unwrap_or(0);
        let right_dist = distance.get(right).copied().unwrap_or(0);
        left_dist.cmp(&right_dist).then_with(|| right.cmp(left))
    }) else {
        return Vec::new();
    };

    let mut path = Vec::new();
    let mut cursor = Some(end);
    while let Some(node) = cursor {
        path.push(node.clone());
        cursor = predecessor.get(&node).cloned().flatten();
    }
    path.reverse();
    path
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use gunbc_ir::{Edge, Node, Port};

    use super::*;
    use crate::workflow::ledger::{
        append_global_ledger_entry, store_output_payload, LedgerStatus, RunLedgerEntry,
    };
    use crate::workflow::process_registry::{ProcessUnitRef, ProcessUnitSpec, UnitClaim};
    use crate::workflow::schema::{
        required_input_contract, required_output_contract, WorkflowId, WorkflowSpec, WorkflowUnit,
    };

    fn temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "gunbc-workflow-plan-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    fn two_node_spec() -> WorkflowSpec {
        let mut dag = gunbc_ir::Dag::new();
        dag.add_node(Node::opaque(
            "wf.a",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "wf.a",
            ))),
        ));
        dag.add_node(Node::opaque(
            "wf.b",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "wf.b",
            ))),
        ));
        dag.add_edge(Edge::new("wf.a", "result", "wf.b", "after"));
        WorkflowSpec::new(WorkflowId::new("wf"), dag, 1)
    }

    fn two_node_registry() -> ProcessUnitRegistry {
        let mut registry = ProcessUnitRegistry::new();
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("wf", "wf.a"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ));
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("wf", "wf.b"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ));
        registry
    }

    #[test]
    fn planner_keys_are_deterministic_for_same_inputs() {
        let root = temp_root();
        let spec = two_node_spec();
        let registry = two_node_registry();
        let inputs = PlannerInputs::new();

        let a = plan_workflow(&spec, &registry, &inputs, &root).expect("plan a");
        let b = plan_workflow(&spec, &registry, &inputs, &root).expect("plan b");
        let key_a = &a.nodes[0].key.digest;
        let key_b = &b.nodes[0].key.digest;
        assert_eq!(key_a, key_b);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn planner_reports_typed_miss_reason_for_input_drift() {
        let root = temp_root();
        let spec = two_node_spec();
        let registry = two_node_registry();

        let mut first_inputs = PlannerInputs::new();
        first_inputs.insert(
            NodeId::from("wf.a"),
            BTreeMap::from([(PortName::from("after"), Value::Str("alpha".to_string()))]),
        );

        let first_plan = plan_workflow(&spec, &registry, &first_inputs, &root).expect("first plan");
        let first = first_plan
            .nodes
            .iter()
            .find(|node| node.node_id == NodeId::from("wf.a"))
            .expect("wf.a plan");
        let result_payload = Value::Str("cached".to_string());
        let hash = store_output_payload(&root, &result_payload).expect("store payload");
        append_global_ledger_entry(
            &root,
            RunLedgerEntry {
                exec_node_id: first.node_id.clone(),
                work_id: first.work_id.clone(),
                key: first.key.clone(),
                status: LedgerStatus::CachedHit {
                    previous_run: "run-1".to_string(),
                },
                output_hashes: BTreeMap::from([(PortName::from("result"), hash)]),
                duration_ms: 1,
            },
        )
        .expect("append previous entry");

        let mut second_inputs = PlannerInputs::new();
        second_inputs.insert(
            NodeId::from("wf.a"),
            BTreeMap::from([(PortName::from("after"), Value::Str("beta".to_string()))]),
        );
        let second_plan =
            plan_workflow(&spec, &registry, &second_inputs, &root).expect("second plan");
        let wf_a = second_plan
            .nodes
            .iter()
            .find(|node| node.node_id == NodeId::from("wf.a"))
            .expect("wf.a plan");
        assert!(matches!(
            wf_a.action,
            PlanAction::Execute {
                miss_reason: MissReason::InputChanged { .. }
            }
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn planner_rehydrates_outputs_for_cached_hits() {
        let root = temp_root();
        let spec = two_node_spec();
        let registry = two_node_registry();
        let inputs = PlannerInputs::new();

        let initial_plan = plan_workflow(&spec, &registry, &inputs, &root).expect("initial plan");
        let node = initial_plan
            .nodes
            .iter()
            .find(|plan| plan.node_id == NodeId::from("wf.a"))
            .expect("wf.a node");

        let payload = Value::Map(BTreeMap::from([("ok".to_string(), Value::Bool(true))]));
        let payload_hash = store_output_payload(&root, &payload).expect("store payload");
        append_global_ledger_entry(
            &root,
            RunLedgerEntry {
                exec_node_id: node.node_id.clone(),
                work_id: node.work_id.clone(),
                key: node.key.clone(),
                status: LedgerStatus::CachedHit {
                    previous_run: "run-cached".to_string(),
                },
                output_hashes: BTreeMap::from([(PortName::from("result"), payload_hash)]),
                duration_ms: 1,
            },
        )
        .expect("append cache hit");

        let planned = plan_workflow(&spec, &registry, &inputs, &root).expect("replanned");
        let wf_a = planned
            .nodes
            .iter()
            .find(|plan| plan.node_id == NodeId::from("wf.a"))
            .expect("wf.a plan");
        let PlanAction::CachedHit {
            rehydrated_outputs, ..
        } = &wf_a.action
        else {
            panic!("wf.a should be cache hit with rehydrated outputs");
        };
        assert_eq!(
            rehydrated_outputs.get(&PortName::from("result")),
            Some(&payload)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn strict_dry_run_fails_on_missing_required_input() {
        let root = temp_root();
        let mut dag = gunbc_ir::Dag::new();
        let mut inputs = required_input_contract();
        inputs.push(Port::scalar("payload", "String"));
        dag.add_node(Node::opaque(
            "wf.a",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "wf.a",
            ))),
        ));
        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = two_node_registry();

        let err = plan_workflow_with_mode(
            &spec,
            &registry,
            &PlannerInputs::new(),
            &root,
            DryRunMode::Strict,
        )
        .expect_err("strict dry-run should reject unset required input");
        assert!(matches!(
            err,
            WorkflowPlannerError::StrictDryRunMissingInput { .. }
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn strict_dry_run_accepts_provided_required_input() {
        let root = temp_root();
        let mut dag = gunbc_ir::Dag::new();
        let mut inputs = required_input_contract();
        inputs.push(Port::scalar("payload", "String"));
        dag.add_node(Node::opaque(
            "wf.a",
            inputs,
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "wf", "wf.a",
            ))),
        ));
        let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
        let registry = two_node_registry();

        let mut planner_inputs = PlannerInputs::new();
        planner_inputs.insert(
            NodeId::from("wf.a"),
            BTreeMap::from([(PortName::from("payload"), Value::Str("x".to_string()))]),
        );
        plan_workflow_with_mode(&spec, &registry, &planner_inputs, &root, DryRunMode::Strict)
            .expect("strict mode should pass when required input is provided");
        let _ = std::fs::remove_dir_all(root);
    }
}
