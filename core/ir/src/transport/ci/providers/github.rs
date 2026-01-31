//! GitHub Actions CI provider.
//!
//! Implements workflow commands using GitHub Actions magic strings:
//! - `::group::{title}` / `::endgroup::` for collapsible sections
//! - `::error file=X,line=Y::{message}` for error annotations
//! - `::warning::`, `::notice::`, `::debug::` for other levels
//! - `::add-mask::{value}` for secrets masking
//!
//! Environment files (`$GITHUB_OUTPUT`, `$GITHUB_STEP_SUMMARY`) are not
//! directly handled here - those require file I/O at the execution layer.
//!
//! # YAML Rendering
//!
//! This module also implements `CiRenderer` for generating GitHub Actions YAML
//! from DAGs. Each DAG node becomes a workflow step with proper dependencies.

use crate::transport::ci::command::{AnnotationLevel, WorkflowCommand};
use crate::transport::ci::provider::CiProvider;
use crate::transport::ci::render::{dag_to_shared_steps, CiRenderer, RenderConfig, SharedStep};
use crate::transport::ci::runner::Runner;
use crate::transport::github_actions::{
    ubuntu_22_04, ubuntu_24_04, ubuntu_latest, RunnerImage,
};
use crate::Dag;

/// GitHub Actions CI provider.
///
/// Formats workflow commands using GitHub's magic string syntax.
/// All commands are natively supported.
#[derive(Debug, Clone, Copy, Default)]
pub struct GitHubActionsProvider;

impl CiProvider for GitHubActionsProvider {
    fn id(&self) -> &'static str {
        "github-actions"
    }

    fn name(&self) -> &'static str {
        "GitHub Actions"
    }

    fn format(&self, cmd: &WorkflowCommand) -> String {
        match cmd {
            WorkflowCommand::GroupStart { name, .. } => {
                // GitHub Actions ignores the collapsed flag in ::group::
                format!("::group::{}", name)
            }

            WorkflowCommand::GroupEnd { .. } => {
                "::endgroup::".to_string()
            }

            WorkflowCommand::Annotation {
                level,
                message,
                title,
                location,
            } => {
                let level_str = match level {
                    AnnotationLevel::Error => "error",
                    AnnotationLevel::Warning => "warning",
                    AnnotationLevel::Notice => "notice",
                    AnnotationLevel::Debug => "debug",
                };

                // Build parameters
                let mut params = Vec::new();

                if let Some(loc) = location {
                    params.push(format!("file={}", loc.file));
                    if let Some(line) = loc.line {
                        params.push(format!("line={}", line));
                    }
                    if let Some(end_line) = loc.end_line {
                        params.push(format!("endLine={}", end_line));
                    }
                    if let Some(col) = loc.col {
                        params.push(format!("col={}", col));
                    }
                    if let Some(end_col) = loc.end_col {
                        params.push(format!("endColumn={}", end_col));
                    }
                }

                if let Some(t) = title {
                    params.push(format!("title={}", t));
                }

                let params_str = if params.is_empty() {
                    String::new()
                } else {
                    format!(" {}", params.join(","))
                };

                format!("::{}{}::{}", level_str, params_str, message)
            }

            WorkflowCommand::SetOutput { key, value } => {
                // Note: This format is for the old ::set-output syntax
                // Modern GitHub Actions uses $GITHUB_OUTPUT file
                // But we output both for compatibility
                format!("::set-output name={}::{}", key, value)
            }

            WorkflowCommand::MaskValue { value } => {
                format!("::add-mask::{}", value)
            }

            WorkflowCommand::Summary { markdown } => {
                // Summaries go to $GITHUB_STEP_SUMMARY file
                // Here we just format as a comment since file I/O is external
                format!("<!-- SUMMARY -->\n{}\n<!-- /SUMMARY -->", markdown)
            }
        }
    }

    fn supports(&self, _cmd: &WorkflowCommand) -> bool {
        // GitHub Actions supports all commands natively
        true
    }

    fn runners(&self) -> Vec<Box<dyn Runner>> {
        vec![
            Box::new(GitHubRunnerAdapter(ubuntu_latest())),
            Box::new(GitHubRunnerAdapter(ubuntu_24_04())),
            Box::new(GitHubRunnerAdapter(ubuntu_22_04())),
        ]
    }

    fn default_runner(&self) -> Box<dyn Runner> {
        Box::new(GitHubRunnerAdapter(ubuntu_latest()))
    }
}

/// Adapter to make RunnerImage implement Runner trait.
#[derive(Debug, Clone)]
struct GitHubRunnerAdapter(RunnerImage);

impl Runner for GitHubRunnerAdapter {
    fn id(&self) -> &str {
        self.0.id
    }

    fn name(&self) -> &str {
        self.0.name
    }

    fn tools(&self) -> &[&str] {
        self.0.tools()
    }

    fn docs_url(&self) -> Option<&str> {
        Some(self.0.docs_url)
    }
}

// ============================================================================
// YAML Rendering
// ============================================================================

impl CiRenderer for GitHubActionsProvider {
    fn provider_id(&self) -> &'static str {
        "github-actions"
    }

    fn render<T>(&self, dag: &Dag<T>, config: &RenderConfig) -> String {
        let steps = dag_to_shared_steps(dag, config);
        render_github_workflow(&steps, config)
    }

    fn output_path(&self, workflow_name: &str) -> String {
        format!(".github/workflows/{}.yml", workflow_name)
    }
}

/// Render a GitHub Actions workflow from shared steps.
fn render_github_workflow(steps: &[SharedStep], config: &RenderConfig) -> String {
    let mut yaml = String::new();

    // Header from render config — generator name and regen command are set by the caller
    yaml.push_str(&config.header("#"));
    yaml.push_str(&format!("\n\nname: {}\n\n", config.workflow_name));

    // Triggers
    yaml.push_str("on:\n");
    yaml.push_str("  push:\n");
    yaml.push_str("    branches:\n");
    for branch in &config.branches {
        yaml.push_str(&format!("      - {}\n", branch));
    }
    yaml.push_str("  pull_request:\n");
    yaml.push_str("    branches:\n");
    for branch in &config.branches {
        yaml.push_str(&format!("      - {}\n", branch));
    }
    yaml.push('\n');

    // Environment variables
    if !config.env.is_empty() {
        yaml.push_str("env:\n");
        for (key, value) in &config.env {
            yaml.push_str(&format!("  {}: {}\n", key, value));
        }
        yaml.push('\n');
    }

    // Jobs
    yaml.push_str("jobs:\n");
    yaml.push_str(&format!("  {}:\n", config.workflow_name));
    yaml.push_str(&format!("    runs-on: {}\n", config.runner));
    yaml.push_str("    steps:\n");

    // Render each step
    for step in steps {
        yaml.push_str(&render_github_step(step, config));
    }

    yaml
}

/// Render a single GitHub Actions step.
fn render_github_step(step: &SharedStep, _config: &RenderConfig) -> String {
    match step {
        SharedStep::Checkout(checkout) => {
            let mut yaml = String::from("      - name: Checkout\n");
            yaml.push_str("        uses: actions/checkout@v4\n");
            if checkout.fetch_depth.is_some() || checkout.submodules.is_some() {
                yaml.push_str("        with:\n");
                if let Some(depth) = checkout.fetch_depth {
                    yaml.push_str(&format!("          fetch-depth: {}\n", depth));
                }
                if let Some(ref submodules) = checkout.submodules {
                    yaml.push_str(&format!("          submodules: {}\n", submodules));
                }
            }
            yaml
        }

        SharedStep::Run { name, command } => {
            format!(
                "      - name: {}\n        run: {}\n",
                name, command
            )
        }

        SharedStep::DagStep {
            tool,
            node_id,
            depends_on,
        } => {
            let mut yaml = format!(
                "      - name: {}\n        id: {}\n        run: {} step {}\n",
                node_id.0, node_id.0, tool.command(), node_id.0
            );

            // Add environment variables for dependencies
            if !depends_on.is_empty() {
                yaml.push_str("        env:\n");
                for dep in depends_on {
                    // Pass outputs from previous steps
                    yaml.push_str(&format!(
                        "          STEP_{}_SUCCESS: ${{{{ steps.{}.outputs.STEP_{}_SUCCESS }}}}\n",
                        dep.0.to_uppercase(),
                        dep.0,
                        dep.0.to_uppercase()
                    ));
                }
            }

            yaml
        }

        SharedStep::DagRun { tool } => {
            format!(
                "      - name: Run {}\n        run: {}\n",
                tool.binary, tool.command()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cargo::CargoInvocation;
    use crate::transport::ci::command::FileLocation;

    #[test]
    fn test_github_group_start() {
        let provider = GitHubActionsProvider;
        let cmd = WorkflowCommand::group_start("build");
        assert_eq!(provider.format(&cmd), "::group::build");
    }

    #[test]
    fn test_github_group_end() {
        let provider = GitHubActionsProvider;
        let cmd = WorkflowCommand::group_end("build");
        assert_eq!(provider.format(&cmd), "::endgroup::");
    }

    #[test]
    fn test_github_error_simple() {
        let provider = GitHubActionsProvider;
        let cmd = WorkflowCommand::error("test failed");
        assert_eq!(provider.format(&cmd), "::error::test failed");
    }

    #[test]
    fn test_github_error_with_location() {
        let provider = GitHubActionsProvider;
        let cmd = WorkflowCommand::annotation_at(
            AnnotationLevel::Error,
            "syntax error",
            FileLocation::new("src/main.rs").with_line(42),
        );
        assert_eq!(
            provider.format(&cmd),
            "::error file=src/main.rs,line=42::syntax error"
        );
    }

    #[test]
    fn test_github_warning() {
        let provider = GitHubActionsProvider;
        let cmd = WorkflowCommand::warning("deprecated");
        assert_eq!(provider.format(&cmd), "::warning::deprecated");
    }

    #[test]
    fn test_github_mask() {
        let provider = GitHubActionsProvider;
        let cmd = WorkflowCommand::mask("secret123");
        assert_eq!(provider.format(&cmd), "::add-mask::secret123");
    }

    #[test]
    fn test_github_default_runner() {
        let provider = GitHubActionsProvider;
        let runner = provider.default_runner();
        assert_eq!(runner.id(), "ubuntu-latest");
        assert!(runner.has_tool("cargo"));
    }

    #[test]
    fn test_github_render_workflow() {
        use crate::build::{edge, port};
        use crate::{Node, NodeBody};

        #[derive(Debug, Clone)]
        struct DummyOp;

        let mut dag: Dag<DummyOp> = Dag::new();
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
        dag.add_edge(edge("build", "success", "test", "build_success"));

        let provider = GitHubActionsProvider;
        let tool = CargoInvocation::composed("ci", "dag");
        let config = RenderConfig::new("ci", tool)
            .with_runner("ubuntu-latest")
            .with_env("CARGO_TERM_COLOR", "always");

        let yaml = provider.render(&dag, &config);

        let ci_name = crate::cargo::name("ci");
        // Check structure
        assert!(yaml.contains("name: ci"));
        assert!(yaml.contains("runs-on: ubuntu-latest"));
        assert!(yaml.contains("uses: actions/checkout@v4"));
        assert!(yaml.contains(&format!("{ci_name} step build")));
        assert!(yaml.contains(&format!("{ci_name} step test")));
        assert!(yaml.contains("CARGO_TERM_COLOR: always"));
    }

    #[test]
    fn test_github_output_path() {
        let provider = GitHubActionsProvider;
        assert_eq!(provider.output_path("ci"), ".github/workflows/ci.yml");
    }
}
