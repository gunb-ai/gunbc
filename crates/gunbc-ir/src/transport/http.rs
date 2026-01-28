//! HTTP transport layer understanding.
//!
//! This layer wraps TCP to provide HTTP request/response semantics.

use crate::{
    edge, port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId, PortName,
};
use crate::transport::external_types;

/// HTTP layer operations.
#[derive(Debug, Clone)]
pub enum HttpOp {
    /// Format an HTTP request from method, url, headers, body.
    FormatRequest,
    /// Parse an HTTP response into status, headers, body.
    ParseResponse,
    /// Mock: return a canned HTTP response.
    MockResponse { status: u16, body: String },
}

/// Build a real HTTP understanding SubDAG.
///
/// This SubDAG wraps a TCP SubDAG to perform HTTP over TCP.
pub fn build_http_real<T: Clone>(_tcp: Dag<T>) -> Dag<HttpOp> {
    // For now, simplified structure - in practice would wrap tcp SubDAG
    let nodes = vec![
        Node {
            id: NodeId("format_request".into()),
            inputs: vec![
                port("method", "String"),
                port("url", "String"),
                port("headers", "MapStrStr"),
                port("body", "String"),
            ],
            outputs: vec![port("request_bytes", "Bytes")],
            body: NodeBody::Opaque(HttpOp::FormatRequest),
        },
        // In a real implementation, tcp SubDAG would be here
        Node {
            id: NodeId("parse_response".into()),
            inputs: vec![port("response_bytes", "Bytes")],
            outputs: vec![
                port("status", "Int"),
                port("headers", "MapStrStr"),
                port("body", "String"),
            ],
            body: NodeBody::Opaque(HttpOp::ParseResponse),
        },
    ];

    let edges = vec![
        // format_request -> tcp -> parse_response (tcp layer omitted for simplicity)
        edge("format_request", "request_bytes", "parse_response", "response_bytes"),
    ];

    let metadata = DagMetadata {
        boundary_declarations: vec![
            BoundaryDeclaration {
                node: NodeId("format_request".into()),
                port: PortName("request_bytes".into()),
                external_type: external_types::http_request(),
            },
        ],
        export_node: Some(NodeId("parse_response".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

/// Build a mock HTTP understanding SubDAG.
///
/// Returns canned HTTP responses without making network calls.
pub fn build_http_mock(status: u16, body: &str) -> Dag<HttpOp> {
    let nodes = vec![
        Node {
            id: NodeId("format_request".into()),
            inputs: vec![
                port("method", "String"),
                port("url", "String"),
                port("headers", "MapStrStr"),
                port("body", "String"),
            ],
            outputs: vec![port("request_bytes", "Bytes")],
            body: NodeBody::Opaque(HttpOp::FormatRequest),
        },
        Node {
            id: NodeId("mock_response".into()),
            inputs: vec![port("request_bytes", "Bytes")],
            outputs: vec![
                port("status", "Int"),
                port("headers", "MapStrStr"),
                port("body", "String"),
            ],
            body: NodeBody::Opaque(HttpOp::MockResponse {
                status,
                body: body.into(),
            }),
        },
    ];

    let edges = vec![
        edge("format_request", "request_bytes", "mock_response", "request_bytes"),
    ];

    let metadata = DagMetadata {
        export_node: Some(NodeId("mock_response".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::tcp::build_tcp_mock;

    #[test]
    fn http_real_has_boundary_declaration() {
        let tcp = build_tcp_mock(vec![]);
        let dag = build_http_real(tcp);
        assert_eq!(dag.metadata.boundary_declarations.len(), 1);
        assert_eq!(
            dag.metadata.boundary_declarations[0].external_type.0,
            "External::HTTP::Request"
        );
    }

    #[test]
    fn http_mock_has_no_boundary() {
        let dag = build_http_mock(200, "OK");
        assert!(dag.metadata.boundary_declarations.is_empty());
    }
}
