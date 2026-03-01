//! Transport middleware infrastructure.
//!
//! Middleware layers intercept transport requests/responses to provide cross-cutting
//! concerns: rate limiting, retry, credentials, metrics. Each middleware implements
//! `TransportMiddleware` and composes via `TransportPipeline`.
//!
//! # Pipeline Order
//!
//! ```text
//! request → metrics → rate_limit → retry → credential → execute → classify → retry → metrics → response
//! ```
//!
//! Pre-request flows outer→inner; post-response flows inner→outer.

use gunbc_exec::ExecError;
use gunbc_ir::transport::{TransportMiddlewareConfig, TransportRequest, TransportResponse};
use std::sync::Arc;

/// Context passed through the middleware chain.
///
/// Carries request metadata, operation info, and per-request state that middleware
/// layers can read and modify.
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    /// Operation identifier for logging/metrics (e.g., "github.gist.create").
    pub operation_id: String,
    /// Whether the operation is idempotent (safe to retry without side effects).
    pub idempotent: bool,
    /// Whether the operation is read-only (no server-side state change).
    pub readonly: bool,
    /// Middleware configuration from IR.
    pub config: Arc<TransportMiddlewareConfig>,
    /// Attempt number (1 for first attempt, incremented on retry).
    pub attempt: u32,
    /// Shared state for cross-request coordination (rate limits, circuit breaker).
    pub shared_state: Arc<SharedMiddlewareState>,
}

impl MiddlewareContext {
    /// Create a new context for an operation.
    pub fn new(
        operation_id: impl Into<String>,
        idempotent: bool,
        readonly: bool,
        config: Arc<TransportMiddlewareConfig>,
        shared_state: Arc<SharedMiddlewareState>,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            idempotent,
            readonly,
            config,
            attempt: 1,
            shared_state,
        }
    }

    /// Check if the operation is safe to retry (idempotent or readonly).
    pub fn retry_safe(&self) -> bool {
        self.idempotent || self.readonly
    }
}

/// Shared state across middleware invocations.
///
/// Holds stateful middleware components like rate limit buckets and circuit breaker
/// state that must be shared across concurrent requests.
#[derive(Debug, Default)]
pub struct SharedMiddlewareState {
    // Rate limit state is added by TL-1
    // Circuit breaker state is added by TL-2
}

impl SharedMiddlewareState {
    /// Create new shared state.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Result of middleware pre-request processing.
#[derive(Debug)]
pub enum MiddlewareOutcome {
    /// Continue to next middleware or execute request.
    Continue(TransportRequest),
    /// Short-circuit with immediate response (cached, rate-limited waiting, etc.).
    ShortCircuit(TransportResponse),
    /// Abort with error (circuit open, auth failed, etc.).
    Abort(ExecError),
}

/// Result of middleware post-response processing.
#[derive(Debug)]
pub enum PostProcessOutcome {
    /// Return response as-is to caller.
    Complete(TransportResponse),
    /// Retry the request after delay.
    Retry {
        /// Delay before retry in milliseconds.
        delay_ms: u64,
        /// Reason for retry (for logging/metrics).
        reason: String,
    },
    /// Abort with error.
    Abort(ExecError),
}

/// Transport middleware layer trait.
///
/// Middleware intercepts requests before execution and responses after execution.
/// Each method has a default pass-through implementation.
pub trait TransportMiddleware: Send + Sync {
    /// Process request before execution.
    ///
    /// Can modify the request, short-circuit with an immediate response,
    /// or abort with an error.
    fn pre_request(
        &self,
        request: TransportRequest,
        _ctx: &mut MiddlewareContext,
    ) -> MiddlewareOutcome {
        MiddlewareOutcome::Continue(request)
    }

    /// Process response after execution.
    ///
    /// Can transform the response, request retry, or abort with an error.
    fn post_response(
        &self,
        _request: &TransportRequest,
        response: TransportResponse,
        _ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        PostProcessOutcome::Complete(response)
    }

    /// Handle execution error.
    ///
    /// Can transform the error, request retry (for network errors), or pass through.
    fn on_error(
        &self,
        _request: &TransportRequest,
        error: ExecError,
        _ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        PostProcessOutcome::Abort(error)
    }

    /// Middleware name for logging/metrics.
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::LocalRequest;

    struct PassthroughMiddleware;

    impl TransportMiddleware for PassthroughMiddleware {
        fn name(&self) -> &'static str {
            "passthrough"
        }
    }

    #[test]
    fn default_middleware_passes_through() {
        let mw = PassthroughMiddleware;
        let config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test.op", false, true, config, shared);

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });
        let outcome = mw.pre_request(request.clone(), &mut ctx);
        assert!(matches!(outcome, MiddlewareOutcome::Continue(_)));
    }

    #[test]
    fn context_retry_safe_checks_idempotent_or_readonly() {
        let config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());

        let ctx = MiddlewareContext::new("op1", false, false, config.clone(), shared.clone());
        assert!(!ctx.retry_safe());

        let ctx = MiddlewareContext::new("op2", true, false, config.clone(), shared.clone());
        assert!(ctx.retry_safe());

        let ctx = MiddlewareContext::new("op3", false, true, config.clone(), shared.clone());
        assert!(ctx.retry_safe());
    }
}
