//! gunbc-bootstrap: Generate all build infrastructure.
//!
//! This tool generates:
//! - Makefile (via gunbc-makegen logic)
//! - .gitignore
//! - deps.toml template
//! - CI workflow
//!
//! All outputs are boundaries (file writes), all dry-runnable.

pub mod graph;
pub mod ops;

pub use graph::build_bootstrap_graph;
pub use ops::BootstrapOp;
