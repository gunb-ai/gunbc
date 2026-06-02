//! Wave 3 §11.7.2 shadow receipt — Phase 2 live CI host transport (queued eval).
//!
//! Modeled authority: `src/v4/workflow/ci.dag` (`CiSelectionReceipt`, `ci_selection_receipt_shadow_from_git_diff`).
//!
//! **Single-authority discipline (INVARIANTS P2).** This transport does NOT invent a parallel
//! receipt genus. The emitted JSON is a *clearly non-authoritative transport envelope* whose
//! `ci_selection_receipt` field is a faithful serialization of the modeled `CiSelectionReceipt`
//! (exact modeled field names + coproduct tags). The host transport cannot resolve `Change.subject:
//! Node` or compute the `AffectedSet` without the live Dag eval, so `pr`/`affected`/`decisions`/
//! `testclaim_decisions`/`testgen_slots` are serialized in their honest unpopulated form — empty
//! `ChangeSet`/lists and a fail-closed `AffectedSet` (P3) — until the bootstrap eval
//! (`node://adhoc-331899f9-19a`) lands `ci_selection_receipt_shadow_from_git_diff`. Only
//! `component_affected_comparison` is host-computed (same predicate set as `detect-ci-affected-components`).
//! Status/debt live in the envelope, never mixed into the receipt fields.
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

/// Non-authoritative transport envelope tag (NOT a receipt-authority schema — the receipt authority
/// is the modeled `v4.workflow.ci.CiSelectionReceipt`, serialized under `ci_selection_receipt`).
pub const TRANSPORT_ENVELOPE: &str = "gunbc/ci-wave3-shadow-emit/v1";
pub const RECEIPT_AUTHORITY: &str = "v4.workflow.ci.CiSelectionReceipt";
pub const EMIT_STEP_NAME: &str = "emit-ci-wave3-shadow-receipt";

/// Model-declared `Symbol` for the shadow fail-closed `AffectedSetReason.reason`
/// (`src/v4/workflow/ci.dag`: `data ci_selection_shadow_reason: Symbol`). Single authority —
/// do NOT substitute host-invented lifecycle strings here (P1 grounded-Symbol discipline).
pub const SHADOW_REASON: &str = "ci_selection_shadow_reason";

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
        "authority": RECEIPT_AUTHORITY,
        "live_eval_status": "queued",
        "live_eval_debt": WAVE3_LIVE_EVAL_DEBT,
        "event_name": event_name,
        "git_diff_range": git_diff_range,
        "git_diff_read_failed": git_diff_read_failed,
        // Raw git paths are transport-only diagnostics, NOT the modeled `ChangeSet`
        // (which requires `Change.subject: Node` the host cannot resolve without the eval).
        "changed_paths": changed_paths,
        "note": "Non-authoritative transport envelope. `ci_selection_receipt` is a serialization of the modeled v4.workflow.ci CiSelectionReceipt (Shadow/FixtureReceipt). pr/affected/decisions/testclaim_decisions/testgen_slots await ci_selection_receipt_shadow_from_git_diff live eval (live_eval_debt); affected is fail-closed (P3) until then. Only component_affected_comparison is host-computed.",
        "ci_selection_receipt": ci_selection_receipt_to_json(component_affected),
    })
}

/// Faithful serialization of the modeled `CiSelectionReceipt` in its honest queued/shadow form.
/// Field names and coproduct tags mirror `src/v4/workflow/ci.dag` exactly (single authority, P2).
/// The shadow fail-closed `AffectedSet` is the modeled `ci_selection_receipt_shadow_fail_closed_affected`
/// (`src/v4/workflow/ci.dag`) — the `reason` is the model-declared `Symbol` `ci_selection_shadow_reason`,
/// NOT a host-invented lifecycle string (transport status stays in the envelope; P1/P2).
fn ci_selection_receipt_to_json(flags: CiComponentAffected) -> Value {
    json!({
        // `pr: ChangeSet { changes: List<Change> }` — empty: host cannot resolve `Change.subject: Node`.
        "pr": { "changes": [] },
        // `affected: AffectedSet` — fail-closed coproduct (never computed without Dag eval; P3).
        // Mirrors modeled `ci_selection_receipt_shadow_fail_closed_affected`.
        "affected": {
            "AffectedSetFailClosed": {
                "changes": { "changes": [] },
                "evidence": { "AffectedSetReason": { "reason": SHADOW_REASON } }
            }
        },
        // `mode: CiSelectionMode` — nullary `Shadow` variant.
        "mode": "Shadow",
        // `provenance: CiSelectionReceiptProvenance` — `FixtureReceipt` until live populate entry.
        "provenance": "FixtureReceipt",
        "decisions": [],
        "testclaim_decisions": [],
        "testgen_slots": [],
        "component_affected_comparison": component_affected_to_json(flags),
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

pub fn emit_github_notice(receipt: &Value) {
    let changed = receipt["changed_paths"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let status = receipt["live_eval_status"].as_str().unwrap_or("unknown");
    let debt = receipt["live_eval_debt"].as_str().unwrap_or("unknown");
    println!(
        "::notice title=Wave 3 shadow receipt (Class C)::status={status} changed_paths={changed} debt={debt} — CiSelectionReceipt serialization in transport envelope (see step log); pr/affected/claim/testgen partitions queued on bootstrap eval"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_wraps_modeled_receipt_serialization() {
        let receipt = build_queued_shadow_receipt(
            "pull_request",
            GitChangedPathsRead::Ok {
                range: "origin/main...HEAD".to_string(),
                paths: vec!["src/v4/workflow/ci.dag".to_string()],
            },
        );

        // Envelope carries non-authoritative status, NOT mixed into the receipt.
        assert_eq!(receipt["transport_envelope"], TRANSPORT_ENVELOPE);
        assert_eq!(receipt["authority"], RECEIPT_AUTHORITY);
        assert_eq!(receipt["live_eval_status"], "queued");
        assert_eq!(receipt["live_eval_debt"], WAVE3_LIVE_EVAL_DEBT);

        // `ci_selection_receipt` uses modeled field names + coproduct tags (single authority, P2).
        let r = &receipt["ci_selection_receipt"];
        assert_eq!(r["mode"], "Shadow");
        assert_eq!(r["provenance"], "FixtureReceipt");
        assert_eq!(r["pr"]["changes"], json!([]));
        assert_eq!(r["decisions"], json!([]));
        assert_eq!(r["testclaim_decisions"], json!([]));
        assert_eq!(r["testgen_slots"], json!([]));
        // Model-declared Symbol, NOT a host-invented lifecycle string (P1/P2).
        assert_eq!(
            r["affected"]["AffectedSetFailClosed"]["evidence"]["AffectedSetReason"]["reason"],
            SHADOW_REASON
        );
        assert_eq!(r["component_affected_comparison"]["v4"], true);
        // No invented parallel-genus fields on the receipt.
        assert!(r.get("schema").is_none());
        assert!(r.get("testclaim_decisions_populated").is_none());
    }

    #[test]
    fn fail_closed_git_read_marks_affected_reason_and_superset() {
        let receipt = build_queued_shadow_receipt(
            "pull_request",
            GitChangedPathsRead::FailClosed {
                range: "origin/main...HEAD".to_string(),
                detail: "git diff exited 1".to_string(),
            },
        );
        // The git-read-failure fact stays in the envelope, NOT in the modeled receipt.
        assert_eq!(receipt["git_diff_read_failed"], true);
        let r = &receipt["ci_selection_receipt"];
        // Receipt reason is the model-declared Symbol regardless of read outcome.
        assert_eq!(
            r["affected"]["AffectedSetFailClosed"]["evidence"]["AffectedSetReason"]["reason"],
            SHADOW_REASON
        );
        // Fail-closed component superset (all flags true).
        assert_eq!(r["component_affected_comparison"]["v2"], true);
        assert_eq!(r["component_affected_comparison"]["v4"], true);
    }
}
