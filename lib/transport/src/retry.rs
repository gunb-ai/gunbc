//! Retry middleware with exponential/jittered backoff and circuit breaker.
//!
//! Provides automatic retry for transient failures (5xx, network errors, rate limits)
//! with configurable backoff strategies and circuit breaker protection.
//!
//! # Safety
//!
//! Only operations marked as `idempotent` or `readonly` are automatically retried.
//! Non-idempotent operations require explicit retry configuration.
//!
//! # Configuration
//!
//! ```ignore
//! RetryConfig {
//!     max_attempts: 4,
//!     base_delay_ms: 100,
//!     max_delay_ms: 2000,
//!     backoff: RetryBackoff::ExponentialJitter,
//!     retry_statuses: vec![429, 500, 502, 503, 504],
//!     retry_network_errors: true,
//!     require_idempotent_or_readonly: true,
//!     circuit_breaker: Some(CircuitBreakerConfig { ... }),
//! }
//! ```

use crate::classify::{classify_for_middleware, classify_transport_error};
use crate::middleware::{
    MiddlewareContext, MiddlewareOutcome, PostProcessOutcome, TransportMiddleware,
};
use gunbc_exec::ExecError;
use gunbc_ir::transport::{
    CircuitBreakerConfig, RetryBackoff, RetryConfig, TransportRequest, TransportResponse,
};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Circuit breaker state machine.
#[derive(Debug, Clone)]
pub enum CircuitState {
    /// Normal operation, requests flow through.
    Closed {
        /// Consecutive failure count.
        failure_count: u32,
    },
    /// Requests blocked, waiting for reset timeout.
    Open {
        /// Time when circuit opened.
        opened_at: Instant,
        /// Time to wait before half-open.
        reset_timeout: Duration,
    },
    /// Testing recovery, limited requests allowed.
    HalfOpen {
        /// Successful probes in half-open state.
        success_count: u32,
        /// Failed probes in half-open state.
        failure_count: u32,
        /// Maximum probe requests before deciding.
        max_probes: u32,
    },
}

impl CircuitState {
    fn new() -> Self {
        CircuitState::Closed { failure_count: 0 }
    }
}

/// Circuit breaker protecting against cascading failures.
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Mutex<CircuitState>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Mutex::new(CircuitState::new()),
        }
    }

    /// Check if request should be allowed.
    pub fn should_allow(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        match &*state {
            CircuitState::Closed { .. } => true,
            CircuitState::Open {
                opened_at,
                reset_timeout,
            } => {
                // Check if it's time to half-open
                if opened_at.elapsed() >= *reset_timeout {
                    *state = CircuitState::HalfOpen {
                        success_count: 0,
                        failure_count: 0,
                        max_probes: self.config.half_open_max_requests,
                    };
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen {
                success_count,
                failure_count,
                max_probes,
            } => {
                // Allow probe if under limit
                (success_count + failure_count) < *max_probes
            }
        }
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        match &*state {
            CircuitState::Closed { .. } => {
                // Reset failure count on success
                *state = CircuitState::Closed { failure_count: 0 };
            }
            CircuitState::HalfOpen {
                success_count,
                max_probes,
                ..
            } => {
                let new_success = success_count + 1;
                if new_success >= *max_probes {
                    // Recovered, close circuit
                    *state = CircuitState::Closed { failure_count: 0 };
                } else {
                    *state = CircuitState::HalfOpen {
                        success_count: new_success,
                        failure_count: 0,
                        max_probes: *max_probes,
                    };
                }
            }
            CircuitState::Open { .. } => {
                // Should not happen - success while open
            }
        }
    }

    /// Record a failed request.
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        match &*state {
            CircuitState::Closed { failure_count } => {
                let new_count = failure_count + 1;
                if new_count >= self.config.failure_threshold {
                    // Trip the circuit
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                        reset_timeout: Duration::from_millis(self.config.reset_timeout_ms),
                    };
                } else {
                    *state = CircuitState::Closed {
                        failure_count: new_count,
                    };
                }
            }
            CircuitState::HalfOpen { .. } => {
                // Probe failed, reopen circuit
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                    reset_timeout: Duration::from_millis(self.config.reset_timeout_ms),
                };
            }
            CircuitState::Open { .. } => {
                // Already open, nothing to do
            }
        }
    }

    /// Check if circuit is open.
    pub fn is_open(&self) -> bool {
        matches!(&*self.state.lock().unwrap(), CircuitState::Open { .. })
    }
}

/// Calculate backoff delay for retry attempt.
fn calculate_backoff(config: &RetryConfig, attempt: u32) -> Duration {
    let base = config.base_delay_ms as f64;
    let max = config.max_delay_ms as f64;

    let delay_ms = match config.backoff {
        RetryBackoff::Fixed => base,
        RetryBackoff::Exponential => {
            // 2^(attempt-1) * base, capped at max
            let factor = 2_f64.powi((attempt - 1) as i32);
            (base * factor).min(max)
        }
        RetryBackoff::ExponentialJitter => {
            // Exponential with random jitter ±25%
            let factor = 2_f64.powi((attempt - 1) as i32);
            let base_delay = (base * factor).min(max);
            // Simple deterministic "jitter" based on attempt for reproducibility
            let jitter_factor = 0.75 + (((attempt as f64 * 0.37) % 0.5) as f64);
            base_delay * jitter_factor
        }
    };

    Duration::from_millis(delay_ms as u64)
}

/// Check if a status code should be retried.
fn is_retryable_status(config: &RetryConfig, status: u16) -> bool {
    config.retry_statuses.contains(&status)
}

/// Retry middleware.
pub struct RetryMiddleware {
    config: RetryConfig,
    circuit_breaker: Option<CircuitBreaker>,
}

impl RetryMiddleware {
    pub fn new(config: RetryConfig) -> Self {
        let circuit_breaker = config
            .circuit_breaker
            .as_ref()
            .map(|cb| CircuitBreaker::new(cb.clone()));
        Self {
            config,
            circuit_breaker,
        }
    }

    /// Check if operation can be retried.
    fn can_retry(&self, ctx: &MiddlewareContext) -> bool {
        if self.config.require_idempotent_or_readonly {
            ctx.retry_safe()
        } else {
            true
        }
    }

    /// Check if response indicates a retryable error.
    fn should_retry_response(
        &self,
        response: &TransportResponse,
        ctx: &MiddlewareContext,
    ) -> Option<String> {
        // Check if we've exceeded max attempts
        if ctx.attempt >= self.config.max_attempts {
            return None;
        }

        // Check if operation allows retry
        if !self.can_retry(ctx) {
            return None;
        }

        // Classify the response
        if let Some(classified) =
            classify_for_middleware(response, ctx.config.response_classification.as_ref())
        {
            // Check if error kind is retryable
            if !classified.retryable() {
                return None;
            }

            // Check if specific status is in retry list
            if let Some(status) = classified.status {
                if !is_retryable_status(&self.config, status) {
                    return None;
                }
            }

            // Retryable
            return Some(
                classified
                    .message
                    .unwrap_or_else(|| format!("{:?}", classified.kind)),
            );
        }

        None
    }

    /// Check if error indicates a retryable condition.
    fn should_retry_error(&self, _error: &ExecError, ctx: &MiddlewareContext) -> Option<String> {
        // Check limits
        if ctx.attempt >= self.config.max_attempts {
            return None;
        }

        if !self.can_retry(ctx) {
            return None;
        }

        // Network errors are retryable if configured
        if self.config.retry_network_errors {
            Some("network error".to_string())
        } else {
            None
        }
    }
}

impl TransportMiddleware for RetryMiddleware {
    fn pre_request(
        &self,
        request: TransportRequest,
        _ctx: &mut MiddlewareContext,
    ) -> MiddlewareOutcome {
        // Check circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            if !cb.should_allow() {
                return MiddlewareOutcome::Abort(ExecError::new(
                    "Circuit breaker is open - too many recent failures",
                ));
            }
        }

        MiddlewareOutcome::Continue(request)
    }

    fn post_response(
        &self,
        _request: &TransportRequest,
        response: TransportResponse,
        ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        // Check if this was a success (for circuit breaker)
        let classified =
            classify_for_middleware(&response, ctx.config.response_classification.as_ref());
        let is_failure = classified.is_some();

        if let Some(cb) = &self.circuit_breaker {
            if is_failure {
                cb.record_failure();
            } else {
                cb.record_success();
            }
        }

        // Check if we should retry
        if let Some(reason) = self.should_retry_response(&response, ctx) {
            let delay = calculate_backoff(&self.config, ctx.attempt);
            return PostProcessOutcome::Retry {
                delay_ms: delay.as_millis() as u64,
                reason,
            };
        }

        PostProcessOutcome::Complete(response)
    }

    fn on_error(
        &self,
        _request: &TransportRequest,
        error: ExecError,
        ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        // Record failure for circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            cb.record_failure();
        }

        // Classify the error
        let classified = classify_transport_error(&error.to_string());
        let is_retryable = classified.retryable();

        // Check if we should retry
        if is_retryable {
            if let Some(reason) = self.should_retry_error(&error, ctx) {
                let delay = calculate_backoff(&self.config, ctx.attempt);
                return PostProcessOutcome::Retry {
                    delay_ms: delay.as_millis() as u64,
                    reason,
                };
            }
        }

        PostProcessOutcome::Abort(error)
    }

    fn name(&self) -> &'static str {
        "retry"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::SharedMiddlewareState;
    use gunbc_ir::transport::{LocalRequest, RestResponse, TransportMiddlewareConfig};
    use std::sync::Arc;

    fn basic_retry_config() -> RetryConfig {
        RetryConfig {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 1000,
            backoff: RetryBackoff::Exponential,
            retry_statuses: vec![429, 500, 502, 503, 504],
            retry_network_errors: true,
            require_idempotent_or_readonly: true,
            circuit_breaker: None,
        }
    }

    fn cb_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            reset_timeout_ms: 100,
            half_open_max_requests: 2,
        }
    }

    #[test]
    fn backoff_fixed_returns_constant() {
        let config = RetryConfig {
            base_delay_ms: 100,
            max_delay_ms: 1000,
            backoff: RetryBackoff::Fixed,
            ..basic_retry_config()
        };

        assert_eq!(calculate_backoff(&config, 1), Duration::from_millis(100));
        assert_eq!(calculate_backoff(&config, 2), Duration::from_millis(100));
        assert_eq!(calculate_backoff(&config, 3), Duration::from_millis(100));
    }

    #[test]
    fn backoff_exponential_doubles() {
        let config = RetryConfig {
            base_delay_ms: 100,
            max_delay_ms: 10000,
            backoff: RetryBackoff::Exponential,
            ..basic_retry_config()
        };

        assert_eq!(calculate_backoff(&config, 1), Duration::from_millis(100));
        assert_eq!(calculate_backoff(&config, 2), Duration::from_millis(200));
        assert_eq!(calculate_backoff(&config, 3), Duration::from_millis(400));
        assert_eq!(calculate_backoff(&config, 4), Duration::from_millis(800));
    }

    #[test]
    fn backoff_exponential_caps_at_max() {
        let config = RetryConfig {
            base_delay_ms: 100,
            max_delay_ms: 300,
            backoff: RetryBackoff::Exponential,
            ..basic_retry_config()
        };

        assert_eq!(calculate_backoff(&config, 1), Duration::from_millis(100));
        assert_eq!(calculate_backoff(&config, 2), Duration::from_millis(200));
        assert_eq!(calculate_backoff(&config, 3), Duration::from_millis(300)); // capped
        assert_eq!(calculate_backoff(&config, 4), Duration::from_millis(300)); // capped
    }

    #[test]
    fn circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(cb_config());
        assert!(cb.should_allow());
        assert!(!cb.is_open());
    }

    #[test]
    fn circuit_breaker_opens_on_threshold() {
        let cb = CircuitBreaker::new(cb_config());

        // Record failures up to threshold
        cb.record_failure();
        assert!(cb.should_allow());
        cb.record_failure();
        assert!(cb.should_allow());
        cb.record_failure();

        // Should now be open
        assert!(cb.is_open());
        assert!(!cb.should_allow());
    }

    #[test]
    fn circuit_breaker_success_resets_count() {
        let cb = CircuitBreaker::new(cb_config());

        cb.record_failure();
        cb.record_failure();
        cb.record_success(); // Reset
        cb.record_failure();
        cb.record_failure();

        // Should still be closed (count reset)
        assert!(cb.should_allow());
    }

    #[test]
    fn circuit_breaker_half_open_after_timeout() {
        let mut config = cb_config();
        config.reset_timeout_ms = 10; // Very short for test
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_open());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(20));

        // Should allow (half-open)
        assert!(cb.should_allow());
    }

    #[test]
    fn retry_middleware_allows_idempotent() {
        let mw = RetryMiddleware::new(basic_retry_config());
        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());

        // Idempotent operation
        let ctx = MiddlewareContext::new("test", true, false, mw_config.clone(), shared.clone());
        assert!(mw.can_retry(&ctx));

        // Readonly operation
        let ctx = MiddlewareContext::new("test", false, true, mw_config.clone(), shared.clone());
        assert!(mw.can_retry(&ctx));

        // Non-idempotent, non-readonly
        let ctx = MiddlewareContext::new("test", false, false, mw_config.clone(), shared.clone());
        assert!(!mw.can_retry(&ctx));
    }

    #[test]
    fn retry_middleware_respects_max_attempts() {
        let mw = RetryMiddleware::new(basic_retry_config());
        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test", true, false, mw_config, shared);

        let response = TransportResponse::Rest(RestResponse::new(
            500,
            serde_json::json!({"error": "server error"}),
        ));

        // Attempt 1 - should retry
        ctx.attempt = 1;
        let outcome = mw.post_response(&TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        }), response.clone(), &mut ctx);
        assert!(matches!(outcome, PostProcessOutcome::Retry { .. }));

        // Attempt 2 - should retry
        ctx.attempt = 2;
        let outcome = mw.post_response(&TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        }), response.clone(), &mut ctx);
        assert!(matches!(outcome, PostProcessOutcome::Retry { .. }));

        // Attempt 3 (max) - should not retry
        ctx.attempt = 3;
        let outcome = mw.post_response(&TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        }), response.clone(), &mut ctx);
        assert!(matches!(outcome, PostProcessOutcome::Complete(_)));
    }

    #[test]
    fn retry_middleware_checks_status_code() {
        let mw = RetryMiddleware::new(basic_retry_config());
        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test", true, false, mw_config, shared);
        ctx.attempt = 1;

        // 500 should retry
        let response = TransportResponse::Rest(RestResponse::new(500, serde_json::json!({})));
        let outcome = mw.post_response(&TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        }), response, &mut ctx);
        assert!(matches!(outcome, PostProcessOutcome::Retry { .. }));

        // 400 should not retry (not in retry_statuses)
        let response = TransportResponse::Rest(RestResponse::new(400, serde_json::json!({})));
        let outcome = mw.post_response(&TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        }), response, &mut ctx);
        assert!(matches!(outcome, PostProcessOutcome::Complete(_)));
    }

    #[test]
    fn retry_middleware_blocks_when_circuit_open() {
        let config = RetryConfig {
            circuit_breaker: Some(cb_config()),
            ..basic_retry_config()
        };
        let mw = RetryMiddleware::new(config);
        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test", true, false, mw_config, shared);

        // Record failures to open circuit
        let response = TransportResponse::Rest(RestResponse::new(500, serde_json::json!({})));
        for _ in 0..3 {
            ctx.attempt = 1;
            mw.post_response(&TransportRequest::Local(LocalRequest {
                inputs: serde_json::json!({}),
            }), response.clone(), &mut ctx);
        }

        // Now circuit should be open
        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });
        let outcome = mw.pre_request(request, &mut ctx);
        assert!(matches!(outcome, MiddlewareOutcome::Abort(_)));
    }
}
