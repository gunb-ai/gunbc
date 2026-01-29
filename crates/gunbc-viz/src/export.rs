//! Export DAG structures to JSON for visualization.

use gunbc_ir::{detect_boundaries, detect_entrypoints, Dag, NodeBody};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A graph exported for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizGraph {
    /// Graph name/id
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Nodes in Cytoscape format
    pub nodes: Vec<VizNode>,
    /// Edges in Cytoscape format
    pub edges: Vec<VizEdge>,
    /// Metadata about the graph
    pub meta: VizMeta,
}

/// A node for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizNode {
    /// Node ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Parent node ID (for compound/nested nodes)
    pub parent: Option<String>,
    /// Node type: "opaque", "subdag", "port_in", "port_out"
    #[serde(rename = "type")]
    pub node_type: String,
    /// Input ports
    pub inputs: Vec<VizPort>,
    /// Output ports
    pub outputs: Vec<VizPort>,
    /// Is this a boundary node?
    pub is_boundary: bool,
    /// Is this an entrypoint node?
    pub is_entrypoint: bool,
    /// Additional classes for styling
    pub classes: Vec<String>,
}

/// A port for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizPort {
    pub name: String,
    #[serde(rename = "type")]
    pub port_type: String,
    pub is_entrypoint: bool,
    pub is_boundary: bool,
}

/// An edge for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizEdge {
    /// Edge ID
    pub id: String,
    /// Source node
    pub source: String,
    /// Target node
    pub target: String,
    /// Source port name
    pub source_port: String,
    /// Target port name
    pub target_port: String,
    /// Data type flowing through
    pub data_type: String,
}

/// Metadata about the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizMeta {
    pub node_count: usize,
    pub edge_count: usize,
    pub boundary_count: usize,
    pub entrypoint_count: usize,
    pub max_depth: usize,
}

/// Export a DAG to visualization format.
pub fn export_dag<T>(dag: &Dag<T>, name: &str) -> VizGraph
where
    T: std::fmt::Debug,
{
    let boundaries = detect_boundaries(dag);
    let entrypoints = detect_entrypoints(dag);

    let boundary_nodes: HashSet<_> = boundaries.boundary_nodes.iter().collect();
    let entrypoint_ports: HashSet<_> = entrypoints
        .entrypoint_ports
        .iter()
        .map(|(node_id, port_name, _)| (node_id.clone(), port_name.clone()))
        .collect();

    let mut viz_nodes = Vec::new();
    let mut viz_edges = Vec::new();
    let mut max_depth = 0;

    // Export nodes
    for node in &dag.nodes {
        let is_boundary = boundary_nodes.contains(&node.id);
        let is_entrypoint = entrypoints
            .entrypoint_ports
            .iter()
            .any(|(node_id, _, _)| *node_id == node.id);

        let mut classes = vec![];
        if is_boundary {
            classes.push("boundary".to_string());
        }
        if is_entrypoint {
            classes.push("entrypoint".to_string());
        }

        let node_type = match &node.body {
            NodeBody::Opaque(_) => "opaque",
            NodeBody::SubDag(_) => "subdag",
        };

        if matches!(node.body, NodeBody::SubDag(_)) {
            classes.push("compound".to_string());
            max_depth = max_depth.max(1); // TODO: recursive depth
        }

        let inputs: Vec<VizPort> = node
            .inputs
            .iter()
            .map(|p| VizPort {
                name: p.name.to_string(),
                port_type: p.type_id.to_string(),
                is_entrypoint: entrypoint_ports.contains(&(node.id.clone(), p.name.clone())),
                is_boundary: false,
            })
            .collect();

        let outputs: Vec<VizPort> = node
            .outputs
            .iter()
            .map(|p| {
                let is_boundary_port = boundaries
                    .boundary_ports
                    .iter()
                    .any(|(node_id, port_name)| *node_id == node.id && *port_name == p.name);
                VizPort {
                    name: p.name.to_string(),
                    port_type: p.type_id.to_string(),
                    is_entrypoint: false,
                    is_boundary: is_boundary_port,
                }
            })
            .collect();

        viz_nodes.push(VizNode {
            id: node.id.to_string(),
            label: node.id.to_string(),
            parent: None,
            node_type: node_type.to_string(),
            inputs,
            outputs,
            is_boundary,
            is_entrypoint,
            classes,
        });

        // If it's a subdag, recursively export child nodes
        if let NodeBody::SubDag(subdag) = &node.body {
            let sub_export = export_dag(subdag, &node.id.0);
            for mut child in sub_export.nodes {
                child.parent = Some(node.id.to_string());
                child.id = format!("{}_{}", node.id, child.id);
                viz_nodes.push(child);
            }
            for mut edge in sub_export.edges {
                edge.source = format!("{}_{}", node.id, edge.source);
                edge.target = format!("{}_{}", node.id, edge.target);
                edge.id = format!("{}_{}", node.id, edge.id);
                viz_edges.push(edge);
            }
            max_depth = max_depth.max(sub_export.meta.max_depth + 1);
        }
    }

    // Export edges
    for (i, edge) in dag.edges.iter().enumerate() {
        // Find the type from the source port
        let data_type = dag
            .get_node(&edge.from_node)
            .and_then(|n| n.outputs.iter().find(|p| p.name == edge.from_port))
            .map(|p| p.type_id.to_string())
            .unwrap_or_else(|| "?".to_string());

        viz_edges.push(VizEdge {
            id: format!("e{}", i),
            source: edge.from_node.to_string(),
            target: edge.to_node.to_string(),
            source_port: edge.from_port.to_string(),
            target_port: edge.to_port.to_string(),
            data_type,
        });
    }

    VizGraph {
        id: name.to_string(),
        name: name.to_string(),
        nodes: viz_nodes,
        edges: viz_edges,
        meta: VizMeta {
            node_count: dag.nodes.len(),
            edge_count: dag.edges.len(),
            boundary_count: boundaries.boundary_nodes.len(),
            entrypoint_count: entrypoints.entrypoint_ports.len(),
            max_depth,
        },
    }
}

/// Collection of multiple graphs for the visualizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizCollection {
    pub graphs: Vec<VizGraph>,
    pub generated_at: String,
}

impl VizCollection {
    pub fn new() -> Self {
        Self {
            graphs: Vec::new(),
            generated_at: chrono_lite(),
        }
    }

    pub fn add(&mut self, graph: VizGraph) {
        self.graphs.push(graph);
    }
}

impl Default for VizCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timestamp without chrono dependency.
fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build::port, Edge, Node};

    #[derive(Debug, Clone)]
    enum TestOp {
        A,
        B,
    }

    #[test]
    fn test_export_simple_dag() {
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![port("in", "String")],
            vec![port("out", "String")],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![port("in", "String")],
            vec![port("out", "String")],
            TestOp::B,
        ));
        dag.add_edge(Edge::new("a", "out", "b", "in"));

        let viz = export_dag(&dag, "test");

        assert_eq!(viz.nodes.len(), 2);
        assert_eq!(viz.edges.len(), 1);
        assert_eq!(viz.meta.boundary_count, 1); // 'b' is boundary
        assert_eq!(viz.meta.entrypoint_count, 1); // 'a.in' is entrypoint
    }
}
