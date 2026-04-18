// Diagnostics and the enforced mark_unresolved API.
//
// The load-bearing guarantee:
//   Port.value_type == None  iff  DiagnosticTable contains PortId
//
// Enforcement is structural: Dag::clear_port_type is pub(crate) and
// is ONLY called from Dag::mark_unresolved (which also writes the
// diagnostic entry atomically). No other call path can null a port.
//
// G2 guardrail: SourceSpan always carries a non-empty `file` field.
// Cross-artifact diagnostics (M2+ multi-file, multi-language) rely
// on spans being addressable across files. Do NOT replace this with
// a bare (line, col) pair — that forecloses the thesis goal.
//
// ────────────────────────────────────────────────────────────────
// DEFERRED DISSOLUTION: Diagnostic enum is a scaffold.
//
// Current variants (M0.5): TokenizerError, ParseError, TypeMismatch,
// ArityMismatch, ResolveError. The v3 target shape per
// docs/v3-modeling-analysis.md §CompilerDiagnostic is a 5-field
// record:
//
//   Diagnostic {
//     span: SourceSpan,
//     category: DiagnosticCategory,  // Type | Cardinality | ...
//     subject: DiagnosticSubject,    // typed ref to thing flagged
//     detail: DiagnosticDetail,      // category-specific payload
//     correction: Option<Correction>,// the literal fixed code
//     producing_node: Option<NodeId>,
//   }
//
// The `correction` field is the thesis claim "show the correct code"
// (THESIS.md §"Error handling"). Every diagnostic that can structurally
// compute a fix must carry one. Design: SELF_HOSTING.md §14.6.
// Roundtrip test pattern: break → diagnose → apply correction →
// recompile → zero diagnostics. Lands at L1.5.
//
// TRIGGER for dissolution: when typed DeclarationId / FieldRef references
// replace the name + span carriers below. M1(2.5) delivered the
// DeclarationId substrate; the subject/detail refactor is next.
//
// Extension from 3→5 variants at M0.5 is deferred dissolution with
// justification: the fail-closed invariant (C-8) requires that every
// detectable failure produces a diagnostic, and M0's infer + lower
// stages reach failure categories (arity, resolve) that TypeMismatch
// does not cover. Until Subject/Detail infrastructure exists,
// extension is the least-bad interim form.
//
// DO NOT add a severity field. C-8: all diagnostics are errors.
// DO NOT add a warning variant. DO NOT add a "maybe wrong" state.
// If a condition is detected and it's wrong, emit the diagnostic
// and fail. If it's not wrong, emit nothing. There is no middle.
// ────────────────────────────────────────────────────────────────

use std::collections::HashMap;

use crate::dag::{Dag, DeclarationId, FieldValue, PortId};
use crate::types::TypeShape;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: String,
    pub byte_start: u32,
    pub byte_end: u32,
}

impl SourceSpan {
    pub fn new(file: impl Into<String>, byte_start: u32, byte_end: u32) -> Self {
        Self {
            file: file.into(),
            byte_start,
            byte_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    pub description: String,
    pub span: SourceSpan,
    pub new_source: String,
}

#[derive(Debug, Clone)]
pub enum Diagnostic {
    TokenizerError {
        message: String,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    ParseError {
        message: String,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    TypeMismatch {
        expected: TypeShape,
        actual: TypeShape,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    ArityMismatch {
        function: String,
        expected: usize,
        actual: usize,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    ResolveError {
        name: String,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
}

impl Diagnostic {
    pub fn span(&self) -> &SourceSpan {
        match self {
            Diagnostic::TokenizerError { span, .. }
            | Diagnostic::ParseError { span, .. }
            | Diagnostic::TypeMismatch { span, .. }
            | Diagnostic::ArityMismatch { span, .. }
            | Diagnostic::ResolveError { span, .. } => span,
        }
    }

    pub fn fixes(&self) -> &[Correction] {
        match self {
            Diagnostic::TokenizerError { fixes, .. }
            | Diagnostic::ParseError { fixes, .. }
            | Diagnostic::TypeMismatch { fixes, .. }
            | Diagnostic::ArityMismatch { fixes, .. }
            | Diagnostic::ResolveError { fixes, .. } => fixes,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Diagnostic::TokenizerError { message, .. } | Diagnostic::ParseError { message, .. } => {
                message.clone()
            }
            Diagnostic::TypeMismatch {
                expected, actual, ..
            } => format!("expected {expected:?}, got {actual:?}"),
            Diagnostic::ArityMismatch {
                function,
                expected,
                actual,
                ..
            } => format!("{function} expected {expected}, got {actual}"),
            Diagnostic::ResolveError { name, .. } => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStyleTarget {
    Rust,
    Go,
    Python,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticRenderError {
    MissingCorrectionStyle(&'static str),
    MalformedCorrectionStyle {
        declaration: DeclarationId,
        detail: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorrectionStyleBinding {
    indent_unit: String,
    line_ending: String,
    string_quote: String,
    trailing_semicolon: bool,
}

impl CorrectionStyleBinding {
    fn build(dag: &Dag, declaration: DeclarationId) -> Result<Self, DiagnosticRenderError> {
        let fields = match dag.declaration(declaration).value_body.as_ref() {
            Some(crate::dag::ValueBody::Structural { fields }) => fields,
            _ => {
                return Err(DiagnosticRenderError::MalformedCorrectionStyle {
                    declaration,
                    detail: "correction_style declaration must carry a Structural value_body",
                });
            }
        };
        Ok(Self {
            indent_unit: require_string(fields, "indent_unit", declaration)?,
            line_ending: require_string(fields, "line_ending", declaration)?,
            string_quote: require_string(fields, "string_quote", declaration)?.replace("%Q", "\""),
            trailing_semicolon: require_bool(fields, "trailing_semicolon", declaration)?,
        })
    }
}

pub fn render_diagnostic_for_target(
    dag: &Dag,
    target: DiagnosticStyleTarget,
    diagnostic: &Diagnostic,
) -> Result<String, DiagnosticRenderError> {
    let declaration =
        match target {
            DiagnosticStyleTarget::Rust => dag.rust_correction_style_spec().ok_or(
                DiagnosticRenderError::MissingCorrectionStyle("rust_correction_style"),
            )?,
            DiagnosticStyleTarget::Go => dag.go_correction_style_spec().ok_or(
                DiagnosticRenderError::MissingCorrectionStyle("go_correction_style"),
            )?,
            DiagnosticStyleTarget::Python => dag.python_correction_style_spec().ok_or(
                DiagnosticRenderError::MissingCorrectionStyle("python_correction_style"),
            )?,
        };
    let style = CorrectionStyleBinding::build(dag, declaration)?;
    Ok(render_diagnostic_with_style(diagnostic, &style))
}

fn render_diagnostic_with_style(diagnostic: &Diagnostic, style: &CorrectionStyleBinding) -> String {
    let mut lines = vec![format!(
        "ERROR at {}:{}-{}: {}",
        diagnostic.span().file,
        diagnostic.span().byte_start,
        diagnostic.span().byte_end,
        diagnostic.message()
    )];
    for (index, fix) in diagnostic.fixes().iter().enumerate() {
        lines.push(format!("FIX (option {}): {}", index + 1, fix.description));
        lines.push(render_correction_source(fix, style));
    }
    lines.join(&style.line_ending)
}

fn render_correction_source(fix: &Correction, style: &CorrectionStyleBinding) -> String {
    let suffix = if style.trailing_semicolon { ";" } else { "" };
    let normalized = fix.new_source.replace("\r\n", "\n").replace('\r', "\n");
    if !normalized.contains('\n') {
        return format!(
            "{}{}{}{}{}",
            style.indent_unit, style.string_quote, normalized, style.string_quote, suffix
        );
    }

    let body = normalized
        .split('\n')
        .map(|line| format!("{}{}", style.indent_unit, line))
        .collect::<Vec<_>>()
        .join(&style.line_ending);
    format!(
        "{}{}{}{}",
        style.string_quote, style.line_ending, body, style.string_quote
    ) + suffix
}

fn require_string(
    fields: &[(String, FieldValue)],
    label: &'static str,
    declaration: DeclarationId,
) -> Result<String, DiagnosticRenderError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style is missing a required String field",
        })?;
    match value {
        FieldValue::Literal(crate::dag::LiteralBits::String(value)) => Ok(value.clone()),
        _ => Err(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style String field must be a string literal",
        }),
    }
}

fn require_bool(
    fields: &[(String, FieldValue)],
    label: &'static str,
    declaration: DeclarationId,
) -> Result<bool, DiagnosticRenderError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style is missing a required Bool field",
        })?;
    match value {
        FieldValue::Literal(crate::dag::LiteralBits::Bool(value)) => Ok(*value),
        _ => Err(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style Bool field must be a bool literal",
        }),
    }
}

#[derive(Debug, Default, Clone)]
pub struct DiagnosticTable {
    entries: HashMap<PortId, Diagnostic>,
}

impl DiagnosticTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, port: PortId) -> bool {
        self.entries.contains_key(&port)
    }

    pub fn get(&self, port: PortId) -> Option<&Diagnostic> {
        self.entries.get(&port)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterate `(port, diagnostic)` pairs. Used by tests and callers
    /// that need to scan all diagnostics for a specific kind.
    pub fn iter(&self) -> impl Iterator<Item = (PortId, &Diagnostic)> {
        self.entries.iter().map(|(p, d)| (*p, d))
    }

    /// Insert a diagnostic entry for a port. pub(crate) because the
    /// only callers are Dag::mark_unresolved (which atomically also
    /// clears the port's value_type) and should never be called
    /// independently from outside the crate.
    pub(crate) fn insert(&mut self, port: PortId, diagnostic: Diagnostic) {
        self.entries.insert(port, diagnostic);
    }
}
