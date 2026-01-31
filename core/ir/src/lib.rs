//! gunbc-ir: Core IR types for the gunbc DAG framework.
//!
//! This crate provides the fundamental types:
//! - [`Node`]: A node in the DAG (opaque operation or sub-DAG)
//! - [`Dag`]: A directed acyclic graph of nodes
//! - [`Edge`]: Connection between output and input ports
//! - [`Port`]: Input or output port with type and cardinality
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
//! # Types as DAGs
//!
//! Types are also DAGs (`Dag<TypeOp>`) that describe validation and transformation
//! of values. This unifies types with workflows — same infrastructure, same
//! composition rules. See [`type_op`] and [`type_lib`] modules.
//!
//! # No Meta-Annotations
//!
//! Conditional execution is modeled through explicit Branch patterns and
//! optional types (ZeroOrOne cardinality), not through guards on ports.
//! This keeps the type system closed and self-consistent.
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
pub mod builder;
pub mod cargo;
pub mod compose;
pub mod contract;
pub mod dag;
pub mod entrypoint;
pub mod git;
pub mod language;
pub mod node;
pub mod patterns;
pub mod render;
pub mod resource;
pub mod signature;
pub mod transport;
pub mod type_lib;
pub mod type_op;
pub mod type_registry;
pub mod types;
pub mod value;

// Re-exports for convenience
pub use boundary::{detect_boundaries, BoundaryInfo};
pub use builder::{BuilderError, DagBuilder, InputRef, NodeRef, OutputRef, PortKind};
pub use dag::{build, canonical_edge_order, edges_to_port, Dag, Edge, Port};
pub use entrypoint::{detect_entrypoints, EntrypointInfo};
pub use node::{Node, NodeBody};
pub use patterns::{
    AtomicBuilder, BackoffStrategy, FailureClassifier, PollBuilder, RepeatPolicy, RetryBuilder,
    TransactionBuilder, UpsertBuilder, WhileBuilder,
};
pub use signature::{infer_signature, SignatureError, SignaturePort, WorkflowSignature};
pub use contract::TypeContract;
pub use resource::{AccessMode, ResourceAccess, ResourceConflict, ResourceId};
pub use cargo::{
    CargoCommand, CargoEnv, CargoInvocation, Subcommand, TermColor, Warnings,
    PREFIX as CARGO_PREFIX,
};
pub use git::GitConfig;
pub use transport::{TransportRequest, TransportResponse};
pub use type_op::{BaseType, Coercion, Predicate, PredicateValue, TypeOp, WrapperKind};
pub use type_registry::{TypeNotFoundError, TypeRegistry};
pub use types::{Cardinality, CardinalityCase, CardinalityMismatch, NodeId, PortName, TypeId};
pub use value::{SecretString, Value};
pub use render::Renderable;

// Re-exports from language module for common use
pub use language::{
    build_languages_dag, detect_language_from_file, markdown_language_id,
    rust_type, NamingCase, LanguageOp,
    MakefileConfig, MakeTarget, RustConfig, GitignoreConfig, HtmlConfig, MarkdownConfig,
    DEFAULT_GITIGNORE_FILENAME, DEFAULT_MAKEFILE_FILENAME,
    MAKEFILE, RUST, GITIGNORE, HTML, MARKDOWN,
    render_html_document, render_code_block, html_comment, markdown_comment,
};
