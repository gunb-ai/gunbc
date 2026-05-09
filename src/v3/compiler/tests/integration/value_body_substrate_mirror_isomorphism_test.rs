//! **Layer:** integration
//!
//! §1.8 gate #96: `value_body_substrate_mirror_isomorphism_executable`.
//! The Rust `ValueBody` enum is generated from `src/v3/std/substrate.dag`;
//! this gate fails closed when the generated Rust mirror is stale or when the
//! substrate constructors drift from the generated carrier shape.

const SUBSTRATE_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../src/v3/std/substrate.dag"
));
const GENERATED_VALUE_BODY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/dag_value_body_generated.rs"
));
const DAG_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/dag.rs"));

#[test]
fn value_body_substrate_mirror_isomorphism_executable() {
    let substrate_variants = substrate_value_body_variants(SUBSTRATE_DAG);
    let rust_variants = generated_rust_value_body_variants(GENERATED_VALUE_BODY);

    assert_eq!(
        substrate_variants,
        vec!["Unparsed", "Structural", "Scalar", "List", "Map"],
        "unexpected substrate ValueBody constructor inventory"
    );
    assert_eq!(
        rust_variants, substrate_variants,
        "Rust `ValueBody` generated mirror must stay isomorphic with `src/v3/std/substrate.dag`"
    );
    assert!(
        GENERATED_VALUE_BODY.contains("// AUTO-GENERATED from `src/v3/std/substrate.dag`."),
        "`dag_value_body_generated.rs` must declare substrate.dag as its generator authority"
    );
    assert!(
        DAG_RS.contains("include!(\"dag_value_body_generated.rs\");"),
        "`dag.rs` must consume the generated `ValueBody` mirror, not a hand-authored enum"
    );
}

fn substrate_value_body_variants(source: &str) -> Vec<&str> {
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
            variants.push(variant_name(rest));
        }
    }

    variants
}

fn generated_rust_value_body_variants(source: &str) -> Vec<&str> {
    let mut variants = Vec::new();
    let mut in_enum = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "pub enum ValueBody {" {
            in_enum = true;
            continue;
        }
        if !in_enum {
            continue;
        }
        if trimmed == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed == "}," || trimmed.contains(':') {
            continue;
        }
        variants.push(variant_name(trimmed));
    }

    variants
}

fn variant_name(text: &str) -> &str {
    let end = text
        .find(|c: char| c == '(' || c == '{' || c == ',' || c.is_whitespace())
        .unwrap_or(text.len());
    &text[..end]
}

// INVARIANTS P1 / P2: checkable receipt that this gate stays tied to the
// Brian-sanctioned worker brief and the generated-file manifest, not an
// untracked hand-maintained mirror.
const _: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docs/briefs/r3-v-valuebody-substrate-mirror-isomorphism-v1-worker.md"
));

#[test]
fn generated_value_body_file_is_manifested_as_producer_owned() {
    assert!(
        v3_compiler::generated_files::GENERATED_FILES
            .contains(&"src/v3/compiler/src/dag_value_body_generated.rs"),
        "`dag_value_body_generated.rs` must remain in the producer-owned generated-file manifest"
    );
}
