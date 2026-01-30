//! Mock operations for testing.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{Dag, Node, NodeBody, Value};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Mock behavior: either scripted outputs or a function.
#[derive(Clone)]
pub enum MockBehavior {
    /// Return fixed outputs
    Scripted(HashMap<String, Value>),
    /// Compute outputs from inputs
    Func(Arc<dyn Fn(HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> + Send + Sync>),
}

impl MockBehavior {
    /// Create scripted behavior with fixed outputs.
    pub fn scripted(outputs: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Self {
        Self::Scripted(outputs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// Create functional behavior.
    pub fn func<F>(f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> + Send + Sync + 'static,
    {
        Self::Func(Arc::new(f))
    }
}

impl fmt::Debug for MockBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockBehavior::Scripted(outputs) => f.debug_tuple("Scripted").field(outputs).finish(),
            MockBehavior::Func(_) => f.debug_tuple("Func").field(&"<fn>").finish(),
        }
    }
}

/// A mock operation for testing.
#[derive(Clone)]
pub struct MockOp {
    node_id: String,
    behavior: MockBehavior,
}

impl MockOp {
    /// Create a new mock operation with scripted outputs.
    pub fn new(node_id: impl Into<String>, outputs: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Self {
        Self {
            node_id: node_id.into(),
            behavior: MockBehavior::scripted(outputs),
        }
    }

    /// Create a mock operation with functional behavior.
    pub fn with_func<F>(node_id: impl Into<String>, f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> + Send + Sync + 'static,
    {
        Self {
            node_id: node_id.into(),
            behavior: MockBehavior::func(f),
        }
    }
}

impl fmt::Debug for MockOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockOp")
            .field("node_id", &self.node_id)
            .field("behavior", &self.behavior)
            .finish()
    }
}

impl Executable for MockOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match &self.behavior {
            MockBehavior::Scripted(outputs) => Ok(outputs.clone()),
            MockBehavior::Func(f) => f(inputs),
        }
    }
}

/// Builder for creating DAGs with scripted mock behaviors.
pub struct ScriptedDagBuilder<'a, T> {
    dag: &'a Dag<T>,
    behaviors: HashMap<String, MockBehavior>,
}

impl<'a, T> ScriptedDagBuilder<'a, T> {
    /// Create a new builder from an existing DAG.
    pub fn new(dag: &'a Dag<T>) -> Self {
        Self {
            dag,
            behaviors: HashMap::new(),
        }
    }

    /// Set scripted outputs for a node.
    pub fn with_outputs(mut self, node_id: &str, outputs: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Self {
        self.behaviors.insert(node_id.to_string(), MockBehavior::scripted(outputs));
        self
    }

    /// Set functional behavior for a node.
    pub fn with_func<F>(mut self, node_id: &str, f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> + Send + Sync + 'static,
    {
        self.behaviors.insert(node_id.to_string(), MockBehavior::func(f));
        self
    }

    /// Build the mock DAG.
    pub fn build(self) -> Result<Dag<MockOp>, String> {
        let mut nodes = Vec::with_capacity(self.dag.nodes.len());

        for node in &self.dag.nodes {
            let behavior = self
                .behaviors
                .get(&node.id.0)
                .cloned()
                .ok_or_else(|| format!("missing mock behavior for node '{}'", node.id.0))?;

            let body = match &node.body {
                NodeBody::Opaque(_) => NodeBody::Opaque(MockOp {
                    node_id: node.id.0.clone(),
                    behavior,
                }),
                NodeBody::SubDag(_) => {
                    // For now, don't support sub-DAGs in mock builder
                    return Err(format!(
                        "node '{}' is a SubDag — mock builder doesn't support sub-DAGs yet",
                        node.id.0
                    ));
                }
            };

            nodes.push(Node {
                id: node.id.clone(),
                inputs: node.inputs.clone(),
                outputs: node.outputs.clone(),
                body,
                requires_tools: node.requires_tools.clone(),
            });
        }

        let mut result = Dag::new();
        for node in nodes {
            result.add_node(node);
        }
        for edge in &self.dag.edges {
            result.add_edge(edge.clone());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::execute;
    use gunbc_ir::build::*;

    #[test]
    fn test_mock_op_scripted() {
        let op = MockOp::new("test", [("out", Value::Str("hello".to_string()))]);
        let result = op.execute(HashMap::new()).unwrap();
        
        match result.get("out") {
            Some(Value::Str(s)) => assert_eq!(s, "hello"),
            _ => panic!("expected string output"),
        }
    }

    #[test]
    fn test_mock_op_func() {
        let op = MockOp::with_func("test", |inputs| {
            let val = inputs.get("in").cloned().unwrap_or(Value::Unit);
            let mut out = HashMap::new();
            out.insert("out".to_string(), val);
            Ok(out)
        });

        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), Value::Str("echo".to_string()));
        let result = op.execute(inputs).unwrap();

        match result.get("out") {
            Some(Value::Str(s)) => assert_eq!(s, "echo"),
            _ => panic!("expected string output"),
        }
    }

    #[test]
    fn test_scripted_dag_builder() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "S")], vec![port("out", "S")], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let mock_dag = ScriptedDagBuilder::new(&dag)
            .with_outputs("A", [("out", Value::Str("from-A".to_string()))])
            .with_outputs("B", [("out", Value::Str("from-B".to_string()))])
            .build()
            .unwrap();

        let log = execute(&mock_dag).unwrap();
        assert_eq!(log.entries.len(), 2);
    }
}
