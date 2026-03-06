//! FC-P6-c: Pragma DSL parity tests.
//!
//! Validate DSL rendering against canonical DSL policy data declarations.
//! The legacy Rust policy mirror has been deleted.

use std::collections::HashMap;
use std::path::PathBuf;

use daglang_driver::{compile_from_context, DriverContext};

/// Verify DSL `derive_disallowed_methods_allowlist()` produces entries that
/// match `allowlist_patterns` from `config/clippy_policy.dag`.
#[test]
fn allowlist_dsl_matches_policy_data() {
    use gunbc_app::pragma::dsl_render::render_allowlist_via_dsl;

    let data_values = compile_clippy_policy_data();
    let dsl_output = render_allowlist_via_dsl();

    let expected_entries = expected_allowlist_entries(&data_values);
    let dsl_entries = extract_allowlist_entries(&dsl_output);

    assert_eq!(
        expected_entries.len(),
        dsl_entries.len(),
        "entry count mismatch:\nExpected entries: {expected_entries:?}\nDSL entries: {dsl_entries:?}"
    );

    for (i, (expected, dsl)) in expected_entries.iter().zip(dsl_entries.iter()).enumerate() {
        assert_eq!(
            expected.pattern, dsl.pattern,
            "pattern mismatch at entry {i}: expected={}, dsl={}",
            expected.pattern, dsl.pattern
        );
        assert_eq!(
            expected.rationale, dsl.rationale,
            "rationale mismatch at entry {i}: expected={}, dsl={}",
            expected.rationale, dsl.rationale
        );
    }
}

/// Verify DSL `derive_pragma_lint_policy()` produces sections that match
/// canonical `dead_code_allowances` / `pragma_allow_lints` data.
#[test]
fn lint_policy_dsl_matches_policy_data() {
    use gunbc_app::pragma::dsl_render::render_lint_policy_via_dsl;

    let data_values = compile_clippy_policy_data();
    let dsl_output = render_lint_policy_via_dsl();

    let expected_dc = expected_dead_code_paths(&data_values);
    let dsl_dc = extract_section_entries(&dsl_output, "[allow.dead_code]", "[allow.lints]");

    assert_eq!(
        expected_dc.len(),
        dsl_dc.len(),
        "dead_code entry count mismatch:\nExpected: {expected_dc:?}\nDSL: {dsl_dc:?}"
    );
    for (i, (r, d)) in expected_dc.iter().zip(dsl_dc.iter()).enumerate() {
        assert_eq!(r, d, "dead_code path mismatch at entry {i}");
    }

    let expected_lints = expected_allow_lints(&data_values);
    let dsl_lints = extract_section_entries(&dsl_output, "[allow.lints]", "");

    assert_eq!(
        expected_lints.len(),
        dsl_lints.len(),
        "allow_lints count mismatch:\nExpected: {expected_lints:?}\nDSL: {dsl_lints:?}"
    );
    for (i, (r, d)) in expected_lints.iter().zip(dsl_lints.iter()).enumerate() {
        assert_eq!(r, d, "lint mismatch at entry {i}");
    }
}

// DELETED: clippy_toml_dsl_produces_valid_output
// Tracked as FC-CF5 in tasks.md — blocked on recursive types / sum type variant
// tags in data declarations. Re-add when FC-CF5 is complete.

// ── Helpers ──────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
struct AllowlistEntry {
    pattern: String,
    rationale: String,
}

fn extract_allowlist_entries(output: &str) -> Vec<AllowlistEntry> {
    let mut entries = Vec::new();
    let mut current_rationale = String::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("# Generated")
            || trimmed.starts_with("# DO NOT")
            || trimmed.starts_with("# Allowed path")
            || trimmed.starts_with("# Format:")
            || trimmed.starts_with("# Note:")
            || trimmed == "#"
        {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            current_rationale = rest.to_string();
        } else if !trimmed.starts_with('#') {
            entries.push(AllowlistEntry {
                pattern: trimmed.to_string(),
                rationale: current_rationale.clone(),
            });
        }
    }
    entries
}

fn extract_section_entries(output: &str, section_header: &str, next_header: &str) -> Vec<String> {
    let mut in_section = false;
    let mut entries = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed == section_header {
            in_section = true;
            continue;
        }
        if !next_header.is_empty() && trimmed == next_header {
            break;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            entries.push(trimmed.to_string());
        }
    }
    entries
}

fn compile_clippy_policy_data() -> HashMap<String, serde_json::Value> {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let dag_file = dsl_root.join("config/clippy_policy.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file),
    };
    let output = compile_from_context(&context).expect("clippy_policy.dag should compile");
    output.data_values
}

fn expected_allowlist_entries(
    data_values: &HashMap<String, serde_json::Value>,
) -> Vec<AllowlistEntry> {
    data_values["allowlist_patterns"]
        .as_array()
        .expect("allowlist_patterns should be an array")
        .iter()
        .map(|item| AllowlistEntry {
            pattern: item["pattern"]
                .as_str()
                .expect("allowlist pattern should be string")
                .to_string(),
            rationale: item["rationale"]
                .as_str()
                .expect("allowlist rationale should be string")
                .to_string(),
        })
        .collect()
}

fn expected_dead_code_paths(data_values: &HashMap<String, serde_json::Value>) -> Vec<String> {
    data_values["dead_code_allowances"]
        .as_array()
        .expect("dead_code_allowances should be an array")
        .iter()
        .map(|item| {
            item["fallback_path"]
                .as_str()
                .expect("fallback_path should be string")
                .to_string()
        })
        .collect()
}

fn expected_allow_lints(data_values: &HashMap<String, serde_json::Value>) -> Vec<String> {
    data_values["pragma_allow_lints"]
        .as_array()
        .expect("pragma_allow_lints should be an array")
        .iter()
        .map(|item| item.as_str().expect("lint should be string").to_string())
        .collect()
}
