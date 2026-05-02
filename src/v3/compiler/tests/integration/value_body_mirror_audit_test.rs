//! **Layer:** integration
//!
//! ValueBody mirror audit: `src/v3/std/substrate.dag` sum type vs `dag::ValueBody` (Rust).
//! Ratchets drift detection without hand-maintaining parallel lists — substrate text is the
//! authority for the reflected sum; Rust is the runtime carrier (Evaluator retirement / #1531).

use std::collections::HashSet;

use v3_compiler::dag::{FieldMap, LiteralBits, ValueBody};
use v3_compiler::SourceSpan;

const SUBSTRATE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../std/substrate.dag");

/// Parse constructor names from the `type ValueBody` sum in `substrate.dag` (lines between the
/// `type ValueBody` header and the `// Type substrate.` sentinel).
fn substrate_value_body_constructors_from_source() -> Vec<String> {
    let substrate = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../std/substrate.dag"));
    let start = substrate
        .find("type ValueBody")
        .unwrap_or_else(|| panic!("{SUBSTRATE_PATH}: missing `type ValueBody`"));
    let tail = &substrate[start..];
    let end = tail.find("\n// Type substrate.").unwrap_or_else(|| {
        panic!("{SUBSTRATE_PATH}: missing `// Type substrate.` after ValueBody")
    });
    let block = &tail[..end];
    let mut out = Vec::new();
    for line in block.lines() {
        let t = line.trim_start();
        if !(t.starts_with('=') || t.starts_with('|')) {
            continue;
        }
        let rest = t.trim_start_matches(['=', '|']).trim_start();
        let name = rest
            .split(|c: char| c == '(' || c == '{' || c.is_whitespace())
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                panic!("{SUBSTRATE_PATH}: malformed ValueBody variant line: {line:?}")
            });
        out.push(name.to_string());
    }
    out
}

fn rust_value_body_variant_tag(body: &ValueBody) -> &'static str {
    match body {
        ValueBody::Unparsed(_) => "Unparsed",
        ValueBody::Structural { .. } => "Structural",
        ValueBody::Scalar(_) => "Scalar",
        ValueBody::List(_) => "List",
        ValueBody::Map(_) => "Map",
    }
}

fn sample_instances_covering_all_rust_variants() -> Vec<ValueBody> {
    let span = SourceSpan::new("value_body_mirror_audit_test.v3", 0, 1);
    vec![
        ValueBody::Unparsed(span),
        ValueBody::Structural { fields: Vec::new() },
        ValueBody::Scalar(LiteralBits::Int(0)),
        ValueBody::List(Vec::new()),
        ValueBody::Map(FieldMap::from_entries(Vec::new()).expect("empty FieldMap")),
    ]
}

#[test]
fn substrate_value_body_sum_matches_parsed_constructors() {
    let parsed = substrate_value_body_constructors_from_source();
    assert_eq!(
        parsed,
        vec![
            "ValueBodyUnparsed".to_string(),
            "ValueBodyStructural".to_string(),
            "ValueBodyMap".to_string(),
        ],
        "`{SUBSTRATE_PATH}` `type ValueBody` must expose exactly these three constructors until substrate regen adds Scalar/List; update this ratchet when the sum changes"
    );
}

#[test]
fn rust_value_body_runtime_variants_are_exhaustively_tagged() {
    let tags: HashSet<&str> = sample_instances_covering_all_rust_variants()
        .iter()
        .map(|b| rust_value_body_variant_tag(b))
        .collect();
    assert_eq!(
        tags,
        HashSet::from(["Unparsed", "Structural", "Scalar", "List", "Map"]),
        "dag::ValueBody gained/lost a variant — update rust_value_body_variant_tag + this test"
    );
}

#[test]
fn value_body_substrate_rust_mirror_audit_documents_known_gap() {
    let substrate_constructors = substrate_value_body_constructors_from_source();
    let rust_tags: HashSet<&str> = sample_instances_covering_all_rust_variants()
        .iter()
        .map(|b| rust_value_body_variant_tag(b))
        .collect();

    assert_eq!(substrate_constructors.len(), 3);
    assert_eq!(rust_tags.len(), 5);

    // Missing generation surface (Disposition #1 debt paid target / Evaluator retirement):
    // extend `substrate.dag` + bootstrap/regen when `ValueBodyScalar` / `ValueBodyList` (and
    // refined map carrier) are generated from the Rust mirror — see `dag.rs` ValueBody docs.
    assert!(
        rust_tags.contains("Scalar") && rust_tags.contains("List"),
        "Rust carries Scalar/List top-level bodies; substrate sum must eventually reflect them"
    );
}
