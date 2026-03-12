//! Provider-neutral agent adapter trait.
//!
//! Implementations of `AgentAdapter` bridge between the issue workflow and
//! specific agent providers (Codex SDK, local shell loop, dry-run stub).
//! The trait is intentionally minimal: spawn, poll, cancel.

use super::agent::{AgentError, AgentHandle, AgentStatus, HandoffSpec};

/// Provider-neutral interface for spawning and monitoring implementation agents.
///
/// Adapters are stateless: all state is externalized in the `AgentHandle` and
/// the agent ledger. This allows the workflow worker to be restarted without losing
/// track of running agents.
pub trait AgentAdapter: Send + Sync {
    /// Spawn an agent to implement the given handoff spec.
    ///
    /// Returns a handle that can be used to poll status or cancel.
    fn spawn(&self, spec: &HandoffSpec) -> Result<AgentHandle, AgentError>;

    /// Poll the current status of a spawned agent.
    fn poll_status(&self, handle: &AgentHandle) -> Result<AgentStatus, AgentError>;

    /// Cancel a running agent. No-op if already completed.
    fn cancel(&self, handle: &AgentHandle) -> Result<(), AgentError>;

    /// Human-readable provider name (e.g., "codex", "local", "stub").
    fn provider_name(&self) -> &str;
}

/// Stub adapter for testing and dry-run mode.
///
/// Returns `Completed` immediately with a pre-configured branch and sha,
/// allowing end-to-end pipeline testing without a real agent.
pub struct StubAgentAdapter {
    pub branch: String,
    pub commit_sha: String,
}

impl StubAgentAdapter {
    pub fn new(branch: impl Into<String>, commit_sha: impl Into<String>) -> Self {
        Self {
            branch: branch.into(),
            commit_sha: commit_sha.into(),
        }
    }
}

impl AgentAdapter for StubAgentAdapter {
    fn spawn(&self, spec: &HandoffSpec) -> Result<AgentHandle, AgentError> {
        Ok(AgentHandle {
            provider: "stub".to_string(),
            session_id: format!("stub-{}", spec.intake_key),
            intake_key: spec.intake_key.clone(),
            spawned_at_epoch_ms: 0,
        })
    }

    fn poll_status(&self, _handle: &AgentHandle) -> Result<AgentStatus, AgentError> {
        Ok(AgentStatus::Completed {
            branch: self.branch.clone(),
            commit_sha: self.commit_sha.clone(),
        })
    }

    fn cancel(&self, _handle: &AgentHandle) -> Result<(), AgentError> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "stub"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::agent::{AgentConstraints, DesignArtifact};

    fn test_spec() -> HandoffSpec {
        HandoffSpec {
            intent_id: "test".into(),
            issue_id: 1,
            intake_key: "key-1".into(),
            run_key: "run-1".into(),
            repo_url: "https://github.com/test/repo".into(),
            base_branch: "main".into(),
            target_branch: "feature/test".into(),
            design_artifact: DesignArtifact {
                content: "design".into(),
                content_hash: "hash".into(),
            },
            constraints: AgentConstraints::default_rust(),
        }
    }

    #[test]
    fn stub_adapter_spawns_and_completes_immediately() {
        let adapter = StubAgentAdapter::new("feature/test", "abc123");
        let handle = adapter.spawn(&test_spec()).expect("spawn");
        assert_eq!(handle.provider, "stub");
        assert_eq!(handle.intake_key, "key-1");

        let status = adapter.poll_status(&handle).expect("poll");
        assert!(
            matches!(status, AgentStatus::Completed { branch, commit_sha }
            if branch == "feature/test" && commit_sha == "abc123")
        );
    }

    #[test]
    fn stub_adapter_cancel_is_noop() {
        let adapter = StubAgentAdapter::new("b", "c");
        let handle = adapter.spawn(&test_spec()).expect("spawn");
        adapter.cancel(&handle).expect("cancel should succeed");
    }
}
