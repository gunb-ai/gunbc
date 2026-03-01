//! Transport middleware pipeline composition.
//!
//! Composes middleware layers into a pipeline that wraps transport execution.
//! Middleware is applied in order: outer layers see the request first on the
//! way in, and the response first on the way out.
//!
//! # Pipeline Order (standard pipeline)
//!
//! ```text
//! Request flow:  metrics → retry → rate_limit → execute
//! Response flow: execute → rate_limit → retry → metrics
//! ```
//!
//! Note: The credential layer is NOT included in the standard pipeline.
//! Use `.layer(Arc::new(CredentialMiddleware::new(config)))` to add it.
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_lib_transport::pipeline::TransportPipelineBuilder;
//!
//! let pipeline = TransportPipelineBuilder::standard(&middleware_config)
//!     .with_metrics_sink(Arc::new(LogMetricsSink::new()))
//!     .build();
//!
//! let response = pipeline.execute(request, "my.operation", true, false)?;
//! ```

use crate::metrics::{MetricsMiddleware, MetricsSink, NullMetricsSink};
use crate::middleware::{
    MiddlewareContext, MiddlewareOutcome, PostProcessOutcome, SharedMiddlewareState,
    TransportMiddleware,
};
use crate::rate_limit::RateLimitMiddleware;
use crate::retry::RetryMiddleware;
use gunbc_exec::{ExecError, IntoExecResult};
use gunbc_ir::transport::{TransportMiddlewareConfig, TransportRequest, TransportResponse};
use std::sync::Arc;

/// Executor function type for actual transport execution.
pub type ExecutorFn =
    Box<dyn Fn(&TransportRequest) -> Result<TransportResponse, ExecError> + Send + Sync>;

/// Builder for constructing middleware pipelines.
pub struct TransportPipelineBuilder {
    layers: Vec<Arc<dyn TransportMiddleware>>,
    config: TransportMiddlewareConfig,
    metrics_sink: Option<Arc<dyn MetricsSink>>,
    executor: Option<ExecutorFn>,
}

impl TransportPipelineBuilder {
    /// Create a new empty pipeline builder.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            config: TransportMiddlewareConfig::default(),
            metrics_sink: None,
            executor: None,
        }
    }

    /// Set the middleware configuration.
    pub fn with_config(mut self, config: TransportMiddlewareConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a custom middleware layer.
    ///
    /// Layers are applied in order: first added is outermost.
    pub fn layer(mut self, middleware: Arc<dyn TransportMiddleware>) -> Self {
        self.layers.push(middleware);
        self
    }

    /// Set a custom metrics sink.
    pub fn with_metrics_sink(mut self, sink: Arc<dyn MetricsSink>) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    /// Set a custom executor function.
    ///
    /// By default, the pipeline will use the crate's internal executor.
    pub fn with_executor(mut self, executor: ExecutorFn) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Build a standard pipeline based on the middleware configuration.
    ///
    /// Creates middleware layers based on what's configured:
    /// - Always adds metrics (outermost)
    /// - Adds rate_limit if configured
    /// - Adds retry if configured
    pub fn standard(config: &TransportMiddlewareConfig) -> Self {
        let mut builder = Self::new().with_config(config.clone());

        // Metrics is always added (outermost)
        // Will be added in build() with the configured sink

        // Retry is outer to rate_limit so that rate_limit sees 429 responses first,
        // updates its pause_until state from Retry-After headers, then passes
        // the response to retry middleware for the actual retry decision.
        if let Some(retry_config) = &config.retry {
            builder = builder.layer(Arc::new(RetryMiddleware::new(retry_config.clone())));
        }

        // Rate limit is inner - it sees responses before retry and can extract
        // Retry-After headers to update its internal state
        if let Some(rate_config) = &config.rate_limit {
            builder = builder.layer(Arc::new(RateLimitMiddleware::new(rate_config.clone())));
        }

        builder
    }

    /// Build the pipeline.
    pub fn build(self) -> TransportPipeline {
        let mut layers = Vec::new();

        // Metrics is always outermost
        let metrics_sink = self
            .metrics_sink
            .unwrap_or_else(|| Arc::new(NullMetricsSink::new()));
        layers.push(Arc::new(MetricsMiddleware::new(metrics_sink)) as Arc<dyn TransportMiddleware>);

        // Then the configured layers
        layers.extend(self.layers);

        TransportPipeline {
            layers,
            config: Arc::new(self.config),
            shared_state: Arc::new(SharedMiddlewareState::new()),
            executor: self.executor,
        }
    }
}

impl Default for TransportPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Composed middleware pipeline for transport execution.
pub struct TransportPipeline {
    layers: Vec<Arc<dyn TransportMiddleware>>,
    config: Arc<TransportMiddlewareConfig>,
    shared_state: Arc<SharedMiddlewareState>,
    executor: Option<ExecutorFn>,
}

impl TransportPipeline {
    /// Execute a request through the middleware pipeline.
    ///
    /// # Arguments
    ///
    /// * `request` - The transport request to execute
    /// * `operation_id` - Identifier for logging/metrics
    /// * `idempotent` - Whether the operation is idempotent
    /// * `readonly` - Whether the operation is read-only
    pub fn execute(
        &self,
        request: TransportRequest,
        operation_id: &str,
        idempotent: bool,
        readonly: bool,
    ) -> Result<TransportResponse, ExecError> {
        let mut ctx = MiddlewareContext::new(
            operation_id,
            idempotent,
            readonly,
            self.config.clone(),
            self.shared_state.clone(),
        );

        self.execute_with_retry(request, &mut ctx)
    }

    /// Execute with retry loop.
    fn execute_with_retry(
        &self,
        request: TransportRequest,
        ctx: &mut MiddlewareContext,
    ) -> Result<TransportResponse, ExecError> {
        let max_attempts = self
            .config
            .retry
            .as_ref()
            .map_or(1, |r| r.max_attempts);

        loop {
            let result = self.execute_once(request.clone(), ctx);

            match result {
                Ok(PostProcessOutcome::Complete(response)) => return Ok(response),
                Ok(PostProcessOutcome::Retry { delay_ms, reason }) => {
                    if ctx.attempt >= max_attempts {
                        return Err(ExecError::new(format!(
                            "Max retry attempts ({}) exceeded: {}",
                            max_attempts, reason
                        )));
                    }
                    ctx.attempt += 1;
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    // Continue loop
                }
                Ok(PostProcessOutcome::Abort(error)) => return Err(error),
                Err(error) => return Err(error),
            }
        }
    }

    /// Execute request once through the pipeline.
    ///
    /// Tracks which layers have run `pre_request` to ensure proper cleanup
    /// when inner layers abort or request retry.
    fn execute_once(
        &self,
        request: TransportRequest,
        ctx: &mut MiddlewareContext,
    ) -> Result<PostProcessOutcome, ExecError> {
        // Track how many layers successfully ran pre_request
        let mut layers_completed_pre = 0;
        // Keep a clone for cleanup in case of early exit
        let request_for_cleanup = request.clone();
        let mut current_request = request;

        // Pre-request processing (outer to inner)
        for layer in &self.layers {
            match layer.pre_request(current_request, ctx) {
                MiddlewareOutcome::Continue(req) => {
                    current_request = req;
                    layers_completed_pre += 1;
                }
                MiddlewareOutcome::ShortCircuit(response) => {
                    // Cleanup layers that already ran pre_request
                    self.cleanup_layers(layers_completed_pre, &request_for_cleanup, ctx);
                    return Ok(PostProcessOutcome::Complete(response));
                }
                MiddlewareOutcome::Abort(error) => {
                    // Cleanup layers that already ran pre_request
                    self.cleanup_layers(layers_completed_pre, &request_for_cleanup, ctx);
                    return Ok(PostProcessOutcome::Abort(error));
                }
            }
        }

        // Use the final transformed request for execution
        let request = current_request;

        // Execute the actual transport
        let result = if let Some(executor) = &self.executor {
            executor(&request)
        } else {
            // Use crate-internal executor
            crate::backend::execute_transport_with_backend(&request)
                .exec_context("transport error")
        };

        // Post-process result
        match result {
            Ok(mut response) => {
                // Post-response processing (inner to outer)
                // All layers ran pre_request, so process all in reverse
                for (idx, layer) in self.layers.iter().enumerate().rev() {
                    match layer.post_response(&request, response, ctx) {
                        PostProcessOutcome::Complete(resp) => response = resp,
                        outcome @ PostProcessOutcome::Retry { .. } => {
                            // Clean up remaining outer layers that haven't seen post_response
                            self.cleanup_layers(idx, &request, ctx);
                            return Ok(outcome);
                        }
                        outcome @ PostProcessOutcome::Abort(_) => {
                            // Clean up remaining outer layers
                            self.cleanup_layers(idx, &request, ctx);
                            return Ok(outcome);
                        }
                    }
                }
                Ok(PostProcessOutcome::Complete(response))
            }
            Err(error) => {
                // Error processing (inner to outer)
                let mut outcome = PostProcessOutcome::Abort(error);
                for (idx, layer) in self.layers.iter().enumerate().rev() {
                    if let PostProcessOutcome::Abort(e) = outcome {
                        outcome = layer.on_error(&request, e, ctx);
                    } else {
                        // Inner layer transformed error to Retry/Complete
                        // Clean up remaining outer layers (indices 0..=idx)
                        self.cleanup_layers(idx + 1, &request, ctx);
                        break;
                    }
                }
                Ok(outcome)
            }
        }
    }

    /// Clean up layers that ran pre_request but won't see post_response.
    ///
    /// Calls on_error with a synthetic cleanup error for the first N layers
    /// (outer layers) that successfully ran pre_request.
    fn cleanup_layers(
        &self,
        count: usize,
        request: &TransportRequest,
        ctx: &mut MiddlewareContext,
    ) {
        // Create a synthetic cleanup error - layers use this to clean up state
        let cleanup_error = ExecError::new("pipeline cleanup (request did not complete)");

        // Call on_error for layers 0..count in reverse order (inner to outer cleanup)
        for layer in self.layers[..count].iter().rev() {
            // Ignore the outcome - we're just cleaning up
            let _ = layer.on_error(request, cleanup_error.clone(), ctx);
        }
    }

    /// Get the number of middleware layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get the middleware layer names.
    pub fn layer_names(&self) -> Vec<&'static str> {
        self.layers.iter().map(|l| l.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::InMemoryMetricsSink;
    use gunbc_ir::transport::{
        LocalRequest, LocalResponse, RateLimitAlgorithm, RateLimitConfig, RetryBackoff,
        RetryConfig,
    };

    fn mock_executor() -> ExecutorFn {
        Box::new(|_req| {
            Ok(TransportResponse::Local(LocalResponse {
                outputs: serde_json::json!({"result": "ok"}),
            }))
        })
    }

    fn failing_executor(fail_count: Arc<std::sync::atomic::AtomicU32>) -> ExecutorFn {
        Box::new(move |_req| {
            let count = fail_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count < 2 {
                Err(ExecError::new("transient failure"))
            } else {
                Ok(TransportResponse::Local(LocalResponse {
                    outputs: serde_json::json!({"result": "ok"}),
                }))
            }
        })
    }

    use std::sync::atomic::AtomicU32;

    #[test]
    fn empty_pipeline_executes_directly() {
        let pipeline = TransportPipelineBuilder::new()
            .with_executor(mock_executor())
            .build();

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });

        let response = pipeline.execute(request, "test.op", false, true);
        assert!(response.is_ok());
    }

    #[test]
    fn standard_pipeline_adds_metrics() {
        let config = TransportMiddlewareConfig::default();
        let pipeline = TransportPipelineBuilder::standard(&config)
            .with_executor(mock_executor())
            .build();

        assert!(pipeline.layer_names().contains(&"metrics"));
    }

    #[test]
    fn standard_pipeline_adds_rate_limit_when_configured() {
        let config = TransportMiddlewareConfig {
            rate_limit: Some(RateLimitConfig {
                scope_key: "test".to_string(),
                algorithm: RateLimitAlgorithm::TokenBucket,
                max_burst: 10,
                requests: 60,
                window_seconds: 60,
                honor_retry_after: true,
            }),
            ..Default::default()
        };

        let pipeline = TransportPipelineBuilder::standard(&config)
            .with_executor(mock_executor())
            .build();

        assert!(pipeline.layer_names().contains(&"rate_limit"));
    }

    #[test]
    fn standard_pipeline_adds_retry_when_configured() {
        let config = TransportMiddlewareConfig {
            retry: Some(RetryConfig {
                max_attempts: 3,
                base_delay_ms: 10,
                max_delay_ms: 100,
                backoff: RetryBackoff::Fixed,
                retry_statuses: vec![500],
                retry_network_errors: true,
                require_idempotent_or_readonly: true,
                circuit_breaker: None,
            }),
            ..Default::default()
        };

        let pipeline = TransportPipelineBuilder::standard(&config)
            .with_executor(mock_executor())
            .build();

        assert!(pipeline.layer_names().contains(&"retry"));
    }

    #[test]
    fn pipeline_records_metrics() {
        let sink = Arc::new(InMemoryMetricsSink::new());
        let pipeline = TransportPipelineBuilder::new()
            .with_metrics_sink(sink.clone())
            .with_executor(mock_executor())
            .build();

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });

        let _ = pipeline.execute(request, "test.op", false, true);

        assert_eq!(sink.request_count(), 1);
        assert_eq!(sink.response_count(), 1);
    }

    #[test]
    fn pipeline_retries_on_error() {
        let config = TransportMiddlewareConfig {
            retry: Some(RetryConfig {
                max_attempts: 5,
                base_delay_ms: 1, // Minimal delay for test
                max_delay_ms: 10,
                backoff: RetryBackoff::Fixed,
                retry_statuses: vec![500],
                retry_network_errors: true,
                require_idempotent_or_readonly: true,
                circuit_breaker: None,
            }),
            ..Default::default()
        };

        let fail_count = Arc::new(AtomicU32::new(0));
        let pipeline = TransportPipelineBuilder::standard(&config)
            .with_executor(failing_executor(fail_count.clone()))
            .build();

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });

        // Should succeed after retries (idempotent = true)
        let response = pipeline.execute(request, "test.op", true, false);
        assert!(response.is_ok());

        // Should have tried 3 times (2 failures + 1 success)
        assert_eq!(fail_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn pipeline_respects_idempotency_requirement() {
        let config = TransportMiddlewareConfig {
            retry: Some(RetryConfig {
                max_attempts: 5,
                base_delay_ms: 1,
                max_delay_ms: 10,
                backoff: RetryBackoff::Fixed,
                retry_statuses: vec![500],
                retry_network_errors: true,
                require_idempotent_or_readonly: true,
                circuit_breaker: None,
            }),
            ..Default::default()
        };

        let fail_count = Arc::new(AtomicU32::new(0));
        let pipeline = TransportPipelineBuilder::standard(&config)
            .with_executor(failing_executor(fail_count.clone()))
            .build();

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });

        // Non-idempotent should NOT retry
        let response = pipeline.execute(request, "test.op", false, false);
        assert!(response.is_err());

        // Should have only tried once
        assert_eq!(fail_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn layer_names_returns_correct_order() {
        let config = TransportMiddlewareConfig {
            rate_limit: Some(RateLimitConfig {
                scope_key: "test".to_string(),
                algorithm: RateLimitAlgorithm::TokenBucket,
                max_burst: 10,
                requests: 60,
                window_seconds: 60,
                honor_retry_after: true,
            }),
            retry: Some(RetryConfig {
                max_attempts: 3,
                base_delay_ms: 10,
                max_delay_ms: 100,
                backoff: RetryBackoff::Fixed,
                retry_statuses: vec![500],
                retry_network_errors: true,
                require_idempotent_or_readonly: true,
                circuit_breaker: None,
            }),
            ..Default::default()
        };

        let pipeline = TransportPipelineBuilder::standard(&config)
            .with_executor(mock_executor())
            .build();

        let names = pipeline.layer_names();
        // Order: metrics (outermost) → retry → rate_limit (innermost)
        // Rate limit is inner to see 429 responses before retry consumes them
        assert_eq!(names, vec!["metrics", "retry", "rate_limit"]);
    }
}
