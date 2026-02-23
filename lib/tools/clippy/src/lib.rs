#![recursion_limit = "1024"]
//! Clippy tool configuration.
//!
//! This crate provides Clippy integration configuration modeling
//! for clippy.toml generation.
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

#![deny(dead_code)]
pub mod config;
pub mod lint;
pub mod ops;
pub mod policy;

pub use config::{
    generate_clippy_toml, ClippyConfig, ClippyConfigRenderer, CrateAllowance, DisallowedMethod,
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
    dsl_module = "clippy",
    consumes = "clippy.toml",
    returns_result
)]
pub fn clippy_tool() {}
