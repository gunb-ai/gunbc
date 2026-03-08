//! Plain text CI provider for local development.
//!
//! This provider produces simple, human-readable output without any
//! CI-specific magic strings. Used as the fallback when no CI environment
//! is detected.

use crate::transport::ci::command::{AnnotationLevel, WorkflowCommand};
use crate::transport::ci::provider::CiProvider;
use crate::transport::ci::runner::Runner;

/// Plain text CI provider for local development.
///
/// Produces simple, human-readable output without CI magic strings.
/// Used as the fallback when not running in any CI environment.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainTextProvider;

impl CiProvider for PlainTextProvider {
    fn id(&self) -> &'static str {
        "plain"
    }

    fn name(&self) -> &'static str {
        "Plain Text (Local)"
    }

    fn format(&self, cmd: &WorkflowCommand) -> String {
        match cmd {
            WorkflowCommand::GroupStart { name, collapsed } => {
                let marker = if *collapsed { "[collapsed] " } else { "" };
                format!("=== {}{} ===", marker, name)
            }

            WorkflowCommand::GroupEnd { name } => {
                format!("=== /{} ===", name)
            }

            WorkflowCommand::Annotation {
                level,
                message,
                title,
                location,
            } => {
                let prefix = match level {
                    AnnotationLevel::Error => "[ERROR]",
                    AnnotationLevel::Warning => "[WARNING]",
                    AnnotationLevel::Notice => "[NOTICE]",
                    AnnotationLevel::Debug => "[DEBUG]",
                };

                let title_part = title
                    .as_ref()
                    .map(|t| format!(" {}: ", t))
                    .unwrap_or_else(|| " ".to_string());

                let loc_part = location
                    .as_ref()
                    .map(|l| {
                        let line = l.line.map(|n| format!(":{}", n)).unwrap_or_default();
                        format!(" ({}{})", l.file, line)
                    })
                    .unwrap_or_default();

                format!("{}{}{}{}", prefix, loc_part, title_part, message)
            }

            WorkflowCommand::SetOutput { key, value } => {
                format!("Output: {}={}", key, value)
            }

            WorkflowCommand::MaskValue { .. } => "[masked value]".to_string(),

            WorkflowCommand::Summary { markdown } => {
                format!("--- Summary ---\n{}\n---------------", markdown)
            }
        }
    }

    fn supports(&self, cmd: &WorkflowCommand) -> bool {
        // Plain text supports everything as fallback
        let _ = cmd;
        true
    }

    fn runners(&self) -> Vec<Box<dyn Runner>> {
        vec![Box::new(LocalRunner)]
    }

    fn default_runner(&self) -> Box<dyn Runner> {
        Box::new(LocalRunner)
    }
}

/// Local development "runner" - represents the local machine.
#[derive(Debug, Clone, Copy)]
struct LocalRunner;

impl Runner for LocalRunner {
    fn id(&self) -> &str {
        "local"
    }

    fn name(&self) -> &str {
        "Local Development Environment"
    }

    fn tools(&self) -> &[&str] {
        // Local environment has whatever is installed
        // We return an empty list since we can't know
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::ci::command::FileLocation;

    #[test]
    fn test_plain_group_start() {
        let provider = PlainTextProvider;
        let cmd = WorkflowCommand::group_start("test");
        assert_eq!(provider.format(&cmd), "=== test ===");
    }

    #[test]
    fn test_plain_group_start_collapsed() {
        let provider = PlainTextProvider;
        let cmd = WorkflowCommand::group_start_collapsed("test");
        assert_eq!(provider.format(&cmd), "=== [collapsed] test ===");
    }

    #[test]
    fn test_plain_group_end() {
        let provider = PlainTextProvider;
        let cmd = WorkflowCommand::group_end("test");
        assert_eq!(provider.format(&cmd), "=== /test ===");
    }

    #[test]
    fn test_plain_error() {
        let provider = PlainTextProvider;
        let cmd = WorkflowCommand::error("something broke");
        assert_eq!(provider.format(&cmd), "[ERROR] something broke");
    }

    #[test]
    fn test_plain_error_with_location() {
        let provider = PlainTextProvider;
        let cmd = WorkflowCommand::annotation_at(
            AnnotationLevel::Error,
            "syntax error",
            FileLocation::new("src/main.rs").with_line(42),
        );
        assert_eq!(
            provider.format(&cmd),
            "[ERROR] (src/main.rs:42) syntax error"
        );
    }

    #[test]
    fn test_plain_output() {
        let provider = PlainTextProvider;
        let cmd = WorkflowCommand::output("result", "success");
        assert_eq!(provider.format(&cmd), "Output: result=success");
    }
}
