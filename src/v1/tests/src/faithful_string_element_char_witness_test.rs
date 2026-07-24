use im::OrdSet as BTreeSet;
use std::rc::Rc;

use v1_compiler::v1_compiler_emit_rust::{
    render_rust_text_carrier, rust_applied_type_base, rust_named_type_base,
    rust_string_grounded_type_alias_decl_line,
};
use v1_compiler::v1_compiler_infer_emit_info::RustCorpusRepr;

#[test]
fn faithful_string_type_alias_decl_grounds_to_native_string() {
    let decl = rust_string_grounded_type_alias_decl_line();
    assert!(
        decl.contains("String = std::string::String"),
        "String type alias must ground to native std::string::String (Gate-1 text carrier), got {decl:?}"
    );
    assert!(
        !decl.contains("FreeMonoid"),
        "grounded String alias must not render FreeMonoid (got {decl:?})"
    );
}

#[test]
fn faithful_string_leaf_base_grounds_to_native_string() {
    let leaf = rust_named_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        leaf, "String",
        "faithful String leaf must ground to native String (Gate-1 text-carrier grounding), got {leaf:?}"
    );
    assert_ne!(
        leaf, "FreeMonoid<Char>",
        "faithful String must not render FreeMonoid<Char> after grounding"
    );
}

#[test]
fn faithful_string_text_carrier_grounds_to_native_string() {
    let carrier = render_rust_text_carrier(Rc::new(BTreeSet::new()));
    assert_eq!(carrier, "String");
    assert_ne!(carrier, "FreeMonoid<Char>");
}

#[test]
fn faithful_string_applied_base_grounds_to_native_string() {
    let base = rust_applied_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        base, "String",
        "applied String base must ground to native String, got {base:?}"
    );
    assert!(
        !base.contains("FreeMonoid"),
        "applied base must not render FreeMonoid after grounding (got {base:?})"
    );
}

#[test]
fn faithful_string_applied_single_application_stays_string() {
    let base = rust_applied_type_base("String".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    let applied = format!("{base}<{}>", "Char");
    assert_eq!(
        applied, "String<Char>",
        "applied String after grounding is still the native carrier (got {applied:?})"
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
