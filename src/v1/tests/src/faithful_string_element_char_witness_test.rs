//! Faithful-carrier String element witness — String = FreeMonoid<Char> renders the DECLARED
//! Char element (std/string_type.dag: "String is the free monoid over Char"), uniformly at
//! leaf AND applied, as a SINGLE application.
//!
//! Discriminating / RED-on-revert: the old emitter baked the WRONG element Nat — leaf
//! "FreeMonoid<Nat>" and applied DOUBLE "FreeMonoid<Nat><Char>". Reverting the de-double fix
//! in 05_emit_rust.dag (rust_named_type_base / render_rust_text_carrier / rust_applied_type_base)
//! returns "FreeMonoid<Nat>" / the bare-base helper disappears, and these assertions fail.
//!
//! These exercise the emit render helpers directly (no Node graph needed): the leaf base, the
//! text carrier, and the bare applied base whose resolved child drives the single application.

use std::collections::BTreeSet;
use std::rc::Rc;

use v1_compiler::v1_compiler_emit_rust::{
    render_rust_text_carrier, rust_applied_type_base, rust_named_type_base,
};
use v1_compiler::v1_compiler_infer_emit_info::RustCorpusRepr;

/// LEAF base: a faithful String node with no resolved type-arg renders its DECLARED element
/// Char — "FreeMonoid<Char>", NOT the old "FreeMonoid<Nat>".
#[test]
fn faithful_string_leaf_base_is_freemonoid_char() {
    let leaf = rust_named_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        leaf, "FreeMonoid<Char>",
        "faithful String leaf must render the declared Char element (got {leaf:?})"
    );
    // Discriminating: the old wrong-element bake.
    assert_ne!(
        leaf, "FreeMonoid<Nat>",
        "leaf must not bake the wrong Nat element"
    );
}

/// TEXT CARRIER (the other leaf surface) renders FreeMonoid<Char> too.
#[test]
fn faithful_string_text_carrier_is_freemonoid_char() {
    let carrier = render_rust_text_carrier(Rc::new(BTreeSet::new()));
    assert_eq!(carrier, "FreeMonoid<Char>");
    assert_ne!(carrier, "FreeMonoid<Nat>");
}

/// APPLIED base is the BARE container: when the String node carries a resolved type-arg child,
/// the element comes from that child (single application), so the base must NOT carry an
/// element literal — else concat(base, "<", child, ">") double-applies.
#[test]
fn faithful_string_applied_base_is_bare_freemonoid() {
    let base = rust_applied_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        base, "FreeMonoid",
        "applied String base must be bare (element from the resolved child)"
    );
    // Discriminating: a non-bare base double-applies.
    assert!(
        !base.contains('<'),
        "applied base must hold no element literal (got {base:?})"
    );
}

/// SINGLE APPLICATION end-to-end: bare base + the resolved Char child = "FreeMonoid<Char>",
/// NOT the old double "FreeMonoid<Nat><Char>".
#[test]
fn faithful_string_applied_single_application_not_doubled() {
    let base = rust_applied_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    // render_rust_applied_type composes concat(base, "<", <resolved child = "Char">, ">").
    let applied = format!("{base}<{}>", "Char");
    assert_eq!(
        applied, "FreeMonoid<Char>",
        "applied String must be a single application"
    );
    assert_ne!(
        applied, "FreeMonoid<Nat><Char>",
        "applied String must not double-apply"
    );
}

/// HOST control (the fix is faithful-only): the HostNative branch is unchanged — String stays
/// the host representation, never FreeMonoid. Guards sign-bar (c) at the unit level.
#[test]
fn host_string_unchanged_by_faithful_fix() {
    let host = rust_named_type_base("String".to_string(), RustCorpusRepr::HostNative);
    assert!(
        !host.contains("FreeMonoid"),
        "host String must not render FreeMonoid (got {host:?})"
    );
}
