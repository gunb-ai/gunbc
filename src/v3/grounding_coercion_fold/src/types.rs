//! LanguageSpec projection + target inhabitance carriers for the structural fold.

/// How the fold obtains LanguageSpec / target-primitive substrate facts.
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — `Undeclared` awaits a declared
/// LanguageSpec projection; `ScratchIntExamples` is an interim checkpoint. Named dissolution:
/// #1133 / #1286 (declared projection replaces this carrier).
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
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — bounded to Examples 1–2 only;
/// superseded when a declared projection generalizes Int examples (#1133 / #1286).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntScratchExample {
    /// Example 1 — `data x: Int = 0` → `EmissionDiagnostic::UnderRefined`.
    DesignDocExample1UnrefinedInt,
    /// Example 2 — `data count: Int(0..2^32) = 100` → unique Rust `u32`.
    DesignDocExample2BoundedU32,
}

/// Unique target inhabitance for a binding (Examples 1–2 surface only).
///
/// Practice 4 (`docs/modeling-discipline.md`): single-variant placeholder — N \< 2, so no 🟢/🟡/🔴
/// coproduct checkpoint until a second inhabitant is introduced. When a second variant lands,
/// classify this enum explicitly (substrate vs Rust mirror / dissolution) — do not grow ad hoc
/// without the same checkpoint discipline as the projection carriers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetInhabitance {
    RustU32,
}
