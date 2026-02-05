//! GitHub Actions external dependency modeling.
//!
//! This module provides typed representations of GitHub Actions concepts:
//! - [`Permissions`]: GITHUB_TOKEN permission scopes for workflow/job configuration
//! - [`Integration`]: GitHub Actions (e.g., checkout, rust-toolchain) with required permissions
//! - [`RunnerImage`]: Runner environment with pre-installed tools
//!
//! # Permissions Model
//!
//! The permission scopes in this module correspond to **GITHUB_TOKEN** scopes,
//! not GitHub OAuth scopes. The GITHUB_TOKEN is automatically provided by the
//! Actions runner and its permissions can be configured at the workflow or job level.
//!
//! Reference: <https://docs.github.com/en/actions/security-guides/automatic-token-authentication>
//!
//! # Design
//!
//! GitHub Actions is modeled as an external dependency interface. Our CI tool
//! provides an implementation that declares which integrations it uses, and
//! permissions flow automatically from integrations to workflow configuration.
//!
//! This module builds on the [`super::github`] platform layer for shared
//! GitHub concepts (API versioning, authentication patterns).
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::github_actions::*;
//!
//! // Declare integrations used by a workflow
//! let perms = merge_permissions(&[
//!     checkout().required_permissions(),
//!     rust_toolchain().required_permissions(),
//! ]);
//!
//! // Check if runner has required tools
//! assert!(ubuntu_latest().has_tool("cargo"));
//! ```

use crate::render::Renderable;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Permission Model
// ============================================================================

/// GitHub Actions GITHUB_TOKEN permission scope.
///
/// These scopes control what the GITHUB_TOKEN can access in Actions workflows.
/// The token is automatically provided by the Actions runner.
///
/// Reference: <https://docs.github.com/en/actions/security-guides/automatic-token-authentication>
/// Scope list: <https://docs.github.com/en/actions/using-jobs/assigning-permissions-to-jobs>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionScope {
    /// Workflow run artifacts and caches
    Actions,
    /// Repository contents (code, commits, branches)
    Contents,
    /// Pull request metadata and comments
    PullRequests,
    /// GitHub Packages (container registry, npm, etc.)
    Packages,
    /// OIDC token for cloud provider authentication
    IdToken,
    /// Repository issues
    Issues,
    /// Repository deployments
    Deployments,
    /// Commit statuses
    Statuses,
}

impl PermissionScope {
    /// Get the YAML key for this permission scope.
    pub fn as_yaml_key(&self) -> &'static str {
        match self {
            Self::Actions => "actions",
            Self::Contents => "contents",
            Self::PullRequests => "pull-requests",
            Self::Packages => "packages",
            Self::IdToken => "id-token",
            Self::Issues => "issues",
            Self::Deployments => "deployments",
            Self::Statuses => "statuses",
        }
    }
}

/// GitHub Actions permission level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// No access (explicit denial)
    None,
    /// Read-only access
    Read,
    /// Read and write access
    Write,
}

impl PermissionLevel {
    /// Get the YAML value for this permission level.
    pub fn as_yaml_value(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

/// Permission set mapping scopes to levels.
pub type Permissions = HashMap<PermissionScope, PermissionLevel>;

/// Merge multiple permission sets, taking the highest level for each scope.
///
/// Write > Read > None. This computes the minimum permissions required
/// to satisfy all input permission sets.
pub fn merge_permissions(permission_sets: &[Permissions]) -> Permissions {
    let mut result = Permissions::new();

    for perms in permission_sets {
        for (scope, level) in perms {
            let existing = result.get(scope).copied().unwrap_or(PermissionLevel::None);
            if *level > existing {
                result.insert(*scope, *level);
            }
        }
    }

    result
}

/// Create a Permissions map from scope-level pairs.
///
/// Convenience macro for defining permission sets inline.
#[macro_export]
macro_rules! permissions {
    ($($scope:expr => $level:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($scope, $level);)*
        map
    }};
}

// ============================================================================
// Integration Model
// ============================================================================

/// A GitHub Actions integration (action) with its permission requirements.
///
/// Integrations are typed references to GitHub Actions with the permissions
/// they require to function. This enables automatic permission propagation
/// from step declarations to workflow configuration.
///
/// Integrations can also declare what tools they provide (install), enabling
/// satisfiability checks against required tools.
#[derive(Debug, Clone)]
pub struct Integration {
    /// Unique identifier (e.g., "checkout", "rust-toolchain")
    pub id: &'static str,
    /// Action reference (e.g., "actions/checkout@v4")
    pub uses: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Permissions required for this integration to work
    required_permissions: Permissions,
    /// Tool IDs that this integration provides/installs
    provides_tools: Vec<&'static str>,
}

impl Integration {
    /// Create a new integration.
    pub fn new(id: &'static str, uses: &'static str, description: &'static str) -> Self {
        Self {
            id,
            uses,
            description,
            required_permissions: HashMap::new(),
            provides_tools: Vec::new(),
        }
    }

    /// Get the required permissions for this integration.
    pub fn required_permissions(&self) -> Permissions {
        self.required_permissions.clone()
    }

    /// Create an integration with permissions.
    pub fn with_permissions(mut self, perms: Permissions) -> Self {
        self.required_permissions = perms;
        self
    }

    /// Get the tools this integration provides/installs.
    pub fn provides_tools(&self) -> &[&'static str] {
        &self.provides_tools
    }

    /// Create an integration that provides specific tools.
    pub fn with_provides_tools(mut self, tools: Vec<&'static str>) -> Self {
        self.provides_tools = tools;
        self
    }
}

// ============================================================================
// Integration Catalog
// ============================================================================

/// Checkout action - clones repository contents.
///
/// Uses: actions/checkout@v4
/// Requires: contents:read
/// Provides: git (ensures git is available for the workflow)
pub fn checkout() -> Integration {
    Integration::new(
        "checkout",
        "actions/checkout@v4",
        "Clone repository contents",
    )
    .with_permissions(permissions! {
        PermissionScope::Contents => PermissionLevel::Read,
    })
    .with_provides_tools(vec!["git"])
}

/// Checkout with push - clones repository and enables pushing changes.
///
/// Uses: actions/checkout@v4
/// Requires: contents:write
/// Provides: git (ensures git is available for the workflow)
pub fn checkout_push() -> Integration {
    Integration::new(
        "checkout-push",
        "actions/checkout@v4",
        "Clone repository and push changes",
    )
    .with_permissions(permissions! {
        PermissionScope::Contents => PermissionLevel::Write,
    })
    .with_provides_tools(vec!["git"])
}

/// Rust toolchain action - installs Rust via rustup.
///
/// Uses: dtolnay/rust-toolchain@stable
/// Requires: (none - just needs checkout first)
/// Provides: rust, cargo, clippy, rustfmt (full Rust toolchain)
pub fn rust_toolchain() -> Integration {
    Integration::new(
        "rust-toolchain",
        "dtolnay/rust-toolchain@stable",
        "Install Rust toolchain via rustup",
    )
    .with_provides_tools(vec!["rust", "cargo", "clippy", "rustfmt"])
    // No special permissions required
}

/// GitHub Container Registry push - push images to ghcr.io.
///
/// Uses: docker/login-action (typically)
/// Requires: packages:write
pub fn ghcr_push() -> Integration {
    Integration::new(
        "ghcr-push",
        "docker/login-action@v3",
        "Push container images to GitHub Container Registry",
    )
    .with_permissions(permissions! {
        PermissionScope::Packages => PermissionLevel::Write,
    })
}

/// GCP Workload Identity Federation - authenticate to GCP via OIDC.
///
/// Uses: google-github-actions/auth@v2
/// Requires: id-token:write
pub fn gcp_workload_identity() -> Integration {
    Integration::new(
        "gcp-wif",
        "google-github-actions/auth@v2",
        "Authenticate to GCP via Workload Identity Federation",
    )
    .with_permissions(permissions! {
        PermissionScope::IdToken => PermissionLevel::Write,
    })
}

/// Upload workflow artifact.
///
/// Uses: actions/upload-artifact@v4
/// Requires: actions:write
pub fn upload_artifact() -> Integration {
    Integration::new(
        "upload-artifact",
        "actions/upload-artifact@v4",
        "Upload workflow artifacts",
    )
    .with_permissions(permissions! {
        PermissionScope::Actions => PermissionLevel::Write,
    })
}

/// All known integrations.
pub fn all_integrations() -> Vec<Integration> {
    vec![
        checkout(),
        checkout_push(),
        rust_toolchain(),
        ghcr_push(),
        gcp_workload_identity(),
        upload_artifact(),
    ]
}

// ============================================================================
// Runner Image Model
// ============================================================================

/// GitHub Actions runner image with pre-installed tools.
///
/// Models what software is available on a GitHub-hosted runner.
/// This enables checking if dependencies are satisfied without
/// explicit installation steps.
#[derive(Debug, Clone)]
pub struct RunnerImage {
    /// Runner label (e.g., "ubuntu-latest", "ubuntu-24.04")
    pub id: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// Pre-installed tools available on this runner
    tools: Vec<&'static str>,
    /// Source documentation URL
    pub docs_url: &'static str,
}

impl RunnerImage {
    /// Create a new runner image definition.
    pub fn new(id: &'static str, name: &'static str, docs_url: &'static str) -> Self {
        Self {
            id,
            name,
            tools: Vec::new(),
            docs_url,
        }
    }

    /// Add tools to the runner image.
    pub fn with_tools(mut self, tools: Vec<&'static str>) -> Self {
        self.tools = tools;
        self
    }

    /// Check if a tool is pre-installed on this runner.
    pub fn has_tool(&self, tool: &str) -> bool {
        self.tools.contains(&tool)
    }

    /// Get all pre-installed tools.
    pub fn tools(&self) -> &[&'static str] {
        &self.tools
    }

    /// Check if all specified tools are available.
    pub fn has_all_tools(&self, required: &[&str]) -> bool {
        required.iter().all(|t| self.has_tool(t))
    }

    /// Get tools that are missing from this runner.
    pub fn missing_tools<'a>(&self, required: &[&'a str]) -> Vec<&'a str> {
        required
            .iter()
            .filter(|t| !self.has_tool(t))
            .copied()
            .collect()
    }
}

// ============================================================================
// Runner Image Catalog
// ============================================================================

/// Ubuntu Latest runner image.
///
/// This is the default runner for most GitHub Actions workflows.
/// Tools listed are commonly available; the actual list is much larger.
///
/// Source: https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md
pub fn ubuntu_latest() -> RunnerImage {
    RunnerImage::new(
        "ubuntu-latest",
        "GitHub Actions Ubuntu Latest Runner",
        "https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md",
    )
    .with_tools(vec![
        // Rust toolchain (via rustup)
        "cargo",
        "rustc",
        "rustup",
        // GitHub CLI
        "gh",
        // Git
        "git",
        // Build tools
        "make",
        "cmake",
        "gcc",
        "g++",
        "clang",
        // Package managers
        "npm",
        "yarn",
        "pip",
        "pip3",
        // Container tools
        "docker",
        "docker-compose",
        // Cloud CLIs
        "aws",
        "az",
        "gcloud",
        // Utilities
        "curl",
        "wget",
        "jq",
        "zip",
        "unzip",
    ])
}

/// Ubuntu 24.04 runner image (explicit version).
pub fn ubuntu_24_04() -> RunnerImage {
    RunnerImage::new(
        "ubuntu-24.04",
        "GitHub Actions Ubuntu 24.04 Runner",
        "https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md",
    )
    .with_tools(ubuntu_latest().tools().to_vec())
}

/// Ubuntu 22.04 runner image.
pub fn ubuntu_22_04() -> RunnerImage {
    RunnerImage::new(
        "ubuntu-22.04",
        "GitHub Actions Ubuntu 22.04 Runner",
        "https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2204-Readme.md",
    )
    .with_tools(ubuntu_latest().tools().to_vec())
}

/// All known runner images.
pub fn all_runner_images() -> Vec<RunnerImage> {
    vec![ubuntu_latest(), ubuntu_24_04(), ubuntu_22_04()]
}

/// Get a runner image by ID.
pub fn runner_image_by_id(id: &str) -> Option<RunnerImage> {
    all_runner_images().into_iter().find(|r| r.id == id)
}

// ============================================================================
// Workflow Configuration
// ============================================================================

/// GitHub Actions workflow configuration.
///
/// Contains all the metadata needed to generate a complete workflow YAML,
/// with permissions automatically computed from declared integrations.
///
/// This struct is the spec for workflow generation. Use the [`Renderable`]
/// trait implementation to generate the actual YAML file.
///
/// # Example
///
/// ```ignore
/// use gunbc_ir::transport::github_actions::*;
///
/// let config = WorkflowConfig::new(
///     "CI",
///     ubuntu_latest(),
///     vec![checkout(), rust_toolchain()],
/// );
///
/// // Permissions are auto-computed from integrations
/// assert!(config.has_permissions());
///
/// // Generate the YAML
/// let yaml = config.render();
/// ```
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    /// Workflow name
    pub name: &'static str,
    /// Target runner image
    pub runner: RunnerImage,
    /// Integrations (actions) used by this workflow
    pub integrations: Vec<Integration>,
    /// Computed permissions from integrations
    pub permissions: Permissions,
    /// Command to run in the CI step
    pub run_command: String,
}

impl WorkflowConfig {
    /// Create a new workflow configuration.
    ///
    /// Permissions are automatically computed from all integrations.
    pub fn new(name: &'static str, runner: RunnerImage, integrations: Vec<Integration>) -> Self {
        // Compute permissions from all integrations
        let permission_sets: Vec<Permissions> = integrations
            .iter()
            .map(|i| i.required_permissions())
            .collect();
        let permissions = merge_permissions(&permission_sets);

        Self {
            name,
            runner,
            integrations,
            permissions,
            run_command: String::new(),
        }
    }

    /// Set the run command for the CI step.
    pub fn with_run_command(mut self, cmd: impl Into<String>) -> Self {
        self.run_command = cmd.into();
        self
    }

    /// Check if this workflow has any special permissions.
    pub fn has_permissions(&self) -> bool {
        !self.permissions.is_empty()
    }

    /// Get all action references (uses: fields) for the workflow.
    pub fn action_refs(&self) -> Vec<&'static str> {
        self.integrations.iter().map(|i| i.uses).collect()
    }

    /// Get all tools available to this workflow.
    ///
    /// This combines tools provided by the runner image with tools
    /// provided by the integrations (actions) used in the workflow.
    pub fn available_tools(&self) -> HashSet<&str> {
        let mut tools: HashSet<&str> = self.runner.tools().iter().copied().collect();
        for integration in &self.integrations {
            tools.extend(integration.provides_tools().iter().copied());
        }
        tools
    }

    /// Check if all required tools are available.
    ///
    /// Returns Ok(()) if all tools are available, or Err with the list of
    /// missing tool IDs.
    pub fn check_satisfiability<'a>(&self, required: &[&'a str]) -> Result<(), Vec<&'a str>> {
        let available = self.available_tools();
        let missing: Vec<&'a str> = required
            .iter()
            .filter(|t| !available.contains(*t))
            .copied()
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

/// Composed generator name for WorkflowConfig.
/// Must match `cargo::name("ci")` — verified by test.
const CI_GENERATOR_NAME: &str = "gunbc-ci";

impl Renderable for WorkflowConfig {
    fn generator_name(&self) -> &str {
        CI_GENERATOR_NAME
    }

    fn regenerate_command(&self) -> &str {
        "make ci-yaml"
    }

    fn format_id(&self) -> &str {
        "yaml"
    }

    fn render_content(&self) -> String {
        let mut yaml = String::new();

        yaml.push_str(&format!("name: {}\n\n", self.name));

        // Triggers
        yaml.push_str("on:\n");
        yaml.push_str("  push:\n");
        yaml.push_str("    branches: [main, master]\n");
        yaml.push_str("  pull_request:\n");
        yaml.push_str("    branches: [main, master]\n\n");

        // Jobs
        yaml.push_str("jobs:\n");
        yaml.push_str("  ci:\n");
        yaml.push_str(&format!("    runs-on: {}\n", self.runner.id));
        yaml.push_str("    steps:\n");

        // Integration steps (actions)
        for integration in &self.integrations {
            yaml.push_str(&format!("      - uses: {}\n", integration.uses));
        }

        // Run command
        if !self.run_command.is_empty() {
            yaml.push_str("      - name: Run CI\n");
            yaml.push_str(&format!("        run: {}\n", self.run_command));
        }

        yaml
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_scope_yaml_key() {
        assert_eq!(PermissionScope::Contents.as_yaml_key(), "contents");
        assert_eq!(PermissionScope::PullRequests.as_yaml_key(), "pull-requests");
        assert_eq!(PermissionScope::IdToken.as_yaml_key(), "id-token");
    }

    #[test]
    fn test_permission_level_ordering() {
        assert!(PermissionLevel::Write > PermissionLevel::Read);
        assert!(PermissionLevel::Read > PermissionLevel::None);
    }

    #[test]
    fn test_merge_permissions() {
        let set1 = permissions! {
            PermissionScope::Contents => PermissionLevel::Read,
        };
        let set2 = permissions! {
            PermissionScope::Contents => PermissionLevel::Write,
            PermissionScope::Packages => PermissionLevel::Read,
        };

        let merged = merge_permissions(&[set1, set2]);

        // Write wins over Read for Contents
        assert_eq!(
            merged.get(&PermissionScope::Contents),
            Some(&PermissionLevel::Write)
        );
        // Packages is included
        assert_eq!(
            merged.get(&PermissionScope::Packages),
            Some(&PermissionLevel::Read)
        );
    }

    #[test]
    fn test_checkout_integration() {
        let checkout = checkout();
        assert_eq!(checkout.id, "checkout");
        assert_eq!(checkout.uses, "actions/checkout@v4");

        let perms = checkout.required_permissions();
        assert_eq!(
            perms.get(&PermissionScope::Contents),
            Some(&PermissionLevel::Read)
        );
    }

    #[test]
    fn test_rust_toolchain_no_permissions() {
        let toolchain = rust_toolchain();
        assert!(toolchain.required_permissions().is_empty());
    }

    #[test]
    fn test_ubuntu_latest_has_cargo() {
        let runner = ubuntu_latest();
        assert!(runner.has_tool("cargo"));
        assert!(runner.has_tool("gh"));
        assert!(runner.has_tool("git"));
    }

    #[test]
    fn test_runner_missing_tools() {
        let runner = ubuntu_latest();
        let missing = runner.missing_tools(&["cargo", "nonexistent-tool"]);
        assert_eq!(missing, vec!["nonexistent-tool"]);
    }

    #[test]
    fn test_all_integrations() {
        let integrations = all_integrations();
        assert!(integrations.len() >= 4);

        // Verify we can find checkout
        assert!(integrations.iter().any(|i| i.id == "checkout"));
    }

    #[test]
    fn test_runner_image_by_id() {
        assert!(runner_image_by_id("ubuntu-latest").is_some());
        assert!(runner_image_by_id("ubuntu-24.04").is_some());
        assert!(runner_image_by_id("nonexistent").is_none());
    }

    // ========================================================================
    // WorkflowConfig and Renderable Tests
    // ========================================================================

    #[test]
    fn test_workflow_config_new() {
        let config =
            WorkflowConfig::new("Test", ubuntu_latest(), vec![checkout(), rust_toolchain()]);

        assert_eq!(config.name, "Test");
        assert_eq!(config.runner.id, "ubuntu-latest");
        assert_eq!(config.integrations.len(), 2);
        // Should have computed permissions from checkout
        assert!(config.has_permissions());
    }

    #[test]
    fn test_workflow_config_with_run_command() {
        let config = WorkflowConfig::new("CI", ubuntu_latest(), vec![checkout()])
            .with_run_command("cargo test");

        assert_eq!(config.run_command, "cargo test");
    }

    #[test]
    fn test_workflow_config_render() {
        let config = WorkflowConfig::new("CI", ubuntu_latest(), vec![checkout(), rust_toolchain()])
            .with_run_command("cargo test");

        let yaml = config.render();

        // Check header
        assert!(yaml.contains("# Generated by gunbc-ci"));
        assert!(yaml.contains("# DO NOT EDIT - regenerate with: make ci-yaml"));

        // Check content
        assert!(yaml.contains("name: CI"));
        assert!(yaml.contains("runs-on: ubuntu-latest"));
        assert!(yaml.contains("- uses: actions/checkout@v4"));
        assert!(yaml.contains("- uses: dtolnay/rust-toolchain@stable"));
        assert!(yaml.contains("run: cargo test"));
    }

    #[test]
    fn test_workflow_config_render_content_only() {
        let config = WorkflowConfig::new("CI", ubuntu_latest(), vec![checkout()])
            .with_run_command("cargo build");

        let content = config.render_content();

        // Should NOT have header (render_content is just the body)
        assert!(!content.contains("Generated by"));

        // Should have the yaml content
        assert!(content.contains("name: CI"));
        assert!(content.contains("on:"));
        assert!(content.contains("push:"));
        assert!(content.contains("branches: [main, master]"));
    }

    #[test]
    fn test_workflow_config_available_tools() {
        let config = WorkflowConfig::new("CI", ubuntu_latest(), vec![checkout(), rust_toolchain()]);

        let tools = config.available_tools();

        // Should have tools from runner
        assert!(tools.contains("cargo"));
        assert!(tools.contains("git"));

        // Should have tools from integrations
        // checkout provides git, rust_toolchain provides cargo, clippy, rustfmt
        assert!(tools.contains("clippy"));
        assert!(tools.contains("rustfmt"));
    }

    #[test]
    fn test_workflow_config_check_satisfiability() {
        let config = WorkflowConfig::new("CI", ubuntu_latest(), vec![checkout(), rust_toolchain()]);

        // Should be satisfied - all tools available
        assert!(config
            .check_satisfiability(&["cargo", "git", "clippy"])
            .is_ok());

        // Should fail - missing tool
        let result = config.check_satisfiability(&["cargo", "nonexistent"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["nonexistent"]);
    }
}
