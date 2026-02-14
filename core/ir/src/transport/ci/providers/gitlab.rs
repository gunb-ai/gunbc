//! GitLab CI provider.
//!
//! Implements workflow commands using GitLab's escape sequence syntax:
//! - `\e[0Ksection_start:TIMESTAMP:NAME\r\e[0K{header}` for collapsible sections
//! - `\e[0Ksection_end:TIMESTAMP:NAME\r\e[0K` for section end
//! - ANSI colored text for annotations (no native support)
//!
//! GitLab CI has limited native support compared to GitHub Actions:
//! - No inline error annotations (falls back to colored text)
//! - No job summaries (falls back to formatted output)
//! - Secrets masking is handled by CI variables, not inline commands
//!
//! # YAML Rendering
//!
//! This module also implements `CiRenderer` for generating GitLab CI YAML
//! from DAGs. Uses stages for parallelism and `needs` for dependencies.

use crate::language::NamingCase;
use crate::transport::ci::command::{AnnotationLevel, WorkflowCommand};
use crate::transport::ci::provider::CiProvider;
use crate::transport::ci::render::{dag_to_shared_steps, CiRenderer, RenderConfig, SharedStep};
use crate::transport::ci::runner::{
    gitlab_saas_linux_large, gitlab_saas_linux_medium, gitlab_saas_linux_small, Runner,
};
use crate::Dag;
use std::collections::HashSet;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// GitLab CI provider.
///
/// Formats workflow commands using GitLab's escape sequence syntax.
/// Commands without native support get graceful ANSI-colored fallbacks.
#[derive(Debug, Clone)]
pub struct GitLabCiProvider {
    /// Whether to use timestamps (default: true)
    use_timestamps: bool,
}

impl GitLabCiProvider {
    /// Create a new GitLab CI provider.
    pub fn new() -> Self {
        Self {
            use_timestamps: true,
        }
    }

    /// Create a provider with fixed timestamps (for testing).
    #[cfg(test)]
    fn with_fixed_timestamp() -> Self {
        Self {
            use_timestamps: false,
        }
    }

    /// Get the current Unix timestamp.
    fn timestamp(&self) -> u64 {
        if self.use_timestamps {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        } else {
            // Fixed timestamp for testing
            1234567890
        }
    }

    /// Sanitize a section name for GitLab.
    /// GitLab section names can only contain letters, numbers, _, ., -
    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }
}

impl Default for GitLabCiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CiProvider for GitLabCiProvider {
    fn id(&self) -> &'static str {
        "gitlab-ci"
    }

    fn name(&self) -> &'static str {
        "GitLab CI"
    }

    fn format(&self, cmd: &WorkflowCommand) -> String {
        match cmd {
            WorkflowCommand::GroupStart { name, collapsed } => {
                let ts = self.timestamp();
                let sanitized = Self::sanitize_name(name);
                let opts = if *collapsed { "[collapsed=true]" } else { "" };
                // GitLab section format with escape sequences
                format!(
                    "\x1b[0Ksection_start:{}:{}{}\r\x1b[0K{}",
                    ts, sanitized, opts, name
                )
            }

            WorkflowCommand::GroupEnd { name } => {
                let ts = self.timestamp();
                let sanitized = Self::sanitize_name(name);
                format!("\x1b[0Ksection_end:{}:{}\r\x1b[0K", ts, sanitized)
            }

            // GitLab doesn't have inline annotations - use ANSI colors
            WorkflowCommand::Annotation {
                level,
                message,
                title,
                location,
            } => {
                let (color, prefix) = match level {
                    AnnotationLevel::Error => ("\x1b[31m", "[ERROR]"), // Red
                    AnnotationLevel::Warning => ("\x1b[33m", "[WARNING]"), // Yellow
                    AnnotationLevel::Notice => ("\x1b[36m", "[NOTICE]"), // Cyan
                    AnnotationLevel::Debug => ("\x1b[90m", "[DEBUG]"), // Gray
                };
                let reset = "\x1b[0m";

                let loc_part = location
                    .as_ref()
                    .map(|l| {
                        let line = l.line.map(|n| format!(":{}", n)).unwrap_or_default();
                        format!(" ({}{})", l.file, line)
                    })
                    .unwrap_or_default();

                let title_part = title
                    .as_ref()
                    .map(|t| format!(" {}: ", t))
                    .unwrap_or_else(|| " ".to_string());

                format!(
                    "{}{}{}{}{}{}",
                    color, prefix, loc_part, title_part, message, reset
                )
            }

            // GitLab uses artifacts for outputs
            WorkflowCommand::SetOutput { key, value } => {
                // Format as dotenv-style output (could be written to artifact)
                format!("{}={}", key, value)
            }

            // GitLab doesn't have inline masking - CI variables handle this
            WorkflowCommand::MaskValue { .. } => {
                // Can't mask inline in GitLab, just acknowledge
                "\x1b[90m[value masked via CI variable]\x1b[0m".to_string()
            }

            // GitLab doesn't have job summaries
            WorkflowCommand::Summary { markdown } => {
                // Format as a visible section
                let ts = self.timestamp();
                format!(
                    "\x1b[0Ksection_start:{}:summary[collapsed=true]\r\x1b[0K\x1b[1mJob Summary\x1b[0m\n{}\n\x1b[0Ksection_end:{}:summary\r\x1b[0K",
                    ts, markdown, ts
                )
            }
        }
    }

    fn supports(&self, cmd: &WorkflowCommand) -> bool {
        match cmd {
            // Native support
            WorkflowCommand::GroupStart { .. } | WorkflowCommand::GroupEnd { .. } => true,
            // Graceful fallback (not native)
            WorkflowCommand::Annotation { .. }
            | WorkflowCommand::SetOutput { .. }
            | WorkflowCommand::MaskValue { .. }
            | WorkflowCommand::Summary { .. } => false,
        }
    }

    fn runners(&self) -> Vec<Box<dyn Runner>> {
        vec![
            Box::new(gitlab_saas_linux_small()),
            Box::new(gitlab_saas_linux_medium()),
            Box::new(gitlab_saas_linux_large()),
        ]
    }

    fn default_runner(&self) -> Box<dyn Runner> {
        Box::new(gitlab_saas_linux_small())
    }
}

// ============================================================================
// YAML Rendering
// ============================================================================

impl CiRenderer for GitLabCiProvider {
    fn provider_id(&self) -> &'static str {
        "gitlab-ci"
    }

    fn render<T>(&self, dag: &Dag<T>, config: &RenderConfig) -> String {
        let steps = dag_to_shared_steps(dag, config);
        render_gitlab_ci(&steps, config)
    }

    fn output_path(&self, _workflow_name: &str) -> String {
        // GitLab CI always uses .gitlab-ci.yml
        ".gitlab-ci.yml".to_string()
    }
}

/// Render a GitLab CI configuration from shared steps.
fn render_gitlab_ci(steps: &[SharedStep], config: &RenderConfig) -> String {
    use crate::transport::ci::yaml_block;

    let mut yaml = String::new();

    yaml.push_str(&config.header("#"));
    write!(yaml, "\n\nimage: {}\n\n", config.runner.id).unwrap();

    // Variables — derived from cargo env + manual overrides
    yaml_block(&mut yaml, "variables:", &config.all_env(), |(k, v)| {
        format!("  {}: \"{}\"", k, v)
    });

    // Stages — computed from DAG structure
    yaml_block(&mut yaml, "stages:", &compute_stages(steps), |s| {
        format!("  - {}", s)
    });

    for step in steps {
        yaml.push_str(&render_gitlab_job(step, config));
    }

    yaml
}

/// Compute stages from shared steps.
///
/// GitLab CI uses stages for parallel execution. Jobs in the same stage
/// run in parallel, jobs in later stages wait for earlier stages.
///
/// Strategy: Build a stage for each "level" of the DAG based on dependencies.
fn compute_stages(steps: &[SharedStep]) -> Vec<String> {
    let mut stages = Vec::new();
    let mut seen_checkout = false;

    for step in steps {
        match step {
            SharedStep::Checkout(_) => {
                if !seen_checkout {
                    stages.push("prepare".to_string());
                    seen_checkout = true;
                }
            }
            SharedStep::DagStep {
                node_id,
                depends_on,
                ..
            } => {
                // If no dependencies (other than checkout), it's a "build" stage
                // Otherwise, determine stage based on depth
                let stage_name = if depends_on.is_empty() {
                    node_id.0.clone()
                } else {
                    // Use the node id as stage name for simplicity
                    // A more sophisticated impl would compute DAG levels
                    node_id.0.clone()
                };
                if !stages.contains(&stage_name) {
                    stages.push(stage_name);
                }
            }
            SharedStep::DagRun { tool } => {
                let stage = format!("{}-run", NamingCase::SnakeCase.apply(&tool.binary));
                if !stages.contains(&stage) {
                    stages.push(stage);
                }
            }
            SharedStep::Run { name, .. } => {
                if !stages.contains(name) {
                    stages.push(name.clone());
                }
            }
        }
    }

    stages
}

/// Render a single GitLab CI job.
fn render_gitlab_job(step: &SharedStep, _config: &RenderConfig) -> String {
    match step {
        SharedStep::Checkout(_checkout) => {
            // GitLab CI automatically checks out code, but we can add a prepare job
            "prepare:\n  stage: prepare\n  script:\n    - git --version\n    - ls -la\n  rules:\n    - if: $CI_PIPELINE_SOURCE == \"push\" || $CI_PIPELINE_SOURCE == \"merge_request_event\"\n\n"
                .to_string()
        }

        SharedStep::Run { name, command } => {
            format!(
                "{}:\n  stage: {}\n  script:\n    - {}\n\n",
                name.replace(' ', "_").to_lowercase(),
                name,
                command
            )
        }

        SharedStep::DagStep {
            tool,
            node_id,
            depends_on,
        } => {
            let mut yaml = format!(
                "{}:\n  stage: {}\n  script:\n    - {} step {}\n",
                node_id.0,
                node_id.0,
                tool.command(),
                node_id.0
            );

            // Add dependencies using `needs`
            if !depends_on.is_empty() {
                yaml.push_str("  needs:\n");
                let seen: HashSet<_> = depends_on.iter().map(|d| &d.0).collect();
                for dep in seen {
                    writeln!(yaml, "    - {}", dep).unwrap();
                }
            }

            // Artifacts for passing data
            yaml.push_str("  artifacts:\n");
            yaml.push_str("    reports:\n");
            writeln!(yaml, "      dotenv: {}.env", node_id.0).unwrap();

            yaml.push('\n');
            yaml
        }

        SharedStep::DagRun { tool } => {
            let stage_name = format!("{}-run", NamingCase::SnakeCase.apply(&tool.binary));
            format!(
                "{}:\n  stage: {}\n  script:\n    - {}\n\n",
                stage_name,
                stage_name,
                tool.command()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cargo::CargoInvocation;
    use crate::transport::ci::command::FileLocation;

    fn test_provider() -> GitLabCiProvider {
        GitLabCiProvider::with_fixed_timestamp()
    }

    #[test]
    fn test_gitlab_group_start() {
        let provider = test_provider();
        let cmd = WorkflowCommand::group_start("build");
        let output = provider.format(&cmd);
        assert!(output.contains("section_start:1234567890:build"));
        assert!(output.contains("\r\x1b[0Kbuild"));
    }

    #[test]
    fn test_gitlab_group_start_collapsed() {
        let provider = test_provider();
        let cmd = WorkflowCommand::group_start_collapsed("build");
        let output = provider.format(&cmd);
        assert!(output.contains("[collapsed=true]"));
    }

    #[test]
    fn test_gitlab_group_end() {
        let provider = test_provider();
        let cmd = WorkflowCommand::group_end("build");
        let output = provider.format(&cmd);
        assert!(output.contains("section_end:1234567890:build"));
    }

    #[test]
    fn test_gitlab_name_sanitization() {
        let provider = test_provider();
        let cmd = WorkflowCommand::group_start("build/test unit");
        let output = provider.format(&cmd);
        // Spaces and slashes should be replaced with underscores in the section name
        assert!(output.contains("build_test_unit"));
    }

    #[test]
    fn test_gitlab_error_fallback() {
        let provider = test_provider();
        let cmd = WorkflowCommand::error("test failed");
        let output = provider.format(&cmd);
        // Should use ANSI red color
        assert!(output.contains("\x1b[31m"));
        assert!(output.contains("[ERROR]"));
        assert!(output.contains("test failed"));
        assert!(output.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_gitlab_error_with_location() {
        let provider = test_provider();
        let cmd = WorkflowCommand::annotation_at(
            AnnotationLevel::Error,
            "syntax error",
            FileLocation::new("src/main.rs").with_line(42),
        );
        let output = provider.format(&cmd);
        assert!(output.contains("(src/main.rs:42)"));
    }

    #[test]
    fn test_gitlab_supports() {
        let provider = test_provider();

        // Native support
        assert!(provider.supports(&WorkflowCommand::group_start("x")));
        assert!(provider.supports(&WorkflowCommand::group_end("x")));

        // Fallback (not native)
        assert!(!provider.supports(&WorkflowCommand::error("x")));
        assert!(!provider.supports(&WorkflowCommand::output("k", "v")));
    }

    #[test]
    fn test_gitlab_default_runner() {
        let provider = test_provider();
        let runner = provider.default_runner();
        assert_eq!(runner.id(), "saas-linux-small-amd64");
    }

    #[test]
    fn test_gitlab_render_ci() {
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
            examples: Vec::new(),
            log_detail: None,
        });
        dag.add_node(Node {
            id: "test".into(),
            inputs: vec![port("build_success", "Bool")],
            outputs: vec![port("success", "Bool")],
            body: NodeBody::Opaque(DummyOp),
            examples: Vec::new(),
            log_detail: None,
        });
        dag.add_edge(edge("build", "success", "test", "build_success"));

        let provider = test_provider();
        let tool = CargoInvocation::composed("ci", "dag");
        let cargo_env = crate::cargo::CargoEnv {
            term_color: crate::cargo::TermColor::Always,
            warnings: crate::cargo::Warnings::Default,
        };
        let config = RenderConfig::new("ci", tool)
            .with_runner(crate::transport::github_actions::ubuntu_latest())
            .with_cargo_env(cargo_env);

        let yaml = provider.render(&dag, &config);

        let ci_name = crate::cargo::name("ci");
        // Check structure
        assert!(yaml.contains("image: ubuntu-latest"));
        assert!(yaml.contains("stages:"));
        assert!(yaml.contains(&format!("{ci_name} step build")));
        assert!(yaml.contains(&format!("{ci_name} step test")));
        assert!(yaml.contains("needs:"));
        assert!(yaml.contains("CARGO_TERM_COLOR: \"always\""));
    }

    #[test]
    fn test_gitlab_output_path() {
        let provider = test_provider();
        assert_eq!(provider.output_path("ci"), ".gitlab-ci.yml");
    }
}
