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
//! The L4b extraction has since grown the native Stage 2 bridge for
//! `Operation`-based effect classification; see `operation_effect_shape` for
//! the live transitional contract and dissolution receipt.

use std::collections::BTreeMap;

use super::{BoolPortRef, Dag, DeclarationId, ElementRef, NodeId, NonSingletonList};

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

/// 🟢 **TERMINAL.** Path token used by `Operation.endpoint.path`; mirrors the
/// path-template authority imported by `services.dag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlPathToken {
    LiteralToken { text: String },
    ParamToken { name: String },
}

/// 🟢 **TERMINAL.** REST path template carried by `Operation.endpoint`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PathTemplate {
    pub tokens: Vec<UrlPathToken>,
}

/// 🟢 **TERMINAL.** Per-input metadata slot keyed by `Operation.inputs`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputField {}

/// 🟡 **TRANSITIONAL.** Native mirror of `services.dag::CallableRef`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableRef {
    pub decl: DeclarationId,
}

/// 🟢 **TERMINAL.** Native mirror of `services.dag::RestEndpointBinding`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestEndpointBinding {
    pub method: HttpMethodScalar,
    pub path: PathTemplate,
}

/// 🟢 **TERMINAL.** Canonical service operation row.
///
/// This intentionally has no authored effect-shape field. Stage 2 derives the
/// effect partition from `endpoint` until callable inhabitance facts are
/// executable through the bootstrapped evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub callable: CallableRef,
    pub inputs: BTreeMap<String, InputField>,
    pub endpoint: RestEndpointBinding,
}

/// 🟢 **TERMINAL at current Stage 2b scope.** Result of linear
/// `compose_effects` — mirrors `CompositionVerdict` in `effects.dag`.
/// `ElementRef<Operation>` closes the "copied standalone breaker
/// record" hole by replacing the copied payload with a validated index,
/// but it does not by itself preserve the owner list identity or prove
/// the pointed operation is breaking. Those facts are still established
/// by [`compose_operation_effects`] and by callers resolving against the
/// matching workflow evidence chain, and are tracked as the same
/// constructor-validation asymmetry class as other reflected handles until
/// the substrate grows an owner-bound, breaking-only witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionVerdict {
    IdempotentComposition,
    BrokenBy {
        first_breaker: ElementRef<Operation>,
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
        ops: Vec<Operation>,
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
    pub fn operation_at(&self, element: ElementRef<Operation>) -> Option<&Operation> {
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

// ── Lane 2 Stage 2b — native `Dag` substrate realization (std.effects shape) ──
//
// Authoritative algebra is **authored** in `src/v3/std/effects.dag`
// (`compose_effects`, `lane2_workflow_idempotency_report`, …).
//
// **R3 gate #4 receipt (codex / inline review 2026-05-11):** the §Acceptance
// clause “crate API consumes **emitted/evaluated** `std.effects` as sole
// authority” is **not** satisfied by this Rust alone: in the bootstrapped
// compiler `Dag`, `lane2_workflow_idempotency_report` is still
// `ArrowBody::Unparsed` (see `lane2_stage_2b_db18_test` ratchet below / snapshot
// `Declaration` for that name). Until that body lowers and a host bridge calls
// through `evaluate_body` (or emitted Rust is `include!`d as the sole
// implementation), the compiler keeps one **native** projection here,
// co-located with the structural carriers — **P5** dissolution receipt for the
// **retired parallel module** `workflow_idempotency.rs` (single **P2**
// co-located realization vs a second hand-authored module), not a claim of full
// evaluator-backed dissolution.

pub fn operation_effect_shape(dag: &Dag, effect: &Operation) -> EffectShape {
    let callable = effect.callable.decl;
    let Some(methods) = StdEffectMethodAnchors::resolve(dag) else {
        return transport_effect_shape(effect);
    };
    if callable == methods.append && effect.endpoint.method == HttpMethodScalar::Post {
        return EffectShape::IsBreaking(BreakingShape::AppendEffect);
    }
    if callable == methods.concat && effect.endpoint.method == HttpMethodScalar::Post {
        return EffectShape::IsBreaking(BreakingShape::CreateEffect {
            cause: CreateCause::PostAlways,
        });
    }
    if methods.reads.contains(&callable) && method_is_read(effect.endpoint.method) {
        return EffectShape::IsIdempotent(IdempotentShape::ReadEffect);
    }
    if methods.upserts.contains(&callable) && method_is_upsert(effect.endpoint.method) {
        return keyed_upsert_or_keyless_break(effect);
    }
    if callable == methods.delete && effect.endpoint.method == HttpMethodScalar::Delete {
        return keyed_delete_or_keyless_break(effect);
    }
    transport_effect_shape(effect)
}

struct StdEffectMethodAnchors {
    append: DeclarationId,
    concat: DeclarationId,
    reads: [DeclarationId; 7],
    upserts: [DeclarationId; 3],
    delete: DeclarationId,
}

impl StdEffectMethodAnchors {
    fn resolve(dag: &Dag) -> Option<Self> {
        Some(Self {
            append: unique_decl_id(dag, "append_method")?,
            concat: unique_decl_id(dag, "concat_method")?,
            reads: [
                unique_decl_id(dag, "get_method")?,
                unique_decl_id(dag, "lookup_method")?,
                unique_decl_id(dag, "map_get_method")?,
                unique_decl_id(dag, "has_method")?,
                unique_decl_id(dag, "map_has_method")?,
                unique_decl_id(dag, "count_method")?,
                unique_decl_id(dag, "length_method")?,
            ],
            upserts: [
                unique_decl_id(dag, "map_insert_method")?,
                unique_decl_id(dag, "replace_method")?,
                unique_decl_id(dag, "with_method")?,
            ],
            delete: unique_decl_id(dag, "diff_method")?,
        })
    }
}

fn unique_decl_id(dag: &Dag, name: &str) -> Option<DeclarationId> {
    let mut matches = dag
        .declarations()
        .iter()
        .filter(|decl| decl.name.as_deref() == Some(name))
        .map(|decl| decl.id);
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first)
}

fn transport_effect_shape(effect: &Operation) -> EffectShape {
    match effect.endpoint.method {
        HttpMethodScalar::Get | HttpMethodScalar::Head | HttpMethodScalar::Options => {
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect)
        }
        HttpMethodScalar::Put | HttpMethodScalar::Patch => keyed_upsert_or_keyless_break(effect),
        HttpMethodScalar::Delete => keyed_delete_or_keyless_break(effect),
        HttpMethodScalar::Post => EffectShape::IsBreaking(BreakingShape::CreateEffect {
            cause: CreateCause::PostAlways,
        }),
    }
}

fn keyed_upsert_or_keyless_break(effect: &Operation) -> EffectShape {
    match operation_resource_key(effect) {
        Some(param) => EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
            key_source: KeySource::PathParam { param },
        }),
        None => keyless_break(effect),
    }
}

fn keyed_delete_or_keyless_break(effect: &Operation) -> EffectShape {
    match operation_resource_key(effect) {
        Some(param) => EffectShape::IsIdempotent(IdempotentShape::DeleteEffect {
            key_source: KeySource::PathParam { param },
        }),
        None => keyless_break(effect),
    }
}

fn keyless_break(effect: &Operation) -> EffectShape {
    EffectShape::IsBreaking(BreakingShape::CreateEffect {
        cause: CreateCause::KeylessFallback {
            method: effect.endpoint.method,
        },
    })
}

fn method_is_read(method: HttpMethodScalar) -> bool {
    matches!(
        method,
        HttpMethodScalar::Get | HttpMethodScalar::Head | HttpMethodScalar::Options
    )
}

fn method_is_upsert(method: HttpMethodScalar) -> bool {
    matches!(method, HttpMethodScalar::Put | HttpMethodScalar::Patch)
}

fn operation_resource_key(effect: &Operation) -> Option<String> {
    last_path_param(effect)
}

fn last_path_param(effect: &Operation) -> Option<String> {
    effect.endpoint.path.tokens.iter().rev().find_map(|token| {
        let UrlPathToken::ParamToken { name } = token else {
            return None;
        };
        effect.inputs.contains_key(name).then(|| name.clone())
    })
}

pub(crate) fn compose_operation_effects(dag: &Dag, effects: &[Operation]) -> CompositionVerdict {
    for (index, effect) in effects.iter().enumerate() {
        if matches!(
            operation_effect_shape(dag, effect),
            EffectShape::IsBreaking(_)
        ) {
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

/// Native-Dag entry for the `std.effects::lane2_workflow_idempotency_report`
/// algebra (same cases as the `.dag` `match`).
pub fn lane2_workflow_idempotency_report(
    dag: &Dag,
    workflow: &WorkflowEffect,
) -> WorkflowIdempotencyReport {
    project_workflow_idempotency_report(dag, workflow)
}

pub(crate) fn project_workflow_idempotency_report(
    dag: &Dag,
    workflow: &WorkflowEffect,
) -> WorkflowIdempotencyReport {
    match workflow {
        WorkflowEffect::LinearEffect { ops } => WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            compose_operation_effects(dag, ops.as_slice()),
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
/// The `.dag` entry reads the reflected substrate with `lane2_workflow_at`;
/// this Rust entry reads the same projection from the native `Dag` fields used
/// before emission.
pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    let Some(workflow) = d.lane2_workflow_effect_at(&workflow_root) else {
        return WorkflowIdempotencyReport::IdempotencyUnsupported(IdempotencyUnsupportedDetail {
            variant_name: "Lane2WorkflowRoot".to_string(),
            downstream_stage: "lane2_stage2b_idempotency_lens".to_string(),
            reason: "no WorkflowEffect at this substrate root - populate `lane2_workflow` on `Value`/`Bind` via lowering or `try_register_lane2_workflow_effect`"
                .to_string(),
        });
    };
    project_workflow_idempotency_report(d, workflow)
}
