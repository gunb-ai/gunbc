//! Inclusive `i128` bounds for `std/integer.dag` surface declarations used by
//! R2 **infer/lower** magnitude checks (`MagnitudeOutOfRange`).
//!
//! **Authority (compiler path):** [`range_for_std_integer_name`] is a **name-keyed**
//! table that must match `std/integer.dag`’s `Int8` / `UInt8` / `Int` / … meanings.
//! This is **not** read from the DAG at runtime; it is the v3 compiler’s
//! current single source for *narrowing* against *std integer decl names*.
//!
//! **Parallel authority (T-Ground / pilot):** `dsl/extdeps/languages/rust/primitives.dag`
//! carries `range_min_inclusive` / `range_max_inclusive` on `IntegerPrimitive` (decimal
//! strings) for the Rust *target* primitive table and grounding-engine validation. Those
//! fields are **not** what `infer`/`lower` consult today — the two must stay
//! **semantically aligned** (same min/max for the same width) or they can drift.
//!
//! **Named dissolution: T-Ground-IntegerRangeSingleAuthority** — one consumer-facing
//! range source (e.g. read pilot `.dag` or a generated `include!` from one table), then
//! delete the duplicate.
//!
//! **Walk cap (32):** local guard on alias / instantiation chains; not a substitute for
//! substrate-typed chain depth (hoist if alias depth becomes a first-class fact).

use crate::dag::{AtomPayload, Dag, DeclarationId, TypeConnective};

/// Returns inclusive `[min, max]` for a fixed-width or default integer declaration,
/// or `None` if the declaration is not a modeled integer leaf.
pub fn i128_range_for_integer_decl(dag: &Dag, mut decl_id: DeclarationId) -> Option<(i128, i128)> {
    for _ in 0..32 {
        let decl = dag.declaration(decl_id);
        if let Some(name) = decl.name.as_deref() {
            if let Some(r) = range_for_std_integer_name(name) {
                return Some(r);
            }
        }
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => {
                decl_id = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                decl_id = *next;
            }
            _ => return None,
        }
    }
    None
}

fn range_for_std_integer_name(name: &str) -> Option<(i128, i128)> {
    match name {
        "Int" | "Int64" => Some((i64::MIN as i128, i64::MAX as i128)),
        "Int8" => Some((-128, 127)),
        "Int16" => Some((-32_768, 32_767)),
        "Int32" => Some((-2_147_483_648, 2_147_483_647)),
        "UInt8" => Some((0, 255)),
        "UInt16" => Some((0, 65_535)),
        "UInt32" => Some((0, 4_294_967_295)),
        "UInt64" | "UInt" => Some((0, u64::MAX as i128)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_range() {
        assert_eq!(range_for_std_integer_name("UInt8"), Some((0, 255)));
    }
}
