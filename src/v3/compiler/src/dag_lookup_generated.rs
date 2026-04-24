// AUTO-GENERATED from `src/v3/std/lookup.dag`.
// Regenerate instead of hand-editing.

// `lookup.dag` fn items are not mirrored here — see script comment
// on `DAG_LOOKUP_TEMPLATE` in `regen_runtime_mirrors.py`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup<T> {
    Miss,
    Hit(T),
}
