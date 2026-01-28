//! Lowering phase: flatten SubDags into a single-level DAG.
//!
//! This is the compiler step described in SPEC.md §5.2. After lowering,
//! every node is `Opaque` — the executor has no knowledge of SubDags.
//!
//! Algorithm:
//! 1. Find SubDag nodes
//! 2. Deduce port mappings from structure:
//!    - Source ports: internal nodes with input ports that have no incoming edges
//!    - Sink ports: export_node's outputs (or nodes with unconnected outputs)
//! 3. Verify type agreement at the boundary
//! 4. Inline internal nodes with prefixed IDs
//! 5. Rewire edges: parent edges connect to internal source/sink nodes by port name

use std::collections::{HashMap, HashSet};

use gunbc_ir::{Dag, Edge, Node, NodeBody, NodeId, PortName};

/// Errors that can occur during lowering.
#[derive(Debug, Clone)]
pub enum LowerError {
    /// Parent input port has no matching internal source port
    UnmappedInput {
        parent_id: String,
        port_name: String,
    },
    /// Parent output port has no matching internal sink port
    UnmappedOutput {
        parent_id: String,
        port_name: String,
    },
    /// Multiple internal nodes have the same open input port name
    AmbiguousSourcePort {
        parent_id: String,
        port_name: String,
        candidates: Vec<String>,
    },
    /// Type mismatch at SubDag boundary
    BoundaryTypeMismatch {
        parent_id: String,
        port_name: String,
        wrapper_type: String,
        inner_type: String,
    },
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LowerError::UnmappedInput { parent_id, port_name } => {
                write!(f, "SubDag '{}' has no internal source for input port '{}'", parent_id, port_name)
            }
            LowerError::UnmappedOutput { parent_id, port_name } => {
                write!(f, "SubDag '{}' has no internal sink for output port '{}'", parent_id, port_name)
            }
            LowerError::AmbiguousSourcePort { parent_id, port_name, candidates } => {
                write!(f, "SubDag '{}' has multiple candidates for input port '{}': {:?}",
                       parent_id, port_name, candidates)
            }
            LowerError::BoundaryTypeMismatch { parent_id, port_name, wrapper_type, inner_type } => {
                write!(f, "SubDag '{}' boundary type mismatch on port '{}': wrapper={}, inner={}",
                       parent_id, port_name, wrapper_type, inner_type)
            }
        }
    }
}

impl std::error::Error for LowerError {}

/// Lower a DAG by recursively inlining all SubDags.
///
/// Returns a flat DAG where all nodes are Opaque.
pub fn lower<T: Clone>(dag: &Dag<T>) -> Result<Dag<T>, LowerError> {
    let mut result = dag.clone();

    // Keep inlining until no SubDags remain
    loop {
        let subdag_node = result.nodes.iter()
            .find(|n| matches!(n.body, NodeBody::SubDag(_)));

        match subdag_node {
            Some(node) => {
                let node_id = node.id.clone();
                result = inline_subdag(&result, &node_id)?;
            }
            None => break,
        }
    }

    Ok(result)
}

/// Inline a single SubDag node into the parent DAG.
fn inline_subdag<T: Clone>(dag: &Dag<T>, parent_id: &NodeId) -> Result<Dag<T>, LowerError> {
    let parent_node = dag.nodes.iter()
        .find(|n| n.id == *parent_id)
        .expect("parent node must exist");

    let inner = match &parent_node.body {
        NodeBody::SubDag(inner) => inner,
        NodeBody::Opaque(_) => panic!("expected SubDag"),
    };

    // Recursively lower the inner DAG first
    let lowered_inner = lower(inner)?;

    // Find source ports (internal nodes with open input ports)
    let source_map = find_source_ports(&lowered_inner, parent_id)?;

    // Find sink ports (export_node outputs or unconnected outputs)
    let sink_map = find_sink_ports(&lowered_inner);

    // Build a lookup of inner port types for boundary checking
    let inner_port_types = build_inner_port_types(&lowered_inner);

    // Validate inputs: every parent input port must have a matching internal source
    // with matching type
    for input_port in &parent_node.inputs {
        let inner_id = source_map.get(&input_port.name).ok_or_else(|| {
            LowerError::UnmappedInput {
                parent_id: parent_id.0.clone(),
                port_name: input_port.name.0.clone(),
            }
        })?;

        // Check type agreement at boundary
        let inner_key = (inner_id.0.as_str(), input_port.name.0.as_str());
        if let Some(inner_type) = inner_port_types.get(&inner_key) {
            if *inner_type != input_port.type_id.0 {
                return Err(LowerError::BoundaryTypeMismatch {
                    parent_id: parent_id.0.clone(),
                    port_name: input_port.name.0.clone(),
                    wrapper_type: input_port.type_id.0.clone(),
                    inner_type: inner_type.to_string(),
                });
            }
        }
    }

    // Validate outputs: every parent output port must have a matching internal sink
    // with matching type
    for output_port in &parent_node.outputs {
        let inner_id = sink_map.get(&output_port.name).ok_or_else(|| {
            LowerError::UnmappedOutput {
                parent_id: parent_id.0.clone(),
                port_name: output_port.name.0.clone(),
            }
        })?;

        // Check type agreement at boundary
        if let Some(inner_node) = lowered_inner.nodes.iter().find(|n| n.id == *inner_id) {
            if let Some(inner_port) = inner_node.outputs.iter().find(|p| p.name == output_port.name) {
                if inner_port.type_id != output_port.type_id {
                    return Err(LowerError::BoundaryTypeMismatch {
                        parent_id: parent_id.0.clone(),
                        port_name: output_port.name.0.clone(),
                        wrapper_type: output_port.type_id.0.clone(),
                        inner_type: inner_port.type_id.0.clone(),
                    });
                }
            }
        }
    }

    // Build new node list (all except the parent, plus prefixed inner nodes)
    let mut new_nodes: Vec<Node<T>> = dag.nodes.iter()
        .filter(|n| n.id != *parent_id)
        .cloned()
        .collect();

    for inner_node in &lowered_inner.nodes {
        let mut new_node = inner_node.clone();
        new_node.id = prefix_id(parent_id, &inner_node.id);
        new_nodes.push(new_node);
    }

    // Build new edge list
    let mut new_edges = Vec::new();

    // Add inner edges with prefixed node IDs
    for edge in &lowered_inner.edges {
        new_edges.push(Edge {
            from_node: prefix_id(parent_id, &edge.from_node),
            from_port: edge.from_port.clone(),
            to_node: prefix_id(parent_id, &edge.to_node),
            to_port: edge.to_port.clone(),
        });
    }

    // Rewire external edges
    for edge in &dag.edges {
        if edge.to_node == *parent_id {
            // Edge pointing TO the parent — rewire to internal source node
            if let Some(internal_id) = source_map.get(&edge.to_port) {
                new_edges.push(Edge {
                    from_node: edge.from_node.clone(),
                    from_port: edge.from_port.clone(),
                    to_node: prefix_id(parent_id, internal_id),
                    to_port: edge.to_port.clone(),
                });
            }
        } else if edge.from_node == *parent_id {
            // Edge coming FROM the parent — rewire from internal sink node
            if let Some(internal_id) = sink_map.get(&edge.from_port) {
                new_edges.push(Edge {
                    from_node: prefix_id(parent_id, internal_id),
                    from_port: edge.from_port.clone(),
                    to_node: edge.to_node.clone(),
                    to_port: edge.to_port.clone(),
                });
            }
        } else {
            new_edges.push(edge.clone());
        }
    }

    Ok(Dag {
        nodes: new_nodes,
        edges: new_edges,
        metadata: dag.metadata.clone(),
    })
}

/// Find source ports: internal nodes with input ports that have no incoming edges.
///
/// Returns a map from port name to the node ID that has that open port.
/// Returns an error if two different nodes have open input ports with the same name
/// (ambiguous — the parent can't know which one to wire to).
fn find_source_ports<T>(dag: &Dag<T>, parent_id: &NodeId) -> Result<HashMap<PortName, NodeId>, LowerError> {
    let connected_inputs: HashSet<(String, String)> = dag.edges.iter()
        .map(|e| (e.to_node.0.clone(), e.to_port.0.clone()))
        .collect();

    // Collect ALL open ports first, tracking candidates per port name
    let mut candidates: HashMap<PortName, Vec<NodeId>> = HashMap::new();

    for node in &dag.nodes {
        for input_port in &node.inputs {
            let key = (node.id.0.clone(), input_port.name.0.clone());
            if !connected_inputs.contains(&key) {
                candidates
                    .entry(input_port.name.clone())
                    .or_default()
                    .push(node.id.clone());
            }
        }
    }

    let mut result = HashMap::new();
    for (port_name, nodes) in candidates {
        if nodes.len() > 1 {
            return Err(LowerError::AmbiguousSourcePort {
                parent_id: parent_id.0.clone(),
                port_name: port_name.0.clone(),
                candidates: nodes.iter().map(|n| n.0.clone()).collect(),
            });
        }
        result.insert(port_name, nodes.into_iter().next().unwrap());
    }

    Ok(result)
}

/// Find sink ports: export_node's outputs, or nodes with unconnected outputs.
/// Returns a map from port name to the node ID that produces that output.
fn find_sink_ports<T>(dag: &Dag<T>) -> HashMap<PortName, NodeId> {
    if let Some(export_id) = &dag.metadata.export_node {
        if let Some(export_node) = dag.nodes.iter().find(|n| n.id == *export_id) {
            return export_node.outputs.iter()
                .map(|p| (p.name.clone(), export_node.id.clone()))
                .collect();
        }
    }

    // Fallback: find nodes with output ports that have no outgoing edges
    let connected_outputs: HashSet<(String, String)> = dag.edges.iter()
        .map(|e| (e.from_node.0.clone(), e.from_port.0.clone()))
        .collect();

    let mut result = HashMap::new();
    for node in &dag.nodes {
        for output_port in &node.outputs {
            let key = (node.id.0.clone(), output_port.name.0.clone());
            if !connected_outputs.contains(&key) {
                result.insert(output_port.name.clone(), node.id.clone());
            }
        }
    }
    result
}

/// Build a lookup of (node_id, port_name) → type_id for input ports.
fn build_inner_port_types<'a, T>(dag: &'a Dag<T>) -> HashMap<(&'a str, &'a str), &'a str> {
    let mut map = HashMap::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            map.insert((node.id.0.as_str(), port.name.0.as_str()), port.type_id.0.as_str());
        }
    }
    map
}

fn prefix_id(parent: &NodeId, child: &NodeId) -> NodeId {
    NodeId(format!("{}/{}", parent.0, child.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::*;

    #[test]
    fn lower_flat_dag_unchanged() {
        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    inputs: vec![],
                    outputs: vec![port("out", "String")],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("b".into()),
                    inputs: vec![port("in", "String")],
                    outputs: vec![],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![edge("a", "out", "b", "in")],
            metadata: DagMetadata::default(),
        };

        let lowered = lower(&dag).unwrap();
        assert_eq!(lowered.nodes.len(), 2);
        assert_eq!(lowered.edges.len(), 1);
    }

    #[test]
    fn lower_simple_subdag() {
        let inner: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("inner_source".into()),
                    inputs: vec![port("x", "String")], // Open input — boundary
                    outputs: vec![port("y", "String")],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("inner_sink".into()),
                    inputs: vec![port("y", "String")],
                    outputs: vec![port("z", "String")],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![edge("inner_source", "y", "inner_sink", "y")],
            metadata: DagMetadata {
                export_node: Some(NodeId("inner_sink".into())),
                ..Default::default()
            },
        };

        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("producer".into()),
                    inputs: vec![],
                    outputs: vec![port("x", "String")],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("wrapper".into()),
                    inputs: vec![port("x", "String")],
                    outputs: vec![port("z", "String")],
                    body: NodeBody::SubDag(inner),
                },
                Node {
                    id: NodeId("consumer".into()),
                    inputs: vec![port("z", "String")],
                    outputs: vec![],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![
                edge("producer", "x", "wrapper", "x"),
                edge("wrapper", "z", "consumer", "z"),
            ],
            metadata: DagMetadata::default(),
        };

        let lowered = lower(&dag).unwrap();
        assert_eq!(lowered.nodes.len(), 4);

        let ids: Vec<&str> = lowered.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(ids.contains(&"producer"));
        assert!(ids.contains(&"wrapper/inner_source"));
        assert!(ids.contains(&"wrapper/inner_sink"));
        assert!(ids.contains(&"consumer"));

        assert_eq!(lowered.edges.len(), 3);

        let has_edge = |from: &str, fp: &str, to: &str, tp: &str| {
            lowered.edges.iter().any(|e|
                e.from_node.0 == from &&
                e.from_port.0 == fp &&
                e.to_node.0 == to &&
                e.to_port.0 == tp
            )
        };
        assert!(has_edge("producer", "x", "wrapper/inner_source", "x"));
        assert!(has_edge("wrapper/inner_source", "y", "wrapper/inner_sink", "y"));
        assert!(has_edge("wrapper/inner_sink", "z", "consumer", "z"));
    }

    #[test]
    fn find_source_ports_works() {
        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    inputs: vec![port("open", "String")],
                    outputs: vec![port("out", "String")],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("b".into()),
                    inputs: vec![port("connected", "String")],
                    outputs: vec![],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![edge("a", "out", "b", "connected")],
            metadata: DagMetadata::default(),
        };

        let parent = NodeId("test_parent".into());
        let sources = find_source_ports(&dag, &parent).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources.get(&PortName("open".into())).unwrap().0, "a");
    }

    #[test]
    fn find_sink_ports_uses_export_node() {
        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    inputs: vec![],
                    outputs: vec![port("x", "String")],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("b".into()),
                    inputs: vec![port("x", "String")],
                    outputs: vec![port("result", "String")],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![edge("a", "x", "b", "x")],
            metadata: DagMetadata {
                export_node: Some(NodeId("b".into())),
                ..Default::default()
            },
        };

        let sinks = find_sink_ports(&dag);
        assert_eq!(sinks.len(), 1);
        assert_eq!(sinks.get(&PortName("result".into())).unwrap().0, "b");
    }

    #[test]
    fn ambiguous_source_ports_detected() {
        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    inputs: vec![port("x", "String")], // open
                    outputs: vec![],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("b".into()),
                    inputs: vec![port("x", "String")], // also open, same name
                    outputs: vec![],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![],
            metadata: DagMetadata::default(),
        };

        let parent = NodeId("wrapper".into());
        let err = find_source_ports(&dag, &parent).unwrap_err();
        assert!(matches!(err, LowerError::AmbiguousSourcePort { .. }));
    }

    #[test]
    fn boundary_type_mismatch_detected() {
        let inner: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("inner".into()),
                    inputs: vec![port("x", "Int")], // Inner expects Int
                    outputs: vec![port("y", "String")],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![],
            metadata: DagMetadata {
                export_node: Some(NodeId("inner".into())),
                ..Default::default()
            },
        };

        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("producer".into()),
                    inputs: vec![],
                    outputs: vec![port("x", "String")],
                    body: NodeBody::Opaque(()),
                },
                Node {
                    id: NodeId("wrapper".into()),
                    inputs: vec![port("x", "String")], // Wrapper declares String
                    outputs: vec![port("y", "String")],
                    body: NodeBody::SubDag(inner),
                },
            ],
            edges: vec![
                edge("producer", "x", "wrapper", "x"),
            ],
            metadata: DagMetadata::default(),
        };

        let err = lower(&dag).unwrap_err();
        assert!(matches!(err, LowerError::BoundaryTypeMismatch { .. }));
    }
}
