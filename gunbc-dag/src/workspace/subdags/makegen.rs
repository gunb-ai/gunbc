//! Makegen SubDag builder.
//!
//! Wraps the makegen tool as a SubDag node using `DynOp`.

use crate::makegen::MakegenOp;
use crate::workspace::WorkspaceOp;
use gunbc_exec::DynOp;
use gunbc_ir::build::*;
use gunbc_ir::{DagBuilder, Node};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;

/// Build the makegen SubDag node.
pub fn build_makegen_subdag() -> Node<WorkspaceOp> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    let load_registry = builder
        .add_root_node(Node::opaque(
            "load_registry",
            vec![],
            vec![
                scalar("tool_count", "Int"),
                non_empty_list("tool_names", "NonEmptyStringList"),
                scalar("registry", "Json"),
            ],
            DynOp::new(MakegenOp::LoadRegistry),
        ))
        .expect("load_registry node");

    let render_makefile = builder
        .add_node_after(
            Node::opaque(
                "render_makefile",
                vec![scalar("registry", "Json")],
                vec![scalar("makefile_content", "String")],
                DynOp::new(MakegenOp::RenderMakefile),
            ),
            &load_registry,
        )
        .expect("render_makefile node");

    let prepare_file_write = builder
        .add_node_after(
            Node::opaque(
                "prepare_file_write",
                vec![port("content", "String"), port("path", "String")],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                DynOp::new(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                    PrepareFileWriteOp,
                )),
            ),
            &render_makefile,
        )
        .expect("prepare_file_write node");

    let execute_transport = builder
        .add_node_after(
            Node::opaque(
                "execute_transport",
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                vec![
                    port("response", "TransportResponse"),
                    port("written_path", "String"),
                    port("content", "String"),
                ],
                DynOp::new(TransportOps::Execute),
            ),
            &prepare_file_write,
        )
        .expect("execute_transport node");

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
    builder
        .add_edge(
            prepare_file_write.out("skip"),
            execute_transport.in_port("skip"),
        )
        .expect("skip edge");

    let inner_dag = builder.build();
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

        assert!(node.inputs.iter().any(|p| p.name.0 == "path"));
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
