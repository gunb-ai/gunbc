//! Transport operations and executors for gunbc I/O.
//!
//! This library provides:
//! - `TransportOps` - DAG node operations for transport execution
//! - `execute_transport` - The actual I/O executor
//!
//! The transport layer separates pure business logic from I/O:
//! - Pure ops prepare `TransportRequest` values
//! - `TransportOps::Execute` is the boundary that does actual I/O
//!
//! In dry-run mode, the boundary is mocked to intercept I/O.
//!
//! # Note
//!
//! This is the ONLY crate (besides codegen and deprecated primitives) that
//! should perform direct I/O operations via std::fs and std::process::Command.
//! All other crates should use PrepareXxxOp + TransportOps::Execute.

// This crate IS the transport layer - it's allowed to use direct I/O
#![allow(clippy::disallowed_methods)]

pub mod executor;
pub mod ops;

pub use executor::execute_transport;
pub use ops::{execute_request, TransportOps};
