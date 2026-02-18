//! Built-in resource definitions.
//!
//! These are shared resource declarations used across the repo to ensure
//! hashing inputs stay centralized and consistent.

use super::{InputPattern, ResourceDef};
use crate::{ResourceId, WorkspaceLayout};
use std::sync::OnceLock;

/// Fallback input globs used when workspace layout discovery is unavailable.
pub const CODEGEN_INPUT_GLOBS: &[&str] = &["core/codegen/src/**/*.rs", "core/ir/src/**/*.rs"];

/// Fallback individual files used when workspace layout discovery is unavailable.
pub const CODEGEN_INPUT_FILES: &[&str] = &["core/codegen/Cargo.toml", "core/ir/Cargo.toml"];

static DERIVED_CODEGEN_INPUTS: OnceLock<(Vec<String>, Vec<String>)> = OnceLock::new();

/// Derive codegen input globs/files from workspace crate locations.
pub fn codegen_input_patterns() -> (Vec<String>, Vec<String>) {
    DERIVED_CODEGEN_INPUTS
        .get_or_init(derive_codegen_input_patterns)
        .clone()
}

fn derive_codegen_input_patterns() -> (Vec<String>, Vec<String>) {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata());
    let Ok(layout) = layout else {
        return (
            CODEGEN_INPUT_GLOBS.iter().map(|s| s.to_string()).collect(),
            CODEGEN_INPUT_FILES.iter().map(|s| s.to_string()).collect(),
        );
    };

    let mut globs = Vec::new();
    let mut files = Vec::new();

    for crate_name in ["gunbc-codegen", "gunbc-ir"] {
        let Some(crate_dir) = layout.crate_dir(crate_name) else {
            continue;
        };
        let rel = layout
            .relative_path(&layout.workspace_root, crate_dir)
            .to_string_lossy()
            .replace('\\', "/");
        globs.push(format!("{rel}/src/**/*.rs"));
        files.push(format!("{rel}/Cargo.toml"));
    }

    if globs.is_empty() || files.is_empty() {
        return (
            CODEGEN_INPUT_GLOBS.iter().map(|s| s.to_string()).collect(),
            CODEGEN_INPUT_FILES.iter().map(|s| s.to_string()).collect(),
        );
    }

    globs.sort();
    globs.dedup();
    files.sort();
    files.dedup();
    (globs, files)
}

/// Resource definition for codegen outputs (`build:generated_cli`).
pub fn codegen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(ResourceId::build("generated_cli"));
    let (globs, files) = codegen_input_patterns();

    for pattern in globs {
        def = def.with_input(InputPattern::glob(pattern));
    }
    for path in files {
        def = def.with_input(InputPattern::file(path));
    }

    // Hash rustc version directly via command output instead of relying on
    // a RUSTC_VERSION env var that defaults to empty when unset.
    def = def.with_input(InputPattern::command_output("rustc", &["--version"]));

    def
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_codegen_input_patterns_include_core_codegen_and_ir() {
        let (globs, files) = codegen_input_patterns();
        assert!(
            globs.iter().any(|g| g == "core/codegen/src/**/*.rs"),
            "expected codegen source glob, got {globs:?}"
        );
        assert!(
            globs.iter().any(|g| g == "core/ir/src/**/*.rs"),
            "expected ir source glob, got {globs:?}"
        );
        assert!(
            files.iter().any(|f| f == "core/codegen/Cargo.toml"),
            "expected codegen manifest path, got {files:?}"
        );
        assert!(
            files.iter().any(|f| f == "core/ir/Cargo.toml"),
            "expected ir manifest path, got {files:?}"
        );
    }
}
