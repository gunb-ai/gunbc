// AUTO-GENERATED from `src/v3/std/substrate.dag, src/v3/std/diagnostics.dag`.
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
