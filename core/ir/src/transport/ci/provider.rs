//! CI provider trait and detection.
//!
//! This module defines the `CiProvider` trait that abstracts CI-specific
//! command formatting across different CI systems. Each provider implements
//! its own formatting rules while sharing common command types.
//!
//! # Design Principle: Coordinated Provider-Specific Rendering
//!
//! Following the same pattern as `Renderable`:
//! - **Shared types**: `WorkflowCommand` defines *what* to emit
//! - **Provider-specific**: `format()` implements *how* to emit
//! - **Graceful degradation**: Providers render what they can
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::ci::{CiProvider, WorkflowCommand, detect_provider};
//! use std::collections::HashMap;
//!
//! let env: HashMap<String, String> = std::env::vars().collect();
//! let provider = detect_provider(&env);
//! let cmd = WorkflowCommand::group_start("build");
//! println!("{}", provider.format(&cmd));
//! // GitHub Actions: "::group::build"
//! // GitLab CI: "\x1b[0Ksection_start:1234:build\r\x1b[0Kbuild"
//! // Plain: "=== build ==="
//! ```

use super::command::WorkflowCommand;
use super::runner::Runner;
use std::collections::HashMap;

/// Trait for CI provider-specific output formatting.
///
/// Providers implement this trait to format workflow commands according
/// to their platform's syntax. The trait also provides auto-detection
/// based on environment variables.
pub trait CiProvider: Send + Sync {
    /// Provider identifier (e.g., "github-actions", "gitlab-ci", "plain").
    fn id(&self) -> &'static str;

    /// Human-readable provider name.
    fn name(&self) -> &'static str;

    /// Format a workflow command for this provider's syntax.
    ///
    /// Each provider implements this method to produce the correct
    /// output format. Commands that aren't supported should produce
    /// graceful fallbacks (e.g., plain text).
    fn format(&self, cmd: &WorkflowCommand) -> String;

    /// Check if this provider supports a specific command natively.
    ///
    /// Returns true if the provider has native support for the command.
    /// Even if false, `format()` should still produce usable output.
    fn supports(&self, cmd: &WorkflowCommand) -> bool {
        // Default: support all commands
        let _ = cmd;
        true
    }

    /// Get available runners for this provider.
    fn runners(&self) -> Vec<Box<dyn Runner>>;

    /// Get the default runner for this provider.
    fn default_runner(&self) -> Box<dyn Runner>;
}

/// Detect the current CI provider from an environment map.
///
/// Checks for provider-specific environment variables in order:
/// 1. `GITHUB_ACTIONS` → GitHub Actions
/// 2. `GITLAB_CI` → GitLab CI
/// 3. (fallback) → Plain text provider
///
/// # Example
///
/// ```ignore
/// let env: HashMap<String, String> = std::env::vars().collect();
/// let provider = detect_provider(&env);
/// println!("Running on: {}", provider.name());
/// ```
pub fn detect_provider(env: &HashMap<String, String>) -> Box<dyn CiProvider> {
    // Check for GitHub Actions
    if env.contains_key("GITHUB_ACTIONS") {
        return Box::new(super::providers::GitHubActionsProvider);
    }

    // Check for GitLab CI
    if env.contains_key("GITLAB_CI") {
        return Box::new(super::providers::GitLabCiProvider::new());
    }

    // Fallback to plain text
    Box::new(super::providers::PlainTextProvider)
}

/// Check if running in any CI environment.
pub fn is_ci(env: &HashMap<String, String>) -> bool {
    env.contains_key("CI") || env.contains_key("GITHUB_ACTIONS") || env.contains_key("GITLAB_CI")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ci_detection() {
        let env = HashMap::new();
        assert!(!is_ci(&env));
    }

    #[test]
    fn test_detect_provider_returns_something() {
        // Should always return a provider (at least PlainText)
        let env = HashMap::new();
        let provider = detect_provider(&env);
        assert!(!provider.id().is_empty());
    }
}
