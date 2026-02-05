//! Makegen SubDag builder.
//!
//! Wraps the makegen tool as a SubDag node using WorkspaceOp.

use crate::makegen::MakegenOp;
use crate::workspace::WorkspaceOp;
use gunbc_ir::build::*;
use gunbc_ir::{DagBuilder, Node};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;

/// Build the makegen SubDag node.
///
/// This wraps the makegen workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # I/O Interface
///
/// Inputs:
/// - `output_path`: String (optional) - Path for generated Makefile
///
/// Outputs:
/// - `response`: TransportResponse - File write response
/// - `written_path`: String - Actual path written to
/// - `content`: String - Generated content
/// - `tool_count`: Int - Number of tools in registry
/// - `tool_names`: List - Names of registered tools
pub fn build_makegen_subdag() -> Node<WorkspaceOp> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    // Node: LoadRegistry (makegen-specific) - generation 0
    let load_registry = builder
        .add_root_node(Node::opaque(
            "load_registry",
            vec![],
            vec![
                scalar("tool_count", "Int"),
                non_empty_list("tool_names", "String"),
                scalar("registry", "Json"),
            ],
            WorkspaceOp::Makegen(MakegenOp::LoadRegistry),
        ))
        .expect("load_registry node");

    // Node: RenderMakefile (makegen-specific) - generation 1
    let render_makefile = builder
        .add_node_after(
            Node::opaque(
                "render_makefile",
                vec![scalar("registry", "Json")],
                vec![scalar("makefile_content", "String")],
                WorkspaceOp::Makegen(MakegenOp::RenderMakefile),
            ),
            &load_registry,
        )
        .expect("render_makefile node");

    // Node: PrepareFileWrite (primitive - PURE) - generation 2
    let prepare_file_write = builder
        .add_node_after(
            Node::opaque(
                "prepare_file_write",
                vec![port("content", "String"), optional("output_path", "String")],
                vec![port("request", "TransportRequest")],
                WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                    PrepareFileWriteOp,
                )),
            ),
            &render_makefile,
        )
        .expect("prepare_file_write node");

    // Node: ExecuteTransport (transport - BOUNDARY) - generation 3
    let execute_transport = builder
        .add_node_after(
            Node::opaque(
                "execute_transport",
                vec![port("request", "TransportRequest")],
                vec![
                    port("response", "TransportResponse"),
                    port("written_path", "String"),
                    port("content", "String"),
                ],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_file_write,
        )
        .expect("execute_transport node");

    // Wire up the pipeline
    builder
        .add_edge(
            load_registry.out("registry"),
            render_makefile.in_port("registry"),
        )
        .expect("registry edge");
    builder
        .add_edge(
            render_makefile.out("makefile_content"),
            prepare_file_write.in_port("content"),
        )
        .expect("content edge");
    builder
        .add_edge(
            prepare_file_write.out("request"),
            execute_transport.in_port("request"),
        )
        .expect("request edge");

    let inner_dag = builder.build();

    // Wrap as SubDag with explicit I/O interface
    Node::subdag("makegen", inner_dag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_makegen_subdag_is_subdag() {
        let node = build_makegen_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "makegen");
    }

    #[test]
    fn test_makegen_subdag_interface() {
        let node = build_makegen_subdag();

        // Check inputs
        assert!(node.inputs.iter().any(|p| p.name.0 == "output_path"));

        // Check outputs
        assert!(node.outputs.iter().any(|p| p.name.0 == "response"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "written_path"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "content"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "tool_count"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "tool_names"));
    }

    #[test]
    fn test_makegen_subdag_structure() {
        let node = build_makegen_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // 4 nodes: load_registry, render_makefile, prepare_file_write, execute_transport
                assert_eq!(dag.nodes.len(), 4);
                // 3 edges
                assert_eq!(dag.edges.len(), 3);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"load_registry"));
                assert!(node_ids.contains(&"render_makefile"));
                assert!(node_ids.contains(&"prepare_file_write"));
                assert!(node_ids.contains(&"execute_transport"));
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
