//! **Stage 8 — Resolve Ops**: Transforms a `Dag<LoweredOp>` into a
//! `Dag<DynOp>` of executable operations.
//!
//! # Pipeline position
//!
//! - **Before**: [`daglang-driver`] has produced a compiled `Dag<LoweredOp>`
//! - **After**: [`gunbc-exec`] executes the resolved `Dag<DynOp>`
//!
//! # Sequential steps
//!
//! 1. Walk every `LoweredOp` node in the DAG
//! 2. Map each variant to a concrete `DynOp` implementation (transport ops,
//!    pattern ops, service ops, filesystem env nodes)
//! 3. Wire filesystem environment edges and dry-run mocks when applicable
//! 4. Return a fully executable `Dag<DynOp>`
//!
//! # Purity
//!
//! May access the build-time filesystem for dagbin cache (documented
//! exception). Otherwise pure in-memory translation.
//!
//! # Failure
//!
//! Returns `ResolveError` when a `LoweredOp` variant cannot be mapped
//! to a concrete operation.

pub mod builder;
pub mod dry_run;
pub mod fs_env;
pub mod resolve;
pub mod service_ops;

pub use builder::{BuildOpts, CompileLoweredResult, DslGraphResult};
pub use dry_run::wire_fs_env_write_mock;
pub use fs_env::{add_fs_env_root_node, wire_fs_env_write_edges};
pub use resolve::{resolve_lowered_dag_with, ResolveError};
