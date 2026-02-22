//! Provider-agnostic SDLC issue adapter for GitHub issues.
//!
//! This module defines the canonical issue snapshot shape consumed by SDLC
//! runtime code, plus capability-gate checks required for real-mode execution.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical SDLC lifecycle stages represented as GitHub labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueLifecycleStage {
    Idea,
    Design,
    DesignReview,
    Accepted,
    Implementing,
    CodeReview,
    Testing,
    Done,
    TerminalFailed,
}

impl IssueLifecycleStage {
    /// Canonical stage label used on provider issues.
    pub const fn as_label(self) -> &'static str {
        match self {
            IssueLifecycleStage::Idea => "idea",
            IssueLifecycleStage::Design => "design",
            IssueLifecycleStage::DesignReview => "design-review",
            IssueLifecycleStage::Accepted => "accepted",
            IssueLifecycleStage::Implementing => "implementing",
            IssueLifecycleStage::CodeReview => "code-review",
            IssueLifecycleStage::Testing => "testing",
            IssueLifecycleStage::Done => "done",
            IssueLifecycleStage::TerminalFailed => "terminal-failed",
        }
    }
}

/// Canonical issue identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueRef {
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

/// Provider-agnostic issue snapshot consumed by SDLC runtime operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackedIssue {
    pub reference: IssueRef,
    pub title: String,
    pub body: String,
    pub state: String,
    pub labels: Vec<String>,
    pub stage: IssueLifecycleStage,
}

/// Minimal GitHub issue record shape used by the adapter mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubIssueRecord {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub labels: Vec<String>,
}

/// Required issue-transport capabilities for SDLC real-mode execution (IM12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdlcIssueCapabilities {
    pub issue_read: bool,
    pub issue_comment_upsert: bool,
    pub issue_label_compare_and_set: bool,
    pub managed_issue_search: bool,
    pub deterministic_issue_identity: bool,
}

impl SdlcIssueCapabilities {
    /// Capabilities provided by GitHub issue transport.
    pub const fn github() -> Self {
        Self {
            issue_read: true,
            issue_comment_upsert: true,
            issue_label_compare_and_set: true,
            managed_issue_search: true,
            deterministic_issue_identity: true,
        }
    }

    /// Stub capabilities where nothing is available.
    pub const fn none() -> Self {
        Self {
            issue_read: false,
            issue_comment_upsert: false,
            issue_label_compare_and_set: false,
            managed_issue_search: false,
            deterministic_issue_identity: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageLabelError {
    ConflictingStages(Vec<IssueLifecycleStage>),
}

impl fmt::Display for StageLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingStages(stages) => {
                write!(f, "issue has multiple conflicting stage labels; reconcile loop cannot proceed without human intervention to fix corrupted state: {:?}", stages)
            }
        }
    }
}

impl std::error::Error for StageLabelError {}

/// Map a GitHub issue record into canonical tracked-issue form.
pub fn map_github_issue(record: GitHubIssueRecord) -> Result<TrackedIssue, String> {
    let stage = stage_from_labels(&record.labels)
        .map_err(|e| e.to_string())?
        .unwrap_or(IssueLifecycleStage::Idea);
    Ok(TrackedIssue {
        reference: IssueRef {
            owner: record.owner,
            repo: record.repo,
            number: record.number,
        },
        title: record.title,
        body: record.body,
        state: record.state,
        labels: record.labels,
        stage,
    })
}

/// Resolve lifecycle stage from issue labels. Returns `None` when no stage
/// label is present.
pub fn stage_from_labels(
    labels: &[String],
) -> Result<Option<IssueLifecycleStage>, StageLabelError> {
    let mut stages = Vec::new();
    let has = |label: &'static str| labels.iter().any(|candidate| candidate == label);
    if has(IssueLifecycleStage::Idea.as_label()) {
        stages.push(IssueLifecycleStage::Idea);
    }
    if has(IssueLifecycleStage::Design.as_label()) {
        stages.push(IssueLifecycleStage::Design);
    }
    if has(IssueLifecycleStage::DesignReview.as_label()) {
        stages.push(IssueLifecycleStage::DesignReview);
    }
    if has(IssueLifecycleStage::Accepted.as_label()) {
        stages.push(IssueLifecycleStage::Accepted);
    }
    if has(IssueLifecycleStage::Implementing.as_label()) {
        stages.push(IssueLifecycleStage::Implementing);
    }
    if has(IssueLifecycleStage::CodeReview.as_label()) {
        stages.push(IssueLifecycleStage::CodeReview);
    }
    if has(IssueLifecycleStage::Testing.as_label()) {
        stages.push(IssueLifecycleStage::Testing);
    }
    if has(IssueLifecycleStage::Done.as_label()) {
        stages.push(IssueLifecycleStage::Done);
    }
    if has(IssueLifecycleStage::TerminalFailed.as_label()) {
        stages.push(IssueLifecycleStage::TerminalFailed);
    }

    if stages.len() > 1 {
        return Err(StageLabelError::ConflictingStages(stages));
    }
    Ok(stages.into_iter().next())
}

/// Replace lifecycle stage labels with a single canonical stage label while
/// preserving non-stage labels.
pub fn set_stage_label(existing_labels: &[String], stage: IssueLifecycleStage) -> Vec<String> {
    let mut labels: Vec<String> = existing_labels
        .iter()
        .filter(|label| !is_stage_label(label))
        .cloned()
        .collect();
    labels.push(stage.as_label().to_string());
    labels.sort();
    labels.dedup();
    labels
}

/// Compare-and-set transition helper for issue stage labels.
///
/// Fails closed when the current stage does not match `expected_stage`.
pub fn compare_and_set_stage_label(
    existing_labels: &[String],
    expected_stage: IssueLifecycleStage,
    next_stage: IssueLifecycleStage,
) -> Result<Vec<String>, String> {
    let current_stage = stage_from_labels(existing_labels)
        .map_err(|e| e.to_string())?
        .unwrap_or(IssueLifecycleStage::Idea);
    if current_stage != expected_stage {
        return Err(format!(
            "stage compare-and-set failed: expected `{}`, found `{}`",
            expected_stage.as_label(),
            current_stage.as_label()
        ));
    }
    Ok(set_stage_label(existing_labels, next_stage))
}

/// Validate that provider capabilities satisfy SDLC real-mode requirements.
///
/// This gate blocks real-mode execution only; dry-run mode bypasses it so
/// local development works without a fully-wired provider (IM12).
pub fn ensure_sdlc_issue_capabilities(capabilities: SdlcIssueCapabilities) -> Result<(), String> {
    let mut missing = Vec::new();
    if !capabilities.issue_read {
        missing.push("issue_read");
    }
    if !capabilities.issue_comment_upsert {
        missing.push("issue_comment_upsert");
    }
    if !capabilities.issue_label_compare_and_set {
        missing.push("issue_label_compare_and_set");
    }
    if !capabilities.managed_issue_search {
        missing.push("managed_issue_search");
    }
    if !capabilities.deterministic_issue_identity {
        missing.push("deterministic_issue_identity");
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "missing required SDLC issue transport capabilities: {}",
        missing.join(", ")
    ))
}

fn is_stage_label(label: &str) -> bool {
    matches!(
        label,
        "idea" | "design" | "design-review" | "accepted" | "implementing" | "code-review" | "testing" | "done" | "terminal-failed"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_github_issue_defaults_to_idea_when_no_stage_label() {
        let tracked = map_github_issue(GitHubIssueRecord {
            owner: "gunb-ai".to_string(),
            repo: "gunbc".to_string(),
            number: 42,
            title: "Intent".to_string(),
            body: "Details".to_string(),
            state: "open".to_string(),
            labels: vec!["priority:M".to_string()],
        })
        .unwrap();
        assert_eq!(tracked.stage, IssueLifecycleStage::Idea);
    }

    #[test]
    fn stage_from_labels_fails_closed_on_multiple_stages() {
        let labels = vec![
            "design".to_string(),
            "accepted".to_string(),
            "priority:M".to_string(),
        ];
        let err = stage_from_labels(&labels).unwrap_err();
        assert!(err
            .to_string()
            .contains("issue has multiple conflicting stage labels"));
    }

    #[test]
    fn set_stage_label_replaces_existing_stage_labels() {
        let labels = vec![
            "design".to_string(),
            "priority:M".to_string(),
            "design-review".to_string(),
        ];
        assert_eq!(
            set_stage_label(&labels, IssueLifecycleStage::Accepted),
            vec!["accepted".to_string(), "priority:M".to_string()]
        );
    }

    #[test]
    fn compare_and_set_stage_label_fails_closed_on_mismatch() {
        let labels = vec!["design-review".to_string()];
        let err = compare_and_set_stage_label(
            &labels,
            IssueLifecycleStage::Design,
            IssueLifecycleStage::Accepted,
        )
        .expect_err("mismatched stage should fail compare-and-set");
        assert!(err.contains("expected `design`"));
        assert!(err.contains("found `design-review`"));
    }

    #[test]
    fn compare_and_set_stage_label_updates_when_expected_matches() {
        let labels = vec!["design-review".to_string(), "priority:M".to_string()];
        let next = compare_and_set_stage_label(
            &labels,
            IssueLifecycleStage::DesignReview,
            IssueLifecycleStage::Accepted,
        )
        .expect("matching stage should update");
        assert_eq!(next, vec!["accepted".to_string(), "priority:M".to_string()]);
    }

    #[test]
    fn capability_gate_fails_closed_when_required_capabilities_missing() {
        let err = ensure_sdlc_issue_capabilities(SdlcIssueCapabilities {
            issue_read: true,
            issue_comment_upsert: false,
            issue_label_compare_and_set: false,
            managed_issue_search: true,
            deterministic_issue_identity: true,
        })
        .expect_err("missing required capabilities should fail closed");
        assert!(err.contains("issue_comment_upsert"));
        assert!(err.contains("issue_label_compare_and_set"));
    }

    #[test]
    fn capability_gate_passes_when_all_capabilities_present() {
        ensure_sdlc_issue_capabilities(SdlcIssueCapabilities::github())
            .expect("full github capabilities should pass gate");
    }

    #[test]
    fn capability_gate_fails_closed_when_no_capabilities_present() {
        let err = ensure_sdlc_issue_capabilities(SdlcIssueCapabilities::none())
            .expect_err("no capabilities should fail closed");
        assert!(err.contains("issue_read"));
        assert!(err.contains("issue_comment_upsert"));
        assert!(err.contains("issue_label_compare_and_set"));
        assert!(err.contains("managed_issue_search"));
        assert!(err.contains("deterministic_issue_identity"));
    }

    #[test]
    fn capability_gate_fails_closed_when_managed_issue_search_missing() {
        let err = ensure_sdlc_issue_capabilities(SdlcIssueCapabilities {
            issue_read: true,
            issue_comment_upsert: true,
            issue_label_compare_and_set: true,
            managed_issue_search: false,
            deterministic_issue_identity: true,
        })
        .expect_err("missing managed_issue_search should fail closed");
        assert!(err.contains("managed_issue_search"));
        assert!(!err.contains("issue_read"));
    }

    #[test]
    fn capability_gate_fails_closed_when_deterministic_issue_identity_missing() {
        let err = ensure_sdlc_issue_capabilities(SdlcIssueCapabilities {
            issue_read: true,
            issue_comment_upsert: true,
            issue_label_compare_and_set: true,
            managed_issue_search: true,
            deterministic_issue_identity: false,
        })
        .expect_err("missing deterministic_issue_identity should fail closed");
        assert!(err.contains("deterministic_issue_identity"));
        assert!(!err.contains("managed_issue_search"));
    }
}
