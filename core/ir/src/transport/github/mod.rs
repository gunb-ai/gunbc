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
//! └── cli.rs      ← gh CLI: config, commands, install methods
//! ```
//!
//! Services like [`super::gist`] and [`super::github_actions`] build on this layer.

pub mod api;
pub mod cli;
pub mod issues;

// Re-export for convenience
pub use api::{github_rest_request, GitHubApi, GITHUB_API};
pub use cli::{gh_cli_commands, gh_cli_request, GHCommand, GH_TOOL};
pub use issues::{
    compare_and_set_stage_label, ensure_sdlc_issue_capabilities, map_github_issue,
    set_stage_label, stage_from_labels, GitHubIssueRecord, IssueLifecycleStage, IssueRef,
    SdlcIssueCapabilities, TrackedIssue,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants_are_set() {
        assert_eq!(GITHUB_API_VERSION, "2022-11-28");
        assert_eq!(GH_CLI_MIN_VERSION, "2.40.0");
        assert!(GITHUB_CONTRACT_VERSION.starts_with("2026."));
    }
}
