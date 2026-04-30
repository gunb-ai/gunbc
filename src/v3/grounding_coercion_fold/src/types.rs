//! Open / stub carriers for the public fold signature until LanguageSpec + lowering land.

/// Placeholder for the structural **LanguageSpec** projection the fold will consume.
///
/// **Shape TBD** — manager dispatch (#1203 / #1133): substrate may declare a projection
/// carrier, or the fold may walk [`v3_compiler::dag::Dag`] directly. **Do not** treat this
/// type as the final API. No row consumption until LanguageSpec Phase 1.5 (#1227) and
/// related substrate gates land.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct LanguageSpecProjectionUndeclared;

/// Stub for **unique per-binding target inhabitance** (primitive + refinement witness).
///
/// Populated by the real Coercion-Fold body when T-Ground-LanguageSpec + extraction
/// gates unblock; the scaffold never returns `Ok` with populated values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetInhabitance;
