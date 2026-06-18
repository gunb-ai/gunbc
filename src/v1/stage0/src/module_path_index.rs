//! Qualified module path → source file path index (DESIGN.md §3 single authority).
//!
//! **SCAFFOLD (DESIGN.md §7)** — thin adapter over `cli_run::build_module_path_index`.
//!
//! Hand-Rust scaffold receipt (P5 / §7):
//! - **Deleted wrong scaffold:** `src/v2/lens/filepath_literal.dag` (text-scan census — contradicted
//!   QN-in design; withdrawn in-branch before merge).
//! - **Lens census shrink:** `extdeps_shape_transport_policy.dag` retired `filesystem_read`,
//!   `FilesystemReadResult`, and every `path: String` fact/violation row; corpus witnesses now use
//!   structural `*_for_qualified_name` projection (no filepath literals in witnesses).
//! - **This module (+ seed bridge):** `module_path_index.rs` (adapter only); pub
//!   `cli_run::build_module_path_index` (+47 lines, reuses existing walk); eight
//!   `*_for_qualified_name` builtins in `v1_interpreter.rs` / `04_method.dag` /
//!   `v1_compiler_infer_method.rs`; `source_path_for_module_path` is Rust-internal (undispatched).
//! - **Dissolve-on (ROADMAP):** `ROADMAP.md` Now → Lane 3a `SourceRootIngest` (#5126) — delete this
//!   module and the QN builtins when the overlay owns module-path lookup in `.dag`.

use std::collections::HashMap;

pub fn workspace_root() -> std::path::PathBuf {
    crate::cli_run::workspace_root()
}

fn default_source_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("dsl").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

/// module_path (serialized QualifiedName) → repo-relative `.dag` path.
pub fn build_module_path_index() -> HashMap<String, String> {
    crate::cli_run::build_module_path_index(&default_source_roots())
}

pub fn source_path_for_module_path(module_path: String) -> String {
    let index = build_module_path_index();
    index
        .get(&module_path)
        .cloned()
        .unwrap_or_else(|| panic!("module_path_index: unknown module path '{module_path}'"))
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
