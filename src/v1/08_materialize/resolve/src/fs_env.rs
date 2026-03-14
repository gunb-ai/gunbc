//! Shared FsEnv graph-builder helpers.

use gunbc_ir::filename::{self, FilesystemHandle, Scope};
use gunbc_ir::{BuilderError, DagBuilder, InputRef, Node, NodeRef, Port};
use gunbc_ir::{FILE_HANDLE_READ_PORT, FILE_HANDLE_WRITE_PORT};

/// Filesystem environment — acquires a FilesystemHandle.
#[derive(Debug, Clone)]
pub struct FsEnv {
    pub scope: Scope,
}

impl FsEnv {
    pub const READ_PORT: &'static str = FILE_HANDLE_READ_PORT;
    pub const WRITE_PORT: &'static str = FILE_HANDLE_WRITE_PORT;

    pub fn new(scope: Scope) -> Self {
        Self { scope }
    }

    pub fn output_port(&self) -> &'static str {
        match self.scope {
            Scope::Read => Self::READ_PORT,
            Scope::Write => Self::WRITE_PORT,
        }
    }

    /// Mock outputs for DryRun/testgen.
    pub fn mock_outputs(&self) -> std::collections::HashMap<String, gunbc_ir::Value> {
        let fs = FilesystemHandle::cross_platform(self.scope);
        gunbc_exec::env_single_output(self.output_port(), fs)
    }
}

impl gunbc_exec::EnvNode for FsEnv {
    fn env_outputs(
        &self,
    ) -> Result<std::collections::HashMap<String, gunbc_ir::Value>, gunbc_exec::ExecError> {
        Ok(self.mock_outputs())
    }

    fn mock_outputs(&self) -> std::collections::HashMap<String, gunbc_ir::Value> {
        FsEnv::mock_outputs(self)
    }
}

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
    use gunbc_exec::{DynOp, ExecError, Executable};
    use gunbc_ir::Value;
    use gunbc_ir::{DagBuilder, Node, Port};
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    struct NoopOp;

    impl Executable for NoopOp {
        fn execute(
            &self,
            _inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, ExecError> {
            Ok(HashMap::new())
        }
    }

    #[test]
    fn add_fs_env_root_node_uses_standard_shape() {
        let mut builder = DagBuilder::new();
        let fs_env = add_fs_env_root_node(&mut builder, DynOp::new).expect("fs_env root");
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
                    DynOp::new(NoopOp),
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
