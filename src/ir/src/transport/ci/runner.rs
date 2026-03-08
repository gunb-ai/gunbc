//! CI runner abstraction.
//!
//! This module provides a `Runner` trait that generalizes the concept of
//! CI runner environments across different providers. The existing `RunnerImage`
//! in `github_actions.rs` implements this trait.
//!
//! # Design
//!
//! Runners model the execution environment where CI jobs run:
//! - Pre-installed tools (cargo, git, docker, etc.)
//! - Resource constraints (CPU, memory)
//! - Operating system and architecture
//!
//! By abstracting runners behind a trait, code can check tool availability
//! without knowing which CI provider is being used.
//!
//! # Example
//!
//! ```text
//! use gunbc_ir::transport::ci::Runner;
//!
//! fn check_requirements(runner: &dyn Runner) -> Result<(), Vec<&str>> {
//!     let required = ["cargo", "git", "docker"];
//!     let missing = runner.missing_tools(&required);
//!     if missing.is_empty() {
//!         Ok(())
//!     } else {
//!         Err(missing)
//!     }
//! }
//! ```

/// Trait for CI runner environments.
///
/// Runners represent the execution environment where CI jobs run.
/// Different CI providers have different runner options with varying
/// pre-installed tools and capabilities.
pub trait Runner: Send + Sync {
    /// Runner identifier (e.g., "ubuntu-latest", "saas-linux-small-amd64").
    fn id(&self) -> &str;

    /// Human-readable name for the runner.
    fn name(&self) -> &str {
        self.id()
    }

    /// Pre-installed tools available on this runner.
    fn tools(&self) -> &[&str];

    /// Check if a tool is available.
    fn has_tool(&self, tool: &str) -> bool {
        self.tools().contains(&tool)
    }

    /// Get missing tools from a required set.
    fn missing_tools<'a>(&self, required: &[&'a str]) -> Vec<&'a str> {
        required
            .iter()
            .filter(|t| !self.has_tool(t))
            .copied()
            .collect()
    }

    /// Check if all required tools are available.
    fn has_all_tools(&self, required: &[&str]) -> bool {
        required.iter().all(|t| self.has_tool(t))
    }

    /// Documentation URL for this runner.
    fn docs_url(&self) -> Option<&str> {
        None
    }
}

// ============================================================================
// GitLab Runner Implementation
// ============================================================================

/// GitLab CI runner definition.
///
/// GitLab runners are typically Docker-based, with tools depending on
/// the container image used. The SaaS shared runners have predictable
/// tool availability.
#[derive(Debug, Clone)]
pub struct GitLabRunner {
    /// Runner tag/identifier
    pub id: &'static str,
    /// Human-readable name
    pub runner_name: &'static str,
    /// Pre-installed tools
    tools: Vec<&'static str>,
    /// Documentation URL
    pub docs_url: &'static str,
}

impl Runner for GitLabRunner {
    fn id(&self) -> &str {
        self.id
    }

    fn name(&self) -> &str {
        self.runner_name
    }

    fn tools(&self) -> &[&str] {
        &self.tools
    }

    fn docs_url(&self) -> Option<&str> {
        Some(self.docs_url)
    }
}

// ============================================================================
// GitLab Runner Catalog
// ============================================================================

/// GitLab SaaS Linux Small runner.
///
/// Shared runner with 2 vCPU, 8GB RAM.
/// Uses Docker executor with pre-installed tools.
pub fn gitlab_saas_linux_small() -> GitLabRunner {
    GitLabRunner {
        id: "saas-linux-small-amd64",
        runner_name: "GitLab SaaS Linux Small Runner",
        tools: vec![
            "git", "docker", "curl", "wget", "jq", "zip",
            "unzip",
            // Note: cargo/rustc not pre-installed, need to use rust image
        ],
        docs_url: "https://docs.gitlab.com/ee/ci/runners/saas/linux_saas_runner.html",
    }
}

/// GitLab SaaS Linux Medium runner.
///
/// Shared runner with 4 vCPU, 16GB RAM.
pub fn gitlab_saas_linux_medium() -> GitLabRunner {
    GitLabRunner {
        id: "saas-linux-medium-amd64",
        runner_name: "GitLab SaaS Linux Medium Runner",
        tools: vec!["git", "docker", "curl", "wget", "jq", "zip", "unzip"],
        docs_url: "https://docs.gitlab.com/ee/ci/runners/saas/linux_saas_runner.html",
    }
}

/// GitLab SaaS Linux Large runner.
///
/// Shared runner with 8 vCPU, 32GB RAM.
pub fn gitlab_saas_linux_large() -> GitLabRunner {
    GitLabRunner {
        id: "saas-linux-large-amd64",
        runner_name: "GitLab SaaS Linux Large Runner",
        tools: vec!["git", "docker", "curl", "wget", "jq", "zip", "unzip"],
        docs_url: "https://docs.gitlab.com/ee/ci/runners/saas/linux_saas_runner.html",
    }
}

/// All known GitLab runners.
pub fn all_gitlab_runners() -> Vec<GitLabRunner> {
    vec![
        gitlab_saas_linux_small(),
        gitlab_saas_linux_medium(),
        gitlab_saas_linux_large(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_runner_has_tool() {
        let runner = gitlab_saas_linux_small();
        assert!(runner.has_tool("git"));
        assert!(runner.has_tool("docker"));
        assert!(!runner.has_tool("cargo")); // Not pre-installed
    }

    #[test]
    fn test_gitlab_runner_missing_tools() {
        let runner = gitlab_saas_linux_small();
        let missing = runner.missing_tools(&["git", "cargo", "rustc"]);
        assert_eq!(missing, vec!["cargo", "rustc"]);
    }

    #[test]
    fn test_gitlab_runner_has_all_tools() {
        let runner = gitlab_saas_linux_small();
        assert!(runner.has_all_tools(&["git", "docker"]));
        assert!(!runner.has_all_tools(&["git", "cargo"]));
    }
}
