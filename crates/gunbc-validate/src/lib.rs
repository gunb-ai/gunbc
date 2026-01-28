use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use gunbc_ir::{Dag, Node, NodeBody};

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
    UnknownNode {
        edge_desc: String,
        node_id: String,
    },
    UnknownPort {
        edge_desc: String,
        node_id: String,
        port_name: String,
    },
    DuplicateInputEdge {
        node: String,
        port: String,
    },
    ExportNodeNotFound {
        wrapper_node: String,
        export_node: String,
    },
    ExportNodeMissingPort {
        wrapper_node: String,
        export_node: String,
        port: String,
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
            Self::UnknownNode { edge_desc, node_id } => {
                write!(f, "edge {edge_desc} references unknown node '{node_id}'")
            }
            Self::UnknownPort { edge_desc, node_id, port_name } => {
                write!(f, "edge {edge_desc} references unknown port '{port_name}' on node '{node_id}'")
            }
            Self::DuplicateInputEdge { node, port } => {
                write!(f, "multiple edges target input port '{port}' on node '{node}'")
            }
            Self::ExportNodeNotFound { wrapper_node, export_node } => {
                write!(f, "subdag in node '{wrapper_node}' references nonexistent export_node '{export_node}'")
            }
            Self::ExportNodeMissingPort { wrapper_node, export_node, port } => {
                write!(f, "export_node '{export_node}' in subdag of '{wrapper_node}' is missing output port '{port}'")
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
    check_unique_input_edges(dag, &mut errors);
    check_pattern_decisions(dag, &mut errors);
    check_export_nodes(dag, &mut errors);

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
        let edge_desc = format!("{}.{} -> {}.{}", edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0);

        let from_node = match node_map.get(edge.from_node.0.as_str()) {
            Some(n) => n,
            None => {
                errors.push(ValidationError::UnknownNode {
                    edge_desc,
                    node_id: edge.from_node.0.clone(),
                });
                continue;
            }
        };
        let to_node = match node_map.get(edge.to_node.0.as_str()) {
            Some(n) => n,
            None => {
                errors.push(ValidationError::UnknownNode {
                    edge_desc,
                    node_id: edge.to_node.0.clone(),
                });
                continue;
            }
        };

        let from_type = match from_node.outputs.iter().find(|p| p.name == edge.from_port) {
            Some(p) => &p.type_id,
            None => {
                errors.push(ValidationError::UnknownPort {
                    edge_desc,
                    node_id: edge.from_node.0.clone(),
                    port_name: edge.from_port.0.clone(),
                });
                continue;
            }
        };
        let to_type = match to_node.inputs.iter().find(|p| p.name == edge.to_port) {
            Some(p) => &p.type_id,
            None => {
                errors.push(ValidationError::UnknownPort {
                    edge_desc,
                    node_id: edge.to_node.0.clone(),
                    port_name: edge.to_port.0.clone(),
                });
                continue;
            }
        };

        if from_type != to_type {
            errors.push(ValidationError::TypeMismatch {
                edge_from: format!("{}.{}", edge.from_node.0, edge.from_port.0),
                edge_to: format!("{}.{}", edge.to_node.0, edge.to_port.0),
                from_type: from_type.0.clone(),
                to_type: to_type.0.clone(),
            });
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

fn check_unique_input_edges<T>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for edge in &dag.edges {
        let key = (edge.to_node.0.clone(), edge.to_port.0.clone());
        if !seen.insert(key) {
            errors.push(ValidationError::DuplicateInputEdge {
                node: edge.to_node.0.clone(),
                port: edge.to_port.0.clone(),
            });
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

fn check_export_nodes<T: fmt::Debug>(dag: &Dag<T>, errors: &mut Vec<ValidationError>) {
    for node in &dag.nodes {
        if let NodeBody::SubDag(ref sub) = node.body {
            if let Some(ref export_id) = sub.metadata.export_node {
                let export_node = sub.nodes.iter().find(|n| n.id == *export_id);
                match export_node {
                    None => {
                        errors.push(ValidationError::ExportNodeNotFound {
                            wrapper_node: node.id.0.clone(),
                            export_node: export_id.0.clone(),
                        });
                    }
                    Some(en) => {
                        for output in &node.outputs {
                            let has_port = en.outputs.iter().any(|p| {
                                p.name == output.name && p.type_id == output.type_id
                            });
                            if !has_port {
                                errors.push(ValidationError::ExportNodeMissingPort {
                                    wrapper_node: node.id.0.clone(),
                                    export_node: export_id.0.clone(),
                                    port: output.name.0.clone(),
                                });
                            }
                        }
                    }
                }
            }
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
            export_node: None,
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

    fn subdag_node(id: &str, tool: &str, inputs: Vec<Port>, outputs: Vec<Port>, sub: Dag<String>) -> Node<String> {
        Node {
            id: NodeId(id.into()),
            inputs,
            outputs,
            metadata: NodeMetadata {
                tool: ToolId(tool.into()),
                behavior: BehaviorKind::Pure,
            },
            body: NodeBody::SubDag(sub),
        }
    }

    #[test]
    fn valid_export_node_passes() {
        let inner = Dag {
            nodes: vec![
                simple_node("inner_a", "t", vec![], vec![simple_port("result", "String")]),
            ],
            edges: vec![],
            metadata: DagMetadata {
                pattern_decisions: vec![PatternDecisionEntry {
                    tool: ToolId("t".into()),
                    pattern: "upsert".into(),
                    decision: PatternDecision::Instantiated,
                }],
                export_node: Some(NodeId("inner_a".into())),
            },
        };
        let dag = Dag {
            nodes: vec![
                subdag_node("wrapper", "t", vec![], vec![simple_port("result", "String")], inner),
            ],
            edges: vec![],
            metadata: decisions_for(&["t"]),
        };
        assert!(validate(&dag).is_ok());
    }

    #[test]
    fn export_node_not_found() {
        let inner = Dag {
            nodes: vec![
                simple_node("inner_a", "t", vec![], vec![]),
            ],
            edges: vec![],
            metadata: DagMetadata {
                pattern_decisions: vec![PatternDecisionEntry {
                    tool: ToolId("t".into()),
                    pattern: "upsert".into(),
                    decision: PatternDecision::Instantiated,
                }],
                export_node: Some(NodeId("nonexistent".into())),
            },
        };
        let dag = Dag {
            nodes: vec![
                subdag_node("wrapper", "t", vec![], vec![], inner),
            ],
            edges: vec![],
            metadata: decisions_for(&["t"]),
        };
        let errs = validate(&dag).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::ExportNodeNotFound { wrapper_node, export_node } if wrapper_node == "wrapper" && export_node == "nonexistent")));
    }

    #[test]
    fn export_node_missing_port() {
        let inner = Dag {
            nodes: vec![
                simple_node("inner_a", "t", vec![], vec![]), // no outputs
            ],
            edges: vec![],
            metadata: DagMetadata {
                pattern_decisions: vec![PatternDecisionEntry {
                    tool: ToolId("t".into()),
                    pattern: "upsert".into(),
                    decision: PatternDecision::Instantiated,
                }],
                export_node: Some(NodeId("inner_a".into())),
            },
        };
        let dag = Dag {
            nodes: vec![
                subdag_node("wrapper", "t", vec![], vec![simple_port("result", "String")], inner),
            ],
            edges: vec![],
            metadata: decisions_for(&["t"]),
        };
        let errs = validate(&dag).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::ExportNodeMissingPort { wrapper_node, export_node, port } if wrapper_node == "wrapper" && export_node == "inner_a" && port == "result")));
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
