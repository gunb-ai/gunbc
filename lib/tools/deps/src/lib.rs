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

pub mod env;
pub mod graph;
pub mod installer;
pub mod manifest;
pub mod ops;
pub mod platform;
pub mod tool_upsert;
pub mod upsert;

#[cfg(test)]
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
// Generated Tests (from `make testgen`)
// ============================================================================

#[cfg(test)]
mod generated_tests {
    #![allow(unused_imports)]
    include!("generated_tests.rs");
}
