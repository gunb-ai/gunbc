//! Repo-specific resource definitions for gunbc-dag.
//!
//! Canonical globs and output paths are loaded from `dsl/config/resources.dag`.

use daglang_driver::compile_data_from_sources;
use gunbc_ir::resource::{codegen_resource_def, InputPattern, ResourceDef, ResourceScope};
use gunbc_ir::ResourceId;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

const RESOURCES_DAG_SOURCE: &str = include_str!("../../dsl/config/resources.dag");

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

#[derive(Debug, Clone)]
struct ResourceDslData {
    repo_source_input_globs: Vec<String>,
    repo_config_input_files: Vec<String>,
    testgen_input_globs: Vec<String>,
    output_paths: HashMap<String, String>,
}

static RESOURCE_DSL_DATA: OnceLock<ResourceDslData> = OnceLock::new();

fn resource_dsl_data() -> &'static ResourceDslData {
    RESOURCE_DSL_DATA.get_or_init(load_resource_dsl_data)
}

fn load_resource_dsl_data() -> ResourceDslData {
    let path = Path::new("<embedded>/config/resources.dag");
    let output = compile_data_from_sources(&[(path, RESOURCES_DAG_SOURCE)])
        .expect("config/resources.dag must compile — fix DSL syntax errors before building");

    let repo_source_input_globs =
        json_string_list(output.data_values.get("repo_source_input_globs"))
            .expect("config/resources.dag must declare repo_source_input_globs");
    let repo_config_input_files =
        json_string_list(output.data_values.get("repo_config_input_files"))
            .expect("config/resources.dag must declare repo_config_input_files");
    let testgen_input_globs =
        json_string_list(output.data_values.get("testgen_input_globs"))
            .expect("config/resources.dag must declare testgen_input_globs");
    let output_paths =
        json_output_paths(output.data_values.get("output_paths"))
            .expect("config/resources.dag must declare output_paths");

    ResourceDslData {
        repo_source_input_globs,
        repo_config_input_files,
        testgen_input_globs,
        output_paths,
    }
}

fn json_string_list(value: Option<&serde_json::Value>) -> Option<Vec<String>> {
    let arr = value?.as_array()?;
    arr.iter()
        .map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect()
}

fn json_output_paths(value: Option<&serde_json::Value>) -> Option<HashMap<String, String>> {
    let arr = value?.as_array()?;
    let mut map = HashMap::new();
    for entry in arr {
        let obj = entry.as_object()?;
        let id = obj.get("id")?.as_str()?.to_string();
        let path = obj.get("path")?.as_str()?.to_string();
        map.insert(id, path);
    }
    Some(map)
}

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
    for pattern in &resource_dsl_data().repo_source_input_globs {
        def = def.with_input(InputPattern::glob(pattern));
    }
    for path in &resource_dsl_data().repo_config_input_files {
        def = def.with_input(InputPattern::file(path));
    }

    // Toolchain version changes can affect generated command snippets.
    def.with_input(InputPattern::command_output("rustc", &["--version"]))
}

fn output_path_for(resource_id: &str, fallback: &str) -> String {
    resource_dsl_data()
        .output_paths
        .get(resource_id)
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

/// Resource definition for testgen outputs (`build:generated_tests`).
pub fn testgen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(generated_tests_resource_id());
    for pattern in &resource_dsl_data().testgen_input_globs {
        def = def.with_input(InputPattern::glob(pattern));
    }
    // Testgen depends on codegen output key.
    let codegen_id = codegen_resource_def().id;
    def.with_input(InputPattern::resource(codegen_id))
}

/// Resource definition for generated `Makefile` (`build:makefile`).
pub fn makefile_resource_def() -> ResourceDef {
    let path = output_path_for(BUILD_RESOURCE_MAKEFILE, MAKEFILE_OUTPUT_PATH);
    with_repo_inputs(ResourceDef::new(makefile_resource_id())).with_output(ResourceScope::file(path))
}

/// Resource definition for generated `.gitignore` (`build:gitignore`).
pub fn gitignore_resource_def() -> ResourceDef {
    let path = output_path_for(BUILD_RESOURCE_GITIGNORE, GITIGNORE_OUTPUT_PATH);
    with_repo_inputs(ResourceDef::new(gitignore_resource_id())).with_output(ResourceScope::file(path))
}

/// Resource definition for generated `deps.toml` (`build:deps_config`).
pub fn deps_config_resource_def() -> ResourceDef {
    let path = output_path_for(BUILD_RESOURCE_DEPS_CONFIG, DEPS_CONFIG_OUTPUT_PATH);
    with_repo_inputs(ResourceDef::new(deps_config_resource_id())).with_output(ResourceScope::file(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsl_resource_globs_match_expected_patterns() {
        let data = resource_dsl_data();
        assert!(
            data.repo_source_input_globs
                .contains(&"gunbc-dag/src/**/*.rs".to_string())
        );
        assert!(data.repo_source_input_globs.contains(&"core/**/*.rs".to_string()));
        assert!(data.repo_source_input_globs.contains(&"lib/**/*.rs".to_string()));
        assert!(data.repo_source_input_globs.contains(&"dsl/**/*.dag".to_string()));
        assert!(data.testgen_input_globs.contains(&"core/ir/src/**/*.rs".to_string()));
        assert!(data.repo_config_input_files.contains(&"Cargo.lock".to_string()));
        assert!(
            data.output_paths
                .get(BUILD_RESOURCE_MAKEFILE)
                .map(|s| s.as_str())
                == Some(MAKEFILE_OUTPUT_PATH)
        );
    }
}
