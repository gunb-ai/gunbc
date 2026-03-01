//! Transport operations and executors for gunbc I/O.
//!
//! This library provides:
//! - `TransportOps` - DAG node operations for transport execution
//!
//! The transport layer separates pure business logic from I/O:
//! - Pure ops prepare `TransportRequest` values
//! - `TransportOps::Execute` is the boundary that does actual I/O
//!
//! In dry-run mode, the boundary is mocked to intercept I/O.
//!
//! # Structural I/O Enforcement
//!
//! `execute_transport()` and `execute_request()` are NOT exported from this crate.
//! The primary I/O boundary is `TransportOps::Execute` nodes in a DAG. Tool
//! acquisition/execution helpers live here as well, so CLI tool I/O stays in
//! the transport layer rather than leaking into pure crates.
//!
//! This ensures I/O is:
//! - Visible in the graph structure (for transport requests)
//! - Interceptable by DryRun mode
//! - Auditable
//!
//! # Note
//!
//! This is the ONLY crate that performs direct I/O operations
//! via std::fs and std::process::Command. All other crates MUST use
//! PrepareXxxOp + TransportOps::Execute.

#![deny(dead_code)]
// This crate IS the transport layer - it's allowed to use direct I/O
#![allow(clippy::disallowed_methods)]
#![allow(clippy::disallowed_types)]

pub mod backend;
pub mod classify;
pub mod cli;
pub mod executor;
pub mod freshness_policy;
pub mod metrics;
pub mod middleware;
pub mod rate_limit;
pub mod retry;
pub mod pipeline;

pub mod ops;
pub mod preflight;
pub mod resource_io;
pub mod test_backend;

// STRUCTURAL ENFORCEMENT: TransportOps + transport-layer CLI helpers only
// execute_transport and execute_request are internal - not exported
pub use backend::{TransportBackend, TransportBackendGuard};
pub use freshness_policy::{check_and_plan_freshness, update_freshness_manifest};

pub use ops::TransportOps;

pub use resource_io::TransportIo;

// Middleware infrastructure
pub use classify::{
    classify_for_middleware, classify_rest_response, classify_transport_error,
    extract_status_code, is_success, ClassifiedErrorKind, ClassifiedResponse,
};
pub use metrics::{InMemoryMetricsSink, LogMetricsSink, MetricsMiddleware, MetricsSink, NullMetricsSink};
pub use middleware::{
    MiddlewareContext, MiddlewareOutcome, PostProcessOutcome, SharedMiddlewareState,
    TransportMiddleware,
};
pub use rate_limit::{RateLimitMiddleware, RateLimitState};
pub use retry::{CircuitBreaker, CircuitState, RetryMiddleware};
pub use pipeline::{TransportPipeline, TransportPipelineBuilder};

pub mod system_models;

#[cfg(test)]
mod pragma_lint;
