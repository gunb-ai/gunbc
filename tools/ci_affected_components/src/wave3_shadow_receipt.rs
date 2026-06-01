//! Wave 3 §11.7.2 shadow receipt — Phase 2 live CI host transport (queued eval).
//!
//! Modeled authority: `src/v4/workflow/ci.dag` (`CiSelectionReceipt`, `ci_selection_receipt_shadow_from_git_diff`).
//! Populated claim/testgen rows await bootstrap eval (`node://adhoc-331899f9-19a`); this transport emits an
//! honest queued receipt on every PR without fabricating `affected_set_from_diff`.

use std::io;

use serde_json::{json, Value};

use crate::{
    ci_component_affected_fail_closed, ci_component_affected_from_changed_paths, CiComponentAffected,
};
use crate::git_diff_transport::{GitChangedPathsRead, WAVE3_LIVE_EVAL_DEBT};

pub const RECEIPT_SCHEMA: &str = "gunbc/ci-selection-receipt-shadow/v1";
pub const EMIT_STEP_NAME: &str = "emit-ci-wave3-shadow-receipt";

pub fn build_queued_shadow_receipt(
    event_name: &str,
    git_read: GitChangedPathsRead,
) -> Value {
    let (git_diff_range, changed_paths, component_affected, git_diff_read_failed) = match git_read {
        GitChangedPathsRead::Ok { range, paths } => {
            let flags = ci_component_affected_from_changed_paths(paths.iter().map(String::as_str));
            (range, paths, flags, false)
        }
        GitChangedPathsRead::FailClosed { range } => (
            range,
            Vec::new(),
            ci_component_affected_fail_closed(),
            true,
        ),
    };

    json!({
        "schema": RECEIPT_SCHEMA,
        "mode": "Shadow",
        "provenance": "FixtureReceipt",
        "live_eval_status": "queued",
        "live_eval_debt": WAVE3_LIVE_EVAL_DEBT,
        "event_name": event_name,
        "git_diff_range": git_diff_range,
        "git_diff_read_failed": git_diff_read_failed,
        "changed_paths": changed_paths,
        "component_affected": component_affected_to_json(component_affected),
        "testclaim_decisions_populated": false,
        "testgen_slots_populated": false,
        "note": "Phase 2 host transport: claim/testgen partitions await ci_selection_receipt_shadow_from_git_diff eval (affected_set_from_diff + live Dag); step-only shadow remains FixtureReceipt until debt closes.",
    })
}

fn component_affected_to_json(flags: CiComponentAffected) -> Value {
    json!({
        "v2": flags.v2,
        "v3": flags.v3,
        "v4": flags.v4,
        "testclaim_corpus": flags.testclaim_corpus,
        "workflow_policy": flags.workflow_policy,
        "release_distribution": flags.release_distribution,
        "release_distribution_only": flags.release_distribution_only,
    })
}

pub fn write_receipt_json(path: &str, receipt: &Value) -> io::Result<()> {
    let body = serde_json::to_string_pretty(receipt).map_err(io::Error::other)?;
    std::fs::write(path, format!("{body}\n"))
}

pub fn emit_github_notice(receipt: &Value) -> {
    let changed = receipt["changed_paths"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let status = receipt["live_eval_status"]
        .as_str()
        .unwrap_or("unknown");
    let debt = receipt["live_eval_debt"].as_str().unwrap_or("unknown");
    println!(
        "::notice title=Wave 3 shadow receipt (Class C)::status={status} changed_paths={changed} debt={debt} — receipt JSON on runner (see step log); LivePrGitDiff + claim rows queued on bootstrap eval"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_receipt_tags_eval_debt_and_empty_partitions() {
        let receipt = build_queued_shadow_receipt(
            "pull_request",
            GitChangedPathsRead::Ok {
                range: "origin/main...HEAD".to_string(),
                paths: vec!["src/v4/workflow/ci.dag".to_string()],
            },
        );
        assert_eq!(receipt["provenance"], "FixtureReceipt");
        assert_eq!(receipt["live_eval_status"], "queued");
        assert_eq!(receipt["live_eval_debt"], WAVE3_LIVE_EVAL_DEBT);
        assert_eq!(receipt["testclaim_decisions_populated"], false);
        assert_eq!(receipt["component_affected"]["v4"], true);
    }
}
