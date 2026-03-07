//! FermiDepth (DSL) ↔ FermiCost (Rust) parity guardrails.
//!
//! These tests ensure the parallel enum definitions stay in sync:
//! - Variant count and names
//! - Timeout values per depth level
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::PathBuf;

fn dsl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl")
}

/// Extract FermiDepth variant names from `dsl/std/types.dag` by parsing the AST.
fn dsl_fermi_depth_variants() -> Vec<String> {
    let types_path = dsl_root().join("std/types.dag");
    let source = std::fs::read_to_string(&types_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", types_path.display()));
    let ast = daglang_syntax::parser::parse(&source)
        .unwrap_or_else(|e| panic!("parse {}: {e:?}", types_path.display()));
    for item in &ast.items {
        if let daglang_syntax::ast::Item::TypeDef(def) = &item.node {
            if def.name == "FermiDepth" {
                if let daglang_syntax::ast::TypeBody::Sum(variants) = &def.body {
                    return variants.iter().map(|v| v.name.clone()).collect();
                }
            }
        }
    }
    panic!("FermiDepth type not found in std/types.dag");
}

/// Rust FermiCost variant names (canonical source of truth).
fn rust_fermi_cost_variants() -> Vec<&'static str> {
    vec!["XS", "S", "M", "L", "XL"]
}

/// Extract variant tag from a DSL sum-type value (serialized as `{"_variant":"Xs"}`).
fn extract_variant_tag(value: &serde_json::Value) -> Option<String> {
    value.as_object()?.get("_variant")?.as_str().map(String::from)
}

/// Extract meta_targets from `config/build_targets.dag`.
fn dsl_meta_targets() -> Vec<serde_json::Value> {
    let output = daglang_driver::compile_data_from_module(&dsl_root(), "config/build_targets.dag")
        .expect("config/build_targets.dag should compile");
    let value = output
        .data_values
        .get("meta_targets")
        .expect("meta_targets data should exist");
    serde_json::from_value(value.clone()).expect("meta_targets should be a list")
}

// ============================================================================
// B2: FermiDepth ↔ FermiCost variant parity
// ============================================================================

#[test]
fn fermi_depth_and_cost_have_same_variant_count() {
    let dsl = dsl_fermi_depth_variants();
    let rust = rust_fermi_cost_variants();
    assert_eq!(
        dsl.len(),
        rust.len(),
        "FermiDepth (DSL) has {} variants but FermiCost (Rust) has {}: DSL={:?}, Rust={:?}",
        dsl.len(),
        rust.len(),
        dsl,
        rust,
    );
}

#[test]
fn fermi_depth_and_cost_variant_names_match() {
    let dsl = dsl_fermi_depth_variants();
    let rust = rust_fermi_cost_variants();
    // DSL uses PascalCase (Xs), Rust uses UPPER (XS). Normalize to uppercase.
    let dsl_upper: Vec<String> = dsl.iter().map(|v| v.to_uppercase()).collect();
    let rust_upper: Vec<String> = rust.iter().map(|v| v.to_uppercase()).collect();
    assert_eq!(
        dsl_upper, rust_upper,
        "FermiDepth and FermiCost variants differ (case-normalized): DSL={:?}, Rust={:?}",
        dsl, rust,
    );
}

#[test]
fn fermi_depth_ordinal_ordering_matches_cost_ordering() {
    let dsl = dsl_fermi_depth_variants();
    let rust = rust_fermi_cost_variants();
    for (i, (d, r)) in dsl.iter().zip(rust.iter()).enumerate() {
        assert_eq!(
            d.to_uppercase(),
            r.to_uppercase(),
            "ordinal mismatch at position {i}: DSL has '{d}', Rust has '{r}'",
        );
    }
}

// ============================================================================
// B3: Timeout parity (DSL fermi_timeouts ↔ Rust FermiCost::timeout_ms)
// ============================================================================

#[test]
fn fermi_timeout_values_match_rust() {
    let output = daglang_driver::compile_data_from_module(&dsl_root(), "std/fermi.dag")
        .expect("std/fermi.dag should compile");
    let timeouts_json = output
        .data_values
        .get("fermi_timeouts")
        .expect("fermi_timeouts data should exist");
    let timeouts: Vec<serde_json::Value> =
        serde_json::from_value(timeouts_json.clone()).expect("fermi_timeouts should be a list");

    // Build DSL mapping: depth_variant → timeout_ms
    // DSL serializes sum-type values as {"_variant":"Xs"}, not bare strings.
    let mut dsl_timeouts: BTreeMap<String, u64> = BTreeMap::new();
    for entry in &timeouts {
        let depth_value = entry.get("depth").expect("depth field");
        let depth = extract_variant_tag(depth_value)
            .unwrap_or_else(|| panic!("depth should be a variant tag, got: {depth_value}"));
        let ms = entry
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .expect("timeout_ms field");
        dsl_timeouts.insert(depth.to_uppercase(), ms);
    }

    // Rust canonical timeouts
    let rust_timeouts: BTreeMap<String, u64> = [
        ("XS", gunbc_test::FermiCost::XS.timeout_ms()),
        ("S", gunbc_test::FermiCost::S.timeout_ms()),
        ("M", gunbc_test::FermiCost::M.timeout_ms()),
        ("L", gunbc_test::FermiCost::L.timeout_ms()),
        ("XL", gunbc_test::FermiCost::XL.timeout_ms()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    assert_eq!(
        dsl_timeouts, rust_timeouts,
        "DSL fermi_timeouts and Rust FermiCost::timeout_ms() diverge"
    );
}

#[test]
fn fermi_timeouts_cover_all_depth_variants() {
    let dsl_variants = dsl_fermi_depth_variants();
    let output = daglang_driver::compile_data_from_module(&dsl_root(), "std/fermi.dag")
        .expect("std/fermi.dag should compile");
    let timeouts_json = output
        .data_values
        .get("fermi_timeouts")
        .expect("fermi_timeouts data should exist");
    let timeouts: Vec<serde_json::Value> =
        serde_json::from_value(timeouts_json.clone()).expect("fermi_timeouts should be a list");

    let timeout_depths: Vec<String> = timeouts
        .iter()
        .filter_map(|e| extract_variant_tag(e.get("depth")?))
        .map(|s| s.to_uppercase())
        .collect();

    let variant_upper: Vec<String> = dsl_variants.iter().map(|v| v.to_uppercase()).collect();
    assert_eq!(
        timeout_depths, variant_upper,
        "fermi_timeouts should have exactly one entry per FermiDepth variant in order"
    );
}

// ============================================================================
// G2: Build targets ↔ FermiDepth parity
// ============================================================================

#[test]
fn build_targets_has_one_test_target_per_fermi_depth() {
    let variants = dsl_fermi_depth_variants();
    let targets = dsl_meta_targets();

    // Fermi-depth test targets are named "test-{depth_lower}" with a
    // GUNBC_TEST_MAX_COST prefix. Filter to only those, excluding aliases
    // (test-small etc.), special-purpose targets (test-integration, test-external),
    // and the bare "test" alias.
    let fermi_depth_lower: Vec<String> = variants.iter().map(|v| v.to_lowercase()).collect();
    let test_target_names: Vec<String> = targets
        .iter()
        .filter_map(|t| t.get("name")?.as_str().map(String::from))
        .filter(|name| {
            if let Some(suffix) = name.strip_prefix("test-") {
                fermi_depth_lower.contains(&suffix.to_string())
            } else {
                false
            }
        })
        .collect();

    // Each FermiDepth variant should have a corresponding test-{variant_lower} target.
    let expected: Vec<String> = variants
        .iter()
        .map(|v| format!("test-{}", v.to_lowercase()))
        .collect();

    assert_eq!(
        test_target_names, expected,
        "build_targets.dag should have exactly one test-{{depth}} target per FermiDepth variant"
    );
}

#[test]
fn build_targets_test_prefixes_match_fermi_cost_strings() {
    let targets = dsl_meta_targets();

    // Verify command_prefix values match FermiCost::as_str() format.
    // Only check Fermi-depth targets (test-xs through test-xl), not special targets.
    let fermi_target_names: std::collections::BTreeSet<&str> =
        ["test-xs", "test-s", "test-m", "test-l", "test-xl"]
            .into_iter()
            .collect();
    let prefixes: BTreeMap<String, String> = targets
        .iter()
        .filter_map(|t| {
            let name = t.get("name")?.as_str()?;
            if !fermi_target_names.contains(name) {
                return None;
            }
            let prefix = t.get("command_prefix").and_then(|v| v.as_str())?;
            let cost = prefix.strip_prefix("GUNBC_TEST_MAX_COST=")?;
            Some((name.to_string(), cost.to_string()))
        })
        .collect();

    let expected: BTreeMap<String, String> = [
        ("test-xs", "XS"),
        ("test-s", "S"),
        ("test-m", "M"),
        ("test-l", "L"),
        ("test-xl", "XL"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    assert_eq!(
        prefixes, expected,
        "test target command_prefix values should match FermiCost string representation"
    );
}
