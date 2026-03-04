//! FC-EG3: `push_str` boundary gate.
//!
//! Enforces that string-building operations (`push_str`) appear only in
//! boundary code (transport, codegen, emit, CLI scaffolding). Non-boundary
//! code must use structured types and let the boundary handle serialization.
//!
//! The baseline count is a ratchet: it can only decrease as anemic rendering
//! is migrated to DSL. Any increase is a hard error.

use std::path::{Path, PathBuf};

/// Directories where `push_str` is allowed (their job is string generation).
const ALLOWED_DIRS: &[&str] = &[
    // Code generation (inherently produces strings)
    "core/codegen/",
    // DSL emit pipeline
    "core/daglang/daglang-emit/",
    // DSL CLI tools
    "core/daglang/daglang-cli/",
    // DSL lowerer (eval, expression rendering)
    "core/daglang/daglang-lower/",
    // DSL parser
    "core/daglang/daglang-syntax/",
    // Transport layer (canonical I/O boundary)
    "lib/transport/",
    // Tool wrappers (clippy config, deps toml)
    "lib/tools/",
    // Cloud ops rendering
    "lib/cloud-ops/",
    // GCP ops
    "lib/gcp-ops/",
    // Primitive data formatting
    "lib/primitives/",
    // Makegen (Rust-side rendering, migrating to DSL)
    "gunbc-dag/src/makegen/",
    // Extern ops (recursive/inventory-backed extern function handlers)
    "gunbc-dag/src/extern_ops.rs",
    // Binary entrypoints
    "gunbc-dag/src/bin/",
    // Test generation DAG (mock_interpreter + profile_discovery relocated to core/codegen in B5)
    "gunbc-dag/src/testgen_dag/",
    // Workflow orchestration
    "gunbc-dag/src/workflow/",
    // DSL registry
    "gunbc-dag/src/dsl_registry.rs",
];

/// Current baseline: total `push_str` occurrences in non-boundary Rust files.
/// Update this number downward when anemic rendering is migrated to DSL.
const NON_BOUNDARY_PUSH_STR_BASELINE: usize = 102; // C24: StringInterpolateOp uses push_str for template assembly

#[test]
#[allow(clippy::disallowed_methods)]
fn push_str_boundary_ratchet() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut violations: Vec<(String, usize)> = Vec::new();
    let mut total = 0usize;

    for entry in walkdir_rs(&workspace_root) {
        let path = entry.as_path();
        if path.extension().is_some_and(|e| e == "rs") {
            let rel = path.strip_prefix(&workspace_root).unwrap_or(path);
            let rel_str = rel.to_string_lossy();

            // Skip test files (they legitimately use push_str for test assertions).
            if rel_str.contains("/tests/") || rel_str.contains("/test_") {
                continue;
            }
            // Skip target directory.
            if rel_str.starts_with("target/") {
                continue;
            }

            // Check if in an allowed directory.
            let in_allowed = ALLOWED_DIRS.iter().any(|d| rel_str.starts_with(d));
            if in_allowed {
                continue;
            }

            let content = std::fs::read_to_string(path).unwrap_or_default();
            let count = content.matches(".push_str(").count();
            if count > 0 {
                total += count;
                violations.push((rel_str.to_string(), count));
            }
        }
    }

    violations.sort();

    assert!(
        total <= NON_BOUNDARY_PUSH_STR_BASELINE,
        "push_str usage increased in non-boundary code from {} to {}! \
         New push_str should go in boundary code (transport, codegen, emit) \
         or use structured types in DSL.\nViolations:\n{}",
        NON_BOUNDARY_PUSH_STR_BASELINE,
        total,
        violations
            .iter()
            .map(|(f, c)| format!("  {f}: {c}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    if total < NON_BOUNDARY_PUSH_STR_BASELINE {
        panic!(
            "push_str count decreased from {} to {} — update \
             NON_BOUNDARY_PUSH_STR_BASELINE in boundary_gate.rs!\nRemaining:\n{}",
            NON_BOUNDARY_PUSH_STR_BASELINE,
            total,
            violations
                .iter()
                .map(|(f, c)| format!("  {f}: {c}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
}

#[allow(clippy::disallowed_methods)]
fn walkdir_rs(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip target and .git directories.
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name == "target" || name == ".git" || name == ".claude" {
                        continue;
                    }
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
