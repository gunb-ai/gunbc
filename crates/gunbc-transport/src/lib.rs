//! gunbc-transport: Transport layer executors.
//!
//! This crate provides the `TransportOp` executor that handles all I/O operations.
//! It is the unified boundary for all world interactions:
//!
//! - REST/HTTP API calls
//! - File system operations
//! - TCP connections
//! - Shell command execution
//!
//! # Design
//!
//! Business logic prepares `TransportRequest` values, which flow through edges
//! to `TransportOp::Execute` nodes. These nodes are the only boundaries in the DAG,
//! making dry-run interception uniform across all I/O types.

pub mod executor;
pub mod ops;

pub use executor::{execute_transport, TransportError};
pub use ops::TransportOp;
