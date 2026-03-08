//! Git repository configuration.
//!
//! Models git conventions used by a repository. The CI and rendering layers
//! reference this model instead of hardcoding branch names.
//!
//! # Ownership
//!
//! - This module defines the model (what git offers).
//! - Repo-specific choices (e.g., "our default branch is main") live in
//!   `gunbc-codegen` alongside other repo config.

/// Git repository conventions.
///
/// Captures the branching strategy and conventions for a repository.
/// CI renderers use this to determine trigger branches, and other tools
/// use it to know the default target for operations.
#[derive(Debug, Clone)]
pub struct GitConfig {
    /// The default branch name (e.g., "main", "master", "develop").
    pub default_branch: String,
}

impl GitConfig {
    /// Create a new git config with the given default branch.
    pub fn new(default_branch: &str) -> Self {
        Self {
            default_branch: default_branch.to_string(),
        }
    }

    /// Branches that CI should trigger on (push and PR).
    ///
    /// By default, returns just the default branch. Override or extend
    /// for repos that need CI on additional branches.
    pub fn ci_branches(&self) -> Vec<&str> {
        vec![&self.default_branch]
    }
}

impl Default for GitConfig {
    /// Defaults to "main" as the default branch, matching modern git convention.
    fn default() -> Self {
        Self::new("main")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_branch() {
        let config = GitConfig::default();
        assert_eq!(config.default_branch, "main");
    }

    #[test]
    fn test_custom_branch() {
        let config = GitConfig::new("develop");
        assert_eq!(config.default_branch, "develop");
    }

    #[test]
    fn test_ci_branches() {
        let config = GitConfig::default();
        assert_eq!(config.ci_branches(), vec!["main"]);
    }
}
