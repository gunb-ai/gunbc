//! Family-A regression: a map-literal's KEY must receive the same type-driven
//! String coercion the VALUE already gets, so a `Map<String, V>` data literal emits
//! `HashMap<String, V>` and not `HashMap<&str, V>` (the E0308 cluster).
//!
//! The coercion is DERIVED from the declared key type (`is_rust_string_like` over
//! the map's key type, computed at the emit call site). These witnesses pin both
//! directions of the derived decision plus the real-source emission shape.

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_emit_rust::emit_rust_map_literal_key;
use v1_compiler::v1_compiler_infer_emit_info::RustCorpusRepr;

// --- Discriminating control: the decision is type-DERIVED, not a blanket
// "always .to_string() the key". A non-String declared key is not expressible as a
// record-literal map (record keys are field-name identifiers), so the false branch
// is exercised directly here. ---

#[test]
fn string_key_is_coerced_to_owned_string() {
    // HostNative repr: the host->dag seam is identity, so the coercion is the only
    // transform and is directly observable.
    let out = emit_rust_map_literal_key("alpha".to_string(), true, RustCorpusRepr::HostNative);
    assert_eq!(
        out, "\"alpha\".to_string()",
        "a String-keyed map literal must own its key"
    );
}

#[test]
fn non_string_key_is_left_as_literal() {
    // The wall: a NON-String declared key must NOT be `.to_string()`'d. If this ever
    // emits `.to_string()`, the coercion has degenerated to blanket (a §5 fail-open
    // that mis-coerces non-String key maps).
    let out = emit_rust_map_literal_key("alpha".to_string(), false, RustCorpusRepr::HostNative);
    assert_eq!(
        out, "\"alpha\"",
        "a non-String declared key must stay an unowned literal"
    );
}

#[test]
fn faithful_repr_routes_owned_key_through_the_text_seam() {
    // Under the faithful FreeMonoid repr the key still becomes an owned String first,
    // then crosses the host->dag text seam (which consumes a `String`).
    let out = emit_rust_map_literal_key(
        "alpha".to_string(),
        true,
        RustCorpusRepr::FaithfulFreeMonoid,
    );
    assert_eq!(
        out, "crate::v2_std_text::host_string_text_from_rust_host(\"alpha\".to_string())",
        "faithful repr must hand the seam an owned String"
    );
}

// --- Source-fixture proof: a real `Map<String, Int>` data literal emits owned
// String keys (so the inferred map matches the declared `HashMap<String, i64>`),
// while the Int value is left uncoerced (the slots are independently type-driven). ---

#[test]
fn map_string_int_literal_emits_owned_keys_only() {
    let source = "module mapkey.fixture\n\ndata lookup: Map<String, Int> = { alpha: 1, beta: 2 }\n";
    let result = crate::helpers::compile_dag_target(source, RenderTarget::Rust);
    let emitted: String = result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("\"alpha\".to_string()"),
        "expected owned String key (coerced via .to_string()) in emitted map literal, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("1.to_string()"),
        "the Int value must not be coerced to String (per-slot type derivation):\n{emitted}"
    );
}
