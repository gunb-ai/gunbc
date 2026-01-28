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
//! let dag = build_gistgen_dag(".", "**/*.rs", UnderstandingMode::Real);
//! let log = gunbc_exec::execute(&dag).unwrap();
//! ```

mod ops;
mod graph;
mod generated;

// Contract definitions — source of truth for port names, types, and topology.
// Consumed by codegen and SetSpec-based test generation.
pub mod contracts;

// SetSpec declarations — each type declares its 0/1/N/null behavior.
// Tests are generated from these, not written manually.
pub mod setspec;

// Behavioral specs for test generation.
pub mod behavior;

// Re-export public API
pub use ops::{GistgenCoreOp, GistgenOp};
pub use graph::{build_gistgen_dag, build_gistgen_dag_with_payload, GistPayloadMode, UnderstandingMode};
