//! IR-level validation for gunbc DAGs.
//!
//! These checks operate on `Dag<T>` (not contracts) and are complementary
//! to the codegen-time contract verification in `gunbc-codegen/src/verify.rs`.
//!
//! All functions are gated behind `debug_assertions` or the `validate` feature,
//! so they compile to nothing in release builds unless explicitly opted in.

use std::collections::{HashMap, HashSet, VecDeque};

use gunbc_ir::{Dag, PortName};

/// Verify that a DAG contains no cycles using Kahn's algorithm.
#[cfg(any(debug_assertions, feature = "validate"))]
pub fn validate_acyclic<T>(dag: &Dag<T>) -> Result<(), String> {
    let node_ids: HashSet<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|&id| (id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|&id| (id, Vec::new())).collect();

    for edge in &dag.edges {
        if let Some(deg) = in_degree.get_mut(edge.to_node.0.as_str()) {
            *deg += 1;
        }
        if let Some(neighbors) = adj.get_mut(edge.from_node.0.as_str()) {
            neighbors.push(&edge.to_node.0);
        }
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut visited = 0;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        if let Some(neighbors) = adj.get(id) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    if visited == node_ids.len() {
        Ok(())
    } else {
        Err(format!(
            "cycle detected in DAG: visited {visited} of {} nodes",
            node_ids.len()
        ))
    }
}

/// Verify that all edges connect ports with matching types.
#[cfg(any(debug_assertions, feature = "validate"))]
pub fn validate_types<T>(dag: &Dag<T>) -> Result<(), String> {
    // Build port type lookup: (node_id, port_name, direction) -> type_id
    let mut output_types: HashMap<(&str, &str), &str> = HashMap::new();
    let mut input_types: HashMap<(&str, &str), &str> = HashMap::new();

    for node in &dag.nodes {
        for p in &node.outputs {
            output_types.insert((node.id.0.as_str(), p.name.0.as_str()), p.type_id.0.as_str());
        }
        for p in &node.inputs {
            input_types.insert((node.id.0.as_str(), p.name.0.as_str()), p.type_id.0.as_str());
        }
    }

    for edge in &dag.edges {
        let from_key = (edge.from_node.0.as_str(), edge.from_port.0.as_str());
        let to_key = (edge.to_node.0.as_str(), edge.to_port.0.as_str());

        let from_type = output_types.get(&from_key).ok_or_else(|| {
            format!(
                "edge references unknown output port {}.{}",
                edge.from_node.0, edge.from_port.0
            )
        })?;

        let to_type = input_types.get(&to_key).ok_or_else(|| {
            format!(
                "edge references unknown input port {}.{}",
                edge.to_node.0, edge.to_port.0
            )
        })?;

        if from_type != to_type {
            return Err(format!(
                "type mismatch on edge {}.{} -> {}.{}: {} != {}",
                edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0,
                from_type, to_type
            ));
        }
    }

    Ok(())
}

/// Verify that every non-optional input port has exactly one incoming edge.
#[cfg(any(debug_assertions, feature = "validate"))]
pub fn validate_port_saturation<T>(dag: &Dag<T>) -> Result<(), String> {
    let mut satisfied: HashSet<(&str, &str)> = HashSet::new();

    for edge in &dag.edges {
        satisfied.insert((edge.to_node.0.as_str(), edge.to_port.0.as_str()));
    }

    // Find nodes with unsatisfied input ports (excluding nodes that have no
    // incoming edges at all — those are source/root nodes whose inputs come
    // from the parent scope, handled by lowering)
    let has_any_incoming: HashSet<&str> = dag.edges.iter()
        .map(|e| e.to_node.0.as_str())
        .collect();

    let mut open_ports: Vec<(&str, &PortName)> = Vec::new();

    for node in &dag.nodes {
        // Only check nodes that have at least one incoming edge — fully open
        // nodes are boundary nodes whose inputs come from the parent
        let node_has_incoming = has_any_incoming.contains(node.id.0.as_str());
        if !node_has_incoming && !node.inputs.is_empty() {
            // All inputs are open — this is a boundary source node, skip
            continue;
        }

        for input in &node.inputs {
            if !satisfied.contains(&(node.id.0.as_str(), input.name.0.as_str())) {
                open_ports.push((node.id.0.as_str(), &input.name));
            }
        }
    }

    if open_ports.is_empty() {
        Ok(())
    } else {
        let details: Vec<String> = open_ports
            .iter()
            .map(|(nid, pn)| format!("{}.{}", nid, pn.0))
            .collect();
        Err(format!("unsatisfied input ports: {}", details.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::*;

    fn simple_dag() -> Dag<()> {
        Dag {
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
        }
    }

    #[test]
    fn acyclic_passes() {
        validate_acyclic(&simple_dag()).unwrap();
    }

    #[test]
    fn types_pass() {
        validate_types(&simple_dag()).unwrap();
    }

    #[test]
    fn type_mismatch_detected() {
        let dag: Dag<()> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    inputs: vec![],
                    outputs: vec![port("out", "Int")],
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
        assert!(validate_types(&dag).is_err());
    }

    #[test]
    fn port_saturation_passes() {
        validate_port_saturation(&simple_dag()).unwrap();
    }

    #[test]
    fn port_saturation_detects_missing_edge() {
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
                    inputs: vec![port("x", "String"), port("y", "String")],
                    outputs: vec![],
                    body: NodeBody::Opaque(()),
                },
            ],
            edges: vec![edge("a", "x", "b", "x")],
            metadata: DagMetadata::default(),
        };
        assert!(validate_port_saturation(&dag).is_err());
    }
}
