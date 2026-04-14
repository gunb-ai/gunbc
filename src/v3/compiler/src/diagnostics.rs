// Diagnostics and the enforced mark_unresolved API.
//
// The load-bearing guarantee:
//   Port.value_type == None  iff  DiagnosticTable contains PortId
//
// Enforcement is structural: Dag::clear_port_type is pub(crate) and
// is ONLY called from DiagnosticTable::mark_unresolved (which also
// writes the diagnostic entry). No other call path can null a port.
//
// G2 guardrail: SourceSpan always carries a non-empty `file` field.
// Cross-artifact diagnostics (M2+ multi-file, multi-language) rely
// on spans being addressable across files. Do NOT replace this with
// a bare (line, col) pair — that forecloses the thesis goal.

use std::collections::HashMap;

use crate::dag::{Dag, PortId};
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
}

impl Diagnostic {
    pub fn span(&self) -> &SourceSpan {
        match self {
            Diagnostic::TokenizerError { span, .. }
            | Diagnostic::ParseError { span, .. }
            | Diagnostic::TypeMismatch { span, .. } => span,
        }
    }
}

#[derive(Debug, Default)]
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

    /// The ONLY code path that sets Port.value_type to None.
    /// Atomically: writes the diagnostic entry AND nulls the
    /// port's type. Linked-by-construction invariant enforced.
    ///
    /// Callers must pass &mut Dag so that the null and the write
    /// happen in one call — there is no way to write the diagnostic
    /// without nulling the port, or vice versa.
    pub fn mark_unresolved(&mut self, dag: &mut Dag, port: PortId, diagnostic: Diagnostic) {
        dag.clear_port_type(port);
        self.entries.insert(port, diagnostic);
    }
}
