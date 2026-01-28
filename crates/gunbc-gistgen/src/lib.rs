//! Gistgen core library.
//!
//! This crate provides the core DAG definition and operations for gistgen,
//! a tool that creates GitHub Gists from repository files.
//!
//! ## Usage
//!
//! ```ignore
//! use gunbc_gistgen::{build_gistgen_dag, GistgenOp};
//!
//! let dag = build_gistgen_dag(".", "**/*.rs", true);
//! let log = gunbc_exec::execute(&dag).unwrap();
//! ```

mod ops;
mod graph;
mod generated;

// Contract definitions — source of truth for port names, types, and topology.
// Currently consumed only by verification tests; codegen binary will read these directly.
#[cfg(test)]
mod contracts;

// Re-export public API
pub use ops::GistgenOp;
pub use graph::{build_gistgen_dag, UnderstandingMode};
