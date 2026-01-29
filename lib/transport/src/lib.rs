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

pub mod executor;
pub mod ops;

pub use executor::execute_transport;
pub use ops::{execute_request, TransportOps};
