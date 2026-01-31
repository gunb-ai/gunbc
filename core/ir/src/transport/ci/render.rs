//! CI YAML rendering from DAGs.
//!
//! This module provides the `CiRenderer` trait for generating CI workflow YAML
//! from DAG structures. Each CI provider (GitHub Actions, GitLab CI) implements
//! its own rendering strategy while sharing common node definitions.
//!
//! # Architecture
//!
//! ```text
//! Dag<T>                     CiRenderer::render()
//!   │                              │
//!   │  ┌───────────────────────────┼───────────────────────────┐
//!   │  │                           │                           │
//!   ▼  ▼                           ▼                           ▼
//! GitHubActionsRenderer      GitLabCiRenderer           (future providers)
//!   │                           │
//!   ▼                           ▼
//! steps:                    stages:
//!   - uses: actions/checkout    - build
//!   - run: cargo build          - test
//!   - run: cargo test           - lint
//! ```
//!
//! # Design Principles
//!
//! 1. **Provider-owned mapping**: Each provider owns its DAG→YAML mapping
//! 2. **Shared node definitions**: Common DAG nodes are reused across providers
//! 3. **Step granularity**: Each DAG node becomes a CI step for visibility
//! 4. **Data passing**: Node outputs become step outputs (GitHub) or artifacts (GitLab)

use crate::{Dag, NodeId};
use std::collections::HashMap;

/// Trait for rendering DAGs to CI workflow YAML.
///
/// Each CI provider implements this trait to generate its native YAML format.
/// The trait provides a uniform interface while allowing provider-specific
/// rendering strategies.
pub trait CiRenderer {
    /// The CI provider identifier (e.g., "github-actions", "gitlab-ci").
    fn provider_id(&self) -> &'static str;

    /// Render a DAG to CI workflow YAML.
    ///
    /// # Arguments
    ///
    /// * `dag` - The DAG to render
    /// * `config` - Rendering configuration (tool name, runner, etc.)
    ///
    /// # Returns
    ///
    /// The generated YAML as a string.
    fn render<T>(&self, dag: &Dag<T>, config: &RenderConfig) -> String;

    /// Get the file path where this CI config should be written.
    ///
    /// For GitHub Actions: `.github/workflows/{name}.yml`
    /// For GitLab CI: `.gitlab-ci.yml`
    fn output_path(&self, workflow_name: &str) -> String;
}

/// Configuration for CI YAML rendering.
#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    /// Name of the workflow/pipeline.
    pub workflow_name: String,

    /// The CLI tool binary name (e.g., "gunbc-ci").
    pub tool_binary: String,

    /// The package name if the binary lives in a different package (e.g., "gunbc-dag").
    /// When set, cargo commands use `-p <tool_package> --bin <tool_binary>`.
    /// When None, cargo commands use `-p <tool_binary>`.
    pub tool_package: Option<String>,

    /// Runner/image to use (e.g., "ubuntu-latest", "saas-linux-small-amd64").
    pub runner: String,

    /// Whether to use step mode (each node as separate CI step).
    pub step_mode: bool,

    /// Additional environment variables to set.
    pub env: HashMap<String, String>,

    /// Checkout action/config (provider-specific).
    pub checkout: Option<CheckoutConfig>,

    /// Branches to trigger on (for push/PR).
    pub branches: Vec<String>,

    /// Caching configuration.
    pub cache: Option<CacheConfig>,
}

impl RenderConfig {
    /// Create a new render config with defaults.
    pub fn new(workflow_name: &str, tool_binary: &str) -> Self {
        Self {
            workflow_name: workflow_name.to_string(),
            tool_binary: tool_binary.to_string(),
            tool_package: None,
            runner: "ubuntu-latest".to_string(),
            step_mode: true,
            env: HashMap::new(),
            checkout: Some(CheckoutConfig::default()),
            branches: vec!["main".to_string()],
            cache: None,
        }
    }

    /// Set the package name when the binary lives in a different package.
    pub fn with_package(mut self, package: &str) -> Self {
        self.tool_package = Some(package.to_string());
        self
    }

    /// Get the `cargo run` command for this tool.
    ///
    /// Returns `cargo run -p <package> --bin <binary>` when tool_package is set,
    /// or `cargo run -p <binary>` when the binary is in its own package.
    pub fn cargo_run_command(&self) -> String {
        match &self.tool_package {
            Some(pkg) => format!("cargo run -p {} --bin {}", pkg, self.tool_binary),
            None => format!("cargo run -p {}", self.tool_binary),
        }
    }

    /// Set the runner.
    pub fn with_runner(mut self, runner: &str) -> Self {
        self.runner = runner.to_string();
        self
    }

    /// Disable step mode (run full DAG in one step).
    pub fn without_step_mode(mut self) -> Self {
        self.step_mode = false;
        self
    }

    /// Add an environment variable.
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Set branches to trigger on.
    pub fn with_branches(mut self, branches: Vec<&str>) -> Self {
        self.branches = branches.into_iter().map(String::from).collect();
        self
    }

    /// Enable caching.
    pub fn with_cache(mut self, cache: CacheConfig) -> Self {
        self.cache = Some(cache);
        self
    }
}

/// Checkout configuration.
#[derive(Debug, Clone)]
pub struct CheckoutConfig {
    /// Fetch depth (0 for full history, 1 for shallow).
    pub fetch_depth: Option<u32>,
    /// Submodules config ("true", "recursive", or empty).
    pub submodules: Option<String>,
}

impl Default for CheckoutConfig {
    fn default() -> Self {
        Self {
            fetch_depth: Some(1),
            submodules: None,
        }
    }
}

/// Cache configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache key pattern.
    pub key: String,
    /// Paths to cache.
    pub paths: Vec<String>,
    /// Restore keys (fallback).
    pub restore_keys: Vec<String>,
}

impl CacheConfig {
    /// Create a Rust/Cargo cache config.
    pub fn rust() -> Self {
        Self {
            key: "cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}".to_string(),
            paths: vec![
                "~/.cargo/bin/".to_string(),
                "~/.cargo/registry/index/".to_string(),
                "~/.cargo/registry/cache/".to_string(),
                "~/.cargo/git/db/".to_string(),
                "target/".to_string(),
            ],
            restore_keys: vec!["cargo-${{ runner.os }}-".to_string()],
        }
    }
}

// ============================================================================
// Shared Node Definitions
// ============================================================================

/// Common CI step that can be rendered by any provider.
///
/// These shared definitions ensure consistency across GitHub Actions and GitLab CI
/// while allowing provider-specific rendering.
#[derive(Debug, Clone)]
pub enum SharedStep {
    /// Checkout code from repository.
    Checkout(CheckoutConfig),
    /// Run a shell command.
    Run { name: String, command: String },
    /// Run a DAG tool in step mode.
    DagStep {
        tool_binary: String,
        node_id: NodeId,
        depends_on: Vec<NodeId>,
    },
    /// Run a DAG tool (full execution).
    DagRun { tool_binary: String },
}

impl SharedStep {
    /// Create a checkout step.
    pub fn checkout() -> Self {
        Self::Checkout(CheckoutConfig::default())
    }

    /// Create a shell command step.
    pub fn run(name: &str, command: &str) -> Self {
        Self::Run {
            name: name.to_string(),
            command: command.to_string(),
        }
    }

    /// Create a DAG step execution.
    pub fn dag_step(tool_binary: &str, node_id: impl Into<NodeId>, depends_on: Vec<NodeId>) -> Self {
        Self::DagStep {
            tool_binary: tool_binary.to_string(),
            node_id: node_id.into(),
            depends_on,
        }
    }

    /// Create a full DAG execution.
    pub fn dag_run(tool_binary: &str) -> Self {
        Self::DagRun {
            tool_binary: tool_binary.to_string(),
        }
    }
}

/// Convert a DAG to a sequence of shared steps.
///
/// This is the shared logic that both GitHub Actions and GitLab CI renderers use.
/// Each provider then renders these shared steps in their native format.
pub fn dag_to_shared_steps<T>(dag: &Dag<T>, config: &RenderConfig) -> Vec<SharedStep> {
    let mut steps = Vec::new();

    // 1. Checkout step
    if let Some(checkout) = &config.checkout {
        steps.push(SharedStep::Checkout(checkout.clone()));
    }

    if config.step_mode {
        // 2. Each DAG node becomes a step
        // Build dependency map
        let mut depends_on: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for edge in &dag.edges {
            depends_on
                .entry(edge.to_node.clone())
                .or_default()
                .push(edge.from_node.clone());
        }

        for node in &dag.nodes {
            let deps = depends_on.get(&node.id).cloned().unwrap_or_default();
            steps.push(SharedStep::dag_step(&config.tool_binary, node.id.clone(), deps));
        }
    } else {
        // 3. Single step: run the full DAG
        steps.push(SharedStep::dag_run(&config.tool_binary));
    }

    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{edge, port};
    use crate::{Dag, Node, NodeBody};

    // Dummy op for testing
    #[derive(Debug, Clone)]
    struct DummyOp;

    fn test_dag() -> Dag<DummyOp> {
        let mut dag = Dag::new();
        dag.add_node(Node {
            id: "build".into(),
            inputs: vec![],
            outputs: vec![port("success", "Bool")],
            body: NodeBody::Opaque(DummyOp),
            requires_tools: vec![],
        });
        dag.add_node(Node {
            id: "test".into(),
            inputs: vec![port("build_success", "Bool")],
            outputs: vec![port("success", "Bool")],
            body: NodeBody::Opaque(DummyOp),
            requires_tools: vec![],
        });
        dag.add_node(Node {
            id: "lint".into(),
            inputs: vec![port("build_success", "Bool")],
            outputs: vec![port("success", "Bool")],
            body: NodeBody::Opaque(DummyOp),
            requires_tools: vec![],
        });
        dag.add_edge(edge("build", "success", "test", "build_success"));
        dag.add_edge(edge("build", "success", "lint", "build_success"));
        dag
    }

    #[test]
    fn test_dag_to_shared_steps_step_mode() {
        let dag = test_dag();
        let config = RenderConfig::new("ci", "gunbc-ci");
        let steps = dag_to_shared_steps(&dag, &config);

        // Should have: checkout + 3 DAG nodes
        assert_eq!(steps.len(), 4);

        // First step is checkout
        assert!(matches!(steps[0], SharedStep::Checkout(_)));

        // Remaining are DAG steps
        for step in &steps[1..] {
            assert!(matches!(step, SharedStep::DagStep { .. }));
        }
    }

    #[test]
    fn test_dag_to_shared_steps_single_mode() {
        let dag = test_dag();
        let config = RenderConfig::new("ci", "gunbc-ci").without_step_mode();
        let steps = dag_to_shared_steps(&dag, &config);

        // Should have: checkout + 1 full DAG run
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0], SharedStep::Checkout(_)));
        assert!(matches!(steps[1], SharedStep::DagRun { .. }));
    }

    #[test]
    fn test_render_config_builder() {
        let config = RenderConfig::new("ci", "gunbc-ci")
            .with_runner("ubuntu-22.04")
            .with_env("CARGO_TERM_COLOR", "always")
            .with_branches(vec!["main", "develop"]);

        assert_eq!(config.runner, "ubuntu-22.04");
        assert_eq!(config.env.get("CARGO_TERM_COLOR"), Some(&"always".to_string()));
        assert_eq!(config.branches, vec!["main", "develop"]);
    }
}
