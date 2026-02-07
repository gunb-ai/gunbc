//! Content upsert helper for the common generate → read → compare → write pattern.
//!
//! Many DAG graphs use a six-node pattern for content upsert operations:
//!
//! 1. **Generate** — pure node that produces content (caller-created)
//! 2. **PrepareRead** — pure node that builds a read `TransportRequest`
//! 3. **ExecuteRead** — transport boundary that reads existing content
//! 4. **Compare** — pure node that compares expected vs actual content
//! 5. **PrepareWrite** — pure node that builds a write `TransportRequest`
//! 6. **ExecuteWrite** — transport boundary that writes content (skippable)
//!
//! This module provides a helper that stamps out the 5 downstream nodes
//! (everything after generate) plus all 8 internal edges in one call,
//! eliminating ~100 lines of boilerplate per chain.
//!
//! The generate node is created by the caller and passed in, since its
//! inputs/outputs vary per workflow.

use crate::build::{optional, port};
use crate::builder::{BuilderError, DagBuilder, NodeRef};
use crate::node::Node;

/// References to the five nodes created by a content upsert chain
/// (excludes the generate node, which the caller creates).
pub struct ContentUpsertChain<T> {
    pub prepare_read: NodeRef<T>,
    pub execute_read: NodeRef<T>,
    pub compare: NodeRef<T>,
    pub prepare_write: NodeRef<T>,
    pub execute_write: NodeRef<T>,
}

/// Add a content upsert chain after a generate node.
///
/// Creates 5 nodes and wires 8 internal edges:
///
/// ```text
/// generate ─┬─→ prepare_read_{name} → execute_read_{name} → compare_{name}_content → execute_{name}_transport
///           └─→ prepare_write_{name} ─────────────────────────────────────────────→ (request)
/// ```
///
/// The generate node must have an output port named `content_port`.
///
/// **Node naming convention:**
/// - `prepare_read_{name}`, `execute_read_{name}`, `compare_{name}_content`
/// - `prepare_write_{name}`, `execute_{name}_transport`
///
/// **Output ports on execute_write:**
/// - `{name}_response`, `{name}_written_path`, `{name}_content` (all optional)
/// - `skip`, `skip_reason` (standard skip propagation)
///
/// **8 internal edges:**
/// 1. `generate.content_port → compare.expected_content`
/// 2. `generate.content_port → prepare_write.content`
/// 3. `prepare_read.request → execute_read.request`
/// 4. `prepare_read.skip → execute_read.skip`
/// 5. `execute_read.response → compare.response`
/// 6. `compare.skip → execute_write.skip`
/// 7. `compare.skip_reason → execute_write.skip_reason`
/// 8. `prepare_write.request → execute_write.request`
#[allow(clippy::too_many_arguments)]
pub fn add_content_upsert_chain<T: Clone>(
    builder: &mut DagBuilder<T>,
    name: &str,
    generate: &NodeRef<T>,
    content_port: &str,
    prepare_read_op: T,
    prepare_write_op: T,
    compare_op: T,
    transport_op: T,
) -> Result<ContentUpsertChain<T>, BuilderError> {
    let prep_read_id = format!("prepare_read_{name}");
    let exec_read_id = format!("execute_read_{name}");
    let compare_id = format!("compare_{name}_content");
    let prep_write_id = format!("prepare_write_{name}");
    let exec_write_id = format!("execute_{name}_transport");
    let response_port = format!("{name}_response");
    let path_port = format!("{name}_written_path");
    let content_out_port = format!("{name}_content");

    // PrepareRead: path entrypoint → request + skip
    let prepare_read = builder.add_node_after(
        Node::opaque(
            prep_read_id.as_str(),
            vec![port("path", "String")],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            prepare_read_op,
        ),
        generate,
    )?;

    // ExecuteRead: transport boundary
    let execute_read = builder.add_node_after(
        Node::opaque(
            exec_read_id.as_str(),
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            transport_op.clone(),
        ),
        &prepare_read,
    )?;

    // Compare: expected vs actual content
    let compare = builder.add_node_after(
        Node::opaque(
            compare_id.as_str(),
            vec![
                port("response", "TransportResponse"),
                port("expected_content", "String"),
                optional("check_mode", "Bool"),
            ],
            vec![
                port("fresh", "Bool"),
                port("skip", "Bool"),
                port("skip_reason", "String"),
            ],
            compare_op,
        ),
        &execute_read,
    )?;

    // PrepareWrite: content + path → request
    let prepare_write = builder.add_node_after(
        Node::opaque(
            prep_write_id.as_str(),
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            prepare_write_op,
        ),
        generate,
    )?;

    // ExecuteWrite: skippable transport boundary
    let execute_write = builder.add_node_after(
        Node::opaque(
            exec_write_id.as_str(),
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional(response_port.as_str(), "TransportResponse"),
                optional(path_port.as_str(), "String"),
                optional(content_out_port.as_str(), "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            transport_op,
        ),
        &compare,
    )?;

    // Wire 8 internal edges
    builder.add_edge(generate.out(content_port), compare.in_port("expected_content"))?;
    builder.add_edge(generate.out(content_port), prepare_write.in_port("content"))?;
    builder.add_edge(prepare_read.out("request"), execute_read.in_port("request"))?;
    builder.add_edge(prepare_read.out("skip"), execute_read.in_port("skip"))?;
    builder.add_edge(execute_read.out("response"), compare.in_port("response"))?;
    builder.add_edge(compare.out("skip"), execute_write.in_port("skip"))?;
    builder.add_edge(compare.out("skip_reason"), execute_write.in_port("skip_reason"))?;
    builder.add_edge(prepare_write.out("request"), execute_write.in_port("request"))?;

    Ok(ContentUpsertChain {
        prepare_read,
        execute_read,
        compare,
        prepare_write,
        execute_write,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum TestOp {
        Generate,
        PrepareRead,
        PrepareWrite,
        Compare,
        Transport,
    }

    #[test]
    fn test_content_upsert_chain() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        // Generate node (root, created by caller)
        let generate = builder
            .add_root_node(Node::opaque(
                "generate_test",
                vec![],
                vec![port("content", "String")],
                TestOp::Generate,
            ))
            .unwrap();

        let chain = add_content_upsert_chain(
            &mut builder,
            "test",
            &generate,
            "content",
            TestOp::PrepareRead,
            TestOp::PrepareWrite,
            TestOp::Compare,
            TestOp::Transport,
        )
        .unwrap();

        let dag = builder.build();

        // 6 nodes: 1 generate + 5 chain
        assert_eq!(dag.nodes.len(), 6);
        // 8 internal edges
        assert_eq!(dag.edges.len(), 8);

        // Verify node names
        assert!(dag.get_node(&"generate_test".into()).is_some());
        assert!(dag.get_node(&"prepare_read_test".into()).is_some());
        assert!(dag.get_node(&"execute_read_test".into()).is_some());
        assert!(dag.get_node(&"compare_test_content".into()).is_some());
        assert!(dag.get_node(&"prepare_write_test".into()).is_some());
        assert!(dag.get_node(&"execute_test_transport".into()).is_some());

        // Verify chain refs are usable
        let _ = chain.prepare_read;
        let _ = chain.execute_read;
        let _ = chain.compare;
        let _ = chain.prepare_write;
        let _ = chain.execute_write;
    }

    #[test]
    fn test_custom_content_port() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        let generate = builder
            .add_root_node(Node::opaque(
                "generate_makefile",
                vec![],
                vec![port("makefile_content", "String")],
                TestOp::Generate,
            ))
            .unwrap();

        let _chain = add_content_upsert_chain(
            &mut builder,
            "makefile",
            &generate,
            "makefile_content",
            TestOp::PrepareRead,
            TestOp::PrepareWrite,
            TestOp::Compare,
            TestOp::Transport,
        )
        .unwrap();

        let dag = builder.build();

        // 6 nodes, 8 edges (same structure)
        assert_eq!(dag.nodes.len(), 6);
        assert_eq!(dag.edges.len(), 8);

        // Verify naming with "makefile" prefix
        assert!(dag.get_node(&"prepare_read_makefile".into()).is_some());
        assert!(dag.get_node(&"execute_read_makefile".into()).is_some());
        assert!(dag.get_node(&"compare_makefile_content".into()).is_some());
        assert!(dag.get_node(&"prepare_write_makefile".into()).is_some());
        assert!(dag.get_node(&"execute_makefile_transport".into()).is_some());
    }
}
