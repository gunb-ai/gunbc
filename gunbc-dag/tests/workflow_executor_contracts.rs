//! Contract tests for workflow executor (WF6/WF7) and SLO instrumentation (WF9).
#![allow(clippy::disallowed_methods)]

use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    check_slo, ci_unit_commands, ci_workflow_spec, default_process_unit_registry,
    default_slo_budgets, execute_workflow_plan, explain_plan, plan_workflow,
    render_execution_report, test_all_unit_commands, test_all_workflow_spec, top_slow_units,
    ExecutionSummary, MissReason, PlannerInputs, SloBudget, SloResult, UnitResult,
};
use gunbc_ir::NodeId;

fn temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-executor-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

// ============================================================================
// WF6: CI workflow executor contracts
// ============================================================================

#[test]
fn ci_workflow_dry_run_plans_all_units_as_execute_on_cold_ledger() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let plan = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root).expect("plan");

    let commands = ci_unit_commands();
    let summary = execute_workflow_plan(&spec, &plan, &commands, &root, true);

    // Cold ledger: all units should be executed (no cache hits).
    assert!(summary.success(), "dry-run should always succeed");
    assert_eq!(summary.total_units, plan.nodes.len());
    assert_eq!(
        summary.cache_hits, 0,
        "cold ledger should have no cache hits"
    );
    assert!(summary.executed > 0, "should have executed units");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ci_unit_commands_map_covers_all_non_report_nodes() {
    let spec = ci_workflow_spec().expect("ci spec");
    let commands = ci_unit_commands();
    for node in &spec.dag.nodes {
        if node.id.0.ends_with(".report") {
            // Report nodes are no-ops; no command expected.
            assert!(
                !commands.contains_key(&node.id),
                "report node '{}' should not have a command",
                node.id.0
            );
        } else {
            assert!(
                commands.contains_key(&node.id),
                "non-report node '{}' should have a command",
                node.id.0
            );
        }
    }
}

#[test]
fn ci_executor_preserves_topological_unit_ordering() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let plan = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root).expect("plan");

    let commands = ci_unit_commands();
    let summary = execute_workflow_plan(&spec, &plan, &commands, &root, true);

    // Results should be in same order as plan nodes.
    assert_eq!(summary.results.len(), plan.nodes.len());
    for (result, node_plan) in summary.results.iter().zip(plan.nodes.iter()) {
        assert_eq!(result.node_id, node_plan.node_id);
    }
    let _ = std::fs::remove_dir_all(root);
}

// ============================================================================
// WF7: test-all workflow executor contracts
// ============================================================================

#[test]
fn test_all_workflow_dry_run_plans_all_units() {
    let root = temp_root();
    let spec = test_all_workflow_spec().expect("test-all spec");
    let registry = default_process_unit_registry();
    let plan = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root).expect("plan");

    let commands = test_all_unit_commands();
    let summary = execute_workflow_plan(&spec, &plan, &commands, &root, true);

    assert!(summary.success());
    assert_eq!(summary.total_units, plan.nodes.len());
    assert_eq!(summary.cache_hits, 0);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn test_all_unit_commands_map_covers_all_non_report_nodes() {
    let spec = test_all_workflow_spec().expect("test-all spec");
    let commands = test_all_unit_commands();
    for node in &spec.dag.nodes {
        if node.id.0.ends_with(".report") {
            assert!(
                !commands.contains_key(&node.id),
                "report node '{}' should not have a command",
                node.id.0
            );
        } else {
            assert!(
                commands.contains_key(&node.id),
                "non-report node '{}' should have a command",
                node.id.0
            );
        }
    }
}

// ============================================================================
// WF9: SLO instrumentation contracts
// ============================================================================

#[test]
fn default_slo_budgets_cover_ci_and_test_all() {
    let budgets = default_slo_budgets();
    assert!(budgets.iter().any(|b| b.workflow_id == "ci"));
    assert!(budgets.iter().any(|b| b.workflow_id == "test-all"));
}

#[test]
fn slo_warm_noop_within_budget_passes() {
    let summary = ExecutionSummary {
        workflow_id: "ci".to_string(),
        total_units: 11,
        cache_hits: 11,
        executed: 0,
        failed: 0,
        pending_approvals: 0,
        skipped: 0,
        results: vec![],
        total_duration_ms: 1_000,
    };
    let budget = SloBudget {
        workflow_id: "ci".to_string(),
        warm_noop_ms: 5_000,
        total_max_ms: 600_000,
    };
    assert!(check_slo(&summary, &budget).is_pass());
}

#[test]
fn slo_warm_noop_exceeding_budget_fails() {
    let summary = ExecutionSummary {
        workflow_id: "ci".to_string(),
        total_units: 11,
        cache_hits: 11,
        executed: 0,
        failed: 0,
        pending_approvals: 0,
        skipped: 0,
        results: vec![],
        total_duration_ms: 8_000,
    };
    let budget = SloBudget {
        workflow_id: "ci".to_string(),
        warm_noop_ms: 5_000,
        total_max_ms: 600_000,
    };
    assert!(matches!(
        check_slo(&summary, &budget),
        SloResult::WarmNoopExceeded { .. }
    ));
}

#[test]
fn slo_total_exceeding_budget_fails() {
    let summary = ExecutionSummary {
        workflow_id: "ci".to_string(),
        total_units: 11,
        cache_hits: 0,
        executed: 11,
        failed: 0,
        pending_approvals: 0,
        skipped: 0,
        results: vec![],
        total_duration_ms: 700_000,
    };
    let budget = SloBudget {
        workflow_id: "ci".to_string(),
        warm_noop_ms: 5_000,
        total_max_ms: 600_000,
    };
    assert!(matches!(
        check_slo(&summary, &budget),
        SloResult::TotalExceeded { .. }
    ));
}

#[test]
fn top_slow_units_sorted_by_duration_descending() {
    let results = vec![
        UnitResult {
            node_id: NodeId::from("fast"),
            success: true,
            cached: false,
            pending_approval: false,
            duration_ms: 100,
            miss_reason: Some(MissReason::NoPriorRun),
        },
        UnitResult {
            node_id: NodeId::from("slow"),
            success: true,
            cached: false,
            pending_approval: false,
            duration_ms: 5_000,
            miss_reason: Some(MissReason::NoPriorRun),
        },
        UnitResult {
            node_id: NodeId::from("medium"),
            success: true,
            cached: false,
            pending_approval: false,
            duration_ms: 2_000,
            miss_reason: Some(MissReason::NoPriorRun),
        },
    ];
    let slow = top_slow_units(&results, 3);
    assert_eq!(slow.len(), 3);
    assert_eq!(slow[0].node_id, "slow");
    assert_eq!(slow[1].node_id, "medium");
    assert_eq!(slow[2].node_id, "fast");
}

#[test]
fn top_slow_units_excludes_cached_hits() {
    let results = vec![
        UnitResult {
            node_id: NodeId::from("cached"),
            success: true,
            cached: true,
            pending_approval: false,
            duration_ms: 0,
            miss_reason: None,
        },
        UnitResult {
            node_id: NodeId::from("executed"),
            success: true,
            cached: false,
            pending_approval: false,
            duration_ms: 1_000,
            miss_reason: Some(MissReason::NoPriorRun),
        },
    ];
    let slow = top_slow_units(&results, 5);
    assert_eq!(slow.len(), 1);
    assert_eq!(slow[0].node_id, "executed");
}

#[test]
fn render_execution_report_includes_slo_and_summary() {
    let root = temp_root();
    let spec = ci_workflow_spec().expect("ci spec");
    let registry = default_process_unit_registry();
    let plan = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root).expect("plan");
    let commands = ci_unit_commands();
    let summary = execute_workflow_plan(&spec, &plan, &commands, &root, true);
    let explain = explain_plan(&spec, &plan);

    let report = render_execution_report(&summary, &explain, &SloResult::Pass);
    assert!(report.contains("workflow: ci"));
    assert!(report.contains("slo: PASS"));
    assert!(report.contains("result: PASS"));
    assert!(report.contains("critical-path:"));
    let _ = std::fs::remove_dir_all(root);
}

// ============================================================================
// WF8: Makefile thinning contract (core target registration)
// ============================================================================

#[test]
fn makefile_ci_and_test_all_registered_as_core_workflows() {
    let workflows = gunbc_dag::makegen::default_core_workflows();
    assert!(
        workflows.iter().any(|w| w.name == "ci"),
        "ci should be registered as a core workflow"
    );
    assert!(
        workflows.iter().any(|w| w.name == "test-all"),
        "test-all should be registered as a core workflow"
    );
}
