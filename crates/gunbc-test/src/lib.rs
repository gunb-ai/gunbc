use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use gunbc_exec::{ExecError, Executable, ExecutionLog, Value};
use gunbc_ir::{Dag, Node, NodeBody};

pub type Outputs = HashMap<String, Value>;

#[derive(Clone)]
pub enum MockBehavior {
    Scripted(Outputs),
    Func(Arc<dyn Fn(HashMap<String, Value>) -> Result<Outputs, ExecError> + Send + Sync>),
}

impl MockBehavior {
    pub fn scripted(outputs: Outputs) -> Self {
        Self::Scripted(outputs)
    }

    pub fn func<F>(f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Result<Outputs, ExecError> + Send + Sync + 'static,
    {
        Self::Func(Arc::new(f))
    }
}

#[derive(Clone)]
pub struct MockOp {
    node_id: String,
    behavior: MockBehavior,
}

impl fmt::Debug for MockOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockOp")
            .field("node_id", &self.node_id)
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

pub struct ScriptedDagBuilder<'a, T> {
    dag: &'a Dag<T>,
    behaviors: HashMap<String, MockBehavior>,
}

impl<'a, T> ScriptedDagBuilder<'a, T> {
    pub fn new(dag: &'a Dag<T>) -> Self {
        Self {
            dag,
            behaviors: HashMap::new(),
        }
    }

    pub fn with_outputs(mut self, node_id: &str, outputs: Outputs) -> Self {
        self.behaviors.insert(node_id.to_string(), MockBehavior::scripted(outputs));
        self
    }

    pub fn with_func<F>(mut self, node_id: &str, f: F) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Result<Outputs, ExecError> + Send + Sync + 'static,
    {
        self.behaviors.insert(node_id.to_string(), MockBehavior::func(f));
        self
    }

    pub fn build(self) -> Result<Dag<MockOp>, String> {
        map_dag(self.dag, &self.behaviors)
    }
}

fn map_dag<T>(dag: &Dag<T>, behaviors: &HashMap<String, MockBehavior>) -> Result<Dag<MockOp>, String> {
    let mut nodes = Vec::with_capacity(dag.nodes.len());
    for node in &dag.nodes {
        let body = match &node.body {
            NodeBody::Opaque(_) => {
                let behavior = behaviors
                    .get(&node.id.0)
                    .cloned()
                    .ok_or_else(|| format!("missing scripted behavior for node '{}'", node.id.0))?;
                NodeBody::Opaque(MockOp {
                    node_id: node.id.0.clone(),
                    behavior,
                })
            }
            NodeBody::SubDag(sub) => NodeBody::SubDag(map_dag(sub, behaviors)?),
        };
        nodes.push(Node {
            id: node.id.clone(),
            inputs: node.inputs.clone(),
            outputs: node.outputs.clone(),
            metadata: node.metadata.clone(),
            body,
        });
    }

    Ok(Dag {
        nodes,
        edges: dag.edges.clone(),
        metadata: dag.metadata.clone(),
    })
}

pub fn execute_scripted<T>(builder: ScriptedDagBuilder<'_, T>) -> Result<ExecutionLog, ExecError> {
    let scripted = builder.build().map_err(ExecError)?;
    gunbc_exec::execute(&scripted)
}

/// Assert that a DAG has the upsert diamond topology:
/// - `check` node feeds both `create` and `resolve`
/// - `create` node feeds `resolve`
/// - `create` has a guard on one of its inputs
/// - `resolve` is the export node
pub fn assert_upsert_topology<T>(dag: &Dag<T>, check: &str, create: &str, resolve: &str) {
    // Verify all three nodes exist
    assert!(
        dag.nodes.iter().any(|n| n.id.0 == check),
        "missing check node '{check}'"
    );
    assert!(
        dag.nodes.iter().any(|n| n.id.0 == create),
        "missing create node '{create}'"
    );
    assert!(
        dag.nodes.iter().any(|n| n.id.0 == resolve),
        "missing resolve node '{resolve}'"
    );

    // Verify diamond edges: check -> create, check -> resolve, create -> resolve
    let has_edge = |from: &str, to: &str| {
        dag.edges.iter().any(|e| e.from_node.0 == from && e.to_node.0 == to)
    };
    assert!(
        has_edge(check, create),
        "missing edge {check} -> {create}"
    );
    assert!(
        has_edge(check, resolve),
        "missing edge {check} -> {resolve}"
    );
    assert!(
        has_edge(create, resolve),
        "missing edge {create} -> {resolve}"
    );

    // Verify create has a guard
    let create_node = dag.nodes.iter().find(|n| n.id.0 == create).unwrap();
    assert!(
        create_node.inputs.iter().any(|p| p.guard.is_some()),
        "create node '{create}' should have a guarded input"
    );

    // Verify export_node is resolve
    assert_eq!(
        dag.metadata.export_node.as_ref().map(|n| n.0.as_str()),
        Some(resolve),
        "export_node should be '{resolve}'"
    );
}

/// Assert that a node produced Skipped outputs when its guard was false.
pub fn assert_upsert_skip_semantics(log: &ExecutionLog, create_id: &str) {
    let entry = log.entries.iter().find(|e| e.node_id == create_id);
    match entry {
        Some(e) => {
            let all_skipped = e.outputs.values().all(|v| matches!(v, Value::Skipped));
            assert!(
                all_skipped,
                "create node '{create_id}' should produce all Skipped outputs when guard is false, got: {:?}",
                e.outputs
            );
        }
        None => {
            panic!("create node '{create_id}' not found in execution log");
        }
    }
}
