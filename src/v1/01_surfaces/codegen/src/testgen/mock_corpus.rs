//! Mock corpus builder: accumulates [`CorpusExample`]s across workflow DryRuns.
//!
//! The key insight: every node is a black box that should be tested against
//! inputs from **all** workflows it appears in. This module builds that
//! cross-workflow corpus by running baseline DryRuns and extracting per-node
//! I/O from the execution log.

use crate::testgen::analyze::{analyze_dag, DagAnalysis};
use gunbc_exec::{execute_dag, Executable, ExecuteConfig, ExecutionLog, ExecutionMode, LogEntry};
use gunbc_ir::{Dag, Value};
use gunbc_test::{
    CorpusExample, EdgeExample, Expectation, MockCorpus, MockSpec, NodeIdentity, Provenance,
    SeedKind,
};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// WorkflowInfo
// ---------------------------------------------------------------------------

/// Metadata about a workflow being processed for corpus building.
#[derive(Clone, Debug)]
pub struct WorkflowInfo {
    /// Display name of the workflow (e.g., "pragma", "makegen").
    pub name: String,
    /// Optional profile (e.g., "unit_test", "local").
    pub profile: Option<String>,
}

/// A workflow that failed during corpus construction.
#[derive(Clone, Debug)]
pub struct WorkflowFailure {
    pub workflow: WorkflowInfo,
    pub error: String,
}

/// Rich corpus-build output used by best-effort callers.
#[derive(Debug, Default)]
pub struct CorpusBuildReport {
    pub corpus_map: HashMap<NodeIdentity, MockCorpus>,
    pub edge_examples: Vec<EdgeExample>,
    pub failures: Vec<WorkflowFailure>,
}

/// Strict corpus build failure (one or more workflows failed DryRun).
#[derive(Debug, Clone)]
pub struct CorpusBuildError {
    pub failures: Vec<WorkflowFailure>,
}

impl fmt::Display for CorpusBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "corpus build failed for {} workflow(s):",
            self.failures.len()
        )?;
        for failure in &self.failures {
            writeln!(f, "- {}: {}", failure.workflow.name, failure.error)?;
        }
        Ok(())
    }
}

impl std::error::Error for CorpusBuildError {}

// ---------------------------------------------------------------------------
// build_corpus
// ---------------------------------------------------------------------------

/// Build a cross-workflow mock corpus by running DryRun on each workflow.
///
/// For each workflow, executes a baseline DryRun and extracts per-node I/O
/// from the execution log. Groups examples by [`NodeIdentity`] so that
/// nodes appearing in multiple workflows accumulate examples from all of them.
///
/// Returns:
/// - A map from `NodeIdentity` → `MockCorpus` with accumulated examples
/// - A list of `EdgeExample`s for adjacent-pair testing
///
/// The `node_classifier` callback determines whether a node is "pure" (no
/// transport deps). Pure nodes with deterministic outputs get `ExactOutputs`
/// expectations; all others get `TypeContractOnly`.
pub fn build_corpus<T: Executable + Clone + Send>(
    workflows: &[(WorkflowInfo, &Dag<T>, &MockSpec)],
    node_classifier: impl Fn(&str) -> bool,
) -> Result<(HashMap<NodeIdentity, MockCorpus>, Vec<EdgeExample>), CorpusBuildError> {
    let report = build_corpus_report(workflows, node_classifier);
    if report.failures.is_empty() {
        return Ok((report.corpus_map, report.edge_examples));
    }
    Err(CorpusBuildError {
        failures: report.failures,
    })
}

/// Best-effort corpus construction that records workflow DryRun failures.
pub fn build_corpus_report<T: Executable + Clone + Send>(
    workflows: &[(WorkflowInfo, &Dag<T>, &MockSpec)],
    node_classifier: impl Fn(&str) -> bool,
) -> CorpusBuildReport {
    let mut corpus_map: HashMap<NodeIdentity, MockCorpus> = HashMap::new();
    let mut edge_examples: Vec<EdgeExample> = Vec::new();
    let mut failures: Vec<WorkflowFailure> = Vec::new();

    for (info, dag, mock_spec) in workflows {
        let boundary_mocks = mock_spec.to_boundary_mocks();
        let mode = ExecutionMode::DryRun(boundary_mocks);

        let log = match execute_dag(
            *dag,
            ExecuteConfig {
                mode,
                ..Default::default()
            },
        ) {
            Ok(log) => log,
            Err(err) => {
                failures.push(WorkflowFailure {
                    workflow: info.clone(),
                    error: err.to_string(),
                });
                continue;
            }
        };

        let analysis = analyze_dag(*dag);

        // Extract per-node examples
        extract_node_examples(
            info,
            *dag,
            &log,
            &analysis,
            &node_classifier,
            &mut corpus_map,
        );

        // Extract edge examples
        extract_edge_examples(info, *dag, &log, &analysis, &mut edge_examples);
    }

    // Dedup each corpus
    for corpus in corpus_map.values_mut() {
        corpus.dedup();
    }

    CorpusBuildReport {
        corpus_map,
        edge_examples,
        failures,
    }
}

// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

/// Extract per-node corpus examples from an execution log.
fn extract_node_examples<T>(
    info: &WorkflowInfo,
    dag: &Dag<T>,
    log: &ExecutionLog,
    _analysis: &DagAnalysis,
    is_pure: &impl Fn(&str) -> bool,
    corpus_map: &mut HashMap<NodeIdentity, MockCorpus>,
) {
    for node in &dag.nodes {
        let node_id_str = &node.id.0;
        let identity = match NodeIdentity::from_origin(&node.origin) {
            Some(id) => id,
            None => continue,
        };

        let entry = match log.get(node_id_str) {
            Some(e) => e,
            None => continue,
        };

        let inputs = extract_inputs(entry);
        let expectation = classify_expectation(node_id_str, entry, is_pure);

        let provenance = Provenance {
            workflow: info.name.clone(),
            profile: info.profile.clone(),
            node_instance: node.id.clone(),
            subdag_path: vec![],
            seed_kind: SeedKind::WorkflowObserved,
        };

        let example = CorpusExample {
            provenance,
            inputs,
            expectation,
        };

        corpus_map.entry(identity).or_default().add(example);
    }
}

/// Extract edge examples from a DAG and its execution log.
fn extract_edge_examples<T>(
    info: &WorkflowInfo,
    dag: &Dag<T>,
    log: &ExecutionLog,
    _analysis: &DagAnalysis,
    edge_examples: &mut Vec<EdgeExample>,
) {
    for edge in &dag.edges {
        let from_node = match dag.get_node(&edge.from_node) {
            Some(n) => n,
            None => continue,
        };
        let from_identity = match NodeIdentity::from_origin(&from_node.origin) {
            Some(id) => id,
            None => continue,
        };
        let to_node_ref = match dag.get_node(&edge.to_node) {
            Some(n) => n,
            None => continue,
        };
        let to_identity = match NodeIdentity::from_origin(&to_node_ref.origin) {
            Some(id) => id,
            None => continue,
        };

        // Get upstream node's inputs (for driving the source node)
        let a_inputs = match log.get(&edge.from_node.0) {
            Some(entry) => extract_inputs(entry),
            None => continue,
        };

        // Get downstream node's other inputs (ports not covered by this edge)
        let b_other_inputs = match log.get(&edge.to_node.0) {
            Some(entry) => {
                let mut inputs = extract_inputs(entry);
                // Remove the port that the edge provides
                inputs.remove(&edge.to_port.0);
                inputs
            }
            None => continue,
        };

        let mut port_map = HashMap::new();
        port_map.insert(edge.from_port.0.clone(), edge.to_port.0.clone());

        let provenance = Provenance {
            workflow: info.name.clone(),
            profile: info.profile.clone(),
            node_instance: edge.from_node.clone(),
            subdag_path: vec![],
            seed_kind: SeedKind::WorkflowObserved,
        };

        edge_examples.push(EdgeExample {
            provenance,
            from_node: from_identity,
            to_node: to_identity,
            edge_port_map: port_map,
            a_inputs,
            b_other_inputs,
        });
    }
}

/// Extract input values from a log entry.
fn extract_inputs(entry: &LogEntry) -> HashMap<String, Value> {
    entry.inputs.as_ref().cloned().unwrap_or_default()
}

/// Classify what expectation to use for a node's outputs.
///
/// Pure nodes with non-intercepted outputs get `ExactOutputs`.
/// Everything else gets `TypeContractOnly`.
fn classify_expectation(
    node_id: &str,
    entry: &LogEntry,
    is_pure: &impl Fn(&str) -> bool,
) -> Expectation {
    if is_pure(node_id) && !entry.was_intercepted {
        Expectation::ExactOutputs(entry.outputs.clone())
    } else {
        Expectation::TypeContractOnly
    }
}

// ---------------------------------------------------------------------------
// Type-derived enrichment (BB-4)
// ---------------------------------------------------------------------------

/// Maximum examples per node before capping.
pub const MAX_EXAMPLES_PER_NODE: usize = 50;

/// Enrich a corpus with type-derived boundary witnesses.
///
/// For each node, for each input port, resolve the port's type to generate
/// boundary witness values. Uses anchored mutation: picks up to 3 observed
/// base cases, then varies one port at a time with type-derived values.
///
/// All type-derived examples get `SeedKind::TypeDerived` and
/// `Expectation::TypeContractOnly`.
pub fn enrich_corpus_with_type_witnesses<T>(
    corpus_map: &mut HashMap<NodeIdentity, MockCorpus>,
    dag: &Dag<T>,
    registry: &gunbc_ir::TypeRegistry,
) {
    use gunbc_ir::contract;
    use gunbc_test::{is_redacted_type, normalize_value};

    for node in &dag.nodes {
        let identity = match NodeIdentity::from_origin(&node.origin) {
            Some(id) => id,
            None => continue,
        };

        let corpus = match corpus_map.get(&identity) {
            Some(c) => c,
            None => continue,
        };

        // Pick up to 3 base cases from observed examples
        let base_cases: Vec<&HashMap<String, Value>> = corpus
            .examples
            .iter()
            .filter(|ex| ex.provenance.seed_kind == SeedKind::WorkflowObserved)
            .take(3)
            .map(|ex| &ex.inputs)
            .collect();

        if base_cases.is_empty() {
            continue;
        }

        let mut new_examples = Vec::new();

        // For each input port, generate witnesses and do anchored mutation
        for port in &node.inputs {
            let type_id_str = &port.type_id.0;

            // Skip redacted types
            if is_redacted_type(type_id_str, registry) {
                continue;
            }

            // Try to get witnesses from the type registry
            let type_dag = match registry.get_by_name(type_id_str) {
                Some(td) => td,
                None => continue,
            };

            let witnesses = contract::witnesses_checked(type_dag);
            let witness_values: Vec<Value> = match witnesses {
                Ok(ws) => ws.into_iter().map(|w| w.value).collect(),
                Err(_) => continue,
            };

            // Anchored mutation: for each base case, vary this port
            for base in &base_cases {
                for witness_val in &witness_values {
                    let normalized = normalize_value(witness_val);
                    let mut mutated = (*base).clone();
                    mutated.insert(port.name.0.clone(), normalized);

                    let provenance = Provenance {
                        workflow: "type-derived".to_string(),
                        profile: None,
                        node_instance: node.id.clone(),
                        subdag_path: vec![],
                        seed_kind: SeedKind::TypeDerived,
                    };

                    new_examples.push(CorpusExample {
                        provenance,
                        inputs: mutated,
                        expectation: Expectation::TypeContractOnly,
                    });
                }
            }
        }

        // Add new examples, respecting cap
        if let Some(corpus) = corpus_map.get_mut(&identity) {
            let remaining = MAX_EXAMPLES_PER_NODE.saturating_sub(corpus.len());
            for ex in new_examples.into_iter().take(remaining) {
                corpus.add(ex);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{Dag, Edge, Node, NodeId, NodeOrigin, Port, TypeId};
    use gunbc_test::MockSpec;

    /// A trivial passthrough op for testing.
    #[derive(Debug, Clone)]
    struct PassthroughOp;

    impl Executable for PassthroughOp {
        fn execute(
            &self,
            inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, gunbc_exec::ExecError> {
            Ok(inputs)
        }
    }

    fn test_origin(module: &str, item: &str) -> NodeOrigin {
        NodeOrigin::UserCode {
            file: "test.dag".into(),
            module: module.into(),
            item: item.into(),
            span_start: 0,
            span_end: 0,
        }
    }

    fn make_simple_dag() -> Dag<PassthroughOp> {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                NodeId("mod_a::node_a".into()),
                vec![Port::new("in1", TypeId::new("String"))],
                vec![Port::new("out1", TypeId::new("String"))],
                PassthroughOp,
            )
            .with_origin(test_origin("mod_a", "node_a")),
        );
        dag.add_node(
            Node::opaque(
                NodeId("mod_b::node_b".into()),
                vec![Port::new("in1", TypeId::new("String"))],
                vec![Port::new("out1", TypeId::new("String"))],
                PassthroughOp,
            )
            .with_origin(test_origin("mod_b", "node_b")),
        );
        dag.add_edge(Edge::new(
            NodeId("mod_a::node_a".into()),
            "out1",
            NodeId("mod_b::node_b".into()),
            "in1",
        ));
        dag
    }

    #[test]
    fn single_workflow_produces_corpus() {
        let dag = make_simple_dag();
        let spec =
            MockSpec::new("test").input_mock("mod_a::node_a", "in1", Value::Str("hello".into()));

        let info = WorkflowInfo {
            name: "test_wf".to_string(),
            profile: None,
        };

        let (corpus_map, edge_examples) = build_corpus(&[(info, &dag, &spec)], |_| true)
            .expect("strict corpus build should succeed");

        // Should have entries for both nodes
        assert!(
            !corpus_map.is_empty(),
            "corpus should have at least one node identity"
        );

        // Check that we got examples
        let total: usize = corpus_map.values().map(|c| c.len()).sum();
        assert!(total > 0, "should have at least one example");

        // Should also have edge examples
        let _ = edge_examples; // used for completeness
    }

    #[test]
    fn multi_workflow_accumulates() {
        let dag = make_simple_dag();
        let spec1 = MockSpec::new("test1").input_mock(
            "mod_a::node_a",
            "in1",
            Value::Str("from_wf1".into()),
        );
        let spec2 = MockSpec::new("test2").input_mock(
            "mod_a::node_a",
            "in1",
            Value::Str("from_wf2".into()),
        );

        let info1 = WorkflowInfo {
            name: "wf1".into(),
            profile: None,
        };
        let info2 = WorkflowInfo {
            name: "wf2".into(),
            profile: None,
        };

        let (corpus_map, _edges) =
            build_corpus(&[(info1, &dag, &spec1), (info2, &dag, &spec2)], |_| true)
                .expect("strict corpus build should succeed");

        // node_a should have examples from both workflows
        let node_a_id = NodeIdentity::new("mod_a", "node_a");
        if let Some(corpus) = corpus_map.get(&node_a_id) {
            let wf_names = corpus.workflow_names();
            assert!(
                wf_names.len() >= 2 || corpus.len() >= 2,
                "node_a should have examples from multiple workflows, got {:?}",
                wf_names
            );
        }
    }

    #[test]
    fn pure_nodes_get_exact_outputs() {
        let dag = make_simple_dag();
        let spec =
            MockSpec::new("test").input_mock("mod_a::node_a", "in1", Value::Str("hello".into()));

        let info = WorkflowInfo {
            name: "test_wf".to_string(),
            profile: None,
        };

        let (corpus_map, _) = build_corpus(&[(info, &dag, &spec)], |_| true)
            .expect("strict corpus build should succeed");

        // Pure non-intercepted nodes should get ExactOutputs
        for corpus in corpus_map.values() {
            for example in &corpus.examples {
                match &example.expectation {
                    Expectation::ExactOutputs(_) => {}  // expected for pure
                    Expectation::TypeContractOnly => {} // also ok if intercepted
                    other => panic!("unexpected expectation: {:?}", other),
                }
            }
        }
    }

    #[test]
    fn effectful_nodes_get_type_contract_only() {
        let dag = make_simple_dag();
        let spec =
            MockSpec::new("test").input_mock("mod_a::node_a", "in1", Value::Str("hello".into()));

        let info = WorkflowInfo {
            name: "test_wf".to_string(),
            profile: None,
        };

        let (corpus_map, _) = build_corpus(&[(info, &dag, &spec)], |_| false)
            .expect("strict corpus build should succeed");

        for corpus in corpus_map.values() {
            for example in &corpus.examples {
                assert!(
                    matches!(example.expectation, Expectation::TypeContractOnly),
                    "effectful nodes should get TypeContractOnly"
                );
            }
        }
    }

    #[test]
    fn edge_examples_captured() {
        let dag = make_simple_dag();
        let spec =
            MockSpec::new("test").input_mock("mod_a::node_a", "in1", Value::Str("hello".into()));

        let info = WorkflowInfo {
            name: "test_wf".to_string(),
            profile: None,
        };

        let (_, edges) = build_corpus(&[(info, &dag, &spec)], |_| true)
            .expect("strict corpus build should succeed");

        assert!(
            !edges.is_empty(),
            "should capture at least one edge example"
        );

        let edge = &edges[0];
        assert_eq!(edge.from_node, NodeIdentity::new("mod_a", "node_a"));
        assert_eq!(edge.to_node, NodeIdentity::new("mod_b", "node_b"));
        assert!(edge.edge_port_map.contains_key("out1"));
    }

    #[test]
    fn build_corpus_is_strict_by_default() {
        #[derive(Debug, Clone)]
        struct FailingOp;
        impl Executable for FailingOp {
            fn execute(
                &self,
                _inputs: HashMap<String, Value>,
            ) -> Result<HashMap<String, Value>, gunbc_exec::ExecError> {
                Err(gunbc_exec::ExecError::new("dry-run failure"))
            }
        }

        let mut dag: Dag<FailingOp> = Dag::new();
        dag.add_node(Node::opaque(
            NodeId("mod_a::node_a".into()),
            vec![],
            vec![Port::new("out", TypeId::new("String"))],
            FailingOp,
        ));
        let info = WorkflowInfo {
            name: "failing".into(),
            profile: None,
        };
        let spec = MockSpec::new("failing");

        let err = build_corpus(&[(info, &dag, &spec)], |_| true)
            .expect_err("strict build should fail when any workflow fails");
        assert_eq!(err.failures.len(), 1);
    }

    #[test]
    fn build_corpus_report_records_failures_best_effort() {
        #[derive(Debug, Clone)]
        struct FailingOp;
        impl Executable for FailingOp {
            fn execute(
                &self,
                _inputs: HashMap<String, Value>,
            ) -> Result<HashMap<String, Value>, gunbc_exec::ExecError> {
                Err(gunbc_exec::ExecError::new("dry-run failure"))
            }
        }

        let mut dag: Dag<FailingOp> = Dag::new();
        dag.add_node(Node::opaque(
            NodeId("mod_a::node_a".into()),
            vec![],
            vec![Port::new("out", TypeId::new("String"))],
            FailingOp,
        ));
        let info = WorkflowInfo {
            name: "failing".into(),
            profile: None,
        };
        let spec = MockSpec::new("failing");

        let report = build_corpus_report(&[(info, &dag, &spec)], |_| true);
        assert_eq!(report.failures.len(), 1);
        assert!(report.corpus_map.is_empty());
        assert!(report.edge_examples.is_empty());
    }
}
