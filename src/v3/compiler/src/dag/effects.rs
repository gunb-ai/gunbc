//! `std.effects` mirror (DB-18 / Lane 2 Stage 2b).
//!
//! Structural carriers aligned with `src/v3/std/effects.dag` — the
//! compiler-side authority for `compose_effects`, [`WorkflowEffect`], and
//! [`BranchArm`] until the self-hosted pipeline consumes the `.dag` forms
//! directly.
//!
//! Each coproduct / boundary carrier below carries its own 🟢/🟡
//! dissolution stamp (modeling-discipline principle 4); do not rely on
//! the module banner alone. 🔴 does not appear in this block — there is
//! no intentionally-wrong deferred carrier here; unsupported control
//! flow is modeled via explicit sums, not silent placeholders.
//!
//! Extracted from `dag.rs` (L4b). No behavior change.

use super::{BoolPortRef, ElementRef, NonSingletonList};

/// 🟢 **TERMINAL.** HTTP verb literals — 1:1 with `std.effects` `HttpMethod`;
/// naming authority is `effects.dag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethodScalar {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// 🟢 **TERMINAL.** Where a stable idempotency key comes from — mirrors
/// `KeySource` in `effects.dag`; no parallel spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    PathParam { param: String },
    InputField { field: String },
    CompositeKey { fields: Vec<String> },
}

/// 🟢 **TERMINAL.** Why a create-shaped op is classified breaking — mirrors
/// `CreateCause` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateCause {
    PostAlways,
    KeylessFallback { method: HttpMethodScalar },
}

/// 🟢 **TERMINAL.** Idempotent-side effect shapes — mirrors `IdempotentShape`
/// in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotentShape {
    ReadEffect,
    UpsertEffect { key_source: KeySource },
    DeleteEffect { key_source: KeySource },
}

/// 🟢 **TERMINAL.** Breaking-side effect shapes — mirrors `BreakingShape` in
/// `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakingShape {
    CreateEffect { cause: CreateCause },
    AppendEffect,
}

/// 🟢 **TERMINAL.** Classified per-op shape — sum of idempotent vs breaking
/// carriers; mirrors `EffectShape` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectShape {
    IsIdempotent(IdempotentShape),
    IsBreaking(BreakingShape),
}

/// 🟢 **TERMINAL.** Named operation plus classified shape — mirrors the
/// `OperationEffect` record in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationEffect {
    pub operation_name: String,
    pub shape: EffectShape,
}

/// 🟢 **TERMINAL at current Stage 2b scope.** Result of linear
/// `compose_effects` — mirrors `CompositionVerdict` in `effects.dag`.
/// `ElementRef<OperationEffect>` closes the "copied standalone breaker
/// record" hole by replacing the copied payload with a validated index,
/// but it does not by itself preserve the owner list identity or prove
/// the pointed operation is breaking. Those facts are still established
/// by `workflow_idempotency::compose_operation_effects` and by callers
/// resolving against the matching workflow evidence chain, and are
/// tracked as the same constructor-validation asymmetry class as other
/// reflected handles until the substrate grows an owner-bound,
/// breaking-only witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionVerdict {
    IdempotentComposition,
    BrokenBy {
        first_breaker: ElementRef<OperationEffect>,
    },
}

/// 🟢 **TERMINAL.** Branch arm: [`BoolPortRef`] condition + nested workflow body.
/// Construct with [`BranchArm::new`] once [`super::Dag::bool_port_of`] has validated the
/// predicate port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchArm {
    condition: BoolPortRef,
    body: Box<WorkflowEffect>,
}

/// 🟢 **TERMINAL.** Four-variant workflow sum aligned with `effects.dag`;
/// Stage 2b analyzes `LinearEffect` only — non-linear variants surface
/// `IdempotencyUnsupported` until branch-wise algebra lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowEffect {
    LinearEffect {
        ops: Vec<OperationEffect>,
    },
    BranchEffect {
        arms: NonSingletonList<BranchArm>,
    },
    LoopEffect {
        body: Box<WorkflowEffect>,
    },
    ParallelEffect {
        branches: NonSingletonList<Box<WorkflowEffect>>,
    },
}

impl BranchArm {
    pub fn new(condition: BoolPortRef, body: WorkflowEffect) -> Self {
        Self {
            condition,
            body: Box::new(body),
        }
    }

    pub fn bool_port(&self) -> BoolPortRef {
        self.condition
    }

    /// Back-compat alias for the condition field name used in early DB-18 tests.
    pub fn branch_predicate(&self) -> BoolPortRef {
        self.condition
    }

    pub fn body(&self) -> &WorkflowEffect {
        &self.body
    }
}

impl WorkflowEffect {
    pub fn operation_at(&self, element: ElementRef<OperationEffect>) -> Option<&OperationEffect> {
        match self {
            Self::LinearEffect { ops } => element.get(ops),
            Self::ParallelEffect { branches } => {
                let mut remaining = element.index_of();
                for branch in branches.iter() {
                    let Self::LinearEffect { ops } = branch.as_ref() else {
                        return None;
                    };
                    if let Some(op) = ops.get(remaining) {
                        return Some(op);
                    }
                    remaining = remaining.checked_sub(ops.len())?;
                }
                None
            }
            Self::BranchEffect { .. } | Self::LoopEffect { .. } => None,
        }
    }
}

/// 🟢 **TERMINAL.** Explicit unsupported payload — names variant + stage +
/// reason; not a silent `Option` alongside a verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyUnsupportedDetail {
    pub variant_name: String,
    pub downstream_stage: String,
    pub reason: String,
}

/// 🟢 **TERMINAL.** Stage 2b lens report sum — success path vs explicit
/// unsupported; mirrors `WorkflowIdempotencyReport` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowIdempotencyReport {
    WorkflowCompositionVerdict(CompositionVerdict),
    IdempotencyUnsupported(IdempotencyUnsupportedDetail),
}

/// 🟢 **TERMINAL.** Stage 2e parallel-lens unsupported classes — mirrors
/// `ParallelismUnsupportedKind` in `effects.dag` (distinct from Stage 2b
/// `IdempotencyUnsupportedDetail.variant_name`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelismUnsupportedKind {
    NoWorkflowProjection,
    NotParallelEffectRoot,
    NonLinearParallelBranch,
    PairwiseNonCommute,
    LensSurfacePending,
}

/// 🟢 **TERMINAL.** Parallelism lens explicit unsupported payload — mirrors
/// `ParallelismUnsupportedDetail` in `effects.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelismUnsupportedDetail {
    pub kind: ParallelismUnsupportedKind,
    pub downstream_stage: String,
    pub reason: String,
}

/// 🟢 **TERMINAL.** Lane 2 Stage 2e parallelism lens report — mirrors
/// `WorkflowParallelismReport` in `effects.dag` (DB-20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowParallelismReport {
    ParallelCompositionVerdict(CompositionVerdict),
    ParallelismUnsupported(ParallelismUnsupportedDetail),
}
