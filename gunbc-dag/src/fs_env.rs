//! Shared FsEnv graph-builder helpers.

use gunbc_ir::{BuilderError, DagBuilder, InputRef, Node, NodeRef, Port};
use gunbc_primitives::{filename, FsEnv};

/// Add a canonical `fs_env` root node with `file:write` capability output.
pub fn add_fs_env_root_node<T, F>(
    builder: &mut DagBuilder<T>,
    make_op: F,
) -> Result<NodeRef<T>, BuilderError>
where
    F: FnOnce(FsEnv) -> T,
{
    builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![Port::new(FsEnv::WRITE_PORT, "FilesystemHandle")],
        make_op(FsEnv::new(filename::Scope::Write)),
    ))
}

/// Wire `fs_env.file:write` to a list of resource input ports.
pub fn wire_fs_env_write_edges<T>(
    builder: &mut DagBuilder<T>,
    fs_env: &NodeRef<T>,
    targets: Vec<InputRef<T>>,
) -> Result<(), BuilderError> {
    for target in targets {
        builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_ops_graph::FileOpsGraph;
    use gunbc_ir::{DagBuilder, Node, Port};

    #[test]
    fn add_fs_env_root_node_uses_standard_shape() {
        let mut builder = DagBuilder::new();
        let fs_env =
            add_fs_env_root_node(&mut builder, FileOpsGraph::<()>::FsEnv).expect("fs_env root");
        let sink = builder
            .add_node_after(
                Node::opaque(
                    "sink",
                    vec![Port::resource(
                        "file",
                        "FilesystemHandle",
                        gunbc_ir::AccessMode::Write,
                    )],
                    vec![],
                    FileOpsGraph::Domain(()),
                ),
                &fs_env,
            )
            .expect("sink");
        wire_fs_env_write_edges(&mut builder, &fs_env, vec![sink.in_port("res:file")])
            .expect("wire fs edges");
        let dag = builder.build();
        assert!(dag.get_node(&"fs_env".into()).is_some());
        assert_eq!(dag.edges.len(), 1);
    }
}
