//! Wave 3 §11.7.2 shadow receipt — Phase 2 live CI host transport (queued eval).
//!
//! Modeled authority: `src/v4/workflow/ci.dag` (`CiSelectionReceipt`, `ci_selection_receipt_shadow_from_git_diff`).
//!
//! **This is a transport STATUS envelope, NOT a `CiSelectionReceipt` serialization (INVARIANTS P2/P3).**
//! The host cannot construct a faithful `CiSelectionReceipt` for a PR: `pr: ChangeSet` needs
//! `Change.subject: Node`, `affected: AffectedSet` needs a Dag eval, and — critically — a *fail-closed*
//! Wave 3 receipt is NOT empty: it preserves the FULL roster as a superset, populating
//! `testclaim_decisions` via `ci_wave3_shadow_testclaim_selection_rows` over
//! `ci_wave3_shadow_claim_roster` (`ci.dag`; `ci_wave3_shadow_fail_closed_selects_full_roster`).
//! Emitting those partitions empty would dilute the modeled fail-closed selection semantics (P3) and
//! make the host a second receipt authority (P2). The host also cannot synthesize the roster superset
//! without mirroring modeled roster data + eval-computed `claim_projection_hash`es (another authority
//! duplication). So the host emits ONLY what it authoritatively computes — `component_affected` (the
//! modeled `CiComponentAffected` frontier, identical to `detect-ci-affected-components`) — plus queued
//! transport status that names the eval debt (`node://adhoc-331899f9-19a`) which constructs the real
//! `CiSelectionReceipt`. It does NOT claim to be a receipt serialization for any partition it cannot
//! faithfully populate.
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

pub fn build_queued_shadow_receipt(event_name: &str, git_read: GitChangedPathsRead) -> Value {
    let (git_diff_range, changed_paths, component_affected, git_diff_read_failed) = match git_read {
        GitChangedPathsRead::Ok { range, paths } => {
            let flags = ci_component_affected_from_changed_paths(paths.iter().map(String::as_str));
            (range, paths, flags, false)
        }
        GitChangedPathsRead::FailClosed { range, detail: _ } => {
            (range, Vec::new(), ci_component_affected_fail_closed(), true)
        }
    };

    json!({
        "transport_envelope": TRANSPORT_ENVELOPE,
        "ci_selection_receipt_status": "queued",
        "modeled_receipt_authority": MODELED_RECEIPT_AUTHORITY,
        "live_eval_debt": WAVE3_LIVE_EVAL_DEBT,
        "event_name": event_name,
        "git_diff_range": git_diff_range,
        "git_diff_read_failed": git_diff_read_failed,
        // Raw git paths are transport-only diagnostics, NOT the modeled `ChangeSet`
        // (which requires `Change.subject: Node` the host cannot resolve without the eval).
        "changed_paths": changed_paths,
        "note": "Host transport STATUS only — NOT a CiSelectionReceipt serialization. The modeled receipt (pr/affected/decisions/testclaim_decisions/testgen_slots — incl. the fail-closed roster SUPERSET, not empty) is constructed by ci_selection_receipt_shadow_from_git_diff under the live eval (live_eval_debt). The host authoritatively computes only component_affected (the modeled CiComponentAffected frontier, same as detect-ci-affected-components).",
        // The one partition the host computes faithfully: the modeled `CiComponentAffected` frontier.
        "component_affected": component_affected_to_json(component_affected),
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

/// Build the GitHub Actions `::notice` line for the receipt. Pure — returns the string for the
/// binary entrypoint to print, keeping the impure stdout write at the CLI edge (CODING.md:
/// library modules do not print; `clippy::disallowed_macros` is allowed only on the bin).
pub fn github_notice_line(receipt: &Value) -> String {
    let changed = receipt["changed_paths"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let status = receipt["ci_selection_receipt_status"]
        .as_str()
        .unwrap_or("unknown");
    let debt = receipt["live_eval_debt"].as_str().unwrap_or("unknown");
    format!(
        "::notice title=Wave 3 shadow receipt (Class C)::ci_selection_receipt_status={status} changed_paths={changed} debt={debt} — transport status only; modeled CiSelectionReceipt (incl. fail-closed roster superset) constructed by the queued live eval"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_is_status_only_with_faithful_component_affected() {
        let receipt = build_queued_shadow_receipt(
            "pull_request",
            GitChangedPathsRead::Ok {
                range: "origin/main...HEAD".to_string(),
                paths: vec!["src/v4/workflow/ci.dag".to_string()],
            },
        );

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

        // The host does NOT emit a CiSelectionReceipt or any partition it cannot faithfully
        // populate — no diluted fail-closed semantics, no second receipt authority (P2/P3).
        assert!(receipt.get("ci_selection_receipt").is_none());
        assert!(receipt.get("testclaim_decisions").is_none());
        assert!(receipt.get("affected").is_none());
        assert!(receipt.get("pr").is_none());
    }

    #[test]
    fn fail_closed_git_read_marks_component_superset() {
        let receipt = build_queued_shadow_receipt(
            "pull_request",
            GitChangedPathsRead::FailClosed {
                range: "origin/main...HEAD".to_string(),
                detail: "git diff exited 1".to_string(),
            },
        );
        // The git-read-failure fact lives in the envelope status, not a faked receipt partition.
        assert_eq!(receipt["git_diff_read_failed"], true);
        assert_eq!(receipt["ci_selection_receipt_status"], "queued");
        // Component frontier is the fail-closed superset (all flags true) — the host's honest P3 default.
        assert_eq!(receipt["component_affected"]["v2"], true);
        assert_eq!(receipt["component_affected"]["v4"], true);
    }
}
