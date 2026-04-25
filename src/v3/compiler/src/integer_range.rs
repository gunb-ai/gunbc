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
    use v3_grounding_pilot::{RustPrimitive, RUST_PILOT_PRIMITIVES};

    /// `std/integer` decl name (what infer narrows on) for each `IntegerPrimitive` Rust
    /// `target_name` in `RUST_PILOT_PRIMITIVES` (e.g. `i8` ↔ `Int8`). Not duplicated in the
    /// .dag: this is a stable bridge between the pilot’s routing key and the compiler table.
    fn std_integer_name_for_pilot_rust_target(target: &str) -> Option<&'static str> {
        match target {
            "i8" => Some("Int8"),
            "i16" => Some("Int16"),
            "i32" => Some("Int32"),
            "i64" => Some("Int64"),
            "u8" => Some("UInt8"),
            "u16" => Some("UInt16"),
            "u32" => Some("UInt32"),
            "u64" => Some("UInt64"),
            _ => None,
        }
    }

    #[test]
    fn u8_range() {
        assert_eq!(range_for_std_integer_name("UInt8"), Some((0, 255)));
    }

    /// `range_min_inclusive` / `range_max_inclusive` in `RUST_PILOT_PRIMITIVES` (mirrors
    /// `dsl/extdeps/.../primitives.dag`) must match this module’s `std/integer` name table
    /// — a mechanical drift check without a *third* copy of decimal strings in this test
    /// (Claude / api-review: **T-Ground-IntegerRangeSingleAuthority** receipt).
    /// **Dissolution:** remove this test when that trigger lands (single authority);
    /// grep for `extdeps_pilot_range_strings` so it is not left as a permanent crutch.
    #[test]
    fn extdeps_pilot_range_strings_match_std_integer_name_table() {
        for p in RUST_PILOT_PRIMITIVES {
            let RustPrimitive::IntegerPrimitive {
                target_name,
                range_min_inclusive: lo_s,
                range_max_inclusive: hi_s,
                ..
            } = p
            else {
                continue;
            };
            let std_name = std_integer_name_for_pilot_rust_target(target_name)
                .unwrap_or_else(|| panic!("std/integer name mapping for {target_name}: add a row in std_integer_name_for_pilot_rust_target (new pilot Integer row?)"));
            let lo: i128 = lo_s
                .parse()
                .unwrap_or_else(|e| panic!("pilot {std_name} min {lo_s:?} ({lo_s}): {e}"));
            let hi: i128 = hi_s
                .parse()
                .unwrap_or_else(|e| panic!("pilot {std_name} max {hi_s:?} ({hi_s}): {e}"));
            let t = range_for_std_integer_name(std_name)
                .unwrap_or_else(|| panic!("std name {std_name} must have a range in this table"));
            assert_eq!(
                t,
                (lo, hi),
                "integer_range vs RUST_PILOT_PRIMITIVES {target_name} ({std_name})"
            );
        }
    }
}
