//! DAG analysis for test generation.

use gunbc_ir::{detect_boundaries, BoundaryInfo, Cardinality, CardinalityCase, Dag, TypeId};

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
    /// Test cases that should be generated for this port
    pub test_cases: Vec<CardinalityCase>,
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

/// Analyze a DAG for test generation.
pub fn analyze_dag<T>(dag: &Dag<T>) -> DagAnalysis {
    let boundaries = detect_boundaries(dag);
    let edge_types = analyze_edges(dag);
    let port_cardinalities = analyze_port_cardinalities(dag);

    DagAnalysis {
        boundaries,
        edge_types,
        port_cardinalities,
        node_count: dag.nodes.len(),
        edge_count: dag.edges.len(),
    }
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
                test_cases: port.cardinality.test_cases(),
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
                test_cases: port.cardinality.test_cases(),
            });
        }
    }

    results
}

/// Analyze edge types in a DAG.
fn analyze_edges<T>(dag: &Dag<T>) -> Vec<EdgeTypeInfo> {
    let mut results = Vec::new();

    for edge in &dag.edges {
        let from_node = dag.get_node(&edge.from_node);
        let to_node = dag.get_node(&edge.to_node);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let from_port = from.outputs.iter().find(|p| p.name == edge.from_port);
            let to_port = to.inputs.iter().find(|p| p.name == edge.to_port);

            if let (Some(fp), Some(tp)) = (from_port, to_port) {
                let compatible = types_compatible(&fp.type_id, &tp.type_id);
                
                results.push(EdgeTypeInfo {
                    from_node: edge.from_node.0.clone(),
                    from_port: edge.from_port.0.clone(),
                    to_node: edge.to_node.0.clone(),
                    to_port: edge.to_port.0.clone(),
                    from_type: fp.type_id.clone(),
                    to_type: tp.type_id.clone(),
                    compatible,
                });
            }
        }
    }

    results
}

/// Check if two types are compatible.
fn types_compatible(from: &TypeId, to: &TypeId) -> bool {
    from.0 == to.0 || from.0 == "Any" || to.0 == "Any"
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build::*, Dag, Node};

    #[test]
    fn test_analyze_simple_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "String")], vec![port("out", "String")], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let analysis = analyze_dag(&dag);

        assert_eq!(analysis.node_count, 2);
        assert_eq!(analysis.edge_count, 1);
        assert_eq!(analysis.boundaries.boundary_nodes.len(), 1);
        assert!(analysis.edge_types[0].compatible);
    }
}
