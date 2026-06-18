//! Qualified module path → source file path index (DESIGN.md §3 single authority).
//!
//! **SCAFFOLD (DESIGN.md §7)** — reuses the same walk semantics as `cli_run::build_module_index`.
//! Shrink target: delete when `SourceRootIngest` overlay owns lookup in `.dag`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn extract_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return Some(trimmed["module ".len()..].trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

fn collect_dag_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            files.push(path);
        }
    }
}

fn rel_path_from_workspace(path: &Path) -> String {
    let ws = workspace_root();
    path.strip_prefix(&ws)
        .unwrap_or_else(|_| {
            panic!(
                "module_path_index: path {} is not under workspace {}",
                path.display(),
                ws.display()
            )
        })
        .to_string_lossy()
        .replace('\\', "/")
}

/// module_path (serialized QualifiedName) → repo-relative `.dag` path.
pub fn build_module_path_index() -> HashMap<String, String> {
    let ws = workspace_root();
    let mut index = HashMap::new();
    for root in ["dsl", "src/v2"] {
        let root_path = ws.join(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut dag_files = Vec::new();
        collect_dag_files(&root_path, &mut dag_files);
        for path in dag_files {
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("module_path_index: failed to read {}: {e}", path.display())
            });
            if let Some(module_path) = extract_module_path(&content) {
                let rel = rel_path_from_workspace(&path);
                if let Some(existing) = index.insert(module_path.clone(), rel.clone()) {
                    panic!(
                        "module_path_index: duplicate module path '{module_path}': {existing} vs {rel}"
                    );
                }
            }
        }
    }
    index
}

pub fn source_path_for_module_path(module_path: String) -> String {
    let index = build_module_path_index();
    index.get(&module_path).cloned().unwrap_or_else(|| {
        panic!("module_path_index: unknown module path '{module_path}'")
    })
}

pub fn qualified_name_value_to_module_path(value: &crate::v1_interpreter::Value) -> String {
    crate::v1_interpreter::qualified_name_value_to_module_path(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_build_resolves_by_module_path_not_directory_nickname() {
        let path = source_path_for_module_path("extdeps.cargo_build".to_string());
        assert_eq!(path, "dsl/extdeps/rust/cargo_build.dag");
    }

    #[test]
    fn git_module_resolves() {
        let path = source_path_for_module_path("extdeps.git".to_string());
        assert_eq!(path, "dsl/extdeps/git/git.dag");
    }
}
