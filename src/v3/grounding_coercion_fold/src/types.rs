//! LanguageSpec projection + target inhabitance carriers for the structural fold.

use std::collections::BTreeMap;

use v3_compiler::dag::{DeclarationId, Interval};
use v3_grounding_lifetime::BindingId;

/// How the fold obtains LanguageSpec / target-primitive substrate facts.
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — `Undeclared` remains the
/// fail-closed production default; `DeclaredIntegerIntents` is the executable LanguageSpec
/// projection path for integer inhabitance rows; `ScratchIntExamples` is retained as a
/// compatibility checkpoint until callers migrate (#1133 / #1286).
///
/// **`Undeclared`** — production entry stays fail-closed until a real projection lands
/// (substrate / #1286 / manager #1133).
///
/// **`ScratchIntExamples`** — **checkpoint only**: drives [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md)
/// Int-family examples. Still reads the bootstrap `Dag` once to count
/// `TargetIntegerTypeInhabitance` declarations (`emit_model.dag`, **INVARIANTS.md E-6**
/// same-PR witness). Examples 2 and 8 consume declared integer-row payloads, but the
/// enum remains a scratch driver until real program-bound and algebra-intent extraction
/// can replace it with a declared projection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LanguageSpecProjection {
    #[default]
    Undeclared,
    DeclaredIntegerIntents(BTreeMap<BindingId, IntegerTargetIntent>),
    ScratchIntExamples(IntScratchExample),
}

/// Program-side integer bound projected into the fold.
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

/// Scratch selector aligned to design-emission-model §Worked examples (Int only).
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — bounded to Int-family examples;
/// superseded when a declared projection generalizes Int examples (#1133 / #1286).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntScratchExample {
    /// Example 1 — `data x: Int = 0` → `EmissionDiagnostic::UnderRefined`.
    DesignDocExample1UnrefinedInt,
    /// Example 2 — `data count: Int(0..2^32) = 100` → unique Rust `u32`.
    DesignDocExample2BoundedU32,
    /// Example 5 — `Int(0..2^32)` without algebra annotation → algebra under-refined.
    DesignDocExample5AmbiguousAlgebra,
    /// Example 6 — `Int(0..2^65)` exceeds the Rust Int128 family.
    DesignDocExample6NoInhabitant,
    /// Example 8 — Rust target for `Int(-2^31..2^31)`.
    ///
    /// Kept as three flat variants rather than adding a scratch target coproduct: this matches
    /// the existing per-example driver shape and keeps the transitional selector simple until
    /// the declared LanguageSpec projection replaces it.
    DesignDocExample8Rust,
    /// Example 8 — Python target for `Int(-2^31..2^31)`.
    DesignDocExample8Python,
    /// Example 8 — Go target for `Int(-2^31..2^31)`.
    DesignDocExample8Go,
}

/// Unique target inhabitance for a binding (scratch Int example surface only).
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — lane-local output mirror for
/// hardcoded scratch examples. It retires with `ScratchIntExamples` when the declared
/// LanguageSpec projection can compute target inhabitance structurally.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetInhabitance {
    RustU32,
    RustI32,
    PythonInt,
    GoInt32,
}
