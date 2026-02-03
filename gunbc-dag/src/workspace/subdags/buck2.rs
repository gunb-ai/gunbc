//! Buck2 SubDag builder.
//!
//! Wraps the buck2 tool as a SubDag node using WorkspaceOp.

use crate::workspace::WorkspaceOp;
use gunbc_buck2::Buck2Op;
use gunbc_ir::build::*;
use gunbc_ir::{DagBuilder, Node};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;

/// Build the buck2 SubDag node.
///
/// This wraps the buck2 generation workflow as a `Node<WorkspaceOp>`.
///
/// # I/O Interface
///
/// Inputs:
/// - `cargo_toml_path`: String - Path to Cargo.toml
/// - `output_path`: String - Output path for generated BUCK file
///
/// Outputs:
/// - `response`: TransportResponse - File write response
/// - `written_path`: String - Actual path written to
/// - `content`: String - Generated BUCK file content
pub fn build_buck2_subdag() -> Node<WorkspaceOp> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    // Node: PrepareParseCargoToml (PURE)
    let prepare_parse = builder
        .add_root_node(Node::opaque(
            "prepare_parse_cargo_toml",
            vec![port("cargo_toml_path", "String")],
            vec![
                port("request", "TransportRequest"),
                port("cargo_toml_path", "String"),
            ],
            WorkspaceOp::Buck2(Buck2Op::PrepareParseCargoToml),
        ))
        .expect("prepare_parse_cargo_toml node");

    // Node: Execute parse (BOUNDARY)
    let execute_parse = builder
        .add_node_after(
            Node::opaque(
                "execute_parse_cargo_toml",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_parse,
        )
        .expect("execute_parse_cargo_toml node");

    // Node: ParseCargoTomlResult (PURE)
    let parse_cargo_toml = builder
        .add_node_after(
            Node::opaque(
                "parse_cargo_toml_result",
                vec![
                    port("response", "TransportResponse"),
                    port("cargo_toml_path", "String"),
                ],
                vec![port("cargo_toml", "Json")],
                WorkspaceOp::Buck2(Buck2Op::ParseCargoTomlResult),
            ),
            &execute_parse,
        )
        .expect("parse_cargo_toml_result node");

    // Node: ExtractDeps
    let extract_deps = builder
        .add_node_after(
            Node::opaque(
                "extract_deps",
                vec![port("cargo_toml", "Json")],
                vec![port("members", "List"), port("deps", "Map")],
                WorkspaceOp::Buck2(Buck2Op::ExtractDeps),
            ),
            &parse_cargo_toml,
        )
        .expect("extract_deps node");

    // Node: GenerateBuckTargets
    let generate_targets = builder
        .add_node_after(
            Node::opaque(
                "generate_targets",
                vec![port("members", "List"), port("deps", "Map")],
                vec![port("buck_content", "String")],
                WorkspaceOp::Buck2(Buck2Op::GenerateBuckTargets),
            ),
            &extract_deps,
        )
        .expect("generate_targets node");

    // Node: PrepareFileWrite
    let prepare_file_write = builder
        .add_node_after(
            Node::opaque(
                "prepare_file_write",
                vec![port("content", "String"), port("output_path", "String")],
                vec![port("request", "TransportRequest")],
                WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                    PrepareFileWriteOp,
                )),
            ),
            &generate_targets,
        )
        .expect("prepare_file_write node");

    // Node: ExecuteTransport
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
        .add_edge(prepare_parse.out("request"), execute_parse.in_port("request"))
        .expect("parse request edge");
    builder
        .add_edge(
            execute_parse.out("response"),
            parse_cargo_toml.in_port("response"),
        )
        .expect("parse response edge");
    builder
        .add_edge(
            prepare_parse.out("cargo_toml_path"),
            parse_cargo_toml.in_port("cargo_toml_path"),
        )
        .expect("cargo_toml_path edge");
    builder
        .add_edge(
            parse_cargo_toml.out("cargo_toml"),
            extract_deps.in_port("cargo_toml"),
        )
        .expect("cargo_toml edge");
    builder
        .add_edge(
            extract_deps.out("members"),
            generate_targets.in_port("members"),
        )
        .expect("members edge");
    builder
        .add_edge(extract_deps.out("deps"), generate_targets.in_port("deps"))
        .expect("deps edge");
    builder
        .add_edge(
            generate_targets.out("buck_content"),
            prepare_file_write.in_port("content"),
        )
        .expect("buck_content edge");
    builder
        .add_edge(
            prepare_file_write.out("request"),
            execute_transport.in_port("request"),
        )
        .expect("transport request edge");

    let inner_dag = builder.build();

    Node::subdag(
        "buck2",
        inner_dag,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_buck2_subdag_is_subdag() {
        let node = build_buck2_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "buck2");
    }

    #[test]
    fn test_buck2_subdag_structure() {
        let node = build_buck2_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // 7 nodes in the pipeline
                assert_eq!(dag.nodes.len(), 7);
                // 8 edges
                assert_eq!(dag.edges.len(), 8);
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
