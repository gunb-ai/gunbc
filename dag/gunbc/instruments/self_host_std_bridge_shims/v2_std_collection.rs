// Shared std-bridge shim — curated minimal v2.std.collection surface.
//
// AUTHORITY: src/v2/std/collection.dag. Hand-authored scaffold; see the header of
// v2_std_diagnostic.rs in this directory for why the bridge exists, what dissolves it, and
// why drift against the authority is caught only by the wet receipt's cargo build today.
//
// UNION, NOT A PICK. This file replaces two same-named per-transport copies whose surfaces
// were DISJOINT: the 03_normalize copy provided only the `Optional` import path, the
// use_site_verdict copy only `List`. Repointing both at either copy would have silently
// removed the other's surface — a de-fork that drops symbols is not a de-fork, it is a
// regression wearing one. So the bridge carries the union, and the rule for any future
// consumer is the same: widen this file to a superset, never narrow a consumer to whatever
// happens to be here already.
pub type List<T> = im::Vector<T>;

// Optional import path for dead `use Optional::{Absent, Present}` in emitted entry.
pub mod Optional {
    pub enum Enum<T> {
        Absent,
        Present { value: T },
    }
    pub use Enum::{Absent, Present};
}

pub fn optional_absent<T>() -> Option<T> {
    None
}

pub fn optional_present<T>(value: T) -> Option<T> {
    Some(value)
}
