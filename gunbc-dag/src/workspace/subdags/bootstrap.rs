//! Bootstrap SubDag builder.
//!
//! Wraps the bootstrap tool as a SubDag node using `DynOp`.

use crate::bootstrap::BootstrapOp;
use crate::workspace::WorkspaceOp;
use gunbc_exec::DynOp;
use gunbc_ir::build::*;
use gunbc_ir::{BuilderError, DagBuilder, Node};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;

/// Build the bootstrap SubDag node.
pub fn build_bootstrap_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let mut builder: DagBuilder<WorkspaceOp> = DagBuilder::new();

    let prepare_scan = builder.add_root_node(Node::opaque(
        "prepare_scan_workspace",
        vec![],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        DynOp::new(BootstrapOp::PrepareScanWorkspace),
    ))?;

    let execute_scan = builder.add_node_after(
        Node::opaque(
            "execute_scan_workspace",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_scan,
    )?;

    let scan_workspace = builder.add_node_after(
        Node::opaque(
            "parse_scan_result",
            vec![port("response", "TransportResponse")],
            vec![
                port("crate_count", "Int"),
                list("crate_names", "StringList"),
            ],
            DynOp::new(BootstrapOp::ParseScanResult),
        ),
        &execute_scan,
    )?;

    let generate_makefile = builder.add_node_after(
        Node::opaque(
            "generate_makefile",
            vec![list("crate_names", "StringList")],
            vec![port("makefile_content", "String")],
            DynOp::new(BootstrapOp::GenerateMakefile),
        ),
        &scan_workspace,
    )?;

    let prepare_makefile = builder.add_node_after(
        Node::opaque(
            "prepare_makefile_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                PrepareFileWriteOp,
            )),
        ),
        &generate_makefile,
    )?;

    let execute_makefile = builder.add_node_after(
        Node::opaque(
            "execute_makefile_transport",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![
                port("makefile_response", "TransportResponse"),
                port("makefile_written_path", "String"),
                port("makefile_content", "String"),
            ],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_makefile,
    )?;

    let generate_gitignore = builder.add_node_after(
        Node::opaque(
            "generate_gitignore",
            vec![list("crate_names", "StringList")],
            vec![port("gitignore_content", "String")],
            DynOp::new(BootstrapOp::GenerateGitignore),
        ),
        &scan_workspace,
    )?;

    let prepare_gitignore = builder.add_node_after(
        Node::opaque(
            "prepare_gitignore_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            DynOp::new(gunbc_primitives::PrimitiveOp::PrepareFileWrite(
                PrepareFileWriteOp,
            )),
        ),
        &generate_gitignore,
    )?;

    let execute_gitignore = builder.add_node_after(
        Node::opaque(
            "execute_gitignore_transport",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![
                port("gitignore_response", "TransportResponse"),
                port("gitignore_written_path", "String"),
                port("gitignore_content", "String"),
            ],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_gitignore,
    )?;

    builder.add_edge(prepare_scan.out("request"), execute_scan.in_port("request"))?;
    builder.add_edge(prepare_scan.out("skip"), execute_scan.in_port("skip"))?;
    builder.add_edge(
        execute_scan.out("response"),
        scan_workspace.in_port("response"),
    )?;

    builder.add_edge(
        scan_workspace.out("crate_names"),
        generate_makefile.in_port("crate_names"),
    )?;
    builder.add_edge(
        generate_makefile.out("makefile_content"),
        prepare_makefile.in_port("content"),
    )?;
    builder.add_edge(
        prepare_makefile.out("request"),
        execute_makefile.in_port("request"),
    )?;
    builder.add_edge(
        prepare_makefile.out("skip"),
        execute_makefile.in_port("skip"),
    )?;

    builder.add_edge(
        scan_workspace.out("crate_names"),
        generate_gitignore.in_port("crate_names"),
    )?;
    builder.add_edge(
        generate_gitignore.out("gitignore_content"),
        prepare_gitignore.in_port("content"),
    )?;
    builder.add_edge(
        prepare_gitignore.out("request"),
        execute_gitignore.in_port("request"),
    )?;
    builder.add_edge(
        prepare_gitignore.out("skip"),
        execute_gitignore.in_port("skip"),
    )?;

    let inner_dag = builder.build();
    Ok(Node::subdag("bootstrap", inner_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_subdag_is_subdag() {
        let node = build_bootstrap_subdag().expect("bootstrap subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "bootstrap");
    }
}
