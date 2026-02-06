//! GitHub platform - common types shared by API and CLI.
//!
//! This module provides the foundation for all GitHub interactions in gunbc:
//! - Version constants (single source of truth)
//! - Authentication methods
//! - Re-exports for API and CLI submodules
//!
//! # Architecture
//!
//! ```text
//! github/
//! ├── mod.rs      ← This file: common types, auth, version constants
//! ├── api.rs      ← REST API: config, headers, request builder
//! └── cli.rs      ← gh CLI: config, commands, install methods, upsert interface
//! ```
//!
//! Services like [`super::gist`] and [`super::github_actions`] build on this layer.

pub mod api;
pub mod cli;

// Re-export for convenience
pub use api::{github_rest_request, GitHubApi, GITHUB_API};
pub use cli::{gh_cli_commands, gh_cli_request, is_gh_installed, GHCommand, GH_TOOL};

// ============================================================================
// Version Constants (single source of truth)
// ============================================================================

/// GitHub API version we target.
///
/// Reference: https://docs.github.com/en/rest/about-the-rest-api/api-versions
pub const GITHUB_API_VERSION: &str = "2022-11-28";

/// Minimum gh CLI version required.
///
/// Reference: https://github.com/cli/cli/releases
pub const GH_CLI_MIN_VERSION: &str = "2.40.0";

/// Our integration contract version - increment when behavior changes.
///
/// Format: YYYY.MM.DD.N where N is a revision within the day.
/// This helps track when our usage of GitHub APIs/CLI changes.
pub const GITHUB_CONTRACT_VERSION: &str = "2026.01.29.1";

// ============================================================================
// Authentication (shared by API and CLI)
// ============================================================================

/// GitHub authentication methods.
///
/// Supports multiple auth strategies that work across REST API and CLI.
///
/// **Deprecated**: Use [`Credential`](crate::transport::credential::Credential) with
/// [`AuthScheme`](crate::transport::credential::AuthScheme) instead.
#[deprecated(note = "use Credential with AuthScheme instead")]
#[derive(Debug, Clone, PartialEq)]
pub enum GitHubAuth {
    /// Direct token (e.g., PAT or fine-grained token)
    Token(String),
    /// Environment variable reference (resolved at execution time)
    /// Common values: GITHUB_TOKEN, GH_TOKEN
    EnvVar(&'static str),
    /// Use gh CLI's stored authentication
    /// Requires: `gh auth login` has been run
    CLI,
}

/// Default authentication method.
///
/// Uses GITHUB_TOKEN environment variable, which is:
/// - Automatically set in GitHub Actions
/// - Standard for local development with PATs
#[allow(deprecated)]
pub const DEFAULT_AUTH: GitHubAuth = GitHubAuth::EnvVar("GITHUB_TOKEN");

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants_are_set() {
        assert!(!GITHUB_API_VERSION.is_empty());
        assert!(!GH_CLI_MIN_VERSION.is_empty());
        assert!(!GITHUB_CONTRACT_VERSION.is_empty());
    }

    #[test]
    fn test_github_auth_variants() {
        let token_auth = GitHubAuth::Token("test_token".to_string());
        let env_auth = GitHubAuth::EnvVar("GITHUB_TOKEN");
        let cli_auth = GitHubAuth::CLI;

        assert!(matches!(token_auth, GitHubAuth::Token(_)));
        assert!(matches!(env_auth, GitHubAuth::EnvVar(_)));
        assert!(matches!(cli_auth, GitHubAuth::CLI));
    }

    #[test]
    fn test_default_auth() {
        assert!(matches!(DEFAULT_AUTH, GitHubAuth::EnvVar("GITHUB_TOKEN")));
    }
}
