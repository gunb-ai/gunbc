//! LanguageSpec projection + target inhabitance carriers for the structural fold.

use std::collections::BTreeMap;

use v3_compiler::dag::{DeclarationId, Interval};
use v3_grounding_lifetime::BindingId;

/// How the fold obtains LanguageSpec / target-primitive substrate facts.
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — `Undeclared` remains the
/// fail-closed production default; `DeclaredIntegerIntents` is the executable LanguageSpec
/// projection path for integer inhabitance rows.
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

/// Selected integer target: structural identity from a declared inhabitance row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectedTargetInhabitance {
    pub type_realization: DeclarationId,
}
