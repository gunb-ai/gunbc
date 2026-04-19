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
        span: CompilerSourceSpan,
        fixes: Vec<CompilerCorrection>,
    },
    ParseError {
        message: String,
        span: CompilerSourceSpan,
        fixes: Vec<CompilerCorrection>,
    },
    TypeMismatch {
        expected: TypeShape,
        actual: TypeShape,
        span: CompilerSourceSpan,
        fixes: Vec<CompilerCorrection>,
    },
    ArityMismatch {
        function: String,
        expected: usize,
        actual: usize,
        span: CompilerSourceSpan,
        fixes: Vec<CompilerCorrection>,
    },
    ResolveError {
        name: String,
        span: CompilerSourceSpan,
        fixes: Vec<CompilerCorrection>,
    },
    BranchConditionNotBool {
        port: PortId,
        actual_type: Option<TypeShape>,
        span: CompilerSourceSpan,
        fixes: Vec<CompilerCorrection>,
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
