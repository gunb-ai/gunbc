// AUTO-GENERATED from `src/v3/std/lookup.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup<T> {
    Miss,
    Hit(T),
}
