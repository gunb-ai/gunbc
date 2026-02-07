//! Bootstrap SubDag builder.
//!
//! Wraps the bootstrap tool as a SubDag node using WorkspaceOp.

use crate::bootstrap::BootstrapOp;
use crate::workspace::WorkspaceOp;
use gunbc_ir::build::*;
use gunbc_ir::{DagBuilder, Node};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;

/// Build the bootstrap SubDag node.
///
/// This wraps the bootstrap workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # I/O Interface
///
/// Inputs: None (scans workspace automatically)
///
/// Outputs:
/// - `makefile_response`: TransportResponse
/// - `makefile_written_path`: String
/// - `makefile_content`: String
/// - `gitignore_response`: TransportResponse
/// - `gitignore_written_path`: String
/// - `gitignore_content`: String
/// - `crate_count`: Int
pub fn build_bootstrap_subdag() -> Node<WorkspaceOp> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    // Node: PrepareScanWorkspace (PURE)
    let prepare_scan = builder
        .add_root_node(Node::opaque(
            "prepare_scan_workspace",
            vec![],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            WorkspaceOp::Bootstrap(BootstrapOp::PrepareScanWorkspace),
        ))
        .expect("prepare_scan_workspace node");

    // Node: Execute scan (BOUNDARY)
    let execute_scan = builder
        .add_node_after(
            Node::opaque(
                "execute_scan_workspace",
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                vec![port("response", "TransportResponse")],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_scan,
        )
        .expect("execute_scan_workspace node");

    // Node: ParseScanResult (PURE)
    let scan_workspace = builder
        .add_node_after(
            Node::opaque(
                "parse_scan_result",
                vec![port("response", "TransportResponse")],
                vec![port("crate_count", "Int"), list("crate_names", "String")],
                WorkspaceOp::Bootstrap(BootstrapOp::ParseScanResult),
            ),
            &execute_scan,
        )
        .expect("parse_scan_result node");

    // === Makefile write chain ===

    let generate_makefile = builder
        .add_node_after(
            Node::opaque(
                "generate_makefile",
                vec![list("crate_names", "String")],
                vec![port("makefile_content", "String")],
                WorkspaceOp::Bootstrap(BootstrapOp::GenerateMakefile),
            ),
            &scan_workspace,
        )
        .expect("generate_makefile node");

    let prepare_makefile = builder
        .add_node_after(
            Node::opaque(
                "prepare_makefile_write",
                vec![port("path", "String"), port("content", "String")],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                    PrepareFileWriteOp,
                )),
            ),
            &generate_makefile,
        )
        .expect("prepare_makefile_write node");

    let execute_makefile = builder
        .add_node_after(
            Node::opaque(
                "execute_makefile_transport",
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                vec![
                    port("makefile_response", "TransportResponse"),
                    port("makefile_written_path", "String"),
                    port("makefile_content", "String"),
                ],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_makefile,
        )
        .expect("execute_makefile_transport node");

    // === Gitignore write chain ===

    let generate_gitignore = builder
        .add_node_after(
            Node::opaque(
                "generate_gitignore",
                vec![list("crate_names", "String")],
                vec![port("gitignore_content", "String")],
                WorkspaceOp::Bootstrap(BootstrapOp::GenerateGitignore),
            ),
            &scan_workspace,
        )
        .expect("generate_gitignore node");

    let prepare_gitignore = builder
        .add_node_after(
            Node::opaque(
                "prepare_gitignore_write",
                vec![port("path", "String"), port("content", "String")],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                    PrepareFileWriteOp,
                )),
            ),
            &generate_gitignore,
        )
        .expect("prepare_gitignore_write node");

    let _execute_gitignore = builder
        .add_node_after(
            Node::opaque(
                "execute_gitignore_transport",
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                vec![
                    port("gitignore_response", "TransportResponse"),
                    port("gitignore_written_path", "String"),
                    port("gitignore_content", "String"),
                ],
                WorkspaceOp::Transport(TransportOps::Execute),
            ),
            &prepare_gitignore,
        )
        .expect("execute_gitignore_transport node");

    // Wire up the ScanWorkspace chain
    builder
        .add_edge(prepare_scan.out("request"), execute_scan.in_port("request"))
        .expect("scan request edge");
    builder
        .add_edge(prepare_scan.out("skip"), execute_scan.in_port("skip"))
        .expect("scan skip edge");
    builder
        .add_edge(
            execute_scan.out("response"),
            scan_workspace.in_port("response"),
        )
        .expect("scan response edge");

    // Wire up the Makefile chain
    builder
        .add_edge(
            scan_workspace.out("crate_names"),
            generate_makefile.in_port("crate_names"),
        )
        .expect("makefile crate_names edge");
    builder
        .add_edge(
            generate_makefile.out("makefile_content"),
            prepare_makefile.in_port("content"),
        )
        .expect("makefile content edge");
    builder
        .add_edge(
            prepare_makefile.out("request"),
            execute_makefile.in_port("request"),
        )
        .expect("makefile request edge");
    builder
        .add_edge(
            prepare_makefile.out("skip"),
            execute_makefile.in_port("skip"),
        )
        .expect("makefile skip edge");

    // Wire up the Gitignore chain
    builder
        .add_edge(
            scan_workspace.out("crate_names"),
            generate_gitignore.in_port("crate_names"),
        )
        .expect("gitignore crate_names edge");
    builder
        .add_edge(
            generate_gitignore.out("gitignore_content"),
            prepare_gitignore.in_port("content"),
        )
        .expect("gitignore content edge");
    builder
        .add_edge(
            prepare_gitignore.out("request"),
            _execute_gitignore.in_port("request"),
        )
        .expect("gitignore request edge");
    builder
        .add_edge(
            prepare_gitignore.out("skip"),
            _execute_gitignore.in_port("skip"),
        )
        .expect("gitignore skip edge");

    let inner_dag = builder.build();

    Node::subdag("bootstrap", inner_dag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_subdag_is_subdag() {
        let node = build_bootstrap_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "bootstrap");
    }

}
