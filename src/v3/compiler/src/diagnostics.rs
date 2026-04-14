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
//     producing_node: Option<NodeId>,
//   }
//
// TRIGGER for dissolution: when typed references (TypeRef, FieldRef,
// FunctionRef beyond M0's symbolic string form) exist in the
// substrate and can be used as Subject carriers. That's M1+ work
// after std/ declarations land.
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

use crate::dag::PortId;
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

#[derive(Debug, Clone)]
pub enum Diagnostic {
    TokenizerError {
        message: String,
        span: SourceSpan,
    },
    ParseError {
        message: String,
        span: SourceSpan,
    },
    TypeMismatch {
        expected: TypeShape,
        actual: TypeShape,
        span: SourceSpan,
    },
    ArityMismatch {
        function: String,
        expected: usize,
        actual: usize,
        span: SourceSpan,
    },
    ResolveError {
        name: String,
        span: SourceSpan,
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

    /// Insert a diagnostic entry for a port. pub(crate) because the
    /// only callers are Dag::mark_unresolved (which atomically also
    /// clears the port's value_type) and should never be called
    /// independently from outside the crate.
    pub(crate) fn insert(&mut self, port: PortId, diagnostic: Diagnostic) {
        self.entries.insert(port, diagnostic);
    }
}
