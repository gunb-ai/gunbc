use crate::span::Span;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    Lex,
    Parse,
    Resolve,
    Pipeline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub file: Option<PathBuf>,
    pub span: Option<Span>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            file: None,
            span: None,
            line: None,
            column: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_file(mut self, file: impl AsRef<Path>) -> Self {
        self.file = Some(file.as_ref().to_path_buf());
        self
    }

    pub fn with_line_col(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn render(&self) -> String {
        match (&self.file, self.line, self.column) {
            (Some(file), Some(line), Some(column)) => {
                format!("{}:{line}:{column}: {}", file.display(), self.message)
            }
            (Some(file), _, _) => format!("{}: {}", file.display(), self.message),
            _ => self.message.clone(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Sort diagnostics by kind → file → line → column → message and deduplicate.
pub fn normalize_diagnostics(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort_by_key(|diag| {
        (
            diagnostic_kind_rank(&diag.kind),
            diag.file
                .as_ref()
                .map(|file| file.display().to_string())
                .unwrap_or_default(),
            diag.line.unwrap_or_default(),
            diag.column.unwrap_or_default(),
            diag.message.clone(),
        )
    });
    diagnostics.dedup_by(|a, b| {
        a.kind == b.kind
            && a.file == b.file
            && a.line == b.line
            && a.column == b.column
            && a.message == b.message
    });
    diagnostics
}

fn diagnostic_kind_rank(kind: &DiagnosticKind) -> u8 {
    match kind {
        DiagnosticKind::Lex => 0,
        DiagnosticKind::Parse => 1,
        DiagnosticKind::Resolve => 2,
        DiagnosticKind::Pipeline => 3,
    }
}
