//! Workflow runtime execution + planner input fingerprinting (WF6/WF7/WF9 scaffolding).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use gunbc_infra::hash::ContentHash;
use gunbc_ir::{NodeBody, NodeId, PortName, Value};

use crate::makegen::registry::BuildConfig;

use super::ledger::{
    append_global_ledger_entry, store_output_payload, LedgerStatus, RunLedgerEntry,
    WorkflowLedgerError,
};
use super::planner::{PlanAction, PlannerInputs, WorkflowPlan};
use super::process_registry::ProcessUnitRef;
use super::schema::{WorkflowOp, WorkflowSpec, PORT_RESULT};

const PORT_WORKSPACE_FINGERPRINT: &str = "__workspace_fingerprint";
const PORT_TEST_COST_CLASS: &str = "__test_cost_class";

/// Runtime summary for one executed workflow plan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowExecutionSummary {
    pub executed: Vec<NodeId>,
    pub cached_hits: Vec<NodeId>,
    pub skipped: Vec<NodeId>,
    pub failed: Vec<NodeId>,
    pub failed_errors: BTreeMap<NodeId, String>,
    pub duration_ms_by_node: BTreeMap<NodeId, u64>,
    pub total_duration_ms: u64,
}

/// Runtime failures that prevent execution bootstrap.
#[derive(Debug)]
pub enum WorkflowRuntimeError {
    UnknownNode(NodeId),
    MissingWorkflowUnitBody(NodeId),
    UnsupportedProcessUnit { process_id: String, unit_id: String },
    Ledger(WorkflowLedgerError),
}

impl std::fmt::Display for WorkflowRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowRuntimeError::UnknownNode(node_id) => {
                write!(f, "workflow runtime: unknown node '{}'", node_id.0)
            }
            WorkflowRuntimeError::MissingWorkflowUnitBody(node_id) => write!(
                f,
                "workflow runtime: node '{}' is missing workflow unit body",
                node_id.0
            ),
            WorkflowRuntimeError::UnsupportedProcessUnit {
                process_id,
                unit_id,
            } => write!(
                f,
                "workflow runtime: unsupported process unit '{}::{}'",
                process_id, unit_id
            ),
            WorkflowRuntimeError::Ledger(error) => {
                write!(f, "workflow runtime ledger error: {error}")
            }
        }
    }
}

impl std::error::Error for WorkflowRuntimeError {}

impl From<WorkflowLedgerError> for WorkflowRuntimeError {
    fn from(value: WorkflowLedgerError) -> Self {
        WorkflowRuntimeError::Ledger(value)
    }
}

/// Build conservative default planner inputs from workspace state.
///
/// This keeps keying fail-closed: any repo/toolchain change invalidates cached
/// planner keys for workflow units.
pub fn default_planner_inputs(spec: &WorkflowSpec, workspace_root: &Path) -> PlannerInputs {
    let fingerprint = workspace_fingerprint(workspace_root);
    let mut planner_inputs = PlannerInputs::new();

    for node in &spec.dag.nodes {
        let mut inputs = BTreeMap::new();
        inputs.insert(
            PortName::from(PORT_WORKSPACE_FINGERPRINT),
            Value::Str(fingerprint.clone()),
        );
        if spec.id.0 == "test-all" && node.id == NodeId::from("test_all.cargo_test_xl") {
            inputs.insert(
                PortName::from(PORT_TEST_COST_CLASS),
                Value::Str("XL".to_string()),
            );
        }
        planner_inputs.insert(node.id.clone(), inputs);
    }
    planner_inputs
}

/// Execute a previously planned workflow in topological plan order.
///
/// `legacy_dry_run=true` keeps the planner/execution bookkeeping but skips
/// subprocess execution for `Execute` nodes.
pub fn execute_workflow_plan(
    spec: &WorkflowSpec,
    plan: &WorkflowPlan,
    workspace_root: &Path,
    legacy_dry_run: bool,
) -> Result<WorkflowExecutionSummary, WorkflowRuntimeError> {
    let predecessor_map = ordered_predecessors(spec);
    let mut committed = BTreeSet::new();
    let mut summary = WorkflowExecutionSummary::default();
    let build_config = BuildConfig::cargo();

    for node_plan in &plan.nodes {
        let blocked_by = predecessor_map
            .get(&node_plan.node_id)
            .and_then(|deps| deps.iter().find(|dep| !committed.contains(*dep)).cloned());
        if let Some(blocked_by) = blocked_by {
            summary.skipped.push(node_plan.node_id.clone());
            summary
                .duration_ms_by_node
                .insert(node_plan.node_id.clone(), 0_u64);
            append_global_ledger_entry(
                workspace_root,
                RunLedgerEntry {
                    exec_node_id: node_plan.node_id.clone(),
                    work_id: node_plan.work_id.clone(),
                    key: node_plan.key.clone(),
                    status: LedgerStatus::Skipped { blocked_by },
                    output_hashes: BTreeMap::new(),
                    duration_ms: 0,
                },
            )?;
            continue;
        }

        match &node_plan.action {
            PlanAction::CachedHit {
                previous_run,
                rehydrated_outputs,
            } => {
                summary.cached_hits.push(node_plan.node_id.clone());
                summary
                    .duration_ms_by_node
                    .insert(node_plan.node_id.clone(), 0_u64);
                committed.insert(node_plan.node_id.clone());

                let output_hashes = output_hashes_from_values(workspace_root, rehydrated_outputs)?;
                append_global_ledger_entry(
                    workspace_root,
                    RunLedgerEntry {
                        exec_node_id: node_plan.node_id.clone(),
                        work_id: node_plan.work_id.clone(),
                        key: node_plan.key.clone(),
                        status: LedgerStatus::CachedHit {
                            previous_run: previous_run.clone(),
                        },
                        output_hashes,
                        duration_ms: 0,
                    },
                )?;
            }
            PlanAction::Execute { miss_reason } => {
                let mut duration_ms = 0_u64;
                let steps = steps_for_node(spec, &node_plan.node_id, &build_config)?;
                let mut failed_error: Option<String> = None;

                if !legacy_dry_run {
                    for step in &steps {
                        let started = Instant::now();
                        if let Err(error) = run_shell_step(workspace_root, step) {
                            duration_ms += started.elapsed().as_millis() as u64;
                            failed_error = Some(format!(
                                "node '{}': command failed: {error}",
                                node_plan.node_id.0
                            ));
                            break;
                        }
                        duration_ms += started.elapsed().as_millis() as u64;
                    }
                }

                summary.total_duration_ms += duration_ms;
                summary
                    .duration_ms_by_node
                    .insert(node_plan.node_id.clone(), duration_ms);

                if let Some(error) = failed_error {
                    summary.failed.push(node_plan.node_id.clone());
                    summary
                        .failed_errors
                        .insert(node_plan.node_id.clone(), error.clone());
                    append_global_ledger_entry(
                        workspace_root,
                        RunLedgerEntry {
                            exec_node_id: node_plan.node_id.clone(),
                            work_id: node_plan.work_id.clone(),
                            key: node_plan.key.clone(),
                            status: LedgerStatus::Failed {
                                reason: miss_reason.clone(),
                                error,
                            },
                            output_hashes: BTreeMap::new(),
                            duration_ms,
                        },
                    )?;
                    continue;
                }

                let output_payload = Value::Map(BTreeMap::from([
                    ("ok".to_string(), Value::Bool(true)),
                    ("dry_run".to_string(), Value::Bool(legacy_dry_run)),
                    (
                        "steps".to_string(),
                        Value::List(steps.into_iter().map(Value::Str).collect()),
                    ),
                ]));
                let output_hash = store_output_payload(workspace_root, &output_payload)?;
                append_global_ledger_entry(
                    workspace_root,
                    RunLedgerEntry {
                        exec_node_id: node_plan.node_id.clone(),
                        work_id: node_plan.work_id.clone(),
                        key: node_plan.key.clone(),
                        status: LedgerStatus::Executed {
                            reason: miss_reason.clone(),
                        },
                        output_hashes: BTreeMap::from([(PortName::from(PORT_RESULT), output_hash)]),
                        duration_ms,
                    },
                )?;

                summary.executed.push(node_plan.node_id.clone());
                committed.insert(node_plan.node_id.clone());
            }
        }
    }

    Ok(summary)
}

/// Stable workspace/toolchain fingerprint used as a conservative cache input.
pub fn workspace_fingerprint(workspace_root: &Path) -> String {
    let mut signals = Vec::new();
    signals.push(format!("root={}", workspace_root.display()));
    if let Some(head) = capture_stdout(workspace_root, "git", &["rev-parse", "HEAD"]) {
        signals.push(format!("git_head={head}"));
    }
    if let Some(status) = capture_stdout(
        workspace_root,
        "git",
        &["status", "--porcelain=v1", "--untracked-files=all"],
    ) {
        signals.push(format!("git_status={status}"));
    }
    if let Some(rustc) = capture_stdout(workspace_root, "rustc", &["--version"]) {
        signals.push(format!("rustc={rustc}"));
    }
    let cargo_lock = workspace_root.join("Cargo.lock");
    if cargo_lock.exists() {
        if let Ok(bytes) = fs::read(&cargo_lock) {
            signals.push(format!(
                "cargo_lock={}",
                ContentHash::from_bytes(&bytes).as_str()
            ));
        }
    }
    ContentHash::from_bytes(signals.join("\n").as_bytes())
        .as_str()
        .to_string()
}

fn ordered_predecessors(spec: &WorkflowSpec) -> BTreeMap<NodeId, Vec<NodeId>> {
    let mut predecessors = BTreeMap::<NodeId, Vec<NodeId>>::new();
    for node in &spec.dag.nodes {
        predecessors.entry(node.id.clone()).or_default();
    }
    for edge in &spec.dag.edges {
        if !edge.kind.creates_ordering() {
            continue;
        }
        predecessors
            .entry(edge.to_node.clone())
            .or_default()
            .push(edge.from_node.clone());
    }
    for deps in predecessors.values_mut() {
        deps.sort();
        deps.dedup();
    }
    predecessors
}

fn steps_for_node(
    spec: &WorkflowSpec,
    node_id: &NodeId,
    config: &BuildConfig,
) -> Result<Vec<String>, WorkflowRuntimeError> {
    let node = spec
        .dag
        .get_node(node_id)
        .ok_or_else(|| WorkflowRuntimeError::UnknownNode(node_id.clone()))?;
    let NodeBody::Opaque(unit) = &node.body else {
        return Err(WorkflowRuntimeError::MissingWorkflowUnitBody(
            node_id.clone(),
        ));
    };
    match &unit.op {
        WorkflowOp::InvokeProcessUnit(process_unit) => process_unit_steps(process_unit, config)
            .ok_or_else(|| WorkflowRuntimeError::UnsupportedProcessUnit {
                process_id: process_unit.process_id.0.clone(),
                unit_id: process_unit.unit_id.0.clone(),
            }),
        WorkflowOp::Aggregate(_) | WorkflowOp::Report(_) => Ok(Vec::new()),
    }
}

fn process_unit_steps(process_unit: &ProcessUnitRef, config: &BuildConfig) -> Option<Vec<String>> {
    let lint = config.lint.to_shell();
    let lint_fix = config.lint_fix.to_shell();
    let lint_upsert = format!(
        "{} || ({} && {})",
        strip_command_prefix(&lint),
        strip_command_prefix(&lint_fix),
        strip_command_prefix(&lint)
    );

    match process_unit.unit_id.0.as_str() {
        "ci.lint_upsert" | "test_all.lint_upsert" => Some(vec![
            strip_command_prefix(&config.pragma_shell()),
            lint_upsert,
        ]),
        "ci.codegen" | "test_all.codegen" => {
            Some(vec![strip_command_prefix(&config.ensure_codegen_shell())])
        }
        "ci.bootstrap" => Some(vec![strip_command_prefix(&config.bootstrap_check_shell())]),
        "ci.pragma" => Some(vec![strip_command_prefix(&config.pragma_shell())]),
        "ci.testgen" | "test_all.testgen" => {
            Some(vec![strip_command_prefix(&config.testgen_shell())])
        }
        "ci.build_compile" | "test_all.build_compile" => Some(vec![
            "RUSTFLAGS=\"-D warnings\" cargo test --no-run".to_string(),
        ]),
        "ci.test_run" => Some(vec![strip_command_prefix(&config.test_shell())]),
        "ci.clippy_run" => Some(vec![strip_command_prefix(&config.lint_shell())]),
        "ci.guardrails" => Some(vec![
            "RUSTFLAGS=\"-D warnings\" cargo test -p gunbc-dag --test resource_purity_checks"
                .to_string(),
        ]),
        "ci.verify" => Some(vec![
            strip_command_prefix(&config.deps_config_check_shell()),
            strip_command_prefix(&config.makegen_check_shell()),
            strip_command_prefix(&config.bootstrap_check_shell()),
            strip_command_prefix(&config.testgen_check_shell()),
            strip_command_prefix(&config.pragma_check_shell()),
        ]),
        "test_all.verify_fix" => Some(vec![
            strip_command_prefix(&config.deps_config_ensure_shell()),
            strip_command_prefix(&config.makegen_ensure_shell()),
            strip_command_prefix(&config.bootstrap_ensure_shell()),
            strip_command_prefix(&config.testgen_ensure_shell()),
            strip_command_prefix(&config.pragma_ensure_shell()),
        ]),
        "test_all.cargo_test_xl" => Some(vec![format!(
            "GUNBC_TEST_MAX_COST=XL {}",
            strip_command_prefix(&config.test_shell())
        )]),
        "ci.report" | "test_all.report" => Some(Vec::new()),
        _ => None,
    }
}

fn output_hashes_from_values(
    workspace_root: &Path,
    values: &BTreeMap<PortName, Value>,
) -> Result<BTreeMap<PortName, String>, WorkflowLedgerError> {
    let mut hashes = BTreeMap::new();
    for (port, value) in values {
        hashes.insert(port.clone(), store_output_payload(workspace_root, value)?);
    }
    Ok(hashes)
}

fn run_shell_step(workspace_root: &Path, step: &str) -> Result<(), String> {
    let status = Command::new("bash")
        .arg("-lc")
        .arg(step)
        .current_dir(workspace_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("spawn error: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("exit status {:?}", status.code()))
    }
}

fn capture_stdout(workspace_root: &Path, command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command)
        .args(args)
        .current_dir(workspace_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn strip_command_prefix(command: &str) -> String {
    command.strip_prefix('@').unwrap_or(command).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::spec_builders::ci_workflow_spec;

    #[test]
    fn default_inputs_include_workspace_fingerprint_for_each_node() {
        let spec = ci_workflow_spec().expect("ci spec");
        let inputs = default_planner_inputs(&spec, Path::new("."));
        assert_eq!(inputs.len(), spec.dag.nodes.len());
        assert!(inputs
            .values()
            .all(|ports| ports.contains_key(&PortName::from(PORT_WORKSPACE_FINGERPRINT))));
    }

    #[test]
    fn process_unit_command_mapping_covers_known_ci_units() {
        let config = BuildConfig::cargo();
        let steps = process_unit_steps(&ProcessUnitRef::new("ci", "ci.codegen"), &config)
            .expect("ci.codegen should be mapped");
        assert_eq!(steps.len(), 1);
        assert!(steps[0].contains("gunbc-codegen"));
    }
}
