//! gunbc-deps: Tool dependency management with upsert pattern.
//!
//! This crate provides:
//! - Declarative tool dependency specification via `deps.toml`
//! - deps.toml generation from tool registry (owns the file)
//! - Platform-agnostic installation (apt, brew, cargo, script, etc.)
//! - Idempotent upsert pattern: Check → Create → Resolve
//!
//! # Example deps.toml
//!
//! ```toml
//! [[dependency]]
//! name = "gh"
//! verify = "gh --version"
//!
//! [dependency.install.linux]
//! method = "apt"
//! packages = ["gh"]
//!
//! [dependency.install.macos]
//! method = "brew"
//! packages = ["gh"]
//! ```
//!
//! # Generated File Ownership
//!
//! This crate owns `deps.toml` generation via `build_deps_generate_graph()`.
//! The filename is centralized in `DEFAULT_MANIFEST_FILENAME`.
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

#![deny(dead_code)]
pub mod env;
pub mod graph;
pub mod installer;
pub mod manifest;
pub mod ops;
pub mod platform;
pub mod tool_upsert;
pub mod upsert;

pub mod graph_mock;

pub use env::PlatformEnv;
pub use graph::{
    build_deps_generate_graph, build_deps_graph, deps_generate_signature, deps_signature,
};
pub use installer::{InstallMethod, Installer};
pub use manifest::{
    Dependency, DepsManifest, ManifestConfig, PlatformInstall, DEFAULT_MANIFEST_FILENAME,
    MANIFEST_CONFIG,
};
pub use ops::DepsOp;
pub use platform::Platform;
pub use tool_upsert::{
    find_install_option, generate_deps_toml, generate_deps_toml_from_registry,
    generate_tool_deps_entry, generate_tool_idempotent_script, generate_tool_install_cmd,
    install_inputs_to_platform_install, tool_to_platform_install,
};
pub use upsert::{UpsertPhase, UpsertResult};

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "deps",
    crate_name = "gunbc-deps",
    description = "Install tool dependencies",
    builder = "build_deps_graph",
    import = "use gunbc_deps::build_deps_graph;",
    mock_spec = "gunbc_deps::graph_mock::deps_mock_spec()",
    package = "deps",
    entrypoints = r#"[{"port_name":"manifest_path","type_id":"String","short":"m","help":"Path to deps.toml manifest","make_var":"MANIFEST"}]"#,
    returns_result
)]
pub fn deps_tool() {}

// ============================================================================
// Generated Tests (from `make testgen`)
// ============================================================================

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}
