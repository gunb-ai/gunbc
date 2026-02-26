//! FC-EG2: Extern func count ratchet gate.
//!
//! Prevents silent addition of `extern func` declarations in `.dag` files
//! or extern implementations in `extern_impls.rs`. The count can only go down
//! (as externs are migrated to pure DSL). Any decrease should update the
//! baselines below.

use gunbc_dag::extern_impls::all_extern_symbols;
use std::path::PathBuf;

/// Current baseline: number of `extern func` declarations in all `.dag` files.
const EXTERN_FUNC_DECL_BASELINE: usize = 2;

/// Current baseline: number of extern implementations in `all_extern_symbols()`.
/// Decreased from 8 → 6: allowlist + lint_policy migrated to DSL evaluation (FC-P6-d).
const EXTERN_IMPL_BASELINE: usize = 6;

#[test]
#[allow(clippy::disallowed_methods)]
fn extern_func_declaration_count_ratchet() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let mut count = 0usize;
    let mut locations = Vec::new();

    for entry in walkdir(&dsl_root) {
        let path = entry.as_path();
        if path.extension().is_some_and(|e| e == "dag") {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            for (line_no, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("extern func ") {
                    count += 1;
                    let rel = path.strip_prefix(&dsl_root).unwrap_or(path);
                    locations.push(format!("  {}:{}: {}", rel.display(), line_no + 1, trimmed));
                }
            }
        }
    }

    assert!(
        count <= EXTERN_FUNC_DECL_BASELINE,
        "extern func count increased from {} to {}! New extern func declarations \
         are prohibited — implement in pure DSL instead.\nLocations:\n{}",
        EXTERN_FUNC_DECL_BASELINE,
        count,
        locations.join("\n"),
    );

    // Also detect when the count decreases so the baseline can be updated.
    if count < EXTERN_FUNC_DECL_BASELINE {
        panic!(
            "extern func count decreased from {} to {} — update \
             EXTERN_FUNC_DECL_BASELINE in extern_ratchet.rs!\nRemaining:\n{}",
            EXTERN_FUNC_DECL_BASELINE,
            count,
            locations.join("\n"),
        );
    }
}

#[test]
fn extern_impl_count_ratchet() {
    let symbols = all_extern_symbols();
    let count = symbols.len();

    assert!(
        count <= EXTERN_IMPL_BASELINE,
        "extern impl count increased from {} to {}! New extern implementations \
         are prohibited — implement in pure DSL instead.\nSymbols: {:?}",
        EXTERN_IMPL_BASELINE,
        count,
        symbols,
    );

    if count < EXTERN_IMPL_BASELINE {
        panic!(
            "extern impl count decreased from {} to {} — update \
             EXTERN_IMPL_BASELINE in extern_ratchet.rs!\nRemaining: {:?}",
            EXTERN_IMPL_BASELINE, count, symbols,
        );
    }
}

#[test]
fn extern_symbols_are_sorted() {
    let symbols = all_extern_symbols();
    let mut sorted = symbols.to_vec();
    sorted.sort();
    assert_eq!(
        symbols,
        &sorted[..],
        "all_extern_symbols() must be sorted for auditability"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn every_extern_impl_has_dag_declaration() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let symbols = all_extern_symbols();

    // Collect all extern func declarations from .dag files.
    let mut declared: Vec<(String, String)> = Vec::new();
    for entry in walkdir(&dsl_root) {
        let path = entry.as_path();
        if path.extension().is_some_and(|e| e == "dag") {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            let module = extract_module_name(&content);
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("extern func ") {
                    if let Some(name) = rest.split('(').next() {
                        declared.push((module.clone(), name.trim().to_string()));
                    }
                }
            }
        }
    }

    // Extern impls that are "shadow" impls (the .dag file has a regular fn body
    // but the extern impl overrides it at resolve time) don't need `extern func`
    // declarations. These are the entries where the .dag has fn bodies, not
    // `extern func` declarations.
    let shadow_overrides: Vec<(&str, &str)> = vec![
        ("tools.bootstrap", "render_bootstrap_gitignore"),
        ("tools.bootstrap", "render_bootstrap_makefile"),
        ("tools.gist", "build_snapshot_content"),
        ("tools.makegen", "discover_tools"),
        ("tools.pragma", "render_clippy_toml"),
        // render_disallowed_methods_allowlist and render_pragma_lint_policy
        // migrated to DSL evaluation (FC-P6-d) — no longer in all_extern_symbols().
    ];

    let mut missing = Vec::new();
    for &(module, name) in symbols {
        if shadow_overrides.contains(&(module, name)) {
            continue;
        }
        let has_decl = declared.iter().any(|(m, n)| m == module && n == name);
        if !has_decl {
            missing.push(format!("  {module}::{name}"));
        }
    }

    // render_tree is the only true extern func (needs recursive algorithm).
    // All others are shadow overrides of DSL fn bodies.
    assert!(
        missing.is_empty(),
        "extern implementations without `extern func` declaration (and not in shadow list):\n{}",
        missing.join("\n"),
    );
}

// Helpers ────────────────────────────────────────────────────────────────────

#[allow(clippy::disallowed_methods)]
fn walkdir(root: &std::path::Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    result.push(path);
                }
            }
        }
    }
    result.sort();
    result
}

fn extract_module_name(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            return rest.trim().to_string();
        }
    }
    String::new()
}
