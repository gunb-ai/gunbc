use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use gunbc_exec::{ExecError, Executable, ExecutionLog, Value};
use gunbc_ir::{Dag, Node, NodeBody};
use gunbc_ir::types::{BehaviorKind, PatternDecision};

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

/// Specification for validating an Upsert pattern in a DAG.
///
/// The Upsert pattern has three canonical nodes:
/// - **check** (Observe): Reads world state, outputs decision flag and existing value
/// - **create** (WritesWorld): Guarded by decision, creates new value if needed
/// - **resolve** (Pure): Merges check and create outputs, picks non-skipped value
///
/// The pattern ensures idempotent world-writes: if the value exists, skip creation.
#[derive(Debug, Clone)]
pub struct UpsertSpec<'a> {
    pub tool: &'a str,
    pub check_node: &'a str,
    pub create_node: &'a str,
    pub resolve_node: &'a str,
    pub decision_port: &'a str,
    pub check_value_port: &'a str,
    pub create_value_port: &'a str,
    pub resolve_check_port: &'a str,
    pub resolve_create_port: &'a str,
    pub resolve_output_port: &'a str,
}

/// Validates that a DAG matches the Upsert pattern structure.
///
/// Checks:
/// - All three nodes exist (check, create, resolve)
/// - Behavior classifications match (check=Observe, create=WritesWorld, resolve=Pure)
/// - Required ports exist on each node
/// - Create node has guard on decision port
/// - Edges connect the diamond correctly
/// - Pattern decision is declared as Instantiated
/// - Export node points to resolve
pub fn assert_upsert_shape<T>(dag: &Dag<T>, spec: &UpsertSpec<'_>) -> Result<(), String> {
    let check = find_node(dag, spec.check_node)?;
    let create = find_node(dag, spec.create_node)?;
    let resolve = find_node(dag, spec.resolve_node)?;

    // Behavior classification assertions (Level A - semantic claims)
    assert_behavior(check, BehaviorKind::Observe, "check node must be Observe")?;
    assert_behavior_writes_world(create, "create node must be WritesWorld")?;
    assert_behavior(resolve, BehaviorKind::Pure, "resolve node must be Pure")?;

    assert_has_output(check, spec.decision_port)?;
    assert_has_output(check, spec.check_value_port)?;

    let create_input = assert_has_input(create, spec.decision_port)?;
    if create_input.guard.is_none() {
        return Err(format!(
            "create node '{}' port '{}' is missing guard",
            spec.create_node, spec.decision_port
        ));
    }
    assert_has_output(create, spec.create_value_port)?;

    assert_has_input(resolve, spec.resolve_check_port)?;
    assert_has_input(resolve, spec.resolve_create_port)?;
    assert_has_output(resolve, spec.resolve_output_port)?;

    assert_edge(dag, spec.check_node, spec.decision_port, spec.create_node, spec.decision_port)?;
    assert_edge(dag, spec.check_node, spec.check_value_port, spec.resolve_node, spec.resolve_check_port)?;
    assert_edge(dag, spec.create_node, spec.create_value_port, spec.resolve_node, spec.resolve_create_port)?;

    let has_pattern = dag.metadata.pattern_decisions.iter().any(|pd| {
        pd.tool.0 == spec.tool
            && pd.pattern == "upsert"
            && matches!(pd.decision, PatternDecision::Instantiated)
    });
    if !has_pattern {
        return Err(format!(
            "pattern decision for tool '{}' and pattern 'upsert' is missing or not Instantiated",
            spec.tool
        ));
    }

    match &dag.metadata.export_node {
        Some(node_id) if node_id.0 == spec.resolve_node => {}
        Some(node_id) => {
            return Err(format!(
                "export_node is '{}', expected '{}'",
                node_id.0, spec.resolve_node
            ));
        }
        None => {
            return Err("export_node is missing".to_string());
        }
    }

    Ok(())
}

pub fn run_upsert_contract_tests<T>(
    dag: &Dag<T>,
    spec: &UpsertSpec<'_>,
    check_value: Value,
    create_value: Value,
) -> Result<(), String> {
    assert_upsert_shape(dag, spec)?;

    let resolve_check_port = spec.resolve_check_port.to_string();
    let resolve_create_port = spec.resolve_create_port.to_string();
    let resolve_output_port = spec.resolve_output_port.to_string();

    let exists_case = ScriptedDagBuilder::new(dag)
        .with_outputs(
            spec.check_node,
            outputs_map(vec![
                (spec.decision_port, Value::Bool(false)),
                (spec.check_value_port, check_value.clone()),
            ]),
        )
        .with_outputs(
            spec.create_node,
            outputs_map(vec![(spec.create_value_port, create_value.clone())]),
        )
        .with_func(
            spec.resolve_node,
            make_resolve_fn(
                resolve_check_port.clone(),
                resolve_create_port.clone(),
                resolve_output_port.clone(),
            ),
        );

    let exists_log = execute_scripted(exists_case).map_err(|e| e.to_string())?;
    assert_resolve_output(&exists_log, spec.resolve_node, spec.resolve_output_port, &check_value)?;
    assert_skipped_output(&exists_log, spec.create_node, spec.create_value_port)?;

    let missing_case = ScriptedDagBuilder::new(dag)
        .with_outputs(
            spec.check_node,
            outputs_map(vec![
                (spec.decision_port, Value::Bool(true)),
                (spec.check_value_port, check_value),
            ]),
        )
        .with_outputs(
            spec.create_node,
            outputs_map(vec![(spec.create_value_port, create_value.clone())]),
        )
        .with_func(
            spec.resolve_node,
            make_resolve_fn(resolve_check_port, resolve_create_port, resolve_output_port),
        );

    let missing_log = execute_scripted(missing_case).map_err(|e| e.to_string())?;
    assert_resolve_output(&missing_log, spec.resolve_node, spec.resolve_output_port, &create_value)?;
    assert_not_skipped_output(&missing_log, spec.create_node, spec.create_value_port, &create_value)?;

    Ok(())
}

fn find_node<'a, T>(dag: &'a Dag<T>, node_id: &str) -> Result<&'a Node<T>, String> {
    dag.nodes
        .iter()
        .find(|n| n.id.0 == node_id)
        .ok_or_else(|| format!("node '{}' not found", node_id))
}

fn assert_behavior<T>(node: &Node<T>, expected: BehaviorKind, msg: &str) -> Result<(), String> {
    if node.metadata.behavior == expected {
        Ok(())
    } else {
        Err(format!(
            "{}: node '{}' has {:?}, expected {:?}",
            msg, node.id.0, node.metadata.behavior, expected
        ))
    }
}

fn assert_behavior_writes_world<T>(node: &Node<T>, msg: &str) -> Result<(), String> {
    match &node.metadata.behavior {
        BehaviorKind::WritesWorld(_) => Ok(()),
        other => Err(format!(
            "{}: node '{}' has {:?}, expected WritesWorld",
            msg, node.id.0, other
        )),
    }
}

fn assert_has_output<T>(node: &Node<T>, port: &str) -> Result<(), String> {
    if node.outputs.iter().any(|p| p.name.0 == port) {
        Ok(())
    } else {
        Err(format!("node '{}' missing output port '{}'", node.id.0, port))
    }
}

fn assert_has_input<'a, T>(node: &'a Node<T>, port: &str) -> Result<&'a gunbc_ir::Port, String> {
    node.inputs
        .iter()
        .find(|p| p.name.0 == port)
        .ok_or_else(|| format!("node '{}' missing input port '{}'", node.id.0, port))
}

fn assert_edge<T>(
    dag: &Dag<T>,
    from_node: &str,
    from_port: &str,
    to_node: &str,
    to_port: &str,
) -> Result<(), String> {
    let ok = dag.edges.iter().any(|e| {
        e.from_node.0 == from_node
            && e.from_port.0 == from_port
            && e.to_node.0 == to_node
            && e.to_port.0 == to_port
    });
    if ok {
        Ok(())
    } else {
        Err(format!(
            "missing edge {}:{} -> {}:{}",
            from_node, from_port, to_node, to_port
        ))
    }
}

fn outputs_map(pairs: Vec<(&str, Value)>) -> Outputs {
    let mut out = HashMap::new();
    for (k, v) in pairs {
        out.insert(k.to_string(), v);
    }
    out
}

fn make_resolve_fn(
    resolve_check_port: String,
    resolve_create_port: String,
    resolve_output_port: String,
) -> impl Fn(HashMap<String, Value>) -> Result<Outputs, ExecError> + Send + Sync + 'static {
    move |inputs: HashMap<String, Value>| {
        let create_val = inputs.get(&resolve_create_port).cloned();
        let check_val = inputs.get(&resolve_check_port).cloned();

        let chosen = match create_val {
            Some(Value::Skipped) | None => check_val,
            Some(v) => Some(v),
        }
        .ok_or_else(|| ExecError("resolve node missing both inputs".to_string()))?;

        let mut outputs = HashMap::new();
        outputs.insert(resolve_output_port.clone(), chosen);
        Ok(outputs)
    }
}

fn assert_resolve_output(
    log: &ExecutionLog,
    node_id: &str,
    output_port: &str,
    expected: &Value,
) -> Result<(), String> {
    let entry = log
        .entries
        .iter()
        .find(|e| e.node_id == node_id)
        .ok_or_else(|| format!("log entry for '{}' not found", node_id))?;

    let actual = entry
        .outputs
        .get(output_port)
        .ok_or_else(|| format!("output '{}' missing on node '{}'", output_port, node_id))?;

    if value_eq(actual, expected) {
        Ok(())
    } else {
        Err(format!(
            "output mismatch on '{}:{}' (expected {}, got {})",
            node_id,
            output_port,
            value_label(expected),
            value_label(actual)
        ))
    }
}

fn assert_skipped_output(log: &ExecutionLog, node_id: &str, output_port: &str) -> Result<(), String> {
    let entry = log
        .entries
        .iter()
        .find(|e| e.node_id == node_id)
        .ok_or_else(|| format!("log entry for '{}' not found", node_id))?;

    let actual = entry
        .outputs
        .get(output_port)
        .ok_or_else(|| format!("output '{}' missing on node '{}'", output_port, node_id))?;

    match actual {
        Value::Skipped => Ok(()),
        _ => Err(format!(
            "expected '{}' output '{}' to be Skipped",
            node_id, output_port
        )),
    }
}

fn assert_not_skipped_output(
    log: &ExecutionLog,
    node_id: &str,
    output_port: &str,
    expected: &Value,
) -> Result<(), String> {
    let entry = log
        .entries
        .iter()
        .find(|e| e.node_id == node_id)
        .ok_or_else(|| format!("log entry for '{}' not found", node_id))?;

    let actual = entry
        .outputs
        .get(output_port)
        .ok_or_else(|| format!("output '{}' missing on node '{}'", output_port, node_id))?;

    if matches!(actual, Value::Skipped) {
        return Err(format!(
            "expected '{}' output '{}' to be non-skipped",
            node_id, output_port
        ));
    }

    if value_eq(actual, expected) {
        Ok(())
    } else {
        Err(format!(
            "output mismatch on '{}:{}' (expected {}, got {})",
            node_id,
            output_port,
            value_label(expected),
            value_label(actual)
        ))
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::StrList(x), Value::StrList(y)) => x == y,
        (Value::MapStrStr(x), Value::MapStrStr(y)) => x == y,
        (Value::Secret(x), Value::Secret(y)) => x.as_inner() == y.as_inner(),
        (Value::Skipped, Value::Skipped) => true,
        (Value::Unit, Value::Unit) => true,
        _ => false,
    }
}

fn value_label(v: &Value) -> String {
    match v {
        Value::Bool(b) => format!("Bool({})", b),
        Value::Str(s) => format!("Str({})", s),
        Value::StrList(vs) => format!("StrList({} items)", vs.len()),
        Value::MapStrStr(m) => format!("MapStrStr({} entries)", m.len()),
        Value::Secret(_) => "Secret(<redacted>)".to_string(),
        Value::Skipped => "Skipped".to_string(),
        Value::Unit => "Unit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::metadata::NodeMetadata;
    use gunbc_ir::types::{BehaviorKind, Idempotency, NodeId, PatternDecision, PortName, ToolId, TypeId};

    fn port(name: &str, ty: &str) -> gunbc_ir::Port {
        gunbc_ir::Port {
            name: PortName(name.into()),
            type_id: TypeId(ty.into()),
            guard: None,
        }
    }

    fn guarded_port(name: &str, ty: &str, guard: &str) -> gunbc_ir::Port {
        gunbc_ir::Port {
            name: PortName(name.into()),
            type_id: TypeId(ty.into()),
            guard: Some(guard.into()),
        }
    }

    fn edge(from: &str, from_port: &str, to: &str, to_port: &str) -> gunbc_ir::Edge {
        gunbc_ir::Edge {
            from_node: NodeId(from.into()),
            from_port: PortName(from_port.into()),
            to_node: NodeId(to.into()),
            to_port: PortName(to_port.into()),
        }
    }

    fn meta(tool: &str, behavior: BehaviorKind) -> NodeMetadata {
        NodeMetadata {
            tool: ToolId(tool.into()),
            behavior,
        }
    }

    fn build_auth_like_subdag() -> Dag<()> {
        let nodes = vec![
            Node {
                id: NodeId("auth_check".into()),
                inputs: vec![],
                outputs: vec![port("token", "Secret"), port("needs_create", "Bool")],
                metadata: meta("auth", BehaviorKind::Observe),
                body: NodeBody::Opaque(()),
            },
            Node {
                id: NodeId("auth_create".into()),
                inputs: vec![guarded_port("needs_create", "Bool", "needs_create == true")],
                outputs: vec![port("token", "Secret")],
                metadata: meta("auth", BehaviorKind::WritesWorld(Idempotency::Idempotent)),
                body: NodeBody::Opaque(()),
            },
            Node {
                id: NodeId("auth_resolve".into()),
                inputs: vec![port("check_token", "Secret"), port("create_token", "Secret")],
                outputs: vec![port("token", "Secret")],
                metadata: meta("auth", BehaviorKind::Pure),
                body: NodeBody::Opaque(()),
            },
        ];

        let edges = vec![
            edge("auth_check", "token", "auth_resolve", "check_token"),
            edge("auth_check", "needs_create", "auth_create", "needs_create"),
            edge("auth_create", "token", "auth_resolve", "create_token"),
        ];

        let metadata = gunbc_ir::DagMetadata {
            pattern_decisions: vec![gunbc_ir::PatternDecisionEntry {
                tool: ToolId("auth".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::Instantiated,
            }],
            export_node: Some(NodeId("auth_resolve".into())),
        };

        Dag { nodes, edges, metadata }
    }

    #[test]
    fn upsert_contract_helpers() {
        let dag = build_auth_like_subdag();
        let spec = UpsertSpec {
            tool: "auth",
            check_node: "auth_check",
            create_node: "auth_create",
            resolve_node: "auth_resolve",
            decision_port: "needs_create",
            check_value_port: "token",
            create_value_port: "token",
            resolve_check_port: "check_token",
            resolve_create_port: "create_token",
            resolve_output_port: "token",
        };

        assert_upsert_shape(&dag, &spec).unwrap();
        run_upsert_contract_tests(
            &dag,
            &spec,
            Value::Secret(gunbc_ir::types::Secret("existing".to_string())),
            Value::Secret(gunbc_ir::types::Secret("created".to_string())),
        )
        .unwrap();
    }
}
