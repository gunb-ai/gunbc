//! CI workflow command types.
//!
//! This module defines typed representations of CI workflow commands that can
//! be formatted by different CI providers. The commands represent *what* to emit,
//! while providers implement *how* to format them.
//!
//! # Supported Commands
//!
//! - **Groups**: Collapsible log sections (`GroupStart`, `GroupEnd`)
//! - **Annotations**: Error/warning/notice/debug messages with optional file locations
//! - **Outputs**: Key-value pairs for passing data between steps
//! - **Masking**: Hide sensitive values in logs
//! - **Summaries**: Markdown content for job summaries
//!
//! # Example
//!
//! ```text
//! use gunbc_ir::transport::ci::{WorkflowCommand, AnnotationLevel};
//!
//! let cmd = WorkflowCommand::Annotation {
//!     level: AnnotationLevel::Error,
//!     message: "Missing semicolon".to_string(),
//!     title: Some("Syntax Error".to_string()),
//!     location: Some(FileLocation {
//!         file: "src/main.rs".to_string(),
//!         line: Some(42),
//!         ..Default::default()
//!     }),
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Annotation severity level.
///
/// Maps to different visual treatments in CI provider UIs:
/// - `Error`: Red, creates PR annotations, fails status checks
/// - `Warning`: Yellow, creates PR annotations
/// - `Notice`: Blue/cyan, informational annotations
/// - `Debug`: Gray, only shown when debug logging enabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationLevel {
    /// Debug message (only shown with debug logging enabled)
    Debug,
    /// Informational notice
    Notice,
    /// Warning (non-fatal issue)
    Warning,
    /// Error (fatal issue)
    Error,
}

impl AnnotationLevel {
    /// Get the string representation for this level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for AnnotationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// File location for annotations.
///
/// Used to link annotations to specific lines in source files,
/// enabling PR annotations and IDE integration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileLocation {
    /// File path (relative to repository root)
    pub file: String,
    /// Starting line number (1-indexed)
    pub line: Option<u32>,
    /// Ending line number (1-indexed)
    pub end_line: Option<u32>,
    /// Starting column number (1-indexed)
    pub col: Option<u32>,
    /// Ending column number (1-indexed)
    pub end_col: Option<u32>,
}

impl FileLocation {
    /// Create a new file location with just a file path.
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            ..Default::default()
        }
    }

    /// Set the line number.
    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the line range.
    pub fn with_lines(mut self, start: u32, end: u32) -> Self {
        self.line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Set the column number.
    pub fn with_col(mut self, col: u32) -> Self {
        self.col = Some(col);
        self
    }

    /// Set the column range.
    pub fn with_cols(mut self, start: u32, end: u32) -> Self {
        self.col = Some(start);
        self.end_col = Some(end);
        self
    }
}

/// CI workflow commands - typed representation.
///
/// These commands represent actions that can be performed in CI job output.
/// Each CI provider implements formatting for these commands according to
/// its own syntax and capabilities.
///
/// Providers that don't support a command can provide graceful fallbacks
/// (e.g., GitLab renders annotations as colored plain text).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowCommand {
    /// Start a collapsible log group.
    ///
    /// Groups create expandable/collapsible sections in CI job logs.
    /// Nested groups create hierarchical names (e.g., "build/test/unit").
    GroupStart {
        /// Group name (displayed as the section header)
        name: String,
        /// Whether to start collapsed (default: false)
        collapsed: bool,
    },

    /// End a collapsible log group.
    GroupEnd {
        /// Group name (must match the corresponding GroupStart)
        name: String,
    },

    /// Emit an annotation (error/warning/notice/debug).
    ///
    /// Annotations can be linked to specific file locations, creating
    /// inline comments in PRs and IDE integration.
    Annotation {
        /// Severity level
        level: AnnotationLevel,
        /// Message content
        message: String,
        /// Optional title (short summary)
        title: Option<String>,
        /// Optional file location
        location: Option<FileLocation>,
    },

    /// Set an output variable.
    ///
    /// Outputs are key-value pairs that can be used by subsequent steps
    /// or jobs in the workflow.
    SetOutput {
        /// Variable name
        key: String,
        /// Variable value
        value: String,
    },

    /// Mask a secret value.
    ///
    /// Masked values are replaced with `***` in logs to prevent
    /// accidental exposure of sensitive data.
    MaskValue {
        /// Value to mask
        value: String,
    },

    /// Write to job summary (markdown).
    ///
    /// Summaries are rendered as markdown on the workflow run page,
    /// providing rich formatted output for test results, coverage, etc.
    Summary {
        /// Markdown content
        markdown: String,
    },
}

impl WorkflowCommand {
    /// Create a group start command.
    pub fn group_start(name: impl Into<String>) -> Self {
        Self::GroupStart {
            name: name.into(),
            collapsed: false,
        }
    }

    /// Create a collapsed group start command.
    pub fn group_start_collapsed(name: impl Into<String>) -> Self {
        Self::GroupStart {
            name: name.into(),
            collapsed: true,
        }
    }

    /// Create a group end command.
    pub fn group_end(name: impl Into<String>) -> Self {
        Self::GroupEnd { name: name.into() }
    }

    /// Create an error annotation.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Annotation {
            level: AnnotationLevel::Error,
            message: message.into(),
            title: None,
            location: None,
        }
    }

    /// Create a warning annotation.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::Annotation {
            level: AnnotationLevel::Warning,
            message: message.into(),
            title: None,
            location: None,
        }
    }

    /// Create a notice annotation.
    pub fn notice(message: impl Into<String>) -> Self {
        Self::Annotation {
            level: AnnotationLevel::Notice,
            message: message.into(),
            title: None,
            location: None,
        }
    }

    /// Create a debug annotation.
    pub fn debug(message: impl Into<String>) -> Self {
        Self::Annotation {
            level: AnnotationLevel::Debug,
            message: message.into(),
            title: None,
            location: None,
        }
    }

    /// Create an annotation with a file location.
    pub fn annotation_at(
        level: AnnotationLevel,
        message: impl Into<String>,
        location: FileLocation,
    ) -> Self {
        Self::Annotation {
            level,
            message: message.into(),
            title: None,
            location: Some(location),
        }
    }

    /// Create an output command.
    pub fn output(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::SetOutput {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Create a mask command.
    pub fn mask(value: impl Into<String>) -> Self {
        Self::MaskValue {
            value: value.into(),
        }
    }

    /// Create a summary command.
    pub fn summary(markdown: impl Into<String>) -> Self {
        Self::Summary {
            markdown: markdown.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_level_as_str() {
        assert_eq!(AnnotationLevel::Error.as_str(), "error");
        assert_eq!(AnnotationLevel::Warning.as_str(), "warning");
        assert_eq!(AnnotationLevel::Notice.as_str(), "notice");
        assert_eq!(AnnotationLevel::Debug.as_str(), "debug");
    }

    #[test]
    fn test_file_location_builder() {
        let loc = FileLocation::new("src/main.rs").with_line(42).with_col(10);

        assert_eq!(loc.file, "src/main.rs");
        assert_eq!(loc.line, Some(42));
        assert_eq!(loc.col, Some(10));
    }

    #[test]
    fn test_workflow_command_constructors() {
        let group = WorkflowCommand::group_start("test");
        assert!(
            matches!(group, WorkflowCommand::GroupStart { name, collapsed } 
            if name == "test" && !collapsed)
        );

        let error = WorkflowCommand::error("oops");
        assert!(matches!(
            error,
            WorkflowCommand::Annotation {
                level: AnnotationLevel::Error,
                ..
            }
        ));
    }
}
