//! Virtual backend registry derivation (RT13).
//!
//! Derives `VirtualBackendConfig` metadata from DAG analysis.
//! For each transport executor node, determines the transport class
//! (REST, Shell, File) and prepares mock registry entries that
//! can be used to configure a `VirtualTransportBackend`.
//!
//! # Design
//!
//! Transport nodes in the DAG follow the prepare→execute→parse triplet
//! pattern. The lowerer stamps `ServiceTransportClass` on each node's
//! `transport_class` metadata field; we read it directly from the `Node`.
//! Type-based fallback (`from_node_context`) is only for test DAGs.
//!
//! The derived config is used by `build_fidelity_ladder_section()` to
//! generate S-tier test code that installs a `VirtualTransportBackend`
//! with appropriate mock registries.

use std::collections::BTreeMap;

use crate::testgen::analyze::DagAnalysis;
use gunbc_ir::node::ServiceTransportClass;
use gunbc_ir::Dag;

/// Transport class for a transport executor node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportClass {
    /// REST API calls (HTTP with JSON).
    Rest,
    /// HTTP requests (raw).
    Http,
    /// Shell command execution.
    Shell,
    /// Filesystem operations.
    File,
    /// TCP connections.
    Tcp,
    /// Local/direct (in-process).
    Local,
}

impl TransportClass {
    /// Classify a transport node by examining explicit request/response types.
    fn from_node_context(
        _node_id: &str,
        input_type: Option<&str>,
        output_type: Option<&str>,
    ) -> Self {
        match (input_type, output_type) {
            (Some("TcpRequest"), _) | (_, Some("TcpResponse")) => return Self::Tcp,
            (Some("ShellRequest"), _) | (_, Some("ShellResponse")) => return Self::Shell,
            (Some("FileRequest"), _) | (_, Some("FileResponse")) => return Self::File,
            (Some("HttpRequest"), _) | (_, Some("HttpResponse")) => return Self::Http,
            (Some("LocalRequest"), _) | (_, Some("LocalResponse")) => return Self::Local,
            _ => {}
        }
        // Default: REST (most common transport class).
        Self::Rest
    }
}

fn from_service_transport_class(class: ServiceTransportClass) -> TransportClass {
    match class {
        ServiceTransportClass::RestNetwork => TransportClass::Rest,
        ServiceTransportClass::ShellLocal => TransportClass::Shell,
        ServiceTransportClass::FileBoundary => TransportClass::File,
        ServiceTransportClass::LocalDirect => TransportClass::Local,
        ServiceTransportClass::InterfaceStub => TransportClass::Rest,
        ServiceTransportClass::Unknown => TransportClass::Rest,
    }
}

fn transport_class_from_node_metadata<T>(node: &gunbc_ir::Node<T>) -> Option<TransportClass> {
    node.transport_class.map(from_service_transport_class)
}

/// Info about a single transport executor node.
#[derive(Debug, Clone)]
pub struct TransportNodeInfo {
    /// Node ID of the transport executor.
    pub node_id: String,
    /// Inferred transport class.
    pub transport_class: TransportClass,
}

/// Summary of transport classes present in a DAG.
///
/// Used by codegen to determine which mock registries to set up.
#[derive(Debug, Clone, Default)]
pub struct VirtualBackendRequirements {
    /// Transport nodes grouped by class.
    pub nodes: Vec<TransportNodeInfo>,
    /// Whether the DAG uses REST transport.
    pub needs_rest: bool,
    /// Whether the DAG uses Shell transport.
    pub needs_shell: bool,
    /// Whether the DAG uses File transport.
    pub needs_file: bool,
    /// Whether the DAG uses TCP transport.
    pub needs_tcp: bool,
}

impl VirtualBackendRequirements {
    /// True if any virtual backend configuration is needed.
    pub fn needs_virtual_backend(&self) -> bool {
        self.needs_rest || self.needs_shell || self.needs_file || self.needs_tcp
    }

    /// Count of distinct transport classes used.
    pub fn transport_class_count(&self) -> usize {
        [
            self.needs_rest,
            self.needs_shell,
            self.needs_file,
            self.needs_tcp,
        ]
        .iter()
        .filter(|&&b| b)
        .count()
    }
}

/// Analyze transport executor nodes and derive virtual backend requirements.
///
/// Examines each transport executor in the DAG analysis, determines its
/// transport class, and produces a summary of what mock registries are needed.
pub fn derive_virtual_backend_requirements<T>(
    dag: &Dag<T>,
    analysis: &DagAnalysis,
) -> VirtualBackendRequirements {
    let mut requirements = VirtualBackendRequirements::default();

    // Build a quick lookup of node input/output types.
    let node_types: BTreeMap<&str, (Option<&str>, Option<&str>)> = dag
        .nodes
        .iter()
        .map(|n| {
            let req_input = n
                .inputs
                .iter()
                .find(|p| p.name.0 == "request")
                .map(|p| p.type_id.0.as_str());
            let resp_output = n
                .outputs
                .iter()
                .find(|p| p.name.0 == "response")
                .map(|p| p.type_id.0.as_str());
            (n.id.0.as_str(), (req_input, resp_output))
        })
        .collect();
    let node_by_id: BTreeMap<&str, &gunbc_ir::Node<T>> =
        dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    for executor_id in &analysis.transport_executors {
        let (input_type, output_type) = node_types
            .get(executor_id.as_str())
            .copied()
            .unwrap_or((None, None));

        let transport_class = node_by_id
            .get(executor_id.as_str())
            .and_then(|node| transport_class_from_node_metadata(*node))
            .unwrap_or_else(|| {
                TransportClass::from_node_context(executor_id, input_type, output_type)
            });

        match transport_class {
            TransportClass::Rest | TransportClass::Http => requirements.needs_rest = true,
            TransportClass::Shell => requirements.needs_shell = true,
            TransportClass::File => requirements.needs_file = true,
            TransportClass::Tcp => requirements.needs_tcp = true,
            TransportClass::Local => {} // No mock needed for local/direct.
        }

        requirements.nodes.push(TransportNodeInfo {
            node_id: executor_id.clone(),
            transport_class,
        });
    }

    requirements
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;
    use gunbc_ir::node::NodeKind;
    use gunbc_ir::{Dag, Node};

    #[test]
    fn classify_rest_from_unknown_types_defaults_to_rest() {
        assert_eq!(
            TransportClass::from_node_context(
                "service_transport::execute::github.Gist::Create",
                None,
                None
            ),
            TransportClass::Rest
        );
    }

    #[test]
    fn classify_shell_from_port_types() {
        assert_eq!(
            TransportClass::from_node_context(
                "execute_shell",
                Some("ShellRequest"),
                Some("ShellResponse")
            ),
            TransportClass::Shell
        );
    }

    #[test]
    fn classify_file_from_port_types() {
        assert_eq!(
            TransportClass::from_node_context(
                "execute_file",
                Some("FileRequest"),
                Some("FileResponse")
            ),
            TransportClass::File
        );
    }

    #[test]
    fn classify_tcp_from_port_types() {
        assert_eq!(
            TransportClass::from_node_context(
                "execute_tcp_check",
                Some("TcpRequest"),
                Some("TcpResponse")
            ),
            TransportClass::Tcp
        );
    }

    #[test]
    fn derive_requirements_for_rest_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "service_transport::execute::github.Gist::Create",
                vec![port("request", "TransportRequest")],
                vec![port("response", "TransportResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let analysis = crate::testgen::analyze::analyze_dag(&dag);
        let requirements = derive_virtual_backend_requirements(&dag, &analysis);

        assert!(requirements.needs_rest);
        assert!(!requirements.needs_shell);
        assert!(!requirements.needs_file);
        assert_eq!(requirements.nodes.len(), 1);
        assert_eq!(requirements.nodes[0].transport_class, TransportClass::Rest);
    }

    #[test]
    fn derive_requirements_for_mixed_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(
            Node::opaque(
                "execute_git_status",
                vec![port("request", "ShellRequest")],
                vec![port("response", "ShellResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "execute_file_read",
                vec![port("request", "FileRequest")],
                vec![port("response", "FileResponse")],
                (),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let analysis = crate::testgen::analyze::analyze_dag(&dag);
        let requirements = derive_virtual_backend_requirements(&dag, &analysis);

        assert!(requirements.needs_shell);
        assert!(requirements.needs_file);
        assert!(!requirements.needs_rest);
        assert_eq!(requirements.transport_class_count(), 2);
    }

    #[test]
    fn empty_dag_has_no_requirements() {
        let dag: Dag<()> = Dag::new();
        let analysis = crate::testgen::analyze::analyze_dag(&dag);
        let requirements = derive_virtual_backend_requirements(&dag, &analysis);

        assert!(!requirements.needs_virtual_backend());
        assert_eq!(requirements.transport_class_count(), 0);
    }
}
