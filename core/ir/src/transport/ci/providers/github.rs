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

use crate::transport::ci::command::{AnnotationLevel, WorkflowCommand};
use crate::transport::ci::provider::CiProvider;
use crate::transport::ci::runner::Runner;
use crate::transport::github_actions::{
    ubuntu_22_04, ubuntu_24_04, ubuntu_latest, RunnerImage,
};

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

#[cfg(test)]
mod tests {
    use super::*;
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
}
