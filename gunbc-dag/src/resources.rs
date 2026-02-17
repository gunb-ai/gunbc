//! Repo-specific resource definitions for gunbc-dag.

use gunbc_ir::resource::{codegen_resource_def, InputPattern, ResourceDef, ResourceScope};
use gunbc_ir::ResourceId;

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
    for pattern in REPO_SOURCE_INPUT_GLOBS {
        def = def.with_input(InputPattern::glob(*pattern));
    }
    for path in REPO_CONFIG_INPUT_FILES {
        def = def.with_input(InputPattern::file(*path));
    }

    // Toolchain version changes can affect generated command snippets.
    def.with_input(InputPattern::command_output("rustc", &["--version"]))
}

/// Input globs that affect testgen outputs.
pub const TESTGEN_INPUT_GLOBS: &[&str] = &[
    "gunbc-dag/src/**/*.rs",
    "core/ir/src/**/*.rs",
    "lib/**/*.rs",
];

/// Resource definition for testgen outputs (`build:generated_tests`).
pub fn testgen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(generated_tests_resource_id());

    for pattern in TESTGEN_INPUT_GLOBS {
        def = def.with_input(InputPattern::glob(*pattern));
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
