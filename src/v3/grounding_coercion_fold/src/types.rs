//! LanguageSpec projection + target inhabitance carriers for the structural fold.

/// How the fold obtains LanguageSpec / target-primitive substrate facts.
///
/// **`Undeclared`** — production entry stays fail-closed until a real projection lands
/// (substrate / #1286 / manager #1133).
///
/// **`ScratchIntExamples`** — **checkpoint only**: drives [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md)
/// **Example 1** and **Example 2** without walking Phase 1 Dag rows. Remove or supersede when
/// a declared projection replaces this scratch path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LanguageSpecProjection {
    #[default]
    Undeclared,
    ScratchIntExamples(IntScratchExample),
}

/// Scratch selector aligned to design-emission-model §Worked examples (Int only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntScratchExample {
    /// Example 1 — `data x: Int = 0` → `EmissionDiagnostic::UnderRefined`.
    DesignDocExample1UnrefinedInt,
    /// Example 2 — `data count: Int(0..2^32) = 100` → unique Rust `u32`.
    DesignDocExample2BoundedU32,
}

/// Unique target inhabitance for a binding (Examples 1–2 surface only).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetInhabitance {
    RustU32,
}
