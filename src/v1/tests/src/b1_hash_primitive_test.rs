use v1_compiler::v1_compiler_runtime_rust::rt_hash_ops;
use v1_compiler::v1_rt::{atom_identity_hash, hash_combine, is_hash_digest};

fn sym(name: &str) -> String {
    name.to_string()
}

#[test]
fn atom_identity_hash_is_stable_and_distinct() {
    let a = atom_identity_hash(sym("canonical_tag_conj"));
    let b = atom_identity_hash(sym("canonical_tag_atom"));
    assert_ne!(a, b);
    assert_eq!(a, atom_identity_hash(sym("canonical_tag_conj")));
    assert!(is_hash_digest(&a));
}

#[test]
fn hash_combine_separates_tag_and_payload_namespaces() {
    let conj = atom_identity_hash(sym("canonical_tag_conj"));
    let atom_tagged = hash_combine(
        atom_identity_hash(sym("canonical_tag_atom")),
        atom_identity_hash(sym("canonical_tag_conj")),
    );
    assert_ne!(conj, atom_tagged);
}

#[test]
fn hash_combine_separates_named_and_positional_edge_tags() {
    let positional = atom_identity_hash(sym("canonical_tag_positional_edge"));
    let named_tagged = hash_combine(
        atom_identity_hash(sym("canonical_tag_named_edge")),
        atom_identity_hash(sym("canonical_tag_positional_edge")),
    );
    assert_ne!(positional, named_tagged);
}

#[test]
fn hash_combine_is_sensitive_to_pair_order() {
    let ha = atom_identity_hash(sym("a"));
    let hb = atom_identity_hash(sym("b"));
    let ab = hash_combine(ha.clone(), hb.clone());
    let ba = hash_combine(hb, ha);
    assert_ne!(ab, ba);
}

#[test]
fn hash_combine_rejects_non_carrier_inputs() {
    assert!(!is_hash_digest("a"));
    assert!(!is_hash_digest("a\0b"));
    assert!(!is_hash_digest("not-a-hash-digest"));
}

#[test]
#[should_panic(expected = "16-char hex Hash digest")]
fn hash_combine_rejects_pair_framing_ambiguous_raw_strings() {
    let _ = hash_combine(sym("a\0b"), sym("c"));
}

#[test]
fn emitted_runtime_hash_ops_preserves_hash_carrier_boundary() {
    let emitted = rt_hash_ops();
    assert!(emitted.contains("fn expect_hash_digest"));
    assert!(emitted.contains("pub fn is_hash_digest"));
    assert!(emitted.contains("pub fn hash_combine(a: Hash, b: Hash) -> Hash"));
    assert!(!emitted.contains("pub fn hash_combine(a: String, b: String)"));
}
