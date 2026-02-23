//! Agent handoff contract types for implementation delegation.
//!
//! These types define the provider-neutral interface between the SDLC pipeline's
//! "Accepted" stage and any implementation agent (Codex, local shell loop, etc.).
//! The contract is intentionally stateless and serializable so that any adapter
//! can consume it without coupling to pipeline internals.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Everything an implementation agent needs to begin work on an accepted design.
///
/// Assembled by the SDLC pipeline from the intake ledger, artifact ledger, and
/// repo metadata. The agent reads this spec, implements the design on the target
/// branch, and signals completion (or failure) via the adapter status protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffSpec {
    pub intent_id: String,
    pub issue_id: u64,
    pub intake_key: String,
    pub run_key: String,
    pub repo_url: String,
    pub base_branch: String,
    pub target_branch: String,
    pub design_artifact: DesignArtifact,
    pub constraints: AgentConstraints,
}

/// The canonical design document promoted from the artifact ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignArtifact {
    pub content: String,
    pub content_hash: String,
}

/// Guardrails the agent must satisfy before signaling completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConstraints {
    pub success_criteria: Vec<String>,
    pub test_command: String,
    pub lint_command: String,
    pub max_iterations: u32,
    pub timeout_seconds: u64,
}

impl AgentConstraints {
    pub fn default_rust() -> Self {
        Self {
            success_criteria: Vec::new(),
            test_command: "cargo test --workspace".to_string(),
            lint_command: "cargo clippy --all-targets -- -D warnings".to_string(),
            max_iterations: 10,
            timeout_seconds: 1800,
        }
    }
}

/// Deterministic branch name from an intent ID.
///
/// Normalizes the intent ID to a safe git branch name:
/// lowercase, non-alphanumeric replaced with hyphens, trimmed.
pub fn target_branch_for_intent(intent_id: &str) -> String {
    let normalized: String = intent_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = normalized.trim_matches('-');
    format!("feature/{trimmed}")
}

/// Opaque handle returned by an agent adapter after spawning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentHandle {
    pub provider: String,
    pub session_id: String,
    pub intake_key: String,
    pub spawned_at_epoch_ms: u128,
}

/// Agent execution status as reported by the adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Running { progress: Option<String> },
    Completed { branch: String, commit_sha: String },
    Failed { reason: String, recoverable: bool },
}

impl AgentStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentStatus::Completed { .. } | AgentStatus::Failed { .. }
        )
    }
}

/// Errors from agent adapter operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentError {
    pub message: String,
    pub retryable: bool,
}

impl AgentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
        }
    }

    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: true,
        }
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AgentError {}

/// Pull request specification for automated PR creation after agent completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestSpec {
    pub owner: String,
    pub repo: String,
    pub head_branch: String,
    pub base_branch: String,
    pub title: String,
    pub body: String,
    pub issue_number: u64,
    pub draft: bool,
}

/// Result of creating a pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestResult {
    pub number: u64,
    pub url: String,
    pub head_branch: String,
}

/// Validation result for a PR (review + CI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrValidationResult {
    pub review_passed: bool,
    pub ci_passed: bool,
    pub blocking_findings: Vec<String>,
    pub ci_summary: Option<String>,
}

impl PrValidationResult {
    pub fn all_passed(&self) -> bool {
        self.review_passed && self.ci_passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_branch_deterministic_from_intent() {
        assert_eq!(
            target_branch_for_intent("add-markdown-report"),
            "feature/add-markdown-report"
        );
    }

    #[test]
    fn target_branch_normalizes_special_chars() {
        assert_eq!(
            target_branch_for_intent("Add Markdown Report!"),
            "feature/add-markdown-report"
        );
    }

    #[test]
    fn target_branch_trims_leading_trailing_hyphens() {
        assert_eq!(
            target_branch_for_intent("--my-feature--"),
            "feature/my-feature"
        );
    }

    #[test]
    fn agent_status_terminal_variants() {
        assert!(!AgentStatus::Running { progress: None }.is_terminal());
        assert!(AgentStatus::Completed {
            branch: "b".into(),
            commit_sha: "c".into()
        }
        .is_terminal());
        assert!(AgentStatus::Failed {
            reason: "r".into(),
            recoverable: false
        }
        .is_terminal());
    }

    #[test]
    fn handoff_spec_round_trips_json() {
        let spec = HandoffSpec {
            intent_id: "test-intent".into(),
            issue_id: 42,
            intake_key: "test-key".into(),
            run_key: "run-1".into(),
            repo_url: "https://github.com/test/repo".into(),
            base_branch: "main".into(),
            target_branch: "feature/test-intent".into(),
            design_artifact: DesignArtifact {
                content: "# Design\nDo the thing.".into(),
                content_hash: "abc123".into(),
            },
            constraints: AgentConstraints::default_rust(),
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let round_tripped: HandoffSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec, round_tripped);
    }

    #[test]
    fn pr_validation_all_passed() {
        let result = PrValidationResult {
            review_passed: true,
            ci_passed: true,
            blocking_findings: vec![],
            ci_summary: Some("all green".into()),
        };
        assert!(result.all_passed());

        let result2 = PrValidationResult {
            review_passed: false,
            ci_passed: true,
            blocking_findings: vec!["issue".into()],
            ci_summary: None,
        };
        assert!(!result2.all_passed());
    }
}
