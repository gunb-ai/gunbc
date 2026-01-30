//! gunbc-exec: Execution engine for gunbc DAGs.
//!
//! This crate provides:
//! - [`Executable`]: Trait for operations that can be executed
//! - [`execute`]: Execute a DAG in real mode
//! - [`execute_with_mode`]: Execute with dry-run interception at transport nodes
//! - [`lower`]: Flatten sub-DAGs into a single flat DAG
//!
//! # Dry-Run via Transport Interception
//!
//! Dry-run is not a flag threaded through operations. It's an execution mode
//! that intercepts **transport execution nodes** - nodes that consume
//! `TransportRequest` values. This follows the design principle:
//!
//! > "World I/O is performed only by transport executor nodes"
//! > "DryRun intercepts transport execution nodes, not boundary outputs"
//!
//! This ensures:
//! - Pure nodes always execute (they can't do I/O)
//! - Transport executors are replaced with mocks
//! - Boundaries are just interface definitions, not interception points

pub mod error;
pub mod execute;
pub mod intercept;
pub mod lower;
pub mod topo;

pub use error::ExecError;
pub use execute::{execute, execute_with_mode, ExecutionLog, ExecutionMode, LogEntry};
pub use intercept::{BoundaryMock, BoundaryMocks};
pub use lower::{lower, LowerError};
pub use topo::topo_sort;

use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;

/// Trait that opaque node operations must implement.
pub trait Executable: fmt::Debug {
    /// Execute the operation with the given inputs.
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError>;
}
