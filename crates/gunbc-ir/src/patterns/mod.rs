//! Composable operation patterns for common DAG structures.
//!
//! This module provides builders for common operation patterns:
//!
//! - [`UpsertBuilder`]: Check → Create → Resolve pattern for idempotent resource creation
//! - [`TransactionBuilder`]: Begin → Body → Commit/Rollback pattern for transactional operations
//! - [`AtomicBuilder`]: Precondition → Operation → Postcondition pattern for atomic operations
//!
//! Each builder creates a [`Node`] with a [`NodeBody::SubDag`] containing the pattern's
//! internal structure, with proper guards for conditional execution.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::patterns::UpsertBuilder;
//!
//! let upsert_node = UpsertBuilder::new("install_tool")
//!     .with_check(MyOp::CheckInstalled)
//!     .with_create(MyOp::Install)
//!     .with_resolve(MyOp::Verify)
//!     .build();
//! ```

pub mod atomic;
pub mod transaction;
pub mod upsert;

pub use atomic::AtomicBuilder;
pub use transaction::TransactionBuilder;
pub use upsert::UpsertBuilder;
