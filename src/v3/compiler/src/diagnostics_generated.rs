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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceByteSpan {
    pub byte_start: u32,
    pub byte_end: u32,
}

impl SourceByteSpan {
    pub const fn new(byte_start: u32, byte_end: u32) -> Self {
        Self {
            byte_start,
            byte_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionWitness {
    pub description: String,
    pub span: SourceSpan,
    pub new_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetirementPlan {
    pub owner: String,
    pub exit_condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Correction {
    LiveCorrection { witness: CorrectionWitness },
    DeferredCorrection {
        reason: String,
        retirement_plan: RetirementPlan,
    },
}
