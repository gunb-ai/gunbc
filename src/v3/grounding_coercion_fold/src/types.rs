//! LanguageSpec projection + target inhabitance carriers for the structural fold.

use std::collections::BTreeMap;

use v3_compiler::dag::{DeclarationId, Interval};
use v3_grounding_lifetime::BindingId;

/// How the fold obtains LanguageSpec / target-primitive substrate facts.
///
/// **Scratch path retired (#1980):** integer rows are selected only through
/// `DeclaredIntegerIntents` over `TargetIntegerTypeInhabitance` facts — transitional
/// `ScratchIntExamples` is removed.
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW (residual scope)** — gated items below
/// are explicit until program-bound + lifecycle wiring fully owns the LanguageSpec boundary:
///
/// - **`Undeclared`**: fail-closed placeholder until a substrate-backed reader lands here (#1133 /
///   lifetime facts #1286).
/// - **`DeclaredIntegerIntents` + `IntegerTargetIntent`**: executable projection, but
///   `IntegerBoundProjection` remains a lane-local mirror of program bounds until lowering can
///   pass declared `BoundDeclaration` directly (#1133 / #1286).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LanguageSpecProjection {
    #[default]
    Undeclared,
    DeclaredIntegerIntents(BTreeMap<BindingId, IntegerTargetIntent>),
}

/// Program-side integer bound projected into the fold.
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — mirrors the declared
/// `BoundDeclaration` static/platform split only for the executable
/// LanguageSpec projection carrier; retires as a distinct public coproduct once
/// program-bound extraction can pass substrate bound declarations directly (#1133 / #1286).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerBoundProjection {
    Static(Interval<i64>),
    PlatformDependent,
}

/// Per-binding integer intent consumed against `TargetIntegerTypeInhabitance` rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerTargetIntent {
    pub target_language: DeclarationId,
    pub kernel_integer: DeclarationId,
    pub algebra: DeclarationId,
    pub bound: IntegerBoundProjection,
}

/// Selected integer target: identity from a `TargetIntegerTypeInhabitance` row after the fold’s
/// fail-closed payload check on **`type_realization`** (substrate **`TypeRealization`** meta + row
/// `language`/`kernel_integer` consistency with realization `language`/`target`; see `fold.rs`).
///
/// **API contract:** downstream code must not fabricate this carrier — obtain values only from
/// [`fold_program_to_target`](crate::fold::fold_program_to_target). The field is private so random
/// `DeclarationId` values cannot be mistaken for validated substrate proof outside this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectedTargetInhabitance {
    type_realization: DeclarationId,
}

impl SelectedTargetInhabitance {
    #[must_use]
    pub fn type_realization(self) -> DeclarationId {
        self.type_realization
    }

    pub(crate) fn from_validated_type_realization(type_realization: DeclarationId) -> Self {
        Self { type_realization }
    }
}
