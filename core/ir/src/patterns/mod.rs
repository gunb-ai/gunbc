//! Composable operation patterns for common DAG structures.
//!
//! This module provides builders for common operation patterns:
//!
//! - [`UpsertBuilder`]: Check → Create → Resolve pattern for idempotent resource creation
//! - [`TransactionBuilder`]: Begin → Body → Commit/Rollback pattern for transactional operations
//! - [`AtomicBuilder`]: Precondition → Operation → Postcondition pattern for atomic operations
//! - [`LoopBuilder`]: Iterate over collection, applying body to each element
//! - [`BranchBuilder`]: Conditional execution based on boolean condition
//! - [`RetryBuilder`]: Re-execute on failure with configurable backoff
//! - [`WhileBuilder`]: Re-execute while a condition holds
//! - [`PollBuilder`]: Re-execute at intervals until success or timeout
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
pub mod branch;
pub mod loop_pattern;
pub mod pattern_op;
pub mod repeat;
pub mod transaction;
pub mod upsert;

pub use atomic::AtomicBuilder;
pub use branch::BranchBuilder;
pub use loop_pattern::LoopBuilder;
pub use pattern_op::PatternOp;
pub use repeat::{
    BackoffStrategy, FailureClassifier, PollBuilder, RepeatPolicy, RetryBuilder, WhileBuilder,
};
pub use transaction::TransactionBuilder;
pub use upsert::UpsertBuilder;
