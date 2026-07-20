use im::OrdSet as BTreeSet;
use std::rc::Rc;

use v1_compiler::v1_compiler_emit_rust::{
    render_rust_text_carrier, rust_applied_type_base, rust_named_type_base,
};
use v1_compiler::v1_compiler_infer_emit_info::RustCorpusRepr;

#[test]
fn faithful_string_leaf_base_is_freemonoid_char() {
    let leaf = rust_named_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        leaf, "FreeMonoid<Char>",
        "faithful String leaf must render the declared Char element (got {leaf:?})"
    );
    assert_ne!(
        leaf, "FreeMonoid<Nat>",
        "leaf must not bake the wrong Nat element"
    );
}

#[test]
fn faithful_string_text_carrier_is_freemonoid_char() {
    let carrier = render_rust_text_carrier(Rc::new(BTreeSet::new()));
    assert_eq!(carrier, "FreeMonoid<Char>");
    assert_ne!(carrier, "FreeMonoid<Nat>");
}

#[test]
fn faithful_string_applied_base_is_bare_freemonoid() {
    let base = rust_applied_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        base, "FreeMonoid",
        "applied String base must be bare (element from the resolved child)"
    );
    assert!(
        !base.contains('<'),
        "applied base must hold no element literal (got {base:?})"
    );
}

#[test]
fn faithful_string_applied_single_application_not_doubled() {
    let base = rust_applied_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
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

#[test]
fn host_string_unchanged_by_faithful_fix() {
    let host = rust_named_type_base("String".to_string(), RustCorpusRepr::HostNative);
    assert!(
        !host.contains("FreeMonoid"),
        "host String must not render FreeMonoid (got {host:?})"
    );
}
