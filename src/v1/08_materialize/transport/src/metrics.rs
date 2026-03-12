// LogMetricsSink intentionally writes telemetry to stderr.
#![allow(clippy::disallowed_macros)]

//! Transport metrics hooks.
//!
//! Provides observability for transport operations: request counts, timing,
//! retry tracking, rate limit headroom, and error distribution.
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_lib_transport::metrics::{MetricsSink, LogMetricsSink, MetricsMiddleware};
//!
//! let sink = Arc::new(LogMetricsSink::new());
//! let middleware = MetricsMiddleware::new(sink);
//! ```

use crate::classify::ClassifiedErrorKind;
use crate::middleware::{
    MiddlewareContext, MiddlewareOutcome, PostProcessOutcome, TransportMiddleware,
};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Trait for metrics collection sinks.
///
/// Implementations can log, emit structured events, push to metrics systems,
/// or do nothing (for tests).
pub trait MetricsSink: Send + Sync {
    /// Record that a request is starting.
    fn record_request(&self, operation_id: &str, transport_kind: &str);

    /// Record that a response was received.
    fn record_response(&self, operation_id: &str, status: Option<u16>, duration_ms: u64);

    /// Record that a retry is happening.
    fn record_retry(&self, operation_id: &str, attempt: u32, reason: &str);

    /// Record rate limit headroom for a scope.
    fn record_rate_limit_headroom(&self, scope: &str, headroom: f64);

    /// Record an error classification.
    fn record_error(&self, operation_id: &str, error_kind: ClassifiedErrorKind);
}

/// No-op metrics sink for tests and when metrics are disabled.
#[derive(Debug, Default)]
pub struct NullMetricsSink;

impl NullMetricsSink {
    pub fn new() -> Self {
        Self
    }
}

impl MetricsSink for NullMetricsSink {
    fn record_request(&self, _operation_id: &str, _transport_kind: &str) {}
    fn record_response(&self, _operation_id: &str, _status: Option<u16>, _duration_ms: u64) {}
    fn record_retry(&self, _operation_id: &str, _attempt: u32, _reason: &str) {}
    fn record_rate_limit_headroom(&self, _scope: &str, _headroom: f64) {}
    fn record_error(&self, _operation_id: &str, _error_kind: ClassifiedErrorKind) {}
}

/// Logging metrics sink that writes to stderr.
#[derive(Debug, Default)]
pub struct LogMetricsSink {
    /// Whether to include timestamps in log output.
    pub include_timestamps: bool,
}

impl LogMetricsSink {
    pub fn new() -> Self {
        Self {
            include_timestamps: true,
        }
    }

    pub fn without_timestamps() -> Self {
        Self {
            include_timestamps: false,
        }
    }

    fn timestamp(&self) -> String {
        if self.include_timestamps {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            format!("[{}.{:03}] ", now.as_secs(), now.subsec_millis())
        } else {
            String::new()
        }
    }
}

impl MetricsSink for LogMetricsSink {
    fn record_request(&self, operation_id: &str, transport_kind: &str) {
        eprintln!(
            "{}METRIC request op={} transport={}",
            self.timestamp(),
            operation_id,
            transport_kind
        );
    }

    fn record_response(&self, operation_id: &str, status: Option<u16>, duration_ms: u64) {
        let status_str = status.map_or("N/A".to_string(), |s| s.to_string());
        eprintln!(
            "{}METRIC response op={} status={} duration_ms={}",
            self.timestamp(),
            operation_id,
            status_str,
            duration_ms
        );
    }

    fn record_retry(&self, operation_id: &str, attempt: u32, reason: &str) {
        eprintln!(
            "{}METRIC retry op={} attempt={} reason={}",
            self.timestamp(),
            operation_id,
            attempt,
            reason
        );
    }

    fn record_rate_limit_headroom(&self, scope: &str, headroom: f64) {
        eprintln!(
            "{}METRIC rate_limit scope={} headroom={:.2}",
            self.timestamp(),
            scope,
            headroom
        );
    }

    fn record_error(&self, operation_id: &str, error_kind: ClassifiedErrorKind) {
        eprintln!(
            "{}METRIC error op={} kind={:?}",
            self.timestamp(),
            operation_id,
            error_kind
        );
    }
}

/// In-memory metrics collector for testing.
#[derive(Debug, Default)]
pub struct InMemoryMetricsSink {
    requests: Mutex<Vec<(String, String)>>,
    responses: Mutex<Vec<(String, Option<u16>, u64)>>,
    retries: Mutex<Vec<(String, u32, String)>>,
    rate_limits: Mutex<Vec<(String, f64)>>,
    errors: Mutex<Vec<(String, ClassifiedErrorKind)>>,
}

impl InMemoryMetricsSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    pub fn response_count(&self) -> usize {
        self.responses.lock().unwrap().len()
    }

    pub fn retry_count(&self) -> usize {
        self.retries.lock().unwrap().len()
    }

    pub fn error_count(&self) -> usize {
        self.errors.lock().unwrap().len()
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.responses
            .lock()
            .unwrap()
            .iter()
            .map(|(_, _, d)| d)
            .sum()
    }

    pub fn errors_by_kind(&self) -> HashMap<ClassifiedErrorKind, usize> {
        let errors = self.errors.lock().unwrap();
        let mut counts = HashMap::new();
        for (_, kind) in errors.iter() {
            *counts.entry(*kind).or_insert(0) += 1;
        }
        counts
    }
}

impl MetricsSink for InMemoryMetricsSink {
    fn record_request(&self, operation_id: &str, transport_kind: &str) {
        self.requests
            .lock()
            .unwrap()
            .push((operation_id.to_string(), transport_kind.to_string()));
    }

    fn record_response(&self, operation_id: &str, status: Option<u16>, duration_ms: u64) {
        self.responses
            .lock()
            .unwrap()
            .push((operation_id.to_string(), status, duration_ms));
    }

    fn record_retry(&self, operation_id: &str, attempt: u32, reason: &str) {
        self.retries
            .lock()
            .unwrap()
            .push((operation_id.to_string(), attempt, reason.to_string()));
    }

    fn record_rate_limit_headroom(&self, scope: &str, headroom: f64) {
        self.rate_limits
            .lock()
            .unwrap()
            .push((scope.to_string(), headroom));
    }

    fn record_error(&self, operation_id: &str, error_kind: ClassifiedErrorKind) {
        self.errors
            .lock()
            .unwrap()
            .push((operation_id.to_string(), error_kind));
    }
}

/// Per-request timing state.
struct RequestTiming {
    start: Instant,
}

/// Metrics middleware that records request/response timing and counts.
pub struct MetricsMiddleware {
    sink: Arc<dyn MetricsSink>,
    /// Active request timings, keyed by unique request_id to avoid collision.
    timings: Mutex<HashMap<u64, RequestTiming>>,
}

impl MetricsMiddleware {
    pub fn new(sink: Arc<dyn MetricsSink>) -> Self {
        Self {
            sink,
            timings: Mutex::new(HashMap::new()),
        }
    }
}

impl TransportMiddleware for MetricsMiddleware {
    fn pre_request(
        &self,
        request: TransportRequest,
        ctx: &mut MiddlewareContext,
    ) -> MiddlewareOutcome {
        let transport_kind = transport_kind_str(&request);
        self.sink.record_request(&ctx.operation_id, transport_kind);

        // Store timing for this request, keyed by unique request_id
        let timing = RequestTiming {
            start: Instant::now(),
        };
        self.timings.lock().unwrap().insert(ctx.request_id, timing);

        MiddlewareOutcome::Continue(request)
    }

    fn post_response(
        &self,
        _request: &TransportRequest,
        response: TransportResponse,
        ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        let duration_ms = self
            .timings
            .lock()
            .unwrap()
            .remove(&ctx.request_id)
            .map(|t| t.start.elapsed().as_millis() as u64)
            .unwrap_or(0);

        let status = extract_status(&response);
        self.sink
            .record_response(&ctx.operation_id, status, duration_ms);

        PostProcessOutcome::Complete(response)
    }

    fn on_error(
        &self,
        _request: &TransportRequest,
        error: gunbc_exec::ExecError,
        ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        // Clean up timing state
        self.timings.lock().unwrap().remove(&ctx.request_id);

        // Don't record synthetic cleanup errors as real failures
        let error_msg = error.to_string();
        if error_msg.contains("pipeline cleanup") {
            return PostProcessOutcome::Abort(error);
        }

        // Classify error based on message content
        let error_kind = classify_exec_error(&error_msg);
        self.sink.record_error(&ctx.operation_id, error_kind);

        PostProcessOutcome::Abort(error)
    }

    fn name(&self) -> &'static str {
        "metrics"
    }
}

/// Extract transport kind as a string for logging.
fn transport_kind_str(request: &TransportRequest) -> &'static str {
    match request {
        TransportRequest::Rest(_) => "rest",
        TransportRequest::Http(_) => "http",
        TransportRequest::File(_) => "file",
        TransportRequest::Tcp(_) => "tcp",
        TransportRequest::Shell(_) => "shell",
        TransportRequest::Local(_) => "local",
    }
}

/// Extract HTTP status from response if applicable.
fn extract_status(response: &TransportResponse) -> Option<u16> {
    match response {
        TransportResponse::Rest(r) => Some(r.status),
        TransportResponse::Http(r) => Some(r.status),
        _ => None,
    }
}

/// Classify an execution error based on message content.
///
/// This is a best-effort heuristic for errors that don't have structured
/// classification (e.g., ExecError from transport failures).
fn classify_exec_error(message: &str) -> ClassifiedErrorKind {
    let lower = message.to_ascii_lowercase();

    // Auth errors
    if lower.contains("auth")
        || lower.contains("credential")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("invalid api key")
        || lower.contains("token")
    {
        return ClassifiedErrorKind::Auth;
    }

    // Rate limit errors
    if lower.contains("rate limit") || lower.contains("too many requests") || lower.contains("429")
    {
        return ClassifiedErrorKind::RateLimit;
    }

    // Client errors (config, serialization, validation)
    if lower.contains("invalid")
        || lower.contains("missing")
        || lower.contains("config")
        || lower.contains("serializ")
        || lower.contains("deserializ")
        || lower.contains("parse")
    {
        return ClassifiedErrorKind::Client;
    }

    // Server errors
    if lower.contains("server")
        || lower.contains("internal error")
        || lower.contains("5xx")
        || lower.contains("500")
    {
        return ClassifiedErrorKind::Server;
    }

    // Default to network for connection/timeout issues
    ClassifiedErrorKind::Network
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{LocalRequest, LocalResponse, TransportMiddlewareConfig};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn null_sink_compiles_and_runs() {
        let sink = NullMetricsSink::new();
        sink.record_request("test.op", "rest");
        sink.record_response("test.op", Some(200), 100);
        sink.record_retry("test.op", 2, "rate limit");
        sink.record_rate_limit_headroom("github:core", 0.5);
        sink.record_error("test.op", ClassifiedErrorKind::RateLimit);
    }

    #[test]
    fn in_memory_sink_counts_correctly() {
        let sink = InMemoryMetricsSink::new();
        sink.record_request("op1", "rest");
        sink.record_request("op2", "shell");
        sink.record_response("op1", Some(200), 50);
        sink.record_response("op2", None, 100);
        sink.record_retry("op1", 2, "server error");
        sink.record_error("op1", ClassifiedErrorKind::Server);
        sink.record_error("op2", ClassifiedErrorKind::Server);

        assert_eq!(sink.request_count(), 2);
        assert_eq!(sink.response_count(), 2);
        assert_eq!(sink.retry_count(), 1);
        assert_eq!(sink.total_duration_ms(), 150);
        assert_eq!(sink.error_count(), 2);
        assert_eq!(
            sink.errors_by_kind().get(&ClassifiedErrorKind::Server),
            Some(&2)
        );
    }

    #[test]
    fn metrics_middleware_records_timing() {
        let sink = Arc::new(InMemoryMetricsSink::new());
        let mw = MetricsMiddleware::new(sink.clone());
        let config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(crate::middleware::SharedMiddlewareState::new());
        let mut ctx =
            crate::middleware::MiddlewareContext::new("test.op", false, true, config, shared);

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });

        // Pre-request
        let outcome = mw.pre_request(request, &mut ctx);
        assert!(matches!(outcome, MiddlewareOutcome::Continue(_)));
        assert_eq!(sink.request_count(), 1);

        // Simulate some work
        thread::sleep(Duration::from_millis(10));

        // Post-response
        let response = TransportResponse::Local(LocalResponse {
            outputs: serde_json::json!({}),
        });
        let outcome = mw.post_response(
            &TransportRequest::Local(LocalRequest {
                inputs: serde_json::json!({}),
            }),
            response,
            &mut ctx,
        );
        assert!(matches!(outcome, PostProcessOutcome::Complete(_)));
        assert_eq!(sink.response_count(), 1);
        assert!(sink.total_duration_ms() >= 10);
    }

    #[test]
    fn transport_kind_str_matches_variants() {
        use gunbc_ir::transport::*;

        assert_eq!(
            transport_kind_str(&TransportRequest::Rest(RestRequest::get(
                "https://example.com"
            ))),
            "rest"
        );
        assert_eq!(
            transport_kind_str(&TransportRequest::Shell(ShellRequest {
                command: "echo".to_string(),
                args: vec![],
                cwd: None,
                env: HashMap::new(),
                stdin: None,
                timeout_ms: None,
                passthrough: false,
                semantics: None,
            })),
            "shell"
        );
        assert_eq!(
            transport_kind_str(&TransportRequest::Local(LocalRequest {
                inputs: serde_json::json!({})
            })),
            "local"
        );
    }
}
