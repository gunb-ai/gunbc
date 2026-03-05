//! FC-EG1: Import-direction lint.
//!
//! Enforces that DSL module imports respect the layered dependency hierarchy.
//! Lower layers must never import from higher layers. The layer ordering is:
//!
//!   std(0) → extdeps(1) → config(2) → interfaces(3) → services(4)
//!   → shared(5) → infra(6) → cloud(7) → funcs(8) → tools(9)
//!   → workflows(10) → pipelines(11) → profiles(12)
//!
//! Examples (exempt from the lint).

use std::path::{Path, PathBuf};

fn layer_rank(module_prefix: &str) -> Option<u8> {
    // SDLC provider adapters live under extdeps/ for migration reasons but
    // behave as service-layer implementations that may depend on interfaces.
    if module_prefix.starts_with("extdeps.sdlc.providers") {
        return Some(4);
    }

    // Extract the top-level module segment.
    let top = module_prefix.split('.').next().unwrap_or(module_prefix);
    match top {
        "std" => Some(0),
        "extdeps" => Some(1),
        "config" => Some(2),
        "interfaces" => Some(3),
        "services" => Some(4),
        "shared" => Some(5),
        "infra" => Some(6),
        "cloud" => Some(7),
        "funcs" => Some(8),
        "tools" => Some(9),
        "workflows" => Some(10),
        "pipelines" => Some(11),
        "profiles" => Some(12),
        _ => None, // Unknown — skip (e.g., workspace module)
    }
}

/// Known violations — pre-existing architecture debt. These can only decrease.
/// Each entry is (file, import_path). Adding new entries is prohibited.
const KNOWN_VIOLATIONS: &[(&str, &str)] = &[
    // std.patterns defines compositional patterns (credential_chain, oidc, etc.)
    // that inherently reference service operations. Future: move to a higher layer
    // or split service-specific patterns into a separate module.
    ("std/patterns.dag", "extdeps.cloud.gcp.secret_manager"),
    ("std/patterns.dag", "extdeps.cloud.gcp.iam"),
    ("std/patterns.dag", "extdeps.cloud.gcp.sts"),
    ("std/patterns.dag", "extdeps.shell"),
    // sdlc_stages invokes design tools as workflow steps.
    ("funcs/sdlc_stages.dag", "tools.design"),
];

struct ImportViolation {
    file: String,
    line_no: usize,
    module_layer: String,
    imported_layer: String,
    import_line: String,
}

impl std::fmt::Display for ImportViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  {}:{}: {} (layer {}) imports from {} (layer {}): {}",
            self.file,
            self.line_no,
            self.module_layer,
            layer_rank(&self.module_layer).unwrap_or(99),
            self.imported_layer,
            layer_rank(&self.imported_layer).unwrap_or(99),
            self.import_line,
        )
    }
}

#[test]
#[allow(clippy::disallowed_methods)]
fn import_direction_lint() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let mut violations = Vec::new();

    for entry in walkdir_dag(&dsl_root) {
        let path = entry.as_path();
        let rel = path.strip_prefix(&dsl_root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();

        // Exempt examples from the lint.
        if rel_str.starts_with("examples/") || rel_str.starts_with("examples\\") {
            continue;
        }

        let content = std::fs::read_to_string(path).unwrap_or_default();

        // Extract the module's own layer from `module` declaration.
        let module_name = extract_module(&content);
        let Some(own_rank) = layer_rank(&module_name) else {
            continue;
        };

        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let import_path = if let Some(rest) = trimmed.strip_prefix("import ") {
                // `import foo.bar { X, Y }` or `import foo.bar`
                rest.split(|c: char| c == '{' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim()
            } else {
                continue;
            };

            if import_path.is_empty() {
                continue;
            }

            if let Some(import_rank) = layer_rank(import_path) {
                if import_rank > own_rank {
                    violations.push(ImportViolation {
                        file: rel_str.to_string(),
                        line_no: line_no + 1,
                        module_layer: module_name.clone(),
                        imported_layer: import_path.to_string(),
                        import_line: trimmed.to_string(),
                    });
                }
            }
        }
    }

    // Separate known violations from new ones.
    let (known, new): (Vec<_>, Vec<_>) = violations.iter().partition(|v| {
        KNOWN_VIOLATIONS
            .iter()
            .any(|(f, i)| v.file == *f && v.imported_layer.starts_with(i))
    });

    // New violations are hard errors.
    assert!(
        new.is_empty(),
        "NEW import direction violations (lower layer importing from higher layer):\n{}\n\n\
         Known violations (allowlisted): {}",
        new.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        known.len(),
    );

    // Detect when known violations are fixed so the allowlist can be updated.
    let stale: Vec<_> = KNOWN_VIOLATIONS
        .iter()
        .filter(|(f, i)| {
            !violations
                .iter()
                .any(|v| v.file == *f && v.imported_layer.starts_with(i))
        })
        .collect();
    assert!(
        stale.is_empty(),
        "stale KNOWN_VIOLATIONS entries (these violations have been fixed!) — \
         remove from allowlist in import_direction.rs:\n  {:?}",
        stale,
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn all_dag_files_have_module_declaration() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let mut missing = Vec::new();

    for entry in walkdir_dag(&dsl_root) {
        let path = entry.as_path();
        let rel = path.strip_prefix(&dsl_root).unwrap_or(path);
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let module_name = extract_module(&content);
        if module_name.is_empty() {
            missing.push(rel.display().to_string());
        }
    }

    assert!(
        missing.is_empty(),
        "`.dag` files without `module` declaration:\n  {}",
        missing.join("\n  "),
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn module_names_match_file_paths() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let mut mismatches = Vec::new();

    for entry in walkdir_dag(&dsl_root) {
        let path = entry.as_path();
        let rel = path.strip_prefix(&dsl_root).unwrap_or(path);
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let module_name = extract_module(&content);
        if module_name.is_empty() {
            continue;
        }

        // Expected module name: convert path separators to dots, strip .dag extension.
        let expected = rel
            .with_extension("")
            .to_string_lossy()
            .replace(['/', '\\'], ".");

        if module_name != expected {
            mismatches.push(format!(
                "  {}: declared `module {module_name}`, expected `module {expected}`",
                rel.display(),
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "module names don't match file paths:\n{}",
        mismatches.join("\n"),
    );
}

// Helpers ────────────────────────────────────────────────────────────────────

fn extract_module(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

#[allow(clippy::disallowed_methods)]
fn walkdir_dag(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "dag") {
                    result.push(path);
                }
            }
        }
    }
    result.sort();
    result
}
