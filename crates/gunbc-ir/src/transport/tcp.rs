//! TCP transport layer understanding.
//!
//! This is the bottom layer of the transport stack - actual network connections.

use crate::{
    edge, port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId, PortName,
};
use crate::transport::external_types;

/// TCP layer operations.
#[derive(Debug, Clone)]
pub enum TcpOp {
    /// Establish a TCP connection.
    Connect { host: String, tcp_port: u16 },
    /// Send bytes over a connection.
    Send,
    /// Receive bytes from a connection.
    Receive,
    /// Close the connection.
    Close,
    /// Mock: simulate a connection.
    MockConnect,
    /// Mock: return canned response bytes.
    MockReceive { response: Vec<u8> },
}

/// Build a real TCP understanding SubDAG.
///
/// This SubDAG performs actual network operations.
pub fn build_tcp_real(host: &str, tcp_port: u16) -> Dag<TcpOp> {
    let nodes = vec![
        Node {
            id: NodeId("connect".into()),
            inputs: vec![],
            outputs: vec![port("connection", "TcpConnection")],
            body: NodeBody::Opaque(TcpOp::Connect {
                host: host.into(),
                tcp_port,
            }),
        },
        Node {
            id: NodeId("send".into()),
            inputs: vec![
                port("connection", "TcpConnection"),
                port("data", "Bytes"),
            ],
            outputs: vec![port("connection", "TcpConnection")],
            body: NodeBody::Opaque(TcpOp::Send),
        },
        Node {
            id: NodeId("receive".into()),
            inputs: vec![port("connection", "TcpConnection")],
            outputs: vec![port("data", "Bytes")],
            body: NodeBody::Opaque(TcpOp::Receive),
        },
        Node {
            id: NodeId("close".into()),
            inputs: vec![port("connection", "TcpConnection")],
            outputs: vec![],
            body: NodeBody::Opaque(TcpOp::Close),
        },
    ];

    let edges = vec![
        edge("connect", "connection", "send", "connection"),
        edge("send", "connection", "receive", "connection"),
    ];

    let metadata = DagMetadata {
        boundary_declarations: vec![
            BoundaryDeclaration {
                node: NodeId("connect".into()),
                port: PortName("connection".into()),
                external_type: external_types::tcp_connection(),
            },
        ],
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

/// Build a mock TCP understanding SubDAG.
///
/// This SubDAG simulates network operations without actual connections.
pub fn build_tcp_mock(response: Vec<u8>) -> Dag<TcpOp> {
    let nodes = vec![
        Node {
            id: NodeId("connect".into()),
            inputs: vec![],
            outputs: vec![port("connection", "TcpConnection")],
            body: NodeBody::Opaque(TcpOp::MockConnect),
        },
        Node {
            id: NodeId("send".into()),
            inputs: vec![
                port("connection", "TcpConnection"),
                port("data", "Bytes"),
            ],
            outputs: vec![port("connection", "TcpConnection")],
            body: NodeBody::Opaque(TcpOp::MockConnect), // No-op for mock
        },
        Node {
            id: NodeId("receive".into()),
            inputs: vec![port("connection", "TcpConnection")],
            outputs: vec![port("data", "Bytes")],
            body: NodeBody::Opaque(TcpOp::MockReceive { response }),
        },
    ];

    let edges = vec![
        edge("connect", "connection", "send", "connection"),
        edge("send", "connection", "receive", "connection"),
    ];

    Dag {
        nodes,
        edges,
        metadata: DagMetadata::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcp_real_has_boundary_declaration() {
        let dag = build_tcp_real("example.com", 443);
        assert_eq!(dag.metadata.boundary_declarations.len(), 1);
        assert_eq!(
            dag.metadata.boundary_declarations[0].external_type.0,
            "External::TCP::Connection"
        );
    }

    #[test]
    fn tcp_mock_has_no_boundary() {
        let dag = build_tcp_mock(vec![0, 1, 2]);
        assert!(dag.metadata.boundary_declarations.is_empty());
    }
}
