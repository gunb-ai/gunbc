//! File I/O transport layer.
//!
//! Provides SubDAGs for file operations:
//! - **File Upsert SubDAG** - Check existing file, write/print if needed, resolve status
//!
//! Used by tools like makegen, gitignoregen.

use crate::{
    edge, eq_guarded_port, port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId,
    PortName, Value,
};
use crate::transport::external_types;

/// File I/O layer operations.
#[derive(Debug, Clone)]
pub enum FileOp {
    // Upsert operations (for makegen, gitignoregen)
    /// Check existing file state - reads file, extracts hash
    CheckExisting,
    /// Resolve whether write is needed based on check + generated content
    ResolveUpsert,
    /// Write content to file
    WriteFile,
    /// Print content to stdout (for dry-run)
    PrintStdout,
}

/// Build a real file upsert SubDAG.
///
/// Structure: check → sink → resolve (write)
///
/// Inputs (to be wired by parent DAG):
/// - check.file_path: String - path to the file
/// - check.force: Bool - force regeneration
/// - check.input_hash: String - hash of input content
/// - sink.content: String - generated content to potentially write
///
/// Outputs (from resolve node):
/// - status: String - Created/Updated/Unchanged/DryRun
pub fn build_file_upsert_real<T: Clone, F>(wrap: F) -> Dag<T>
where
    F: Fn(FileOp) -> T + Copy,
{
    let nodes = vec![
        Node {
            id: NodeId("file_check".into()),
            inputs: vec![
                port("file_path", "String"),
                port("force", "Bool"),
                port("input_hash", "String"),
            ],
            outputs: vec![
                port("file_path", "String"),
                port("input_hash", "String"),
                port("needs_write", "Bool"),
                port("file_existed", "Bool"),
            ],
            body: NodeBody::Opaque(wrap(FileOp::CheckExisting)),
        },
        Node {
            id: NodeId("file_sink".into()),
            inputs: vec![
                port("content", "String"),
                eq_guarded_port("needs_write", "Bool", Value::Bool(true)),
                port("file_path", "String"),
                port("file_existed", "Bool"),
            ],
            outputs: vec![port("write_status", "String")],
            body: NodeBody::Opaque(wrap(FileOp::WriteFile)),
        },
        Node {
            id: NodeId("file_resolve".into()),
            inputs: vec![port("needs_write", "Bool"), port("write_status", "String")],
            outputs: vec![port("status", "String")],
            body: NodeBody::Opaque(wrap(FileOp::ResolveUpsert)),
        },
    ];

    let edges = vec![
        // Check to sink
        edge("file_check", "needs_write", "file_sink", "needs_write"),
        edge("file_check", "file_path", "file_sink", "file_path"),
        edge("file_check", "file_existed", "file_sink", "file_existed"),
        // Check + sink to resolve
        edge("file_check", "needs_write", "file_resolve", "needs_write"),
        edge("file_sink", "write_status", "file_resolve", "write_status"),
    ];

    let metadata = DagMetadata {
        boundary_declarations: vec![
            BoundaryDeclaration {
                node: NodeId("file_sink".into()),
                port: PortName("write_status".into()),
                external_type: external_types::fs_write(),
            },
        ],
        export_node: Some(NodeId("file_resolve".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

/// Build a mock file upsert SubDAG (dry-run mode).
///
/// Same structure as real, but sink prints to stdout instead of writing.
pub fn build_file_upsert_mock<T: Clone, F>(wrap: F) -> Dag<T>
where
    F: Fn(FileOp) -> T + Copy,
{
    let nodes = vec![
        Node {
            id: NodeId("file_check".into()),
            inputs: vec![
                port("file_path", "String"),
                port("force", "Bool"),
                port("input_hash", "String"),
            ],
            outputs: vec![
                port("file_path", "String"),
                port("input_hash", "String"),
                port("needs_write", "Bool"),
                port("file_existed", "Bool"),
            ],
            body: NodeBody::Opaque(wrap(FileOp::CheckExisting)),
        },
        Node {
            id: NodeId("file_sink".into()),
            inputs: vec![
                port("content", "String"),
                eq_guarded_port("needs_write", "Bool", Value::Bool(true)),
                port("file_path", "String"),
                port("file_existed", "Bool"),
            ],
            outputs: vec![port("write_status", "String")],
            body: NodeBody::Opaque(wrap(FileOp::PrintStdout)),
        },
        Node {
            id: NodeId("file_resolve".into()),
            inputs: vec![port("needs_write", "Bool"), port("write_status", "String")],
            outputs: vec![port("status", "String")],
            body: NodeBody::Opaque(wrap(FileOp::ResolveUpsert)),
        },
    ];

    let edges = vec![
        // Check to sink
        edge("file_check", "needs_write", "file_sink", "needs_write"),
        edge("file_check", "file_path", "file_sink", "file_path"),
        edge("file_check", "file_existed", "file_sink", "file_existed"),
        // Check + sink to resolve
        edge("file_check", "needs_write", "file_resolve", "needs_write"),
        edge("file_sink", "write_status", "file_resolve", "write_status"),
    ];

    let metadata = DagMetadata {
        export_node: Some(NodeId("file_resolve".into())),
        ..Default::default()
    };

    Dag { nodes, edges, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum DummyOp {
        File(FileOp),
    }

    fn wrap(op: FileOp) -> DummyOp {
        DummyOp::File(op)
    }

    #[test]
    fn file_upsert_real_has_boundary_declaration() {
        let dag = build_file_upsert_real(wrap);
        assert_eq!(dag.metadata.boundary_declarations.len(), 1);
        assert_eq!(
            dag.metadata.boundary_declarations[0].external_type.0,
            "External::FS::Write"
        );
    }

    #[test]
    fn file_upsert_mock_has_no_boundary() {
        let dag = build_file_upsert_mock(wrap);
        assert!(dag.metadata.boundary_declarations.is_empty());
    }

    #[test]
    fn file_upsert_has_correct_node_count() {
        let dag = build_file_upsert_real(wrap);
        assert_eq!(dag.nodes.len(), 3);
    }

    #[test]
    fn file_upsert_has_export_node() {
        let dag = build_file_upsert_real(wrap);
        assert_eq!(dag.metadata.export_node, Some(NodeId("file_resolve".into())));
    }
}
