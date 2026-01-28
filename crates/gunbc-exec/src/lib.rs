use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;

use gunbc_ir::{Dag, Node, NodeBody, NodeId};
use gunbc_ir::types::Secret;

pub mod guards;

/// Runtime value flowing between nodes.
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Str(String),
    StrList(Vec<String>),
    MapStrStr(BTreeMap<String, String>),
    Secret(Secret<String>),
    Skipped,
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::StrList(v) => write!(f, "[{} items]", v.len()),
            Value::MapStrStr(m) => write!(f, "{{{} entries}}", m.len()),
            Value::Secret(_) => write!(f, "<REDACTED>"),
            Value::Skipped => write!(f, "<SKIPPED>"),
            Value::Unit => write!(f, "()"),
        }
    }
}

/// Trait that opaque node operations must implement.
pub trait Executable: fmt::Debug {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError>;
}

#[derive(Debug, Clone)]
pub struct ExecError(pub String);

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ExecError {}

/// A single entry in the execution log.
#[derive(Debug)]
pub struct LogEntry {
    pub node_id: String,
    pub outputs: HashMap<String, Value>,
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.node_id)?;
        for (k, v) in &self.outputs {
            write!(f, " {k}={v}")?;
        }
        Ok(())
    }
}

/// Full execution log.
#[derive(Debug)]
pub struct ExecutionLog {
    pub entries: Vec<LogEntry>,
}

impl fmt::Display for ExecutionLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{entry}")?;
        }
        Ok(())
    }
}

/// Topologically sort the nodes in a DAG using Kahn's algorithm.
pub fn topo_sort<T>(dag: &Dag<T>) -> Vec<NodeId> {
    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|id| (*id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|id| (*id, Vec::new())).collect();

    for edge in &dag.edges {
        *in_degree.get_mut(edge.to_node.0.as_str()).unwrap() += 1;
        adj.get_mut(edge.from_node.0.as_str()).unwrap().push(&edge.to_node.0);
    }

    let mut queue: VecDeque<&str> = in_degree.iter().filter(|(_, &d)| d == 0).map(|(&id, _)| id).collect();
    let mut initial: Vec<&str> = queue.drain(..).collect();
    initial.sort();
    queue.extend(initial);

    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        result.push(NodeId(id.to_string()));
        if let Some(neighbors) = adj.get(id) {
            let mut next = Vec::new();
            for &neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    next.push(neighbor);
                }
            }
            next.sort();
            queue.extend(next);
        }
    }
    result
}

/// Check whether a node should be skipped based on guard expressions.
fn should_skip_node<T>(node: &Node<T>, inputs: &HashMap<String, Value>) -> bool {
    for port in &node.inputs {
        if let Some(guard_expr) = &port.guard {
            if let Some(value) = inputs.get(&port.name.0) {
                if !guards::eval_guard(guard_expr, value) {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    false
}

/// Execute a DAG.
pub fn execute<T: Executable>(dag: &Dag<T>) -> Result<ExecutionLog, ExecError> {
    let order = topo_sort(dag);
    let node_map: HashMap<&str, &Node<T>> = dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    let mut node_outputs: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut entries = Vec::new();

    for node_id in &order {
        let node = node_map[node_id.0.as_str()];

        // Gather inputs from upstream edges
        let mut inputs: HashMap<String, Value> = HashMap::new();
        for edge in &dag.edges {
            if edge.to_node == *node_id {
                if let Some(upstream) = node_outputs.get(&edge.from_node.0) {
                    if let Some(val) = upstream.get(&edge.from_port.0) {
                        inputs.insert(edge.to_port.0.clone(), val.clone());
                    }
                }
            }
        }

        // Check guards
        let skip = should_skip_node(node, &inputs);

        let outputs = if skip {
            node.outputs.iter().map(|p| (p.name.0.clone(), Value::Skipped)).collect()
        } else {
            match &node.body {
                NodeBody::Opaque(op) => op.execute(inputs)?,
                NodeBody::SubDag(sub_dag) => {
                    let sub_log = execute(sub_dag)?;
                    let mut sub_outputs = HashMap::new();
                    if let Some(last_entry) = sub_log.entries.last() {
                        for output_port in &node.outputs {
                            if let Some(val) = last_entry.outputs.get(&output_port.name.0) {
                                sub_outputs.insert(output_port.name.0.clone(), val.clone());
                            }
                        }
                    }
                    for entry in sub_log.entries {
                        entries.push(LogEntry {
                            node_id: format!("{}/{}", node_id.0, entry.node_id),
                            outputs: entry.outputs,
                        });
                    }
                    sub_outputs
                }
            }
        };

        node_outputs.insert(node_id.0.clone(), outputs.clone());
        entries.push(LogEntry {
            node_id: node_id.0.clone(),
            outputs,
        });
    }

    Ok(ExecutionLog { entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::*;
    use gunbc_ir::metadata::NodeMetadata;
    use gunbc_ir::types::{BehaviorKind, ToolId};

    #[derive(Debug, Clone)]
    struct Echo;

    impl Executable for Echo {
        fn execute(&self, mut inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
            let val = inputs.remove("in").unwrap_or(Value::Unit);
            let mut out = HashMap::new();
            out.insert("out".into(), val);
            Ok(out)
        }
    }

    fn port(name: &str, ty: &str) -> Port {
        Port { name: PortName(name.into()), type_id: TypeId(ty.into()), guard: None }
    }

    fn guarded_port(name: &str, ty: &str, guard: &str) -> Port {
        Port { name: PortName(name.into()), type_id: TypeId(ty.into()), guard: Some(guard.into()) }
    }

    fn meta(behavior: BehaviorKind) -> NodeMetadata {
        NodeMetadata { tool: ToolId("test".into()), behavior }
    }

    fn echo_node(id: &str, inputs: Vec<Port>, outputs: Vec<Port>) -> Node<Echo> {
        Node {
            id: NodeId(id.into()),
            inputs,
            outputs,
            metadata: meta(BehaviorKind::Pure),
            body: NodeBody::Opaque(Echo),
        }
    }

    fn empty_dag_metadata() -> DagMetadata {
        DagMetadata::default()
    }

    #[test]
    fn topo_sort_correct_order() {
        let dag = Dag {
            nodes: vec![
                echo_node("b", vec![port("in", "S")], vec![]),
                echo_node("a", vec![], vec![port("out", "S")]),
            ],
            edges: vec![Edge {
                from_node: NodeId("a".into()), from_port: PortName("out".into()),
                to_node: NodeId("b".into()), to_port: PortName("in".into()),
            }],
            metadata: empty_dag_metadata(),
        };
        let order = topo_sort(&dag);
        assert_eq!(order[0].0, "a");
        assert_eq!(order[1].0, "b");
    }

    #[test]
    fn execute_propagates_values() {
        let dag = Dag {
            nodes: vec![
                echo_node("a", vec![], vec![port("out", "S")]),
                echo_node("b", vec![port("in", "S")], vec![port("out", "S")]),
            ],
            edges: vec![Edge {
                from_node: NodeId("a".into()), from_port: PortName("out".into()),
                to_node: NodeId("b".into()), to_port: PortName("in".into()),
            }],
            metadata: empty_dag_metadata(),
        };
        let log = execute(&dag).unwrap();
        assert_eq!(log.entries.len(), 2);
        assert!(matches!(log.entries[1].outputs.get("out"), Some(Value::Unit)));
    }

    #[test]
    fn guard_skips_node_when_false() {
        #[derive(Debug, Clone)]
        struct Produce;
        impl Executable for Produce {
            fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
                let mut out = HashMap::new();
                out.insert("flag".into(), Value::Str("no".into()));
                Ok(out)
            }
        }

        let dag = Dag {
            nodes: vec![
                Node {
                    id: NodeId("a".into()),
                    inputs: vec![],
                    outputs: vec![port("flag", "S")],
                    metadata: meta(BehaviorKind::Pure),
                    body: NodeBody::Opaque(Produce),
                },
                Node {
                    id: NodeId("b".into()),
                    inputs: vec![guarded_port("flag", "S", "flag == yes")],
                    outputs: vec![port("out", "S")],
                    metadata: meta(BehaviorKind::Pure),
                    body: NodeBody::Opaque(Produce),
                },
            ],
            edges: vec![Edge {
                from_node: NodeId("a".into()), from_port: PortName("flag".into()),
                to_node: NodeId("b".into()), to_port: PortName("flag".into()),
            }],
            metadata: empty_dag_metadata(),
        };
        let log = execute(&dag).unwrap();
        let b_entry = log.entries.iter().find(|e| e.node_id == "b").unwrap();
        assert!(matches!(b_entry.outputs.get("out"), Some(Value::Skipped)));
    }

    #[test]
    fn subdag_executes_recursively() {
        let sub_dag = Dag {
            nodes: vec![echo_node("inner", vec![], vec![port("out", "S")])],
            edges: vec![],
            metadata: empty_dag_metadata(),
        };

        let dag = Dag {
            nodes: vec![
                Node {
                    id: NodeId("wrapper".into()),
                    inputs: vec![],
                    outputs: vec![port("out", "S")],
                    metadata: meta(BehaviorKind::Pure),
                    body: NodeBody::SubDag(sub_dag),
                },
            ],
            edges: vec![],
            metadata: empty_dag_metadata(),
        };

        let log = execute(&dag).unwrap();
        assert!(log.entries.iter().any(|e| e.node_id == "wrapper/inner"));
        assert!(log.entries.iter().any(|e| e.node_id == "wrapper"));
    }
}
