use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use gunbc_exec::{ExecError, Executable, ExecutionLog, Value};
use gunbc_ir::{Dag, Node, NodeBody};

// =============================================================================
// SetSpec: Cardinality-based test generation
// =============================================================================

/// Cardinality variants for set-based testing.
///
/// All types can be viewed through set semantics:
/// - Non-nullable scalar (`String`, `Bool`): always cardinality 1 → `One` only
/// - Optional scalar (`Option<T>`): `Zero`, `One`, `Null` (Null = missing input)
/// - Collection (`StrList`, `MapStrStr`): cardinality 0..N → `Zero`, `One`, `N`
/// - Optional collection: `Zero`, `One`, `N`, `Null` (Null = missing input)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cardinality {
    /// Empty set (0 elements) - empty collection, absent optional
    Zero,
    /// Singleton set (1 element) - single element, present optional/scalar
    One,
    /// Multiple elements (N > 1)
    N,
    /// Null/missing input - truly missing/undefined
    Null,
}

impl Cardinality {
    pub fn all() -> &'static [Cardinality] {
        &[Cardinality::Zero, Cardinality::One, Cardinality::N, Cardinality::Null]
    }
}

impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cardinality::Zero => write!(f, "Zero"),
            Cardinality::One => write!(f, "One"),
            Cardinality::N => write!(f, "N"),
            Cardinality::Null => write!(f, "Null"),
        }
    }
}

/// Output contract for a cardinality case.
#[derive(Debug, Clone)]
pub enum SetSpecOutput {
    /// Operation succeeds with these outputs.
    Ok(HashMap<String, Value>),
    /// Operation fails with error containing this substring.
    Err(String),
}

impl SetSpecOutput {
    pub fn ok(outputs: impl IntoIterator<Item = (impl Into<String>, Value)>) -> Self {
        SetSpecOutput::Ok(outputs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    pub fn err(contains: impl Into<String>) -> Self {
        SetSpecOutput::Err(contains.into())
    }
}

/// A single test case: inputs and expected output.
#[derive(Debug, Clone)]
pub struct SetSpecCase {
    pub cardinality: Cardinality,
    pub inputs: HashMap<String, Value>,
    pub expected: SetSpecOutput,
}

/// Trait for types that declare their cardinality behavior.
///
/// Each implementor declares what happens for 0/1/N/null inputs.
/// When composed in a graph, the test framework generates all permutations.
pub trait SetSpec {
    /// Returns test cases for each cardinality.
    fn cases() -> Vec<SetSpecCase>;

    /// Optional: port name that carries the "set" (for automatic wiring).
    fn set_port() -> Option<&'static str> {
        None
    }
}

/// Generate all test permutations for composed SetSpec types.
pub fn generate_permutations<A: SetSpec, B: SetSpec>() -> Vec<(SetSpecCase, SetSpecCase)> {
    let a_cases = A::cases();
    let b_cases = B::cases();

    let mut perms = Vec::new();
    for a in &a_cases {
        for b in &b_cases {
            perms.push((a.clone(), b.clone()));
        }
    }
    perms
}

// =============================================================================
// ProducesSpec / AcceptsSpec: Composition-based bug detection
// =============================================================================

/// What a node PRODUCES for a given input cardinality.
#[derive(Debug, Clone)]
pub enum ProducesCase {
    /// Operation succeeds, producing output with this cardinality.
    Ok(Cardinality),
    /// Operation fails for this input cardinality.
    Err,
}

/// Trait for types that declare what cardinalities they can produce.
///
/// Used in composition checking to verify adjacent nodes are compatible.
pub trait ProducesSpec {
    /// Returns (input_cardinality, output_case) pairs.
    /// Describes what output cardinality results from each input cardinality.
    fn produces() -> Vec<(Cardinality, ProducesCase)>;

    /// Name of this spec for error messages.
    fn name() -> &'static str;
}

/// Trait for types that declare what cardinalities they accept or reject.
///
/// Used in composition checking to verify adjacent nodes are compatible.
pub trait AcceptsSpec {
    /// Cardinalities this node accepts (valid inputs).
    fn accepts() -> Vec<Cardinality>;

    /// Cardinalities this node explicitly rejects (should error).
    fn rejects() -> Vec<Cardinality>;

    /// Name of this spec for error messages.
    fn name() -> &'static str;
}

/// An integration bug detected during composition checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationBug {
    /// Name of the upstream node
    pub from: &'static str,
    /// Name of the downstream node
    pub to: &'static str,
    /// The cardinality that causes the bug
    pub cardinality: Cardinality,
    /// Description of the issue
    pub issue: IntegrationIssue,
}

/// Type of integration issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationIssue {
    /// A produces something B doesn't handle at all (neither accepts nor rejects).
    Unhandled,
    /// A produces something B explicitly rejects - this is a known edge case.
    /// Not necessarily a bug, but must be tested.
    KnownRejection,
}

impl fmt::Display for IntegrationBug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.issue {
            IntegrationIssue::Unhandled => {
                write!(
                    f,
                    "BUG: {} can produce {} but {} doesn't handle it",
                    self.from, self.cardinality, self.to
                )
            }
            IntegrationIssue::KnownRejection => {
                write!(
                    f,
                    "EDGE CASE: {} can produce {} but {} rejects it (test required)",
                    self.from, self.cardinality, self.to
                )
            }
        }
    }
}

/// Result of checking composition between two nodes.
#[derive(Debug, Clone)]
pub struct CompositionResult {
    /// Integration bugs found (Unhandled issues).
    pub bugs: Vec<IntegrationBug>,
    /// Known rejections that should be tested.
    pub edge_cases: Vec<IntegrationBug>,
}

impl CompositionResult {
    /// Returns true if no bugs were found.
    pub fn is_ok(&self) -> bool {
        self.bugs.is_empty()
    }

    /// Returns all issues (both bugs and edge cases).
    pub fn all_issues(&self) -> impl Iterator<Item = &IntegrationBug> {
        self.bugs.iter().chain(self.edge_cases.iter())
    }
}

/// Check composition between producer A and consumer B.
///
/// Returns bugs where A can produce cardinalities that B doesn't handle,
/// and edge cases where A produces cardinalities that B explicitly rejects.
pub fn check_composition<A: ProducesSpec, B: AcceptsSpec>() -> CompositionResult {
    let mut bugs = Vec::new();
    let mut edge_cases = Vec::new();

    let accepts = B::accepts();
    let rejects = B::rejects();

    for (_input_card, output_case) in A::produces() {
        let output_card = match output_case {
            ProducesCase::Ok(card) => card,
            ProducesCase::Err => continue, // A errors, so nothing flows to B
        };

        if rejects.contains(&output_card) {
            // A can produce something B rejects - this is a known edge case
            edge_cases.push(IntegrationBug {
                from: A::name(),
                to: B::name(),
                cardinality: output_card,
                issue: IntegrationIssue::KnownRejection,
            });
        } else if !accepts.contains(&output_card) {
            // A can produce something B doesn't handle at all - BUG!
            bugs.push(IntegrationBug {
                from: A::name(),
                to: B::name(),
                cardinality: output_card,
                issue: IntegrationIssue::Unhandled,
            });
        }
        // If accepts.contains(&output_card), composition is valid
    }

    CompositionResult { bugs, edge_cases }
}

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
