//! gunbc-exec: Execution engine for gunbc DAGs.
//!
//! This crate provides:
//! - [`Executable`]: Trait for operations that can be executed
//! - [`execute`]: Execute a DAG in real mode
//! - [`execute_with_mode`]: Execute with dry-run interception at boundaries
//! - [`lower`]: Flatten sub-DAGs into a single flat DAG
//!
//! # Dry-Run via Boundary Interception
//!
//! Dry-run is not a flag threaded through operations. It's an execution mode
//! that intercepts at boundaries (unconnected outputs). Boundary nodes get
//! their operations replaced with mock implementations.

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
