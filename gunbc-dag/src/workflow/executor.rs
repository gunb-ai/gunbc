//! Workflow executor: runs planned units via shell commands (WF6/WF7).
//!
//! This module takes a computed `WorkflowPlan` and executes it:
//! - CachedHit nodes are skipped (outputs rehydrated from ledger CAS).
//! - Execute nodes run their mapped shell command.
//! - Results are persisted to the global ledger after each unit.
//! - A typed execution summary is returned with timing and hit/miss stats.
//!
//! `Command::new` is used here intentionally: the executor dispatches
//! coarse-grained CLI subprocesses (not fine-grained DAG node operations).
//! These commands correspond to workflow planner units, not transport ops.
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use gunbc_ir::{NodeId, PortName, Value};

use super::key::MissReason;
use super::ledger::{
    append_global_ledger_entry, store_output_payload, LedgerStatus, RunLedgerEntry,
};
use super::planner::{NodePlan, PlanAction, WorkflowPlan};
use super::schema::WorkflowSpec;

/// Shell command to execute for a workflow unit.
#[derive(Debug, Clone)]
pub struct UnitCommand {
    pub program: String,
    pub args: Vec<String>,
    pub label: String,
}

impl UnitCommand {
    pub fn new(label: impl Into<String>, program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            label: label.into(),
            program: program.into(),
            args,
        }
    }

    pub fn cargo(label: impl Into<String>, args: Vec<&str>) -> Self {
        Self::new(
            label,
            "cargo",
            args.into_iter().map(String::from).collect(),
        )
    }
}

/// Result of executing a single workflow unit.
#[derive(Debug, Clone)]
pub struct UnitResult {
    pub node_id: NodeId,
    pub success: bool,
    pub cached: bool,
    pub duration_ms: u64,
    pub miss_reason: Option<MissReason>,
}

/// Summary of a full workflow execution run.
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub workflow_id: String,
    pub total_units: usize,
    pub cache_hits: usize,
    pub executed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub results: Vec<UnitResult>,
    pub total_duration_ms: u64,
}

impl ExecutionSummary {
    /// Whether the entire workflow succeeded (no failures).
    pub fn success(&self) -> bool {
        self.failed == 0
    }
}

/// Execute a workflow plan, running only units marked `Execute`.
///
/// Units are processed in topological order (as produced by the planner).
/// CachedHit units are skipped. Execute units run the mapped shell command.
/// Results are persisted to the global ledger after each unit completes.
///
/// If `dry_run` is true, commands are printed but not executed.
pub fn execute_workflow_plan(
    spec: &WorkflowSpec,
    plan: &WorkflowPlan,
    commands: &BTreeMap<NodeId, UnitCommand>,
    workspace_root: &Path,
    dry_run: bool,
) -> ExecutionSummary {
    let run_start = Instant::now();
    let mut results = Vec::new();
    let mut cache_hits = 0usize;
    let mut executed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut has_failure = false;

    for node_plan in &plan.nodes {
        match &node_plan.action {
            PlanAction::CachedHit { .. } => {
                cache_hits += 1;
                results.push(UnitResult {
                    node_id: node_plan.node_id.clone(),
                    success: true,
                    cached: true,
                    duration_ms: 0,
                    miss_reason: None,
                });
                emit_unit_status(&node_plan.node_id, UnitStatus::CachedHit);
            }
            PlanAction::Execute { miss_reason } => {
                if has_failure {
                    // Skip downstream units after a failure (fail-closed).
                    skipped += 1;
                    results.push(UnitResult {
                        node_id: node_plan.node_id.clone(),
                        success: false,
                        cached: false,
                        duration_ms: 0,
                        miss_reason: Some(miss_reason.clone()),
                    });
                    persist_skipped_entry(workspace_root, node_plan);
                    emit_unit_status(&node_plan.node_id, UnitStatus::Skipped);
                    continue;
                }

                let Some(cmd) = commands.get(&node_plan.node_id) else {
                    // Report nodes and aggregate nodes may not have commands.
                    // Treat them as successful no-ops.
                    executed += 1;
                    results.push(UnitResult {
                        node_id: node_plan.node_id.clone(),
                        success: true,
                        cached: false,
                        duration_ms: 0,
                        miss_reason: Some(miss_reason.clone()),
                    });
                    persist_executed_entry(workspace_root, node_plan, true, 0);
                    emit_unit_status(&node_plan.node_id, UnitStatus::Executed { success: true });
                    continue;
                };

                if dry_run {
                    executed += 1;
                    emit_unit_status(&node_plan.node_id, UnitStatus::DryRun(&cmd.label));
                    results.push(UnitResult {
                        node_id: node_plan.node_id.clone(),
                        success: true,
                        cached: false,
                        duration_ms: 0,
                        miss_reason: Some(miss_reason.clone()),
                    });
                    continue;
                }

                emit_unit_status(&node_plan.node_id, UnitStatus::Running(&cmd.label));
                let unit_start = Instant::now();
                let success = run_unit_command(cmd);
                let duration_ms = unit_start.elapsed().as_millis() as u64;
                executed += 1;

                if !success {
                    failed += 1;
                    has_failure = true;
                }

                persist_executed_entry(workspace_root, node_plan, success, duration_ms);
                emit_unit_status(
                    &node_plan.node_id,
                    UnitStatus::Executed { success },
                );

                results.push(UnitResult {
                    node_id: node_plan.node_id.clone(),
                    success,
                    cached: false,
                    duration_ms,
                    miss_reason: Some(miss_reason.clone()),
                });
            }
        }
    }

    let total_duration_ms = run_start.elapsed().as_millis() as u64;
    ExecutionSummary {
        workflow_id: spec.id.0.clone(),
        total_units: plan.nodes.len(),
        cache_hits,
        executed,
        failed,
        skipped,
        results,
        total_duration_ms,
    }
}

/// Run a shell command, inheriting stdout/stderr. Returns true on success.
fn run_unit_command(cmd: &UnitCommand) -> bool {
    let result = Command::new(&cmd.program)
        .args(&cmd.args)
        .status();
    match result {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!(
                "  error: failed to spawn '{}': {}",
                cmd.program, error
            );
            false
        }
    }
}

/// Persist a ledger entry for an executed unit.
fn persist_executed_entry(
    workspace_root: &Path,
    node_plan: &NodePlan,
    success: bool,
    duration_ms: u64,
) {
    let result_payload = Value::Map(BTreeMap::from([
        ("success".to_string(), Value::Bool(success)),
    ]));
    let hash = match store_output_payload(workspace_root, &result_payload) {
        Ok(h) => h,
        Err(error) => {
            eprintln!(
                "  warning: failed to store output payload for '{}': {}",
                node_plan.node_id.0, error
            );
            return;
        }
    };

    let status = if success {
        LedgerStatus::Executed {
            reason: match &node_plan.action {
                PlanAction::Execute { miss_reason } => miss_reason.clone(),
                _ => MissReason::NoPriorRun,
            },
        }
    } else {
        LedgerStatus::Failed {
            reason: match &node_plan.action {
                PlanAction::Execute { miss_reason } => miss_reason.clone(),
                _ => MissReason::NoPriorRun,
            },
            error: "unit command failed".to_string(),
        }
    };

    let entry = RunLedgerEntry {
        exec_node_id: node_plan.node_id.clone(),
        work_id: node_plan.work_id.clone(),
        key: node_plan.key.clone(),
        status,
        output_hashes: BTreeMap::from([(PortName::from("result"), hash)]),
        duration_ms,
    };
    if let Err(error) = append_global_ledger_entry(workspace_root, entry) {
        eprintln!(
            "  warning: failed to persist ledger entry for '{}': {}",
            node_plan.node_id.0, error
        );
    }
}

/// Persist a ledger entry for a skipped unit (blocked by upstream failure).
fn persist_skipped_entry(workspace_root: &Path, node_plan: &NodePlan) {
    let entry = RunLedgerEntry {
        exec_node_id: node_plan.node_id.clone(),
        work_id: node_plan.work_id.clone(),
        key: node_plan.key.clone(),
        status: LedgerStatus::Skipped {
            blocked_by: node_plan.node_id.clone(),
        },
        output_hashes: BTreeMap::new(),
        duration_ms: 0,
    };
    if let Err(error) = append_global_ledger_entry(workspace_root, entry) {
        eprintln!(
            "  warning: failed to persist skipped entry for '{}': {}",
            node_plan.node_id.0, error
        );
    }
}

enum UnitStatus<'a> {
    CachedHit,
    Running(&'a str),
    Executed { success: bool },
    DryRun(&'a str),
    Skipped,
}

fn emit_unit_status(node_id: &NodeId, status: UnitStatus<'_>) {
    let is_ci = std::env::var("GITHUB_ACTIONS").is_ok();
    match status {
        UnitStatus::CachedHit => {
            println!("  [hit] {}", node_id.0);
        }
        UnitStatus::Running(label) => {
            if is_ci {
                println!("::group::{}", node_id.0);
            }
            println!("  [run] {} ({})", node_id.0, label);
        }
        UnitStatus::Executed { success } => {
            let marker = if success { "ok" } else { "FAIL" };
            println!("  [{}] {}", marker, node_id.0);
            if is_ci {
                println!("::endgroup::");
            }
        }
        UnitStatus::DryRun(label) => {
            println!("  [dry] {} ({})", node_id.0, label);
        }
        UnitStatus::Skipped => {
            println!("  [skip] {} (blocked by upstream failure)", node_id.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::key::{CanonicalKeyPayload, MaterializationKey, WorkIdentity};
    use crate::workflow::planner::WorkflowPlan;
    use crate::workflow::process_registry::ProcessId;
    use crate::workflow::coordination::CoordinationStatus;

    fn make_node_plan(name: &str, action: PlanAction) -> NodePlan {
        let work_id = WorkIdentity::new(ProcessId::new("test"), NodeId::from(name));
        let key = MaterializationKey::new(
            work_id.clone(),
            CanonicalKeyPayload {
                key_format_version: 1,
                op_version: 1,
                input_hashes: BTreeMap::new(),
                upstream_keys: BTreeMap::new(),
                policy_version: 1,
            },
        )
        .expect("key should build");

        NodePlan {
            node_id: NodeId::from(name),
            work_id,
            key,
            action,
        }
    }

    #[test]
    fn cached_hit_nodes_are_counted_not_executed() {
        let spec = crate::workflow::schema::WorkflowSpec::new(
            "test",
            gunbc_ir::Dag::new(),
            1,
        );
        let plan = WorkflowPlan {
            nodes: vec![make_node_plan(
                "a",
                PlanAction::CachedHit {
                    previous_run: "run-1".to_string(),
                    rehydrated_outputs: BTreeMap::new(),
                },
            )],
            coordination: CoordinationStatus {
                ready: vec![],
                blocked: BTreeMap::new(),
            },
        };

        let summary = execute_workflow_plan(
            &spec,
            &plan,
            &BTreeMap::new(),
            Path::new("/tmp/nonexistent"),
            true,
        );
        assert_eq!(summary.cache_hits, 1);
        assert_eq!(summary.executed, 0);
        assert!(summary.success());
    }

    #[test]
    fn execute_nodes_without_commands_succeed_as_noop() {
        let spec = crate::workflow::schema::WorkflowSpec::new(
            "test",
            gunbc_ir::Dag::new(),
            1,
        );
        let plan = WorkflowPlan {
            nodes: vec![make_node_plan(
                "report",
                PlanAction::Execute {
                    miss_reason: MissReason::NoPriorRun,
                },
            )],
            coordination: CoordinationStatus {
                ready: vec![],
                blocked: BTreeMap::new(),
            },
        };

        let summary = execute_workflow_plan(
            &spec,
            &plan,
            &BTreeMap::new(),
            Path::new("/tmp/nonexistent"),
            true,
        );
        assert_eq!(summary.executed, 1);
        assert!(summary.success());
    }

    #[test]
    fn dry_run_does_not_execute_commands() {
        let spec = crate::workflow::schema::WorkflowSpec::new(
            "test",
            gunbc_ir::Dag::new(),
            1,
        );
        let plan = WorkflowPlan {
            nodes: vec![make_node_plan(
                "build",
                PlanAction::Execute {
                    miss_reason: MissReason::NoPriorRun,
                },
            )],
            coordination: CoordinationStatus {
                ready: vec![],
                blocked: BTreeMap::new(),
            },
        };

        let mut commands = BTreeMap::new();
        commands.insert(
            NodeId::from("build"),
            UnitCommand::new("build", "false", vec![]),
        );

        // dry_run=true should not actually run the `false` command.
        let summary = execute_workflow_plan(
            &spec,
            &plan,
            &commands,
            Path::new("/tmp/nonexistent"),
            true,
        );
        assert_eq!(summary.executed, 1);
        assert!(summary.success());
    }

    #[test]
    fn execution_summary_reports_correct_totals() {
        let spec = crate::workflow::schema::WorkflowSpec::new(
            "test",
            gunbc_ir::Dag::new(),
            1,
        );
        let plan = WorkflowPlan {
            nodes: vec![
                make_node_plan(
                    "a",
                    PlanAction::CachedHit {
                        previous_run: "run-1".to_string(),
                        rehydrated_outputs: BTreeMap::new(),
                    },
                ),
                make_node_plan(
                    "b",
                    PlanAction::Execute {
                        miss_reason: MissReason::NoPriorRun,
                    },
                ),
            ],
            coordination: CoordinationStatus {
                ready: vec![],
                blocked: BTreeMap::new(),
            },
        };

        let summary = execute_workflow_plan(
            &spec,
            &plan,
            &BTreeMap::new(),
            Path::new("/tmp/nonexistent"),
            true,
        );
        assert_eq!(summary.total_units, 2);
        assert_eq!(summary.cache_hits, 1);
        assert_eq!(summary.executed, 1);
    }
}
