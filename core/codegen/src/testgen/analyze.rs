//! DAG analysis for test generation.
//!
//! Two-layer analysis:
//! 1. **Structural analysis** (`DagAnalysis`): edges, ports, boundaries — raw facts.
//! 2. **Obligation collection** (`ObligationSet`): proof obligations derived from
//!    the structure + contract tower. Only obligations not discharged statically
//!    produce tests.

use gunbc_ir::resource::ResourceAccess;
use gunbc_ir::{detect_boundaries, BoundaryInfo, Cardinality, Dag, TypeId, TypeRegistry};

use crate::testgen::cardinality::fermi_test_cases;
use crate::testgen::obligation::{collect_obligations, ObligationSet};

/// Analysis of a DAG for test generation.
#[derive(Debug)]
pub struct DagAnalysis {
    /// Boundary information
    pub boundaries: BoundaryInfo,
    /// Edge type compatibility results
    pub edge_types: Vec<EdgeTypeInfo>,
    /// Port cardinality information
    pub port_cardinalities: Vec<PortCardinalityInfo>,
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Transport executor node IDs
    pub transport_executors: Vec<String>,
    /// Tool environment node IDs (emit ToolHandle)
    pub tool_env_nodes: Vec<String>,
    /// Nodes with guarded inputs (node_id, port_name)
    pub guarded_nodes: Vec<(String, String)>,
    /// Pure node IDs (no transport, no tool I/O)
    pub pure_nodes: Vec<String>,
    /// Credential node IDs (emit Credential outputs)
    pub credential_nodes: Vec<String>,
}

/// Information about an edge's types.
#[derive(Debug)]
pub struct EdgeTypeInfo {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
    pub from_type: TypeId,
    pub to_type: TypeId,
    pub compatible: bool,
}

/// Information about a port's cardinality.
#[derive(Debug, Clone)]
pub struct PortCardinalityInfo {
    pub node_id: String,
    pub port_name: String,
    pub is_input: bool,
    pub type_id: TypeId,
    pub cardinality: Cardinality,
    /// Boundary values that should be tested for this port
    pub test_cases: Vec<u32>,
}

impl PortCardinalityInfo {
    /// Check if this port needs cardinality tests.
    pub fn needs_tests(&self) -> bool {
        self.test_cases.len() > 1
    }

    /// Check if this is a list port (ZeroOrMore or OneOrMore).
    pub fn is_list(&self) -> bool {
        self.cardinality.allows_many()
    }
}

/// Analyze a DAG for test generation (structural analysis only).
pub fn analyze_dag<T>(dag: &Dag<T>) -> DagAnalysis {
    let boundaries = detect_boundaries(dag);
    let edge_types = analyze_edges(dag);
    let port_cardinalities = analyze_port_cardinalities(dag);
    let transport_executors = find_transport_executors(dag);
    let tool_env_nodes = find_tool_env_nodes(dag);
    let guarded_nodes = find_guarded_nodes(dag);
    let pure_nodes = find_pure_nodes(dag, &transport_executors, &tool_env_nodes);
    let credential_nodes = find_credential_nodes(dag);

    DagAnalysis {
        boundaries,
        edge_types,
        port_cardinalities,
        node_count: dag.nodes.len(),
        edge_count: dag.edges.len(),
        transport_executors,
        tool_env_nodes,
        guarded_nodes,
        pure_nodes,
        credential_nodes,
    }
}

/// Full analysis: structural + obligation collection.
///
/// This is the main entry point for test generation. It analyzes the DAG
/// structure and collects proof obligations, producing everything needed
/// to generate tests.
pub fn analyze_dag_with_obligations<T>(
    dag: &Dag<T>,
    registry: &TypeRegistry,
    resource_accesses: Option<&[ResourceAccess]>,
) -> (DagAnalysis, ObligationSet) {
    let analysis = analyze_dag(dag);
    let obligations = collect_obligations(dag, registry, resource_accesses);
    (analysis, obligations)
}

/// Find transport executor nodes (consume TransportRequest).
fn find_transport_executors<T>(dag: &Dag<T>) -> Vec<String> {
    dag.nodes
        .iter()
        .filter(|n| n.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"))
        .map(|n| n.id.0.clone())
        .collect()
}

/// Find tool environment nodes (emit ToolHandle).
fn find_tool_env_nodes<T>(dag: &Dag<T>) -> Vec<String> {
    dag.nodes
        .iter()
        .filter(|n| n.outputs.iter().any(|p| p.type_id.0 == "ToolHandle"))
        .map(|n| n.id.0.clone())
        .collect()
}

/// Find nodes with guarded inputs.
fn find_guarded_nodes<T>(dag: &Dag<T>) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            if port.has_guard() {
                result.push((node.id.0.clone(), port.name.0.clone()));
            }
        }
    }
    result
}

/// Find credential nodes (emit Credential outputs).
fn find_credential_nodes<T>(dag: &Dag<T>) -> Vec<String> {
    dag.nodes
        .iter()
        .filter(|n| n.outputs.iter().any(|p| p.type_id.0 == "Credential"))
        .map(|n| n.id.0.clone())
        .collect()
}

/// Find pure nodes (not transport executors, not tool env, not tool consumers, not SubDags).
fn find_pure_nodes<T>(
    dag: &Dag<T>,
    transport_executors: &[String],
    tool_env_nodes: &[String],
) -> Vec<String> {
    dag.nodes
        .iter()
        .filter(|n| {
            let id = &n.id.0;
            // SubDag nodes are composite — not pure opaque nodes.
            n.is_opaque()
                && !transport_executors.contains(id)
                && !tool_env_nodes.contains(id)
                && !n.inputs.iter().any(|p| p.type_id.0 == "ToolHandle")
        })
        .map(|n| n.id.0.clone())
        .collect()
}

/// Analyze port cardinalities in a DAG.
fn analyze_port_cardinalities<T>(dag: &Dag<T>) -> Vec<PortCardinalityInfo> {
    let mut results = Vec::new();

    for node in &dag.nodes {
        // Analyze input ports
        for port in &node.inputs {
            results.push(PortCardinalityInfo {
                node_id: node.id.0.clone(),
                port_name: port.name.0.clone(),
                is_input: true,
                type_id: port.type_id.clone(),
                cardinality: port.cardinality,
                test_cases: fermi_test_cases(port.cardinality),
            });
        }

        // Analyze output ports
        for port in &node.outputs {
            results.push(PortCardinalityInfo {
                node_id: node.id.0.clone(),
                port_name: port.name.0.clone(),
                is_input: false,
                type_id: port.type_id.clone(),
                cardinality: port.cardinality,
                test_cases: fermi_test_cases(port.cardinality),
            });
        }
    }

    results
}

/// Analyze edge types in a DAG.
fn analyze_edges<T>(dag: &Dag<T>) -> Vec<EdgeTypeInfo> {
    let mut results = Vec::new();
    let registry = TypeRegistry::with_core_types();

    for edge in &dag.edges {
        let Some(ports) = dag.resolve_edge_ports(edge) else {
            continue;
        };
        let compatible = types_compatible(ports.from.type_id(), ports.to.type_id(), &registry);

        results.push(EdgeTypeInfo {
            from_node: edge.from_node.0.clone(),
            from_port: edge.from_port.0.clone(),
            to_node: edge.to_node.0.clone(),
            to_port: edge.to_port.0.clone(),
            from_type: ports.from.type_id().clone(),
            to_type: ports.to.type_id().clone(),
            compatible,
        });
    }

    results
}

/// Check if two types are compatible.
fn types_compatible(from: &TypeId, to: &TypeId, registry: &TypeRegistry) -> bool {
    registry.is_compatible(from, to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build::*, Dag, Node};

    #[test]
    fn test_analyze_simple_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "String")],
            vec![port("out", "String")],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        let analysis = analyze_dag(&dag);

        assert_eq!(analysis.node_count, 2);
        assert_eq!(analysis.edge_count, 1);
        assert_eq!(analysis.boundaries.boundary_nodes.len(), 1);
        assert!(analysis.edge_types[0].compatible);
    }

    #[test]
    fn test_analyze_transport_detection() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare",
            vec![],
            vec![port("request", "TransportRequest")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            (),
        ));
        dag.add_edge(edge("prepare", "request", "execute", "request"));

        let analysis = analyze_dag(&dag);

        assert_eq!(analysis.transport_executors, vec!["execute"]);
        assert!(analysis.pure_nodes.contains(&"prepare".to_string()));
        assert!(!analysis.pure_nodes.contains(&"execute".to_string()));
    }

    #[test]
    fn test_analyze_tool_env_detection() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "env",
            vec![],
            vec![port("tool:clippy", "ToolHandle")],
            (),
        ));
        dag.add_node(Node::opaque(
            "lint",
            vec![port("tool:clippy", "ToolHandle")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("env", "tool:clippy", "lint", "tool:clippy"));

        let analysis = analyze_dag(&dag);

        assert_eq!(analysis.tool_env_nodes, vec!["env"]);
        // lint consumes ToolHandle → not pure
        assert!(!analysis.pure_nodes.contains(&"lint".to_string()));
    }

    #[test]
    fn test_analyze_credential_detection() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "cloud_credential",
            vec![],
            vec![port("credential", "Credential")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute",
            vec![port("credential", "Credential")],
            vec![port("response", "String")],
            (),
        ));
        dag.add_edge(edge(
            "cloud_credential",
            "credential",
            "execute",
            "credential",
        ));

        let analysis = analyze_dag(&dag);

        assert_eq!(analysis.credential_nodes, vec!["cloud_credential"]);
    }

    #[test]
    fn test_analyze_with_obligations() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("a", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("a", "out", "b", "in"));

        let registry = TypeRegistry::with_core_types();
        let (analysis, obligations) = analyze_dag_with_obligations(&dag, &registry, None);

        assert_eq!(analysis.node_count, 2);
        assert!(obligations.stats().total > 0);
    }
}
