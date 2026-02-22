#![recursion_limit = "1024"]
//! Clippy tool DAG and configuration.
//!
//! This crate provides Clippy integration using the fractal DAG pattern,
//! plus configuration modeling for clippy.toml generation.
//!
//! # Fractal DAG Usage (Preferred)
//!
//! ```ignore
//! use gunbc_clippy::build_clippy_upsert;
//!
//! // Build a sub-DAG node for clippy
//! let clippy_node = build_clippy_upsert(&["--all-targets"]);
//!
//! // Compose into a larger DAG
//! builder.add_node(clippy_node);
//! ```
//!
//! # Direct Operations (Simple Cases)
//!
//! ```ignore
//! use gunbc_clippy::Clippy;
//!
//! // Imperative upsert for simple scripts
//! let result = Clippy::upsert_and_run(&["--fix"])?;
//! ```
//!
//! # Configuration Generation
//!
//! ```ignore
//! use gunbc_clippy::config::{ClippyConfig, generate_clippy_toml};
//!
//! // Use transport pattern preset
//! let config = ClippyConfig::transport_pattern();
//! let toml = generate_clippy_toml(&config);
//! ```
//!
//! # Design
//!
//! This crate demonstrates the pattern for CLI tool integration:
//! 1. Define tool via `CliToolDef` (in `gunbc_ir::transport::cli`)
//! 2. Use `build_cli_upsert()` for the fractal sub-DAG
//! 3. Provide convenience wrappers for common operations
//! 4. Model configuration as Rust code, generate config files

#![deny(dead_code)]
pub mod config;
pub mod graph;
pub mod graph_mock;
pub mod lint;
pub mod ops;
pub mod policy;

pub use config::{
    generate_clippy_toml, ClippyConfig, ClippyConfigRenderer, CrateAllowance, DisallowedMethod,
};
pub use graph::{
    build_clippy_dag, build_clippy_graph, build_clippy_graph_lint_all, build_clippy_lint_all,
    build_clippy_upsert, ClippyGraphOp,
};
pub use lint::{LintId, LintSource};
pub use ops::{CliToolOp, Clippy};
pub use policy::{CratePolicy, CrateRole};

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "clippy",
    crate_name = "gunbc-clippy",
    description = "Run clippy via upsert (check → install → run)",
    builder = "build_clippy_graph_dsl",
    import = "use gunbc_dag::build_clippy_graph_dsl;",
    mock_spec = "gunbc_clippy::graph_mock::clippy_mock_spec()",
    dsl_module = "clippy",
    returns_result
)]
pub fn clippy_tool() {}

// ============================================================================
// Generated Tests (from `make testgen`)
// ============================================================================

#[cfg(test)]
mod generated_tests {
    include!("generated_tests.rs");
}
