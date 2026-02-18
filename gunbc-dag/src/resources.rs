//! Repo-specific resource definitions for gunbc-dag.

use gunbc_ir::resource::{codegen_resource_def, InputPattern, ResourceDef, ResourceScope};
use gunbc_ir::{ResourceId, WorkspaceLayout};
use std::sync::OnceLock;

// Canonical build resource names for repo-level composition.
pub const BUILD_RESOURCE_GENERATED_CLI: &str = "generated_cli";
pub const BUILD_RESOURCE_GENERATED_TESTS: &str = "generated_tests";
pub const BUILD_RESOURCE_PRAGMA_CONFIG: &str = "pragma_config";
pub const BUILD_RESOURCE_COMPILED_CODE: &str = "compiled_code";
pub const BUILD_RESOURCE_VERIFIED_ARTIFACTS: &str = "verified_artifacts";
pub const BUILD_RESOURCE_DEPS_CONFIG: &str = "deps_config";
pub const BUILD_RESOURCE_MAKEFILE: &str = "makefile";
pub const BUILD_RESOURCE_GITIGNORE: &str = "gitignore";

// Canonical output paths for generated repo artifacts.
pub const MAKEFILE_OUTPUT_PATH: &str = "Makefile";
pub const GITIGNORE_OUTPUT_PATH: &str = ".gitignore";
pub const DEPS_CONFIG_OUTPUT_PATH: &str = "deps.toml";

/// Shared repo source globs that affect generated repo artifacts.
pub const REPO_SOURCE_INPUT_GLOBS: &[&str] =
    &["gunbc-dag/src/**/*.rs", "core/**/*.rs", "lib/**/*.rs"];

/// Shared config files that affect generated repo artifacts.
pub const REPO_CONFIG_INPUT_FILES: &[&str] = &["Cargo.toml", "gunbc-dag/Cargo.toml"];

/// Input globs that affect testgen outputs.
pub const TESTGEN_INPUT_GLOBS: &[&str] = &[
    "gunbc-dag/src/**/*.rs",
    "core/ir/src/**/*.rs",
    "lib/**/*.rs",
];

static DERIVED_REPO_SOURCE_GLOBS: OnceLock<Vec<String>> = OnceLock::new();
static DERIVED_REPO_CONFIG_FILES: OnceLock<Vec<String>> = OnceLock::new();
static DERIVED_TESTGEN_GLOBS: OnceLock<Vec<String>> = OnceLock::new();

pub fn generated_cli_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_GENERATED_CLI)
}

pub fn generated_tests_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_GENERATED_TESTS)
}

pub fn pragma_config_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_PRAGMA_CONFIG)
}

pub fn compiled_code_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_COMPILED_CODE)
}

pub fn verified_artifacts_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_VERIFIED_ARTIFACTS)
}

pub fn deps_config_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_DEPS_CONFIG)
}

pub fn makefile_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_MAKEFILE)
}

pub fn gitignore_resource_id() -> ResourceId {
    ResourceId::build(BUILD_RESOURCE_GITIGNORE)
}

fn with_repo_inputs(mut def: ResourceDef) -> ResourceDef {
    for pattern in repo_source_input_globs() {
        def = def.with_input(InputPattern::glob(pattern));
    }
    for path in repo_config_input_files() {
        def = def.with_input(InputPattern::file(path));
    }

    // Toolchain version changes can affect generated command snippets.
    def.with_input(InputPattern::command_output("rustc", &["--version"]))
}

/// Resource definition for testgen outputs (`build:generated_tests`).
pub fn testgen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(generated_tests_resource_id());

    for pattern in testgen_input_globs() {
        def = def.with_input(InputPattern::glob(pattern));
    }

    // Testgen depends on codegen output key.
    let codegen_id = codegen_resource_def().id;
    def = def.with_input(InputPattern::resource(codegen_id));

    def
}

/// Resource definition for generated `Makefile` (`build:makefile`).
pub fn makefile_resource_def() -> ResourceDef {
    with_repo_inputs(ResourceDef::new(makefile_resource_id()))
        .with_output(ResourceScope::file(MAKEFILE_OUTPUT_PATH))
}

/// Resource definition for generated `.gitignore` (`build:gitignore`).
pub fn gitignore_resource_def() -> ResourceDef {
    with_repo_inputs(ResourceDef::new(gitignore_resource_id()))
        .with_output(ResourceScope::file(GITIGNORE_OUTPUT_PATH))
}

/// Resource definition for generated `deps.toml` (`build:deps_config`).
pub fn deps_config_resource_def() -> ResourceDef {
    with_repo_inputs(ResourceDef::new(deps_config_resource_id()))
        .with_output(ResourceScope::file(DEPS_CONFIG_OUTPUT_PATH))
}

fn repo_source_input_globs() -> Vec<String> {
    DERIVED_REPO_SOURCE_GLOBS
        .get_or_init(derive_repo_source_input_globs)
        .clone()
}

fn repo_config_input_files() -> Vec<String> {
    DERIVED_REPO_CONFIG_FILES
        .get_or_init(derive_repo_config_input_files)
        .clone()
}

fn testgen_input_globs() -> Vec<String> {
    DERIVED_TESTGEN_GLOBS
        .get_or_init(derive_testgen_input_globs)
        .clone()
}

fn derive_repo_source_input_globs() -> Vec<String> {
    let layout = workspace_layout_or_none();
    let Some(layout) = layout else {
        return REPO_SOURCE_INPUT_GLOBS
            .iter()
            .map(|s| s.to_string())
            .collect();
    };
    let mut globs = Vec::new();

    if let Some(gunbc_dag_dir) = layout.crate_dir("gunbc-dag") {
        let rel = layout
            .relative_path(&layout.workspace_root, gunbc_dag_dir)
            .to_string_lossy()
            .replace('\\', "/");
        globs.push(format!("{rel}/src/**/*.rs"));
    }

    let core_root = layout.workspace_root.join("core");
    if layout
        .crates
        .values()
        .any(|path| path.starts_with(core_root.as_path()))
    {
        globs.push("core/**/*.rs".to_string());
    }

    let lib_root = layout.workspace_root.join("lib");
    if layout
        .crates
        .values()
        .any(|path| path.starts_with(lib_root.as_path()))
    {
        globs.push("lib/**/*.rs".to_string());
    }

    if globs.is_empty() {
        return REPO_SOURCE_INPUT_GLOBS
            .iter()
            .map(|s| s.to_string())
            .collect();
    }
    globs.sort();
    globs.dedup();
    globs
}

fn derive_repo_config_input_files() -> Vec<String> {
    let layout = workspace_layout_or_none();
    let Some(layout) = layout else {
        return REPO_CONFIG_INPUT_FILES
            .iter()
            .map(|s| s.to_string())
            .collect();
    };

    let mut files = vec!["Cargo.toml".to_string()];
    if let Some(gunbc_dag_dir) = layout.crate_dir("gunbc-dag") {
        let rel = layout
            .relative_path(&layout.workspace_root, gunbc_dag_dir)
            .to_string_lossy()
            .replace('\\', "/");
        files.push(format!("{rel}/Cargo.toml"));
    }
    files.sort();
    files.dedup();
    files
}

fn derive_testgen_input_globs() -> Vec<String> {
    let layout = workspace_layout_or_none();
    let Some(layout) = layout else {
        return TESTGEN_INPUT_GLOBS.iter().map(|s| s.to_string()).collect();
    };

    let mut globs = Vec::new();

    if let Some(gunbc_dag_dir) = layout.crate_dir("gunbc-dag") {
        let rel = layout
            .relative_path(&layout.workspace_root, gunbc_dag_dir)
            .to_string_lossy()
            .replace('\\', "/");
        globs.push(format!("{rel}/src/**/*.rs"));
    }
    if let Some(ir_dir) = layout.crate_dir("gunbc-ir") {
        let rel = layout
            .relative_path(&layout.workspace_root, ir_dir)
            .to_string_lossy()
            .replace('\\', "/");
        globs.push(format!("{rel}/src/**/*.rs"));
    }

    let lib_root = layout.workspace_root.join("lib");
    if layout
        .crates
        .values()
        .any(|path| path.starts_with(lib_root.as_path()))
    {
        globs.push("lib/**/*.rs".to_string());
    }

    if globs.is_empty() {
        return TESTGEN_INPUT_GLOBS.iter().map(|s| s.to_string()).collect();
    }
    globs.sort();
    globs.dedup();
    globs
}

fn workspace_layout_or_none() -> Option<WorkspaceLayout> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_repo_source_globs_match_expected_patterns() {
        let globs = repo_source_input_globs();
        assert!(globs.iter().any(|g| g == "gunbc-dag/src/**/*.rs"));
        assert!(globs.iter().any(|g| g == "core/**/*.rs"));
        assert!(globs.iter().any(|g| g == "lib/**/*.rs"));
    }

    #[test]
    fn derived_repo_config_files_match_expected_patterns() {
        let files = repo_config_input_files();
        assert!(files.iter().any(|p| p == "Cargo.toml"));
        assert!(files.iter().any(|p| p == "gunbc-dag/Cargo.toml"));
    }

    #[test]
    fn derived_testgen_globs_match_expected_patterns() {
        let globs = testgen_input_globs();
        assert!(globs.iter().any(|g| g == "gunbc-dag/src/**/*.rs"));
        assert!(globs.iter().any(|g| g == "core/ir/src/**/*.rs"));
        assert!(globs.iter().any(|g| g == "lib/**/*.rs"));
    }
}
