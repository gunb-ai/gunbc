//! Declarative behavioral specs for transport executors.
//!
//! These specs describe request/response field contracts and critical routing
//! invariants (for example TCP timeout field routing) in a portable form that
//! can be consumed by generated tests.

use crate::InvocationContract;
use serde::{Deserialize, Serialize};

/// Transport families supported by the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TransportKind {
    Tcp,
    Http,
    Rest,
    File,
    Shell,
    LocalDirect,
}

/// Field-routing invariant within a transport behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRouteSpec {
    /// Request field name.
    pub request_field: String,
    /// Semantic operation the field must drive.
    pub operation: String,
}

impl FieldRouteSpec {
    pub fn new(request_field: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            request_field: request_field.into(),
            operation: operation.into(),
        }
    }
}

/// Behavioral contract for a transport request/response pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportBehavior {
    pub id: String,
    pub transport: TransportKind,
    pub request_type: String,
    pub response_type: String,
    pub required_request_fields: Vec<String>,
    pub optional_request_fields: Vec<String>,
    pub response_fields: Vec<String>,
    pub field_routes: Vec<FieldRouteSpec>,
}

impl TransportBehavior {
    pub fn new(
        id: impl Into<String>,
        transport: TransportKind,
        request_type: impl Into<String>,
        response_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            transport,
            request_type: request_type.into(),
            response_type: response_type.into(),
            required_request_fields: Vec::new(),
            optional_request_fields: Vec::new(),
            response_fields: Vec::new(),
            field_routes: Vec::new(),
        }
    }

    pub fn with_required_fields(mut self, fields: &[&str]) -> Self {
        self.required_request_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn with_optional_fields(mut self, fields: &[&str]) -> Self {
        self.optional_request_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn with_response_fields(mut self, fields: &[&str]) -> Self {
        self.response_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn with_field_routes(mut self, routes: &[(&str, &str)]) -> Self {
        self.field_routes = routes
            .iter()
            .map(|(field, op)| FieldRouteSpec::new(*field, *op))
            .collect();
        self
    }

    /// Shared invocation contract for this transport behavior.
    pub fn invocation_contract(&self) -> InvocationContract {
        let docs = format!("transport behavior contract: {}", self.id);
        match self.transport {
            TransportKind::Tcp => InvocationContract::protocol("tcp", docs),
            TransportKind::Http => InvocationContract::protocol("http", docs),
            TransportKind::Rest => InvocationContract::protocol("rest", docs),
            TransportKind::File => InvocationContract::protocol("file", docs),
            TransportKind::Shell => InvocationContract::protocol("shell", docs),
            TransportKind::LocalDirect => InvocationContract::protocol("local", docs),
        }
    }
}

/// Canonical transport behaviors used by executor and behavioral tests.
pub fn default_transport_behaviors() -> Vec<TransportBehavior> {
    vec![
        TransportBehavior::new(
            "transport.tcp",
            TransportKind::Tcp,
            "TcpRequest",
            "TcpResponse",
        )
        .with_required_fields(&["host", "port"])
        .with_optional_fields(&["data", "read_timeout_ms", "write_timeout_ms"])
        .with_response_fields(&["connected", "data", "bytes_sent", "bytes_received", "error"])
        .with_field_routes(&[
            ("read_timeout_ms", "set_read_timeout"),
            ("write_timeout_ms", "set_write_timeout"),
        ]),
        TransportBehavior::new(
            "transport.http",
            TransportKind::Http,
            "HttpRequest",
            "HttpResponse",
        )
        .with_required_fields(&["url", "method"])
        .with_optional_fields(&["headers", "body", "timeout_ms"])
        .with_response_fields(&["status", "headers", "body", "error"])
        .with_field_routes(&[("timeout_ms", "http_timeout_ms")]),
        TransportBehavior::new(
            "transport.rest",
            TransportKind::Rest,
            "RestRequest",
            "RestResponse",
        )
        .with_required_fields(&["url", "method"])
        .with_optional_fields(&["headers", "body", "timeout_ms"])
        .with_response_fields(&["status", "headers", "body", "error"])
        .with_field_routes(&[("timeout_ms", "rest_timeout_ms")]),
        TransportBehavior::new(
            "transport.file",
            TransportKind::File,
            "FileRequest",
            "FileResponse",
        )
        .with_required_fields(&["path", "operation"])
        .with_optional_fields(&["content", "append", "create_dirs"])
        .with_response_fields(&[
            "path",
            "operation",
            "success",
            "content",
            "exists",
            "error",
        ]),
        TransportBehavior::new(
            "transport.shell",
            TransportKind::Shell,
            "ShellRequest",
            "ShellResponse",
        )
        .with_required_fields(&["command"])
        .with_optional_fields(&["args", "cwd", "env", "stdin", "timeout_ms", "passthrough"])
        .with_response_fields(&["exit_code", "stdout", "stderr", "success", "error"]),
        TransportBehavior::new(
            "transport.local",
            TransportKind::LocalDirect,
            "LocalRequest",
            "LocalResponse",
        )
        .with_required_fields(&["inputs"])
        .with_response_fields(&["outputs"]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_behaviors_cover_all_core_transport_families() {
        let specs = default_transport_behaviors();
        assert_eq!(specs.len(), 6);
        assert!(specs.iter().any(|s| s.transport == TransportKind::Tcp));
        assert!(specs.iter().any(|s| s.transport == TransportKind::Http));
        assert!(specs.iter().any(|s| s.transport == TransportKind::Rest));
        assert!(specs.iter().any(|s| s.transport == TransportKind::File));
        assert!(specs.iter().any(|s| s.transport == TransportKind::Shell));
        assert!(specs
            .iter()
            .any(|s| s.transport == TransportKind::LocalDirect));
    }

    #[test]
    fn tcp_behavior_pins_timeout_field_routes() {
        let specs = default_transport_behaviors();
        let tcp = specs
            .iter()
            .find(|s| s.transport == TransportKind::Tcp)
            .expect("tcp spec present");
        assert!(tcp
            .field_routes
            .contains(&FieldRouteSpec::new("read_timeout_ms", "set_read_timeout")));
        assert!(tcp.field_routes.contains(&FieldRouteSpec::new(
            "write_timeout_ms",
            "set_write_timeout"
        )));
    }

    #[test]
    fn behavior_invocation_contract_matches_transport_kind() {
        let specs = default_transport_behaviors();
        let rest = specs
            .iter()
            .find(|spec| spec.transport == TransportKind::Rest)
            .expect("rest spec present");
        assert!(matches!(
            rest.invocation_contract(),
            InvocationContract::Protocol { protocol, .. } if protocol == "rest"
        ));
    }
}
