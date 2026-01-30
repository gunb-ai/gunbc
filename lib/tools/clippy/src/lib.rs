//! Clippy tool DAG.
//!
//! This crate provides Clippy integration using the fractal DAG pattern.
//! The key function is `build_clippy_upsert()` which returns a `Node<CliToolOp>`
//! containing a sub-DAG that implements: check → install → run.
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
//! # Design
//!
//! This crate demonstrates the pattern for CLI tool integration:
//! 1. Define tool via `CliToolDef` (in `gunbc_ir::transport::cli`)
//! 2. Use `build_cli_upsert()` for the fractal sub-DAG
//! 3. Provide convenience wrappers for common operations

pub mod graph;
pub mod ops;

pub use graph::{build_clippy_dag, build_clippy_lint_all, build_clippy_upsert};
pub use ops::{Clippy, CliToolOp};
