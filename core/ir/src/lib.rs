//! gunbc-ir: Core IR types for the gunbc DAG framework.
//!
//! This crate provides the fundamental types:
//! - [`Node`]: A node in the DAG (opaque operation or sub-DAG)
//! - [`Dag`]: A directed acyclic graph of nodes
//! - [`Edge`]: Connection between output and input ports
//! - [`Port`]: Input or output port with type and optional guard
//! - [`Value`]: Runtime values flowing through the DAG
//! - [`detect_boundaries`]: Find outputs that leave the DAG (world writes)
//! - [`detect_entrypoints`]: Find inputs that enter the DAG (world reads)
//! - [`transport`]: Transport layer types for I/O abstraction
//!
//! # Core Insight
//!
//! **World I/O is structural, not annotated.**
//!
//! - An output port with no downstream edge is a *boundary* — data leaving
//!   that port exits the DAG and goes to the world (world write).
//! - An input port with no upstream edge is an *entrypoint* — data entering
//!   that port comes from outside the DAG (world read).
//!
//! These are detected by [`detect_boundaries`] and [`detect_entrypoints`],
//! not by annotations on nodes.
//!
//! # Transport Layer
//!
//! All world I/O can be modeled as transport requests/responses:
//! - REST/HTTP for web APIs
//! - File operations for filesystem I/O
//! - TCP for raw network connections
//! - Shell for command execution
//!
//! This allows business logic to remain pure while transport execution
//! happens at well-defined boundaries.

pub mod boundary;
pub mod dag;
pub mod entrypoint;
pub mod node;
pub mod patterns;
pub mod transport;
pub mod types;
pub mod validate;
pub mod value;

// Re-exports for convenience
pub use boundary::{detect_boundaries, BoundaryInfo};
pub use dag::{build, Dag, Edge, Guard, Port};
pub use entrypoint::{detect_entrypoints, EntrypointInfo};
pub use node::{Node, NodeBody};
pub use patterns::{AtomicBuilder, TransactionBuilder, UpsertBuilder};
pub use transport::{TransportRequest, TransportResponse};
pub use types::{Cardinality, CardinalityCase, CardinalityMismatch, NodeId, PortName, TypeId};
pub use validate::{
    check_port_saturation_lowered, validate_dag, validate_dag_quick, ValidationError,
    ValidationResult,
};
pub use value::Value;
