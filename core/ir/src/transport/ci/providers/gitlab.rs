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
use crate::transport::ci::command::{AnnotationLevel, WorkflowCommand};
use crate::transport::ci::provider::CiProvider;
use crate::transport::ci::runner::{
    gitlab_saas_linux_large, gitlab_saas_linux_medium, gitlab_saas_linux_small, Runner,
};
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
