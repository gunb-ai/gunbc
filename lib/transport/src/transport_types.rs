//! Transport foundation types for classifying and configuring transport operations.
//!
//! These types provide semantic annotations that guide middleware behavior,
//! test generation, and runtime optimization.

/// Transport class - the fundamental protocol/mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportClass {
    /// REST over HTTP/HTTPS.
    Rest,
    /// Raw HTTP without REST semantics.
    Http,
    /// Shell command execution.
    Shell,
    /// File system operations.
    File,
    /// Raw TCP sockets.
    Tcp,
    /// gRPC (future).
    Grpc,
    /// Streaming connections (SSE, WebSocket).
    Stream,
    /// Pub/sub messaging (future).
    Pubsub,
    /// Custom/plugin transport.
    Custom,
}

impl TransportClass {
    /// Whether this transport class supports connection pooling.
    pub fn supports_pooling(&self) -> bool {
        matches!(self, Self::Rest | Self::Http | Self::Grpc | Self::Tcp)
    }

    /// Whether this transport class is inherently streaming.
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::Stream | Self::Pubsub)
    }

    /// Whether this transport class has request/response semantics.
    pub fn is_request_response(&self) -> bool {
        matches!(
            self,
            Self::Rest | Self::Http | Self::Grpc | Self::Shell | Self::File
        )
    }
}

/// Capabilities of a transport implementation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransportCapabilities {
    /// Supports connection pooling for reuse.
    pub connection_pooling: bool,
    /// Operations are safe to retry on failure.
    pub retry_safe: bool,
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports bidirectional communication.
    pub bidirectional: bool,
    /// Supports multiplexing multiple requests over one connection.
    pub multiplexing: bool,
    /// Has built-in compression support.
    pub compression: bool,
}

/// Endpoint-level behavior hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointBehavior {
    /// Endpoint is rate-limited; respect limits.
    RateLimited,
    /// Response can be cached.
    Cacheable,
    /// Operation is idempotent; safe to retry.
    Idempotent,
    /// Response is paginated; may need multiple requests.
    Paginated,
    /// Endpoint requires authentication.
    Authenticated,
    /// Endpoint supports conditional requests (ETag/If-Modified-Since).
    Conditional,
    /// Endpoint is deprecated; warn on use.
    Deprecated,
    /// Endpoint has known latency; adjust timeouts.
    HighLatency,
}

/// Operation-level behavioral properties.
///
/// These flags guide middleware decisions about retry, caching, and observability.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperationBehavior {
    /// Operation only reads state; no side effects.
    pub readonly: bool,
    /// Operation can be safely retried without side effects.
    pub idempotent: bool,
    /// Operation has no external dependencies (for testing).
    pub hermetic: bool,
    /// Expected behaviors for this endpoint.
    pub behaviors: Vec<EndpointBehavior>,
    /// Custom retry timeout override (ms).
    pub timeout_ms: Option<u64>,
    /// Maximum retry attempts override.
    pub max_retries: Option<u32>,
    /// Known failure modes for this operation.
    pub failure_modes: Vec<FailureMode>,
}

impl OperationBehavior {
    /// Create a new readonly operation.
    pub fn readonly() -> Self {
        Self {
            readonly: true,
            idempotent: true, // readonly implies idempotent
            ..Default::default()
        }
    }

    /// Create a new idempotent operation.
    pub fn idempotent() -> Self {
        Self {
            idempotent: true,
            ..Default::default()
        }
    }

    /// Create a new hermetic operation (for testing).
    pub fn hermetic() -> Self {
        Self {
            hermetic: true,
            ..Default::default()
        }
    }

    /// Whether this operation is safe to retry.
    pub fn retry_safe(&self) -> bool {
        self.readonly || self.idempotent
    }

    /// Check if a specific behavior is declared.
    pub fn has_behavior(&self, behavior: EndpointBehavior) -> bool {
        self.behaviors.contains(&behavior)
    }
}

/// Known failure modes for an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureMode {
    /// Operation may time out under load.
    Timeout,
    /// Operation may be rate limited.
    RateLimited,
    /// Operation may fail due to auth issues.
    AuthenticationRequired,
    /// Operation may fail due to missing resource.
    NotFound,
    /// Operation may fail due to conflict.
    Conflict,
    /// Operation may fail due to validation errors.
    ValidationError,
    /// Operation may fail due to quota exhaustion.
    QuotaExceeded,
    /// Custom failure mode with description.
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_class_capabilities() {
        assert!(TransportClass::Rest.supports_pooling());
        assert!(TransportClass::Rest.is_request_response());
        assert!(!TransportClass::Rest.is_streaming());

        assert!(TransportClass::Stream.is_streaming());
        assert!(!TransportClass::Shell.supports_pooling());
    }

    #[test]
    fn operation_behavior_retry_safe() {
        let readonly = OperationBehavior::readonly();
        assert!(readonly.retry_safe());

        let idempotent = OperationBehavior::idempotent();
        assert!(idempotent.retry_safe());

        let default = OperationBehavior::default();
        assert!(!default.retry_safe());
    }

    #[test]
    fn operation_behavior_has_behavior() {
        let behavior = OperationBehavior {
            behaviors: vec![EndpointBehavior::RateLimited, EndpointBehavior::Cacheable],
            ..Default::default()
        };

        assert!(behavior.has_behavior(EndpointBehavior::RateLimited));
        assert!(behavior.has_behavior(EndpointBehavior::Cacheable));
        assert!(!behavior.has_behavior(EndpointBehavior::Deprecated));
    }
}
