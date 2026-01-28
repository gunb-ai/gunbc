use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use gunbc_ir::{Dag, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateNodeId(String),
    CycleDetected,
    TypeMismatch {
        edge_from: String,
        edge_to: String,
        from_type: String,
        to_type: String,
    },
    UnsatisfiedInput {
        node: String,
        port: String,
    },
    MissingPatternDecision {
        tool: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNodeId(id) => write!(f, "duplicate node id: {id}"),
            Self::CycleDetected => write!(f, "cycle detected in DAG"),
            Self::TypeMismatch { edge_from, edge_to, from_type, to_type } => {
                write!(f, "type mismatch on edge {edge_from} -> {edge_to}: {from_type} != {to_type}")
            }
            Self::UnsatisfiedInput { node, port } => {
                write!(f, "unsatisfied input port '{port}' on node '{node}'")
            }
            Self::MissingPatternDecision { tool } => {
                write!(f, "tool '{tool}' has no pattern decision in DAG metadata")
            }
        }
    }
}

/// Validate a DAG, returning all errors found.
pub fn validate<T: fmt::Debug>(dag: &Dag<T>) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    check_duplicate_ids(dag, &mut errors);
    check_acyclic(dag, &mut errors);
    check_type_agreement(dag, &mut errors);
    check_port_saturation(dag, &mut errors);
    check_pattern_decisions(dag, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_duplicate_ids<T>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    let mut seen = HashSet::new();
    for node in &dag.nodes {
        if !seen.insert(&node.id.0) {
            errors.push(ValidationError::DuplicateNodeId(node.id.0.clone()));
        }
    }
}

fn check_acyclic<T>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|id| (*id, 0usize)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|id| (*id, Vec::new())).collect();

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
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut visited = 0usize;
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

    if visited != node_ids.len() {
        errors.push(ValidationError::CycleDetected);
    }
}

fn check_type_agreement<T>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    let node_map: HashMap<&str, &Node<T>> = dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    for edge in &dag.edges {
        let from_node = node_map.get(edge.from_node.0.as_str());
        let to_node = node_map.get(edge.to_node.0.as_str());

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let from_type = from.outputs.iter().find(|p| p.name == edge.from_port).map(|p| &p.type_id);
            let to_type = to.inputs.iter().find(|p| p.name == edge.to_port).map(|p| &p.type_id);

            if let (Some(ft), Some(tt)) = (from_type, to_type) {
                if ft != tt {
                    errors.push(ValidationError::TypeMismatch {
                        edge_from: format!("{}.{}", edge.from_node.0, edge.from_port.0),
                        edge_to: format!("{}.{}", edge.to_node.0, edge.to_port.0),
                        from_type: ft.0.clone(),
                        to_type: tt.0.clone(),
                    });
                }
            }
        }
    }
}

fn check_port_saturation<T>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    let mut satisfied: HashSet<(String, String)> = HashSet::new();
    for edge in &dag.edges {
        satisfied.insert((edge.to_node.0.clone(), edge.to_port.0.clone()));
    }

    for node in &dag.nodes {
        for input in &node.inputs {
            if !satisfied.contains(&(node.id.0.clone(), input.name.0.clone())) {
                errors.push(ValidationError::UnsatisfiedInput {
                    node: node.id.0.clone(),
                    port: input.name.0.clone(),
                });
            }
        }
    }
}

fn check_pattern_decisions<T>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    // Collect all unique tool IDs from nodes
    let tools_in_nodes: HashSet<&str> = dag.nodes.iter()
        .map(|n| n.metadata.tool.0.as_str())
        .collect();

    // Collect tool IDs that have pattern decisions
    let tools_with_decisions: HashSet<&str> = dag.metadata.pattern_decisions.iter()
        .map(|entry| entry.tool.0.as_str())
        .collect();

    for tool in &tools_in_nodes {
        if !tools_with_decisions.contains(tool) {
            errors.push(ValidationError::MissingPatternDecision {
                tool: tool.to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::*;
    use gunbc_ir::metadata::NodeMetadata;
    use gunbc_ir::types::{BehaviorKind, PatternDecision, ToolId};

    fn simple_port(name: &str, ty: &str) -> Port {
        Port {
            name: PortName(name.into()),
            type_id: TypeId(ty.into()),
            guard: None,
        }
    }

    fn simple_node(id: &str, tool: &str, inputs: Vec<Port>, outputs: Vec<Port>) -> Node<String> {
        Node {
            id: NodeId(id.into()),
            inputs,
            outputs,
            metadata: NodeMetadata {
                tool: ToolId(tool.into()),
                behavior: BehaviorKind::Pure,
            },
            body: NodeBody::Opaque(id.into()),
        }
    }

    fn decisions_for(tools: &[&str]) -> DagMetadata {
        DagMetadata {
            pattern_decisions: tools.iter().map(|t| PatternDecisionEntry {
                tool: ToolId(t.to_string()),
                pattern: "upsert".into(),
                decision: PatternDecision::Instantiated,
            }).collect(),
        }
    }

    #[test]
    fn valid_dag_passes() {
        let dag = Dag {
            nodes: vec![
                simple_node("a", "t", vec![], vec![simple_port("out", "String")]),
                simple_node("b", "t", vec![simple_port("in", "String")], vec![]),
            ],
            edges: vec![Edge {
                from_node: NodeId("a".into()),
                from_port: PortName("out".into()),
                to_node: NodeId("b".into()),
                to_port: PortName("in".into()),
            }],
            metadata: decisions_for(&["t"]),
        };
        assert!(validate(&dag).is_ok());
    }

    #[test]
    fn cycle_detected() {
        let dag = Dag {
            nodes: vec![
                simple_node("a", "t", vec![simple_port("in", "S")], vec![simple_port("out", "S")]),
                simple_node("b", "t", vec![simple_port("in", "S")], vec![simple_port("out", "S")]),
            ],
            edges: vec![
                Edge { from_node: NodeId("a".into()), from_port: PortName("out".into()), to_node: NodeId("b".into()), to_port: PortName("in".into()) },
                Edge { from_node: NodeId("b".into()), from_port: PortName("out".into()), to_node: NodeId("a".into()), to_port: PortName("in".into()) },
            ],
            metadata: decisions_for(&["t"]),
        };
        let errs = validate(&dag).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::CycleDetected)));
    }

    #[test]
    fn type_mismatch_rejected() {
        let dag = Dag {
            nodes: vec![
                simple_node("a", "t", vec![], vec![simple_port("out", "Int")]),
                simple_node("b", "t", vec![simple_port("in", "String")], vec![]),
            ],
            edges: vec![Edge {
                from_node: NodeId("a".into()),
                from_port: PortName("out".into()),
                to_node: NodeId("b".into()),
                to_port: PortName("in".into()),
            }],
            metadata: decisions_for(&["t"]),
        };
        let errs = validate(&dag).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::TypeMismatch { .. })));
    }

    #[test]
    fn unsatisfied_port_rejected() {
        let dag = Dag {
            nodes: vec![
                simple_node("a", "t", vec![simple_port("needed", "S")], vec![]),
            ],
            edges: vec![],
            metadata: decisions_for(&["t"]),
        };
        let errs = validate(&dag).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::UnsatisfiedInput { .. })));
    }

    #[test]
    fn missing_pattern_decision_rejected() {
        let dag = Dag {
            nodes: vec![
                simple_node("a", "tool_a", vec![], vec![]),
                simple_node("b", "tool_b", vec![], vec![]),
            ],
            edges: vec![],
            metadata: decisions_for(&["tool_a"]), // missing tool_b
        };
        let errs = validate(&dag).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::MissingPatternDecision { tool } if tool == "tool_b")));
    }
}
