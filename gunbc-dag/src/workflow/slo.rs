//! Latency SLO instrumentation and guardrails (WF9).
//!
//! Provides run-ledger timing metrics, warm-path budget assertions,
//! and "top slow units" reporting for CI integration.

use super::executor::{ExecutionSummary, UnitResult};
use super::planner::PlanExplain;

/// SLO budget thresholds (milliseconds).
#[derive(Debug, Clone)]
pub struct SloBudget {
    /// Workflow identifier this budget applies to.
    pub workflow_id: String,
    /// Maximum total duration for warm no-op (all CachedHit) in ms.
    pub warm_noop_ms: u64,
    /// Maximum total duration for any execution in ms.
    pub total_max_ms: u64,
}

/// Default SLO budgets from the design doc (Section 12).
pub fn default_slo_budgets() -> Vec<SloBudget> {
    vec![
        SloBudget {
            workflow_id: "ci".to_string(),
            warm_noop_ms: 5_000,
            total_max_ms: 600_000,
        },
        SloBudget {
            workflow_id: "test-all".to_string(),
            warm_noop_ms: 10_000,
            total_max_ms: 600_000,
        },
    ]
}

/// SLO check result.
#[derive(Debug, Clone)]
pub enum SloResult {
    Pass,
    WarmNoopExceeded { budget_ms: u64, actual_ms: u64 },
    TotalExceeded { budget_ms: u64, actual_ms: u64 },
}

impl SloResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, SloResult::Pass)
    }
}

/// Check an execution summary against SLO budgets.
pub fn check_slo(summary: &ExecutionSummary, budget: &SloBudget) -> SloResult {
    let is_warm_noop = summary.executed == 0 && summary.failed == 0;
    if is_warm_noop && summary.total_duration_ms > budget.warm_noop_ms {
        return SloResult::WarmNoopExceeded {
            budget_ms: budget.warm_noop_ms,
            actual_ms: summary.total_duration_ms,
        };
    }
    if summary.total_duration_ms > budget.total_max_ms {
        return SloResult::TotalExceeded {
            budget_ms: budget.total_max_ms,
            actual_ms: summary.total_duration_ms,
        };
    }
    SloResult::Pass
}

/// Slow-unit report entry for "top N slow units" output.
#[derive(Debug, Clone)]
pub struct SlowUnit {
    pub node_id: String,
    pub duration_ms: u64,
    pub cached: bool,
}

/// Extract top N slowest executed (non-cached) units from results.
pub fn top_slow_units(results: &[UnitResult], top_n: usize) -> Vec<SlowUnit> {
    let mut executed: Vec<&UnitResult> = results.iter().filter(|r| !r.cached).collect();
    executed.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
    executed
        .into_iter()
        .take(top_n)
        .map(|r| SlowUnit {
            node_id: r.node_id.0.clone(),
            duration_ms: r.duration_ms,
            cached: r.cached,
        })
        .collect()
}

/// Render human-readable execution summary with SLO status.
pub fn render_execution_report(
    summary: &ExecutionSummary,
    explain: &PlanExplain,
    slo_result: &SloResult,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("workflow: {}\n", summary.workflow_id));
    out.push_str(&format!("total-units: {}\n", summary.total_units));
    out.push_str(&format!("cache-hits: {}\n", summary.cache_hits));
    out.push_str(&format!("executed: {}\n", summary.executed));
    out.push_str(&format!("failed: {}\n", summary.failed));
    out.push_str(&format!("skipped: {}\n", summary.skipped));
    out.push_str(&format!(
        "total-duration-ms: {}\n",
        summary.total_duration_ms
    ));

    // Critical path from planner.
    out.push_str("critical-path:\n");
    for node in &explain.critical_path {
        out.push_str(&format!("  - {}\n", node.0));
    }

    // Top slow units.
    let slow = top_slow_units(&summary.results, 5);
    if !slow.is_empty() {
        out.push_str("top-slow-units:\n");
        for unit in &slow {
            out.push_str(&format!("  - {} ({}ms)\n", unit.node_id, unit.duration_ms));
        }
    }

    // SLO status.
    match slo_result {
        SloResult::Pass => {
            out.push_str("slo: PASS\n");
        }
        SloResult::WarmNoopExceeded {
            budget_ms,
            actual_ms,
        } => {
            out.push_str(&format!(
                "slo: FAIL (warm no-op exceeded: {}ms > {}ms budget)\n",
                actual_ms, budget_ms
            ));
        }
        SloResult::TotalExceeded {
            budget_ms,
            actual_ms,
        } => {
            out.push_str(&format!(
                "slo: FAIL (total exceeded: {}ms > {}ms budget)\n",
                actual_ms, budget_ms
            ));
        }
    }

    // Overall result.
    let overall = if summary.success() && slo_result.is_pass() {
        "PASS"
    } else {
        "FAIL"
    };
    out.push_str(&format!("result: {overall}\n"));
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gunbc_ir::NodeId;

    use super::*;
    use crate::workflow::key::MissReason;

    fn make_summary(
        workflow_id: &str,
        cache_hits: usize,
        executed: usize,
        failed: usize,
        total_ms: u64,
    ) -> ExecutionSummary {
        ExecutionSummary {
            workflow_id: workflow_id.to_string(),
            total_units: cache_hits + executed,
            cache_hits,
            executed,
            failed,
            skipped: 0,
            results: vec![],
            total_duration_ms: total_ms,
        }
    }

    #[test]
    fn warm_noop_within_budget_passes() {
        let summary = make_summary("ci", 11, 0, 0, 2_000);
        let budget = SloBudget {
            workflow_id: "ci".to_string(),
            warm_noop_ms: 5_000,
            total_max_ms: 600_000,
        };
        assert!(check_slo(&summary, &budget).is_pass());
    }

    #[test]
    fn warm_noop_exceeding_budget_fails() {
        let summary = make_summary("ci", 11, 0, 0, 8_000);
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
    fn total_duration_exceeding_budget_fails() {
        let summary = make_summary("ci", 0, 11, 0, 700_000);
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
    fn top_slow_units_returns_correct_ordering() {
        let results = vec![
            UnitResult {
                node_id: NodeId::from("fast"),
                success: true,
                cached: false,
                duration_ms: 100,
                miss_reason: Some(MissReason::NoPriorRun),
            },
            UnitResult {
                node_id: NodeId::from("slow"),
                success: true,
                cached: false,
                duration_ms: 5_000,
                miss_reason: Some(MissReason::NoPriorRun),
            },
            UnitResult {
                node_id: NodeId::from("cached"),
                success: true,
                cached: true,
                duration_ms: 0,
                miss_reason: None,
            },
        ];
        let slow = top_slow_units(&results, 5);
        assert_eq!(slow.len(), 2);
        assert_eq!(slow[0].node_id, "slow");
        assert_eq!(slow[1].node_id, "fast");
    }

    #[test]
    fn render_report_includes_slo_pass() {
        let summary = make_summary("ci", 11, 0, 0, 1_000);
        let explain = PlanExplain {
            execute_set: vec![],
            cache_hit_set: vec![],
            miss_reasons: BTreeMap::new(),
            blocked: BTreeMap::new(),
            ready: vec![],
            critical_path: vec![NodeId::from("ci.lint_upsert")],
            capability_status: BTreeMap::new(),
        };
        let report = render_execution_report(&summary, &explain, &SloResult::Pass);
        assert!(report.contains("slo: PASS"));
        assert!(report.contains("result: PASS"));
    }

    #[test]
    fn render_report_includes_slo_failure() {
        let summary = make_summary("ci", 11, 0, 0, 8_000);
        let explain = PlanExplain {
            execute_set: vec![],
            cache_hit_set: vec![],
            miss_reasons: BTreeMap::new(),
            blocked: BTreeMap::new(),
            ready: vec![],
            critical_path: vec![],
            capability_status: BTreeMap::new(),
        };
        let slo = SloResult::WarmNoopExceeded {
            budget_ms: 5_000,
            actual_ms: 8_000,
        };
        let report = render_execution_report(&summary, &explain, &slo);
        assert!(report.contains("slo: FAIL"));
        assert!(report.contains("result: FAIL"));
    }
}
