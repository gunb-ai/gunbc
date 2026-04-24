// AUTO-GENERATED from `src/v3/std/lookup.dag`.
// Regenerate instead of hand-editing.

// Mirror of `v3.std.lookup` — see `DAG_LOOKUP_TEMPLATE` in
// `regen_runtime_mirrors.py`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup<T> {
    Miss,
    Hit(T),
}

/// `v3.std.lookup::miss_int_lookup` (`.dag` authority).
#[inline]
pub fn miss_int_lookup() -> Lookup<i64> {
    Lookup::Miss
}

/// `v3.std.lookup::hit_int_lookup` (`.dag` authority).
#[inline]
pub fn hit_int_lookup(n: i64) -> Lookup<i64> {
    Lookup::Hit(n)
}

/// `v3.std.lookup::miss_declaration_id_lookup` (`.dag` authority).
#[inline]
pub fn miss_declaration_id_lookup() -> Lookup<DeclarationId> {
    Lookup::Miss
}

/// `v3.std.lookup::hit_declaration_id_lookup` (`.dag` authority).
#[inline]
pub fn hit_declaration_id_lookup(id: DeclarationId) -> Lookup<DeclarationId> {
    Lookup::Hit(id)
}
