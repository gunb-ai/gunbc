#![recursion_limit = "1024"]
//! Clippy tool configuration.
//!
//! This crate provides Clippy integration configuration modeling
//! for clippy.toml generation.
//!
//! # Configuration Generation
//!
//! ```text
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

