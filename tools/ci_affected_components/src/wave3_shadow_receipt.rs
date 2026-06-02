//! Wave 3 §11.7.2 shadow receipt — Phase 2 live CI host transport (queued eval).
//!
//! Modeled authority: `src/v4/workflow/ci.dag` (`CiSelectionReceipt`, `ci_selection_receipt_shadow_from_git_diff`).
//!
//! **This is a transport STATUS envelope, NOT a `CiSelectionReceipt` serialization (INVARIANTS P2/P3).**
//! The host cannot construct a faithful `CiSelectionReceipt` for a PR: `pr: ChangeSet` needs
//! `Change.subject: Node`, `affected: AffectedSet` needs a Dag eval, and a *fail-closed* Wave 3
//! receipt is NOT empty — it preserves the FULL roster as a superset
//! (`ci_wave3_shadow_testclaim_selection_rows` over `ci_wave3_shadow_claim_roster`, `ci.dag`).
//! So the host emits ONLY what it authoritatively computes — `component_affected` (the modeled
//! `CiComponentAffected` frontier, identical to `detect-ci-affected-components`) — plus queued
//! transport status that names the eval debt (`node://adhoc-331899f9-19a`).
//!
//! **Typed carrier (P3 / CODING).** Construction goes through the typed [`ShadowEmit`] /
//! [`GitReadStatus`] carriers, not best-effort JSON re-parsing: a fail-closed git read carries its
//! diagnostic `detail` into BOTH the JSON envelope (`git_diff_read_failed` + `git_diff_read_detail`)
//! and the GitHub notice (`git_read=FAIL-CLOSED(...)`), so a fail-closed read can never collapse to
//! the same visible summary as a successful empty diff.
//!
//! Wired as a non-blocking (Class C) step in the `affected` job (`.github/workflows/ci.yml`),
//! mirrored in the pinned carrier `dsl/gunbc/ci_github_actions_workflow.dag`.

use std::io;

use serde_json::{json, Value};

use crate::git_diff_transport::{GitChangedPathsRead, WAVE3_LIVE_EVAL_DEBT};
use crate::{
    ci_component_affected_fail_closed, ci_component_affected_from_changed_paths,
    CiComponentAffected,
};

/// Transport STATUS envelope tag. This is explicitly NOT a `CiSelectionReceipt` — the receipt
/// authority is the modeled `v4.workflow.ci.CiSelectionReceipt`, constructed by the live eval.
pub const TRANSPORT_ENVELOPE: &str = "gunbc/ci-wave3-shadow-emit/v1";
pub const MODELED_RECEIPT_AUTHORITY: &str = "v4.workflow.ci.CiSelectionReceipt";
pub const EMIT_STEP_NAME: &str = "emit-ci-wave3-shadow-receipt";

/// Typed host git-read outcome. A fail-closed read keeps its diagnostic `detail` so the boundary
/// state survives into the envelope + notice (no fabricated plausible "empty diff" output; P3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitReadStatus {
    Ok,
    FailClosed { detail: String },
}

/// Typed transport-emit summary. Single source for both the JSON envelope and the GitHub notice,
/// so they cannot diverge and the fail-closed state is always surfaced.
#[derive(Debug, Clone)]
pub struct ShadowEmit {
    pub event_name: String,
    pub git_diff_range: String,
    pub git_read: GitReadStatus,
    pub changed_paths: Vec<String>,
    pub component_affected: CiComponentAffected,
}

/// Build the typed emit summary from the shared git-diff transport read.
pub fn build_shadow_emit(event_name: &str, git_read: GitChangedPathsRead) -> ShadowEmit {
    match git_read {
        GitChangedPathsRead::Ok { range, paths } => {
            let component_affected =
                ci_component_affected_from_changed_paths(paths.iter().map(String::as_str));
            ShadowEmit {
                event_name: event_name.to_string(),
                git_diff_range: range,
                git_read: GitReadStatus::Ok,
                changed_paths: paths,
                component_affected,
            }
        }
        // Fail-closed: keep the diagnostic detail; component frontier is the fail-closed superset.
        GitChangedPathsRead::FailClosed { range, detail } => ShadowEmit {
            event_name: event_name.to_string(),
            git_diff_range: range,
            git_read: GitReadStatus::FailClosed { detail },
            changed_paths: Vec::new(),
            component_affected: ci_component_affected_fail_closed(),
        },
    }
}

/// Serialize the typed summary to the transport status envelope JSON.
pub fn shadow_emit_to_json(emit: &ShadowEmit) -> Value {
    let (git_diff_read_failed, git_diff_read_detail) = match &emit.git_read {
        GitReadStatus::Ok => (false, Value::Null),
        GitReadStatus::FailClosed { detail } => (true, Value::String(detail.clone())),
    };
    json!({
        "transport_envelope": TRANSPORT_ENVELOPE,
        "ci_selection_receipt_status": "queued",
        "modeled_receipt_authority": MODELED_RECEIPT_AUTHORITY,
        "live_eval_debt": WAVE3_LIVE_EVAL_DEBT,
        "event_name": emit.event_name,
        "git_diff_range": emit.git_diff_range,
        "git_diff_read_failed": git_diff_read_failed,
        // Diagnostic detail for a fail-closed read (null on success) — keeps the boundary state typed.
        "git_diff_read_detail": git_diff_read_detail,
        // Raw git paths are transport-only diagnostics, NOT the modeled `ChangeSet`.
        "changed_paths": emit.changed_paths,
        "note": "Host transport STATUS only — NOT a CiSelectionReceipt serialization. The modeled receipt (pr/affected/decisions/testclaim_decisions/testgen_slots — incl. the fail-closed roster SUPERSET, not empty) is constructed by ci_selection_receipt_shadow_from_git_diff under the live eval (live_eval_debt). The host authoritatively computes only component_affected (the modeled CiComponentAffected frontier, same as detect-ci-affected-components).",
        // The one partition the host computes faithfully: the modeled `CiComponentAffected` frontier.
        "component_affected": component_affected_to_json(emit.component_affected),
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

/// GitHub Actions `::notice` line, rendered from the TYPED summary (not best-effort JSON). A
/// fail-closed git read renders `git_read=FAIL-CLOSED(<detail>)`, distinct from `git_read=ok`, so
/// it can never look like a successful empty diff. Pure — the bin prints it (impurity at the edge).
pub fn github_notice_line(emit: &ShadowEmit) -> String {
    let git_read = match &emit.git_read {
        GitReadStatus::Ok => "git_read=ok".to_string(),
        GitReadStatus::FailClosed { detail } => format!("git_read=FAIL-CLOSED({detail})"),
    };
    format!(
        "::notice title=Wave 3 shadow receipt (Class C)::ci_selection_receipt_status=queued {git_read} changed_paths={} debt={WAVE3_LIVE_EVAL_DEBT} — transport status only; modeled CiSelectionReceipt (incl. fail-closed roster superset) constructed by the queued live eval",
        emit.changed_paths.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_is_status_only_with_faithful_component_affected() {
        let emit = build_shadow_emit(
            "pull_request",
            GitChangedPathsRead::Ok {
                range: "origin/main...HEAD".to_string(),
                paths: vec!["src/v4/workflow/ci.dag".to_string()],
            },
        );
        let receipt = shadow_emit_to_json(&emit);

        // Transport status envelope, naming the modeled authority + eval debt.
        assert_eq!(receipt["transport_envelope"], TRANSPORT_ENVELOPE);
        assert_eq!(receipt["ci_selection_receipt_status"], "queued");
        assert_eq!(
            receipt["modeled_receipt_authority"],
            MODELED_RECEIPT_AUTHORITY
        );
        assert_eq!(receipt["live_eval_debt"], WAVE3_LIVE_EVAL_DEBT);

        // The one faithfully-computed modeled partition.
        assert_eq!(receipt["component_affected"]["v4"], true);

        // Successful read: not failed, no detail.
        assert_eq!(receipt["git_diff_read_failed"], false);
        assert_eq!(receipt["git_diff_read_detail"], Value::Null);

        // The host does NOT emit a CiSelectionReceipt or any partition it cannot faithfully populate.
        assert!(receipt.get("ci_selection_receipt").is_none());
        assert!(receipt.get("testclaim_decisions").is_none());
        assert!(receipt.get("affected").is_none());
        assert!(receipt.get("pr").is_none());
    }

    #[test]
    fn fail_closed_git_read_surfaces_detail_in_envelope_and_notice() {
        let emit = build_shadow_emit(
            "pull_request",
            GitChangedPathsRead::FailClosed {
                range: "origin/main...HEAD".to_string(),
                detail: "git diff --name-only origin/main...HEAD exited 128".to_string(),
            },
        );
        let receipt = shadow_emit_to_json(&emit);

        // The git-read-failure fact + diagnostic detail survive into the envelope (P3, not discarded).
        assert_eq!(receipt["git_diff_read_failed"], true);
        assert_eq!(
            receipt["git_diff_read_detail"],
            "git diff --name-only origin/main...HEAD exited 128"
        );
        // Component frontier is the fail-closed superset (all flags true) — the host's honest P3 default.
        assert_eq!(receipt["component_affected"]["v2"], true);
        assert_eq!(receipt["component_affected"]["v4"], true);

        // The notice must distinguish fail-closed from a successful empty diff (the reviewed gap).
        let notice = github_notice_line(&emit);
        assert!(notice.contains("git_read=FAIL-CLOSED("));
        assert!(notice.contains("exited 128"));
        assert!(!notice.contains("git_read=ok"));
    }

    #[test]
    fn ok_read_notice_is_distinct_from_fail_closed() {
        let emit = build_shadow_emit(
            "pull_request",
            GitChangedPathsRead::Ok {
                range: "origin/main...HEAD".to_string(),
                paths: vec![],
            },
        );
        let notice = github_notice_line(&emit);
        // A real empty diff reads ok with zero paths — and is NOT labeled fail-closed.
        assert!(notice.contains("git_read=ok"));
        assert!(notice.contains("changed_paths=0"));
        assert!(!notice.contains("FAIL-CLOSED"));
    }
}
