//! **Stage 11 — Transport**: Transforms a `TransportRequest` into a
//! `TransportResponse` by performing actual I/O.
//!
//! # Pipeline position
//!
//! - **Before**: pure ops have prepared a `TransportRequest` value
//! - **After**: parse ops extract structured output from `TransportResponse`
//!
//! # Sequential steps
//!
//! 1. Classify the transport request (REST, shell, file)
//! 2. Apply middleware pipeline (credentials, rate limiting, retry, metrics)
//! 3. Execute the I/O operation via the appropriate backend
//! 4. Return `TransportResponse` (or mock response in DryRun mode)
//!
//! # Purity
//!
//! **NOT PURE — this is the I/O boundary.** This crate is the ONLY crate
//! that performs direct I/O (shell, HTTP, filesystem via `std::fs` and
//! `std::process::Command`). All other crates MUST use
//! `PrepareXxxOp` + `TransportOps::Execute`.
//!
//! # Failure
//!
//! Returns `TransportError` with classified error kinds (network, auth,
//! rate limit, shell exit code, filesystem).

#![deny(dead_code)]

pub mod backend;
pub mod classify;
pub mod cli;
pub mod credential;
pub mod executor;
pub mod freshness_policy;
pub mod metrics;
pub mod middleware;
pub mod pipeline;
pub mod rate_limit;
pub mod retry;
pub mod transport_types;

pub mod ops;
pub mod resource_io;
pub mod test_backend;

// STRUCTURAL ENFORCEMENT: TransportOps + transport-layer CLI helpers only
// execute_transport and execute_request are internal - not exported
pub use backend::{TransportBackend, TransportBackendGuard};
pub use freshness_policy::{
    check_and_plan_freshness, check_and_plan_generation_freshness, update_freshness_manifest,
};

pub use ops::TransportOps;

pub use resource_io::TransportIo;

// Middleware infrastructure
pub use classify::{
    classify_for_middleware, classify_rest_response, classify_transport_error, extract_status_code,
    is_success, ClassifiedErrorKind, ClassifiedResponse,
};
pub use credential::{CredentialCache, CredentialMiddleware};
pub use metrics::{
    InMemoryMetricsSink, LogMetricsSink, MetricsMiddleware, MetricsSink, NullMetricsSink,
};
pub use middleware::{
    MiddlewareContext, MiddlewareOutcome, PostProcessOutcome, SharedMiddlewareState,
    TransportMiddleware,
};
pub use pipeline::{TransportPipeline, TransportPipelineBuilder};
pub use rate_limit::{RateLimitMiddleware, RateLimitState};
pub use retry::{CircuitBreaker, CircuitState, RetryMiddleware};

// Transport foundation types (TL-0)
pub use transport_types::{
    EndpointBehavior, FailureMode, OperationBehavior, TransportCapabilities, TransportClass,
};

pub mod system_models;
