//! `std.effects` substrate carriers + Stage 2b native-`Dag` projection (DB-18).
//!
//! Structural carriers align with `src/v3/std/effects.dag`. Executable Stage 2b
//! helpers (`analyze_workflow`, [`lane2_workflow_idempotency_report`], …) live
//! here as the typed Rust witness for unparsed std arrow bodies in bootstrap
//! (`ArrowBody::Unparsed`) — sole compiler-local implementation surface for this algebra
//! (R3 gate `tier3_effect_carrier_mirror_dissolved`: retired standalone
//! `workflow_idempotency.rs`).
//!
//! Each coproduct / boundary carrier below carries its own 🟢/🟡
//! dissolution stamp (modeling-discipline principle 4); do not rely on
//! the module banner alone. 🔴 does not appear in this block — there is
//! no intentionally-wrong deferred carrier here; unsupported control
//! flow is modeled via explicit sums, not silent placeholders.

use super::{BoolPortRef, Dag, ElementRef, NodeId, NonSingletonList};

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
/// by [`compose_operation_effects`] and by callers
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

// ── Stage 2b projection (native `Dag` / rustc round-trip exports) ───────────

pub(crate) fn compose_operation_effects(effects: &[OperationEffect]) -> CompositionVerdict {
    for (index, effect) in effects.iter().enumerate() {
        if matches!(effect.shape, EffectShape::IsBreaking(_)) {
            // `ElementRef` preserves the validated in-bounds position of the
            // breaker without copying a second breaker record; the breaking
            // subset fact and the owner-list identity still come from this
            // partition check plus callers resolving against the same slice.
            let first_breaker = ElementRef::from_slice(effects, index)
                .expect("enumerated workflow effect index must stay in-bounds");
            return CompositionVerdict::BrokenBy { first_breaker };
        }
    }
    CompositionVerdict::IdempotentComposition
}

/// Pure projection used by Stage 2b — kept aligned with
/// `std.effects::report_unsupported_workflow_variant`. Exported for
/// `emit_rust_module` output from `src/v3/lenses/idempotency.dag` (rustc
/// round-trip in `m2_lens_idempotency_migration_test`).
pub fn report_unsupported_workflow_variant(
    variant_name: &str,
    downstream_stage: &str,
    reason: &str,
) -> WorkflowIdempotencyReport {
    WorkflowIdempotencyReport::IdempotencyUnsupported(IdempotencyUnsupportedDetail {
        variant_name: variant_name.to_string(),
        downstream_stage: downstream_stage.to_string(),
        reason: reason.to_string(),
    })
}

/// Keeps parity with `std.effects::lane2_workflow_idempotency_report`.
pub fn lane2_workflow_idempotency_report(workflow: &WorkflowEffect) -> WorkflowIdempotencyReport {
    project_workflow_idempotency_report(workflow)
}

pub(crate) fn project_workflow_idempotency_report(
    workflow: &WorkflowEffect,
) -> WorkflowIdempotencyReport {
    match workflow {
        WorkflowEffect::LinearEffect { ops } => WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            compose_operation_effects(ops.as_slice()),
        ),
        WorkflowEffect::BranchEffect { .. } => WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "BranchEffect".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "non-linear workflow; branch-wise idempotency composition is not in the Stage 2b algebra"
                    .to_string(),
            },
        ),
        WorkflowEffect::LoopEffect { .. } => WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "LoopEffect".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "non-linear workflow; loop-carried idempotency composition is not in the Stage 2b algebra"
                    .to_string(),
            },
        ),
        WorkflowEffect::ParallelEffect { .. } => WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "ParallelEffect".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "non-linear workflow; parallel idempotency composition is not in the Stage 2b algebra"
                    .to_string(),
            },
        ),
    }
}

/// Native-Dag convenience entry for the declared `lenses.idempotency` surface.
///
/// The `.dag` entry reads the reflected substrate with `lane2_workflow_at`; this
/// Rust entry reads the same projection from the native `Dag` fields used before
/// emission.
pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    let Some(workflow) = d.lane2_workflow_effect_at(&workflow_root) else {
        return WorkflowIdempotencyReport::IdempotencyUnsupported(
            IdempotencyUnsupportedDetail {
                variant_name: "Lane2WorkflowRoot".to_string(),
                downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
                reason: "no WorkflowEffect at this substrate root - populate `lane2_workflow` on `Value`/`Bind` via lowering or `try_register_lane2_workflow_effect`"
                    .to_string(),
            },
        );
    };
    project_workflow_idempotency_report(workflow)
}
