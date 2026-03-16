//! Workflow executor: runs planned units via shell commands (WF6/WF7).
//!
//! This module takes a computed `WorkflowPlan` and executes it:
//! - All nodes are executed (no caching).
//! - A typed execution summary is returned with timing stats.
//!
//! `Command::new` is used here intentionally: the executor dispatches
//! coarse-grained CLI subprocesses (not fine-grained DAG node operations).
//! These commands correspond to workflow planner units, not transport ops.
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use gunbc_ir::NodeId;

use crate::key::MissReason;
use crate::planner::{PlanAction, WorkflowPlan};
use crate::schema::WorkflowSpec;

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
        Self::new(label, "cargo", args.into_iter().map(String::from).collect())
    }
}

/// Result of executing a single workflow unit.
#[derive(Debug, Clone)]
pub struct UnitResult {
    pub node_id: NodeId,
    pub success: bool,
    pub cached: bool,
    pub pending_approval: bool,
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
    pub pending_approvals: usize,
    pub skipped: usize,
    pub results: Vec<UnitResult>,
    pub total_duration_ms: u64,
}

impl ExecutionSummary {
    /// Whether the entire workflow succeeded (no failures).
    pub fn success(&self) -> bool {
        self.failed == 0 && self.pending_approvals == 0
    }
}

const PENDING_APPROVAL_EXIT_CODE: i32 = 42;

/// Execute a workflow plan, running all units.
///
/// Units are processed in topological order (as produced by the planner).
///
/// If `dry_run` is true, commands are printed but not executed.
pub fn execute_workflow_plan(
    spec: &WorkflowSpec,
    plan: &WorkflowPlan,
    commands: &BTreeMap<NodeId, UnitCommand>,
    _workspace_root: &Path,
    dry_run: bool,
) -> ExecutionSummary {
    let run_start = Instant::now();
    let is_ci = std::env::var("GITHUB_ACTIONS").is_ok();
    let mut results = Vec::new();
    let mut executed = 0usize;
    let mut failed = 0usize;
    let mut pending_approvals = 0usize;
    let mut skipped = 0usize;
    let mut has_failure = false;

    for node_plan in &plan.nodes {
        let PlanAction::Execute { miss_reason } = &node_plan.action;

        if has_failure {
            // Skip downstream units after a failure (fail-closed).
            skipped += 1;
            results.push(UnitResult {
                node_id: node_plan.node_id.clone(),
                success: false,
                cached: false,
                pending_approval: false,
                duration_ms: 0,
                miss_reason: Some(miss_reason.clone()),
            });
            emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::Skipped);
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
                pending_approval: false,
                duration_ms: 0,
                miss_reason: Some(miss_reason.clone()),
            });
            emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::Executed { success: true });
            continue;
        };

        if dry_run {
            executed += 1;
            emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::DryRun(&cmd.label));
            results.push(UnitResult {
                node_id: node_plan.node_id.clone(),
                success: true,
                cached: false,
                pending_approval: false,
                duration_ms: 0,
                miss_reason: Some(miss_reason.clone()),
            });
            continue;
        }

        emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::Running(&cmd.label));
        let unit_start = Instant::now();
        let execution_outcome = run_unit_command(cmd);
        let duration_ms = unit_start.elapsed().as_millis() as u64;
        executed += 1;

        let (success, pending_approval) = match execution_outcome {
            CommandExecutionOutcome::Success => {
                emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::Executed { success: true });
                (true, false)
            }
            CommandExecutionOutcome::PendingApproval => {
                pending_approvals += 1;
                has_failure = true;
                emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::PendingApproval);
                (false, true)
            }
            CommandExecutionOutcome::Failure => {
                failed += 1;
                has_failure = true;
                emit_unit_status(&node_plan.node_id, is_ci, UnitStatus::Executed { success: false });
                (false, false)
            }
        };

        results.push(UnitResult {
            node_id: node_plan.node_id.clone(),
            success,
            cached: false,
            pending_approval,
            duration_ms,
            miss_reason: Some(miss_reason.clone()),
        });
    }

    let total_duration_ms = run_start.elapsed().as_millis() as u64;
    ExecutionSummary {
        workflow_id: spec.id.0.clone(),
        total_units: plan.nodes.len(),
        cache_hits: 0,
        executed,
        failed,
        pending_approvals,
        skipped,
        results,
        total_duration_ms,
    }
}

/// Run a shell command, inheriting stdout/stderr. Returns true on success.
enum CommandExecutionOutcome {
    Success,
    PendingApproval,
    Failure,
}

#[allow(clippy::disallowed_macros)]
fn run_unit_command(cmd: &UnitCommand) -> CommandExecutionOutcome {
    let result = Command::new(&cmd.program)
        .args(&cmd.args)
        .env(gunbc_exec::freshness::FRESHNESS_ACTIVE_ENV, "1")
        .status();
    match result {
        Ok(status) if status.success() => CommandExecutionOutcome::Success,
        Ok(status) if status.code() == Some(PENDING_APPROVAL_EXIT_CODE) => {
            CommandExecutionOutcome::PendingApproval
        }
        Ok(_) => CommandExecutionOutcome::Failure,
        Err(error) => {
            eprintln!("  error: failed to spawn '{}': {}", cmd.program, error);
            CommandExecutionOutcome::Failure
        }
    }
}

enum UnitStatus<'a> {
    Running(&'a str),
    Executed { success: bool },
    DryRun(&'a str),
    PendingApproval,
    Skipped,
}

fn emit_unit_status(node_id: &NodeId, is_ci: bool, status: UnitStatus<'_>) {
    match status {
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
        UnitStatus::PendingApproval => {
            println!("  [await] {} (pending approval)", node_id.0);
            if is_ci {
                println!("::endgroup::");
            }
        }
        UnitStatus::Skipped => {
            println!("  [skip] {} (blocked by upstream failure)", node_id.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordination::CoordinationStatus;
    use crate::key::{CanonicalKeyPayload, MaterializationKey, WorkIdentity};
    use crate::planner::WorkflowPlan;
    use crate::process_registry::ProcessId;

    fn make_node_plan(name: &str, action: PlanAction) -> crate::planner::NodePlan {
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

        crate::planner::NodePlan {
            node_id: NodeId::from(name),
            work_id,
            key,
            action,
        }
    }

    #[test]
    fn execute_nodes_without_commands_succeed_as_noop() {
        let spec = crate::schema::WorkflowSpec::new("test", gunbc_ir::Dag::new(), 1);
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
        let spec = crate::schema::WorkflowSpec::new("test", gunbc_ir::Dag::new(), 1);
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
        let summary =
            execute_workflow_plan(&spec, &plan, &commands, Path::new("/tmp/nonexistent"), true);
        assert_eq!(summary.executed, 1);
        assert!(summary.success());
    }

    #[test]
    fn execution_summary_reports_correct_totals() {
        let spec = crate::schema::WorkflowSpec::new("test", gunbc_ir::Dag::new(), 1);
        let plan = WorkflowPlan {
            nodes: vec![
                make_node_plan(
                    "a",
                    PlanAction::Execute {
                        miss_reason: MissReason::NoPriorRun,
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
        assert_eq!(summary.cache_hits, 0);
        assert_eq!(summary.executed, 2);
    }

    #[test]
    fn pending_approval_exit_code_detected() {
        let spec = crate::schema::WorkflowSpec::new("test", gunbc_ir::Dag::new(), 1);
        let plan = WorkflowPlan {
            nodes: vec![make_node_plan(
                "approve",
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
            NodeId::from("approve"),
            UnitCommand::new(
                "await approval",
                "bash",
                vec!["-lc".into(), "exit 42".into()],
            ),
        );

        let root = std::env::temp_dir().join("gunbc-executor-pending-approval-test");

        let summary = execute_workflow_plan(&spec, &plan, &commands, &root, false);
        assert_eq!(summary.pending_approvals, 1);
        assert_eq!(summary.failed, 0);
        assert!(!summary.success());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn first_failure_skips_downstream_units() {
        let spec = crate::schema::WorkflowSpec::new("test", gunbc_ir::Dag::new(), 1);
        let plan = WorkflowPlan {
            nodes: vec![
                make_node_plan(
                    "build",
                    PlanAction::Execute {
                        miss_reason: MissReason::NoPriorRun,
                    },
                ),
                make_node_plan(
                    "publish",
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
        let mut commands = BTreeMap::new();
        commands.insert(
            NodeId::from("build"),
            UnitCommand::new("build", "bash", vec!["-lc".into(), "exit 1".into()]),
        );
        commands.insert(
            NodeId::from("publish"),
            UnitCommand::new("publish", "bash", vec!["-lc".into(), "exit 0".into()]),
        );

        let summary = execute_workflow_plan(
            &spec,
            &plan,
            &commands,
            Path::new("/tmp/nonexistent"),
            false,
        );

        assert_eq!(summary.executed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 1);
        assert!(!summary.success());
        assert_eq!(summary.results.len(), 2);
        assert_eq!(summary.results[0].node_id, NodeId::from("build"));
        assert!(!summary.results[0].success);
        assert_eq!(summary.results[1].node_id, NodeId::from("publish"));
        assert!(!summary.results[1].success);
        assert_eq!(summary.results[1].duration_ms, 0);
        assert!(!summary.results[1].pending_approval);
    }
}
