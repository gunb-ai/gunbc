//! Clippy tool DAG.
//!
//! This crate provides a self-ensuring Clippy DAG that can be composed
//! into other workflows. The DAG handles its own installation (upsert pattern)
//! so consumers just use it without worrying about dependencies.
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_clippy::build_clippy_lint_all;
//!
//! // Get a clippy node that ensures clippy is installed and runs linting
//! let clippy_node = build_clippy_lint_all();
//!
//! // Compose into your workflow
//! builder.add_node(clippy_node);
//! ```
//!
//! # Design
//!
//! The Clippy DAG follows the upsert pattern:
//!
//! 1. **Check**: Verify clippy is installed
//! 2. **Create**: Install via rustup if missing
//! 3. **Resolve**: Run cargo clippy
//!
//! This makes the dependency implicit - by using the Clippy DAG,
//! you depend on clippy, and the DAG ensures it's available.

pub mod graph;
pub mod ops;

pub use graph::{build_clippy_dag, build_clippy_lint_all, build_clippy_upsert};
pub use ops::ClippyOp;
