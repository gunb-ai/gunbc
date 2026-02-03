//! Deps SubDag builders.
//!
//! Wraps the deps tool as SubDag nodes using WorkspaceOp.
//! Provides both install and generate workflows.

use crate::workspace::WorkspaceOp;
use gunbc_deps::DepsOp;
use gunbc_ir::build::*;
use gunbc_ir::{DagBuilder, Node};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;

/// Build the deps install SubDag node.
///
/// This wraps the deps install workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # I/O Interface
///
/// Inputs:
/// - `manifest_path`: String (optional) - Path to deps.toml
///
/// Outputs:
/// - `dep_count`: Int - Number of dependencies
/// - `dep_names`: List - Dependency names
/// - `already_installed`: List - Dependencies already present
/// - `needs_install`: List - Dependencies that need installation
/// - `platform`: String - Current platform
/// - `executed`: Bool - Whether installs were executed
/// - `success`: Bool - Whether installation succeeded
/// - `script`: String - Generated install script
pub fn build_deps_install_subdag() -> Node<WorkspaceOp> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    // Node: PrepareLoadManifest (PURE)
    let prepare_load = builder
        .add_root_node(Node::opaque(
            "prepare_load_manifest",
            vec![optional("manifest_path", "String")],
            vec![
                port("request", "TransportRequest"),
                port("manifest_path", "String"),
            ],
            WorkspaceOp::Deps(DepsOp::PrepareLoadManifest),
        ))
        .expect("prepare_load_manifest node");

    // Node: Execute manifest load (BOUNDARY)
    let execute_load = builder
        .add_node_after(
            Node::opaque(
                "execute_load_manifest",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_load,
        )
        .expect("execute_load_manifest node");

    // Node: ParseManifest (PURE)
    let parse_manifest = builder
        .add_node_after(
            Node::opaque(
                "parse_manifest",
                vec![
                    port("response", "TransportResponse"),
                    port("manifest_path", "String"),
                ],
                vec![
                    scalar("dep_count", "Int"),
                    list("dep_names", "List"),
                    scalar("manifest_path", "String"),
                    scalar("manifest_content", "String"),
                ],
                WorkspaceOp::Deps(DepsOp::ParseManifest),
            ),
            &execute_load,
        )
        .expect("parse_manifest node");

    // Node: GenerateScripts (PURE)
    let generate_scripts = builder
        .add_node_after(
            Node::opaque(
                "generate_scripts",
                vec![scalar("manifest_content", "String")],
                vec![
                    scalar("install_script", "String"),
                    list("already_installed", "List"),
                    list("needs_install", "List"),
                    scalar("platform", "String"),
                ],
                WorkspaceOp::Deps(DepsOp::GenerateScripts),
            ),
            &parse_manifest,
        )
        .expect("generate_scripts node");

    // Node: PrepareExecuteInstalls (PURE)
    let prepare_execute = builder
        .add_node_after(
            Node::opaque(
                "prepare_execute_installs",
                vec![scalar("install_script", "String")],
                vec![port("request", "TransportRequest"), port("script", "String")],
                WorkspaceOp::Deps(DepsOp::PrepareExecuteInstalls),
            ),
            &generate_scripts,
        )
        .expect("prepare_execute_installs node");

    // Node: Execute installs (BOUNDARY)
    let execute_installs = builder
        .add_node_after(
            Node::opaque(
                "execute_installs",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_execute,
        )
        .expect("execute_installs node");

    // Node: ParseExecuteResult (PURE)
    let _parse_result = builder
        .add_node_after(
            Node::opaque(
                "parse_execute_result",
                vec![
                    port("response", "TransportResponse"),
                    port("script", "String"),
                ],
                vec![
                    scalar("executed", "Bool"),
                    scalar("success", "Bool"),
                    scalar("script", "String"),
                    scalar("stdout", "String"),
                    scalar("stderr", "String"),
                ],
                WorkspaceOp::Deps(DepsOp::ParseExecuteResult),
            ),
            &execute_installs,
        )
        .expect("parse_execute_result node");

    // Wire up the pipeline
    builder
        .add_edge(
            prepare_load.out("request"),
            execute_load.in_port("request"),
        )
        .expect("request edge");
    builder
        .add_edge(
            execute_load.out("response"),
            parse_manifest.in_port("response"),
        )
        .expect("response edge");
    builder
        .add_edge(
            prepare_load.out("manifest_path"),
            parse_manifest.in_port("manifest_path"),
        )
        .expect("manifest_path edge");
    builder
        .add_edge(
            parse_manifest.out("manifest_content"),
            generate_scripts.in_port("manifest_content"),
        )
        .expect("manifest_content edge");
    builder
        .add_edge(
            generate_scripts.out("install_script"),
            prepare_execute.in_port("install_script"),
        )
        .expect("install_script edge");
    builder
        .add_edge(
            prepare_execute.out("request"),
            execute_installs.in_port("request"),
        )
        .expect("execute request edge");
    builder
        .add_edge(
            execute_installs.out("response"),
            _parse_result.in_port("response"),
        )
        .expect("execute response edge");
    builder
        .add_edge(
            prepare_execute.out("script"),
            _parse_result.in_port("script"),
        )
        .expect("script edge");

    let inner_dag = builder.build();

    // Wrap as SubDag with explicit I/O interface
    Node::subdag(
        "deps_install",
        inner_dag,
    )
}

/// Build the deps generate SubDag node.
///
/// This wraps the deps.toml generation workflow as a `Node<WorkspaceOp>`.
///
/// # I/O Interface
///
/// Inputs:
/// - `output_path`: String (optional) - Path for generated deps.toml
///
/// Outputs:
/// - `response`: TransportResponse - File write response
/// - `written_path`: String - Actual path written to
/// - `content`: String - Generated content
/// - `tool_count`: Int - Number of tools in registry
/// - `tool_names`: List - Names of registered tools
pub fn build_deps_generate_subdag() -> Node<WorkspaceOp> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    // Node: LoadToolRegistry
    let load_registry = builder
        .add_root_node(Node::opaque(
            "load_tool_registry",
            vec![],
            vec![
                scalar("tool_count", "Int"),
                non_empty_list("tool_names", "List"),
            ],
            WorkspaceOp::Deps(DepsOp::LoadToolRegistry),
        ))
        .expect("load_tool_registry node");

    // Node: RenderDepsToml
    let render_deps_toml = builder
        .add_node_after(
            Node::opaque(
                "render_deps_toml",
                vec![],
                vec![scalar("deps_toml_content", "String")],
                WorkspaceOp::Deps(DepsOp::RenderDepsToml),
            ),
            &load_registry,
        )
        .expect("render_deps_toml node");

    // Node: PrepareFileWrite
    let prepare_write = builder
        .add_node_after(
            Node::opaque(
                "prepare_file_write",
                vec![scalar("content", "String"), optional("output_path", "String")],
                vec![port("request", "TransportRequest")],
                WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                    PrepareFileWriteOp,
                )),
            ),
            &render_deps_toml,
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
            &prepare_write,
        )
        .expect("execute_transport node");

    // Wire up
    builder
        .add_edge(
            render_deps_toml.out("deps_toml_content"),
            prepare_write.in_port("content"),
        )
        .expect("content edge");
    builder
        .add_edge(
            prepare_write.out("request"),
            execute_transport.in_port("request"),
        )
        .expect("request edge");

    let inner_dag = builder.build();

    Node::subdag(
        "deps_generate",
        inner_dag,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_deps_install_subdag_is_subdag() {
        let node = build_deps_install_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "deps_install");
    }

    #[test]
    fn test_deps_install_subdag_structure() {
        let node = build_deps_install_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // 7 nodes in install pipeline
                assert_eq!(dag.nodes.len(), 7);
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_deps_generate_subdag_is_subdag() {
        let node = build_deps_generate_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "deps_generate");
    }

    #[test]
    fn test_deps_generate_subdag_structure() {
        let node = build_deps_generate_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // 4 nodes in generate pipeline
                assert_eq!(dag.nodes.len(), 4);
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
