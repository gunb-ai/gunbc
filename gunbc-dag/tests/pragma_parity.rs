//! FC-P6-c: Pragma DSL parity tests.
//!
//! Compares Rust rendering output against DSL evaluate_fn_body() output
//! for all three pragma renders. These tests gate the Rust → DSL migration.

/// Verify DSL `derive_disallowed_methods_allowlist()` produces output matching
/// the Rust `render_disallowed_methods_allowlist()` (modulo path resolution).
#[test]
fn allowlist_dsl_matches_rust() {
    use gunbc_dag::policy::pragma::render_disallowed_methods_allowlist;
    use gunbc_dag::pragma::dsl_render::render_allowlist_via_dsl;

    let rust_output = render_disallowed_methods_allowlist();
    let dsl_output = render_allowlist_via_dsl();

    // Both outputs should have the same allowlist entries (pattern + rationale).
    // The header may differ slightly (Rust uses StructuredRenderer, DSL builds string).
    // Compare entry-by-entry.
    let rust_entries = extract_allowlist_entries(&rust_output);
    let dsl_entries = extract_allowlist_entries(&dsl_output);

    assert_eq!(
        rust_entries.len(),
        dsl_entries.len(),
        "entry count mismatch:\nRust entries: {rust_entries:?}\nDSL entries: {dsl_entries:?}"
    );

    for (i, (rust, dsl)) in rust_entries.iter().zip(dsl_entries.iter()).enumerate() {
        assert_eq!(
            rust.pattern, dsl.pattern,
            "pattern mismatch at entry {i}: rust={}, dsl={}",
            rust.pattern, dsl.pattern
        );
        assert_eq!(
            rust.rationale, dsl.rationale,
            "rationale mismatch at entry {i}: rust={}, dsl={}",
            rust.rationale, dsl.rationale
        );
    }
}

/// Verify DSL `derive_pragma_lint_policy()` produces output matching
/// the Rust `render_pragma_lint_policy()`.
#[test]
fn lint_policy_dsl_matches_rust() {
    use gunbc_dag::policy::pragma::render_pragma_lint_policy;
    use gunbc_dag::pragma::dsl_render::render_lint_policy_via_dsl;

    let rust_output = render_pragma_lint_policy();
    let dsl_output = render_lint_policy_via_dsl();

    // Extract dead_code paths and allow_lints from both outputs.
    let rust_dc = extract_section_entries(&rust_output, "[allow.dead_code]", "[allow.lints]");
    let dsl_dc = extract_section_entries(&dsl_output, "[allow.dead_code]", "[allow.lints]");

    assert_eq!(
        rust_dc.len(),
        dsl_dc.len(),
        "dead_code entry count mismatch:\nRust: {rust_dc:?}\nDSL: {dsl_dc:?}"
    );
    for (i, (r, d)) in rust_dc.iter().zip(dsl_dc.iter()).enumerate() {
        assert_eq!(r, d, "dead_code path mismatch at entry {i}");
    }

    let rust_lints = extract_section_entries(&rust_output, "[allow.lints]", "");
    let dsl_lints = extract_section_entries(&dsl_output, "[allow.lints]", "");

    assert_eq!(
        rust_lints.len(),
        dsl_lints.len(),
        "allow_lints count mismatch:\nRust: {rust_lints:?}\nDSL: {dsl_lints:?}"
    );
    for (i, (r, d)) in rust_lints.iter().zip(dsl_lints.iter()).enumerate() {
        assert_eq!(r, d, "lint mismatch at entry {i}");
    }
}

/// Verify DSL `derive_clippy_toml()` is blocked on sum type variant tags
/// in data declarations. The `_variant` field is not preserved when
/// `ExemptionScope` values are serialized to JSON during `build_data_values()`.
/// This test documents the blocker — once resolved, remove the #[ignore].
#[test]
#[ignore = "blocked on sum type variant tags in data declarations (FC-CF5 prerequisite)"]
fn clippy_toml_dsl_produces_valid_output() {
    use gunbc_dag::pragma::dsl_render::render_clippy_toml_via_dsl;

    let dsl_output = render_clippy_toml_via_dsl();

    assert!(
        dsl_output.contains("disallowed-methods"),
        "should contain disallowed-methods section"
    );
    assert!(
        dsl_output.contains("disallowed-types"),
        "should contain disallowed-types section"
    );
}

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
        if trimmed.is_empty() || trimmed.starts_with("# Generated")
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
