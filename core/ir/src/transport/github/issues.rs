//! Provider-agnostic SDLC issue adapter for GitHub issues.
//!
//! This module defines the canonical issue snapshot shape consumed by SDLC
//! runtime code, plus capability-gate checks required for real-mode execution.

use serde::{Deserialize, Serialize};

/// Canonical SDLC lifecycle stages represented as GitHub labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueLifecycleStage {
    Idea,
    Design,
    DesignReview,
    Accepted,
    Implementation,
    Closed,
}

impl IssueLifecycleStage {
    /// Canonical stage label used on provider issues.
    pub const fn as_label(self) -> &'static str {
        match self {
            IssueLifecycleStage::Idea => "idea",
            IssueLifecycleStage::Design => "design",
            IssueLifecycleStage::DesignReview => "design-review",
            IssueLifecycleStage::Accepted => "accepted",
            IssueLifecycleStage::Implementation => "implementation",
            IssueLifecycleStage::Closed => "closed",
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

/// Required issue-transport capabilities for SDLC real-mode execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdlcIssueCapabilities {
    pub issue_read: bool,
    pub issue_comment_upsert: bool,
    pub issue_label_compare_and_set: bool,
}

impl SdlcIssueCapabilities {
    /// Capabilities provided by GitHub issue transport.
    pub const fn github() -> Self {
        Self {
            issue_read: true,
            issue_comment_upsert: true,
            issue_label_compare_and_set: true,
        }
    }
}

/// Map a GitHub issue record into canonical tracked-issue form.
pub fn map_github_issue(record: GitHubIssueRecord) -> TrackedIssue {
    let stage = stage_from_labels(&record.labels).unwrap_or(IssueLifecycleStage::Idea);
    TrackedIssue {
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
    }
}

/// Resolve lifecycle stage from issue labels. Returns `None` when no stage
/// label is present.
pub fn stage_from_labels(labels: &[String]) -> Option<IssueLifecycleStage> {
    let has = |label: &'static str| labels.iter().any(|candidate| candidate == label);
    // Most advanced stage wins to avoid ambiguity when stale labels linger.
    if has(IssueLifecycleStage::Closed.as_label()) {
        return Some(IssueLifecycleStage::Closed);
    }
    if has(IssueLifecycleStage::Implementation.as_label()) {
        return Some(IssueLifecycleStage::Implementation);
    }
    if has(IssueLifecycleStage::Accepted.as_label()) {
        return Some(IssueLifecycleStage::Accepted);
    }
    if has(IssueLifecycleStage::DesignReview.as_label()) {
        return Some(IssueLifecycleStage::DesignReview);
    }
    if has(IssueLifecycleStage::Design.as_label()) {
        return Some(IssueLifecycleStage::Design);
    }
    if has(IssueLifecycleStage::Idea.as_label()) {
        return Some(IssueLifecycleStage::Idea);
    }
    None
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

/// Validate that provider capabilities satisfy SDLC real-mode requirements.
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
        "idea" | "design" | "design-review" | "accepted" | "implementation" | "closed"
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
        });
        assert_eq!(tracked.stage, IssueLifecycleStage::Idea);
    }

    #[test]
    fn stage_from_labels_prefers_most_advanced_stage() {
        let labels = vec![
            "design".to_string(),
            "accepted".to_string(),
            "priority:M".to_string(),
        ];
        assert_eq!(
            stage_from_labels(&labels),
            Some(IssueLifecycleStage::Accepted)
        );
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
    fn capability_gate_fails_closed_when_required_capabilities_missing() {
        let err = ensure_sdlc_issue_capabilities(SdlcIssueCapabilities {
            issue_read: true,
            issue_comment_upsert: false,
            issue_label_compare_and_set: false,
        })
        .expect_err("missing required capabilities should fail closed");
        assert!(err.contains("issue_comment_upsert"));
        assert!(err.contains("issue_label_compare_and_set"));
    }
}
