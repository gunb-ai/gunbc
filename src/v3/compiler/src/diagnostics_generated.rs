// AUTO-GENERATED from `src/v3/compiler/runtime_mirrors.dag`.
// Regenerate instead of hand-editing.

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
    BranchConditionNotBool {
        port: PortId,
        actual_type: Option<TypeShape>,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStyleTarget {
    Rust,
    Go,
    Python,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticRenderError {
    MissingCleanEmissionContract(&'static str),
    MalformedCleanEmissionContract {
        declaration: DeclarationId,
        detail: &'static str,
    },
    MissingCorrectionStyle(&'static str),
    MalformedCorrectionStyle {
        declaration: DeclarationId,
        detail: &'static str,
    },
}
