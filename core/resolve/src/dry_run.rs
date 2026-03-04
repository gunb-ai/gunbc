//! Dry-run helper utilities shared across binaries.

use gunbc_exec::BoundaryMocks;
use gunbc_ir::{Dag, NodeId};
use gunbc_primitives::filename;

fn is_fs_env_node(node_id: &NodeId) -> bool {
    node_id
        .0
        .rsplit_once('/')
        .map_or(node_id.0.as_str(), |(_, leaf)| leaf)
        == "fs_env"
}

/// Auto-wire a filesystem write-handle dry-run mock when the DAG declares an
/// `fs_env` node with a `FilesystemHandle` output.
///
/// Returns true when a mock was inserted.
pub fn wire_fs_env_write_mock<T>(dag: &Dag<T>, mocks: &mut BoundaryMocks) -> bool {
    let fs: gunbc_ir::Value =
        filename::FilesystemHandle::cross_platform(filename::Scope::Write).into();
    let mut inserted = false;

    for node in dag.nodes.iter().filter(|node| is_fs_env_node(&node.id)) {
        for port in node
            .outputs
            .iter()
            .filter(|port| port.type_id.0 == "FilesystemHandle")
        {
            mocks.set_value(node.id.0.as_str(), port.name.0.as_str(), fs.clone());
            inserted = true;
        }
    }

    inserted
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{Node, Port, PortName, Value};
    use gunbc_primitives::FsEnv;

    #[test]
    fn wire_fs_env_write_mock_sets_value_when_node_exists() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "scope/fs_env",
            vec![],
            vec![Port::new(FsEnv::WRITE_PORT, "FilesystemHandle")],
            (),
        ));

        let mut mocks = BoundaryMocks::new();
        assert!(wire_fs_env_write_mock(&dag, &mut mocks));
        assert!(mocks.has_mock(
            &NodeId::from("scope/fs_env"),
            &PortName::from(FsEnv::WRITE_PORT)
        ));
        let value = mocks
            .get_mock(
                &NodeId::from("scope/fs_env"),
                &PortName::from(FsEnv::WRITE_PORT),
            )
            .expect("fs_env mock should be registered");
        assert!(!matches!(value.value, Value::Skipped));
    }

    #[test]
    fn wire_fs_env_write_mock_is_noop_when_node_missing() {
        let dag: Dag<()> = Dag::new();
        let mut mocks = BoundaryMocks::new();
        assert!(!wire_fs_env_write_mock(&dag, &mut mocks));
    }

    #[test]
    fn wire_fs_env_write_mock_supports_dsl_port_name() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "fs_env",
            vec![],
            vec![Port::new("FilesystemHandle", "FilesystemHandle")],
            (),
        ));

        let mut mocks = BoundaryMocks::new();
        assert!(wire_fs_env_write_mock(&dag, &mut mocks));
        assert!(mocks.has_mock(&NodeId::from("fs_env"), &PortName::from("FilesystemHandle")));
    }
}
