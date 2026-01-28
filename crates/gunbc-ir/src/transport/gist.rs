//! GitHub Gist transport layer understanding.
//!
//! This layer wraps REST to provide GitHub Gist semantics.

use crate::{
    edge, port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId, PortName,
};
use crate::transport::external_types;

/// GitHub Gist layer operations.
#[derive(Debug, Clone)]
pub enum GistOp {
    /// Format a Create Gist request from the contract shape.
    FormatCreateRequest,
    /// Perform the real GitHub Gist call (implementation in higher layer).
    CallReal,
    /// Mock: return a canned Gist response derived from request.
    CallMock,
    /// Parse a Gist response into contract shape.
    ParseCreateResponse,
    /// Extract html_url as gist_url convenience output.
    ExtractGistUrl,
}

/// Build a real GitHub Gist understanding SubDAG.
///
/// This SubDAG wraps REST to perform Create Gist.
pub fn build_gist_real<T: Clone, F>(wrap: F) -> Dag<T>
where
    F: Fn(GistOp) -> T + Copy,
{
    let nodes = vec![
        Node {
            id: NodeId("format_gist_create".into()),
            inputs: vec![port("request", "GitHub::Gist::CreateRequest")],
            outputs: vec![port("request_json", "Json")],
            body: NodeBody::Opaque(wrap(GistOp::FormatCreateRequest)),
        },
        Node {
            id: NodeId("call_gist_real".into()),
            inputs: vec![
                port("request_json", "Json"),
                port("token", "Secret"),
            ],
            outputs: vec![port("response_json", "Json")],
            body: NodeBody::Opaque(wrap(GistOp::CallReal)),
        },
        Node {
            id: NodeId("parse_gist_response".into()),
            inputs: vec![port("response_json", "Json")],
            outputs: vec![port("response", "GitHub::Gist::CreateResponse")],
            body: NodeBody::Opaque(wrap(GistOp::ParseCreateResponse)),
        },
        Node {
            id: NodeId("extract_gist_url".into()),
            inputs: vec![port("response", "GitHub::Gist::CreateResponse")],
            outputs: vec![
                port("response", "GitHub::Gist::CreateResponse"),
                port("gist_url", "String"),
            ],
            body: NodeBody::Opaque(wrap(GistOp::ExtractGistUrl)),
        },
    ];

    let edges = vec![
        edge("format_gist_create", "request_json", "call_gist_real", "request_json"),
        edge("call_gist_real", "response_json", "parse_gist_response", "response_json"),
        edge("parse_gist_response", "response", "extract_gist_url", "response"),
    ];

    let metadata = DagMetadata {
        boundary_declarations: vec![
            BoundaryDeclaration {
                node: NodeId("extract_gist_url".into()),
                port: PortName("gist_url".into()),
                external_type: external_types::github_gist(),
            },
        ],
        export_node: Some(NodeId("extract_gist_url".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

/// Build a mock GitHub Gist understanding SubDAG.
///
/// Returns canned Gist responses without making external calls.
pub fn build_gist_mock<T: Clone, F>(wrap: F) -> Dag<T>
where
    F: Fn(GistOp) -> T + Copy,
{
    let nodes = vec![
        Node {
            id: NodeId("format_gist_create".into()),
            inputs: vec![port("request", "GitHub::Gist::CreateRequest")],
            outputs: vec![port("request_json", "Json")],
            body: NodeBody::Opaque(wrap(GistOp::FormatCreateRequest)),
        },
        Node {
            id: NodeId("call_gist_mock".into()),
            inputs: vec![
                port("request_json", "Json"),
                port("token", "Secret"),
            ],
            outputs: vec![port("response_json", "Json")],
            body: NodeBody::Opaque(wrap(GistOp::CallMock)),
        },
        Node {
            id: NodeId("parse_gist_response".into()),
            inputs: vec![port("response_json", "Json")],
            outputs: vec![port("response", "GitHub::Gist::CreateResponse")],
            body: NodeBody::Opaque(wrap(GistOp::ParseCreateResponse)),
        },
        Node {
            id: NodeId("extract_gist_url".into()),
            inputs: vec![port("response", "GitHub::Gist::CreateResponse")],
            outputs: vec![
                port("response", "GitHub::Gist::CreateResponse"),
                port("gist_url", "String"),
            ],
            body: NodeBody::Opaque(wrap(GistOp::ExtractGistUrl)),
        },
    ];

    let edges = vec![
        edge("format_gist_create", "request_json", "call_gist_mock", "request_json"),
        edge("call_gist_mock", "response_json", "parse_gist_response", "response_json"),
        edge("parse_gist_response", "response", "extract_gist_url", "response"),
    ];

    let metadata = DagMetadata {
        export_node: Some(NodeId("extract_gist_url".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum DummyOp {
        Gist(GistOp),
    }

    fn wrap(op: GistOp) -> DummyOp {
        DummyOp::Gist(op)
    }

    #[test]
    fn gist_real_has_boundary_declaration() {
        let dag = build_gist_real(wrap);
        assert_eq!(dag.metadata.boundary_declarations.len(), 1);
        assert_eq!(
            dag.metadata.boundary_declarations[0].external_type.0,
            "External::GitHub::Gist"
        );
    }

    #[test]
    fn gist_mock_has_no_boundary() {
        let dag = build_gist_mock(wrap);
        assert!(dag.metadata.boundary_declarations.is_empty());
    }
}
