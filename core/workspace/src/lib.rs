//! gunbc-workspace: Unified WorkspaceOp and fractal DAG composition.
//!
//! This crate provides:
//! - `WorkspaceOp`: A unified enum wrapping all domain operations
//! - `build_*_subdag()`: SubDag builders for each tool
//! - `build_workspace_dag()`: Composes all tool and language SubDags
//!
//! # Fractal DAG Pattern
//!
//! All DAGs in gunbc follow the fractal pattern where:
//! - Every DAG can be wrapped as a SubDag node
//! - SubDags compose within parent DAGs
//! - I/O interfaces are explicit at SubDag boundaries
//!
//! ```text
//! Workspace DAG
//! ├── ci SubDag
//! ├── deps SubDag
//! ├── makegen SubDag
//! ├── gist SubDag
//! ├── bootstrap SubDag
//! ├── buck2 SubDag
//! ├── clippy SubDag
//! └── languages SubDag
//!     ├── rust
//!     ├── makefile
//!     └── gitignore
//! ```

mod ops;
mod subdags;

/// Deprecated aliases for backward compatibility.
/// Use the new `build_*_subdag()` functions instead.
pub mod deprecated;

pub use ops::WorkspaceOp;
pub use subdags::build_workspace_dag;
pub use subdags::bootstrap::build_bootstrap_subdag;
pub use subdags::buck2::build_buck2_subdag;
pub use subdags::ci::build_ci_subdag;
pub use subdags::clippy::{build_clippy_lint_all_subdag, build_clippy_subdag};
pub use subdags::deps::{build_deps_generate_subdag, build_deps_install_subdag};
pub use subdags::gist::{build_gist_rust_subdag, build_gist_subdag};
pub use subdags::languages::build_languages_subdag;
pub use subdags::makegen::build_makegen_subdag;

// Re-export tool ops for convenience
pub use gunbc_bootstrap::BootstrapOp;
pub use gunbc_buck2::Buck2Op;
pub use gunbc_ci::CIOp;
pub use gunbc_clippy::CliToolOp;
pub use gunbc_deps::DepsOp;
pub use gunbc_gist::GistOps;
pub use gunbc_ir::LanguageOp;
pub use gunbc_lib_transport::TransportOps;
pub use gunbc_makegen::MakegenOp;
pub use gunbc_primitives::PrimitiveOp;
