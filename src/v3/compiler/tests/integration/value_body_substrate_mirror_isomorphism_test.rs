//! **Layer:** integration
//!
//! §1.8 gate #96: `value_body_substrate_mirror_isomorphism_executable`.
//! The Rust `ValueBody` enum is generated from `src/v3/std/substrate.dag`;
//! this gate fails closed when the generated Rust mirror is stale or when the
//! substrate constructors drift from the generated carrier shape.

use v3_compiler::dag::{FieldMap, LiteralBits, ValueBody};
use v3_compiler::SourceSpan;

const SUBSTRATE_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../src/v3/std/substrate.dag"
));
const GENERATED_VALUE_BODY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/dag_value_body_generated.rs"
));

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariantShape {
    name: String,
    payload: PayloadShape,
}

// 🟢 TERMINAL test-helper coproduct: generated Rust enum payload syntax for
// this mirror parser is either tuple-shaped or record-shaped.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PayloadShape {
    Tuple(String),
    Record(Vec<(String, String)>),
}

#[test]
fn value_body_substrate_mirror_isomorphism_executable() {
    let substrate_variants = substrate_value_body_variants(SUBSTRATE_DAG);
    let generated_variants = generated_value_body_variants(GENERATED_VALUE_BODY);
    let runtime_variants = rust_value_body_variants();

    assert_generated_value_body_map_uses_field_map(&generated_variants);
    assert_eq!(
        normalize_generated_value_body_variants(&generated_variants),
        substrate_variants,
        "generated Rust `ValueBody` payload shape must stay isomorphic with `src/v3/std/substrate.dag`"
    );
    assert_eq!(
        runtime_variants,
        substrate_variants
            .iter()
            .map(|variant| variant.name.as_str())
            .collect::<Vec<_>>(),
        "live Rust `ValueBody` exhaustiveness witness must stay isomorphic with `src/v3/std/substrate.dag`"
    );
}

#[test]
fn value_body_map_fieldmap_duplicate_key_invariant_is_executable() {
    let duplicate = FieldMap::from_entries(vec![
        (
            "same".to_string(),
            v3_compiler::dag::FieldValue::Literal(LiteralBits::Bool(true)),
        ),
        (
            "same".to_string(),
            v3_compiler::dag::FieldValue::Literal(LiteralBits::Bool(false)),
        ),
    ])
    .expect_err("FieldMap must reject duplicate keys");

    assert_eq!(duplicate.key, "same");
}

fn substrate_value_body_variants(source: &str) -> Vec<VariantShape> {
    let mut variants = Vec::new();
    let mut in_value_body = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "type ValueBody" {
            in_value_body = true;
            continue;
        }
        if !in_value_body {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("type ") {
            break;
        }
        if let Some(rest) = trimmed
            .strip_prefix("= ValueBody")
            .or_else(|| trimmed.strip_prefix("| ValueBody"))
        {
            // Substrate constructor names carry the `ValueBody` family prefix
            // (`ValueBodyUnparsed`); Rust enum variants are the suffix.
            variants.push(parse_substrate_variant(rest));
        }
    }

    variants
}

fn generated_value_body_variants(source: &str) -> Vec<VariantShape> {
    let mut variants = Vec::new();
    let mut lines = source.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if !trimmed.starts_with(char::is_uppercase) || trimmed.starts_with("pub enum ") {
            continue;
        }

        if let Some((name, rest)) = trimmed.split_once('(') {
            variants.push(VariantShape {
                name: name.to_string(),
                payload: PayloadShape::Tuple(rest.trim_end_matches("),").to_string()),
            });
            continue;
        }

        if let Some(name) = trimmed.strip_suffix(" {") {
            let mut fields = Vec::new();
            for field_line in lines.by_ref() {
                let field = field_line.trim();
                if field == "}," {
                    break;
                }
                let (label, ty) = field
                    .trim_end_matches(',')
                    .split_once(": ")
                    .expect("generated ValueBody record field");
                fields.push((label.to_string(), ty.to_string()));
            }
            variants.push(VariantShape {
                name: name.to_string(),
                payload: PayloadShape::Record(fields),
            });
            continue;
        }

        panic!("unsupported generated ValueBody variant shape: {trimmed}");
    }

    variants
}

fn rust_value_body_variants() -> Vec<&'static str> {
    [
        ValueBody::Unparsed(SourceSpan::new("value_body_gate.dag", 0, 0)),
        ValueBody::Structural { fields: Vec::new() },
        ValueBody::Scalar(LiteralBits::Bool(true)),
        ValueBody::List(Vec::new()),
        ValueBody::Map(FieldMap::from_entries(Vec::new()).expect("empty map is valid")),
    ]
    .iter()
    .map(rust_value_body_variant_name)
    .collect()
}

fn rust_value_body_variant_name(body: &ValueBody) -> &'static str {
    match body {
        ValueBody::Unparsed(_) => "Unparsed",
        ValueBody::Structural { .. } => "Structural",
        ValueBody::Scalar(_) => "Scalar",
        ValueBody::List(_) => "List",
        ValueBody::Map(_) => "Map",
    }
}

fn variant_name(text: &str) -> &str {
    let end = text
        .find(|c: char| c == '(' || c == '{' || c == ',' || c.is_whitespace())
        .unwrap_or(text.len());
    &text[..end]
}

fn parse_substrate_variant(text: &str) -> VariantShape {
    let name = variant_name(text);
    let rest = text[name.len()..].trim();

    let payload = if let Some(tuple) = rest.strip_prefix('(') {
        PayloadShape::Tuple(tuple.trim_end_matches(')').to_string())
    } else if rest.starts_with('{') {
        let fields = rest
            .trim_start_matches('{')
            .trim_end_matches('}')
            .split(',')
            .map(str::trim)
            .filter(|field| !field.is_empty())
            .map(|field| {
                let (label, ty) = field.split_once(": ").expect("ValueBody record field");
                (label.to_string(), ty.to_string())
            })
            .collect();
        PayloadShape::Record(fields)
    } else {
        panic!("unsupported ValueBody substrate variant shape: {text}");
    };

    VariantShape {
        name: name.to_string(),
        payload,
    }
}

fn assert_generated_value_body_map_uses_field_map(generated_variants: &[VariantShape]) {
    let map = generated_variants
        .iter()
        .find(|variant| variant.name == "Map")
        .expect("generated ValueBody includes Map variant");

    assert_eq!(
        map.payload,
        PayloadShape::Tuple("FieldMap".to_string()),
        "generated Rust `ValueBody::Map` must use `FieldMap` so duplicate-key rejection stays at the carrier boundary"
    );
}

fn normalize_generated_value_body_variants(
    generated_variants: &[VariantShape],
) -> Vec<VariantShape> {
    generated_variants
        .iter()
        .map(|variant| VariantShape {
            name: variant.name.clone(),
            payload: normalize_generated_payload(&variant.payload),
        })
        .collect()
}

fn normalize_generated_payload(payload: &PayloadShape) -> PayloadShape {
    match payload {
        PayloadShape::Tuple(ty) => PayloadShape::Tuple(normalize_generated_type(ty)),
        PayloadShape::Record(fields) => PayloadShape::Record(
            fields
                .iter()
                .map(|(label, ty)| (label.clone(), normalize_generated_type(ty)))
                .collect(),
        ),
    }
}

fn normalize_generated_type(ty: &str) -> String {
    match ty {
        "Vec<(String, FieldValue)>" | "FieldMap" => "List<FieldEntry>".to_string(),
        "Vec<FieldValue>" => "List<FieldValue>".to_string(),
        other => other.to_string(),
    }
}

#[test]
fn generated_value_body_file_is_manifested_as_producer_owned() {
    assert!(
        v3_compiler::generated_files::GENERATED_FILES
            .contains(&"src/v3/compiler/src/dag_value_body_generated.rs"),
        "`dag_value_body_generated.rs` must remain in the producer-owned generated-file manifest"
    );
}
