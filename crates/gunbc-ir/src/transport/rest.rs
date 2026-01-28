//! REST transport layer understanding.
//!
//! This layer wraps HTTP to provide REST API semantics.

use crate::{
    edge, port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId, PortName,
};
use crate::transport::external_types;

/// REST layer operations.
#[derive(Debug, Clone)]
pub enum RestOp {
    /// Format a REST request (add JSON content-type, etc.).
    FormatRequest,
    /// Parse a REST response (JSON parsing, error handling).
    ParseResponse,
    /// Mock: return a canned REST response.
    MockResponse { body: String },
}

/// Build a real REST understanding SubDAG.
///
/// This SubDAG wraps an HTTP SubDAG to perform REST over HTTP.
pub fn build_rest_real<T: Clone>(_http: Dag<T>) -> Dag<RestOp> {
    let nodes = vec![
        Node {
            id: NodeId("format_rest_request".into()),
            inputs: vec![
                port("method", "String"),
                port("url", "String"),
                port("body", "Json"),
            ],
            outputs: vec![
                port("method", "String"),
                port("url", "String"),
                port("headers", "MapStrStr"),
                port("body", "String"),
            ],
            body: NodeBody::Opaque(RestOp::FormatRequest),
        },
        // In a real implementation, http SubDAG would be embedded here
        Node {
            id: NodeId("parse_rest_response".into()),
            inputs: vec![
                port("status", "Int"),
                port("headers", "MapStrStr"),
                port("body", "String"),
            ],
            outputs: vec![port("response", "Json")],
            body: NodeBody::Opaque(RestOp::ParseResponse),
        },
    ];

    let edges = vec![
        // format_rest_request -> http -> parse_rest_response
        // (http layer omitted for simplicity)
    ];

    let metadata = DagMetadata {
        boundary_declarations: vec![
            BoundaryDeclaration {
                node: NodeId("format_rest_request".into()),
                port: PortName("url".into()),
                external_type: external_types::rest_request(),
            },
        ],
        export_node: Some(NodeId("parse_rest_response".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

/// Build a mock REST understanding SubDAG.
///
/// Returns canned REST responses without making HTTP calls.
pub fn build_rest_mock(body: &str) -> Dag<RestOp> {
    let nodes = vec![
        Node {
            id: NodeId("format_rest_request".into()),
            inputs: vec![
                port("method", "String"),
                port("url", "String"),
                port("body", "Json"),
            ],
            outputs: vec![port("request", "Json")],
            body: NodeBody::Opaque(RestOp::FormatRequest),
        },
        Node {
            id: NodeId("mock_response".into()),
            inputs: vec![port("request", "Json")],
            outputs: vec![port("response", "Json")],
            body: NodeBody::Opaque(RestOp::MockResponse { body: body.into() }),
        },
    ];

    let edges = vec![
        edge("format_rest_request", "request", "mock_response", "request"),
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
    use crate::transport::http::build_http_mock;

    #[test]
    fn rest_real_has_boundary_declaration() {
        let http = build_http_mock(200, "{}");
        let dag = build_rest_real(http);
        assert_eq!(dag.metadata.boundary_declarations.len(), 1);
        assert_eq!(
            dag.metadata.boundary_declarations[0].external_type.0,
            "External::REST::Request"
        );
    }

    #[test]
    fn rest_mock_has_no_boundary() {
        let dag = build_rest_mock("{}");
        assert!(dag.metadata.boundary_declarations.is_empty());
    }
}
