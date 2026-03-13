//! Probe-observer integration test discovery.
//!
//! Discovers non-tautological integration tests by pairing probes (concrete
//! mock values at entry nodes) with observers (developer-specified OutputMatchers
//! at downstream nodes).
//!
//! # Formal Model
//!
//! - **Probe**: A node with concrete input values (from transport_mocks, boundary_mocks,
//!   or NodeExample inputs).
//! - **Observer**: A node with an OutputMatcher (from NodeExample outputs).
//! - **Integration test**: For each (probe, observer) pair where observer is reachable
//!   from probe in the DAG, extract the subgraph between them and generate a test that
//!   injects probe values, executes the chain, and asserts observer matchers.
//!
//! # Observability Invariant
//!
//! Every terminal node reachable from any probe **must** have an OutputMatcher.
//! Testgen emits an error for unobserved terminals.
//!
//! # Intermediate Observers
//!
//! An intermediate observer (non-terminal with OutputMatcher) serves dual roles:
//! 1. **Sink**: Validates the upstream chain from the nearest probe
//! 2. **New probe**: Its matched output becomes a fresh probe for downstream tests
//!
//! This gives compositional segment tests: if A->C passes and C->E passes,
//! A->E works by transitivity.

use crate::testgen::analyze::DagAnalysis;
use gunbc_ir::Dag;
use gunbc_test::{MockSpec, OutputMatcher};
use std::collections::{BTreeMap, HashMap, HashSet};

// ============================================================================
// Types
// ============================================================================

/// A test probe: a node with concrete values that can be injected.
#[derive(Debug, Clone)]
pub struct Probe {
    /// Node ID where values are injected.
    pub node_id: String,
    /// Source of the probe values.
    pub source: ProbeSource,
}

/// Where a probe's concrete values come from.
#[derive(Debug, Clone)]
pub enum ProbeSource {
    /// From a transport_mock in MockSpec (boundary/transport node intercepted in DryRun).
    TransportMock,
    /// From a boundary_mock in MockSpec.
    BoundaryMock,
    /// From a NodeExample's inputs (pure node with developer-specified inputs).
    NodeExampleInputs,
    /// From an input_mock in MockSpec (entry node with developer-specified input values).
    InputMock,
    /// From an intermediate observer that was promoted to a probe.
    /// The observer's OutputMatcher::Exact values become the new probe values.
    IntermediateObserver {
        /// The upstream probe that validated this observer.
        upstream_probe: String,
    },
}

/// A test observer: a node with developer-specified OutputMatchers.
#[derive(Debug, Clone)]
pub struct Observer {
    /// Node ID where outputs are checked.
    pub node_id: String,
    /// Port-level matchers.
    pub matchers: BTreeMap<String, MatcherDescription>,
    /// Whether this is a terminal node (no outgoing edges to non-boundary nodes).
    pub is_terminal: bool,
}

/// A description of an OutputMatcher that can be serialized for reporting.
#[derive(Debug, Clone)]
pub struct MatcherDescription {
    /// Human-readable description of the matcher.
    pub description: String,
    /// Whether this matcher provides a concrete value (Exact) that can be
    /// used as a downstream probe.
    pub has_concrete_value: bool,
    /// Whether this matcher is input-independent (valid regardless of what
    /// input the node receives). Exact matchers are input-dependent.
    pub is_input_independent: bool,
    /// Whether this is a "weak" matcher (Any, IsRequest, IsResponse) that
    /// asserts minimal properties. Weak matchers count as observers for
    /// chain discovery, but an all-weak terminal observer is flagged in the
    /// coverage report as needing stronger assertions.
    pub is_weak: bool,
}

/// A discovered integration test: probe -> observer through a subgraph.
#[derive(Debug, Clone)]
pub struct ProbeObserverTest {
    /// The probe (entry point with concrete values).
    pub probe: Probe,
    /// The observer (exit point with matchers).
    pub observer: Observer,
    /// All node IDs on paths from probe to observer (the subgraph to execute).
    pub subgraph_nodes: Vec<String>,
    /// Depth: number of nodes in the chain.
    pub depth: usize,
}

/// A coverage gap: a terminal node reachable from a probe but with no observer.
#[derive(Debug, Clone)]
pub struct CoverageGap {
    /// The probe that reaches this terminal.
    pub probe_node: String,
    /// The terminal node missing an observer.
    pub terminal_node: String,
}

/// Complete probe-observer analysis result.
#[derive(Debug, Clone)]
pub struct ProbeObserverAnalysis {
    /// All probes discovered from MockSpec.
    pub probes: Vec<Probe>,
    /// All observers discovered from MockSpec.
    pub observers: Vec<Observer>,
    /// Integration tests to generate.
    pub tests: Vec<ProbeObserverTest>,
    /// Coverage gaps (hard errors).
    pub gaps: Vec<CoverageGap>,
}

// ============================================================================
// Helpers
// ============================================================================

/// Check if an OutputMatcher is input-independent (valid for chain tests).
///
/// Input-independent matchers verify structural properties of the output
/// (type, non-emptiness, range) rather than exact values that depend on
/// specific inputs.
fn is_input_independent(matcher: &OutputMatcher) -> bool {
    match matcher {
        // These depend on specific input values — not safe for chains
        OutputMatcher::Exact(_) => false,
        OutputMatcher::Contains(_) => false,
        OutputMatcher::Satisfies { .. } => false,
        // These are structural invariants — safe for chains
        OutputMatcher::NonEmpty => true,
        OutputMatcher::IsBool => true,
        OutputMatcher::IsInt => true,
        OutputMatcher::IsNumeric => true,
        OutputMatcher::IsString => true,
        OutputMatcher::IsSecret => true,
        OutputMatcher::IsRequest => true,
        OutputMatcher::IsResponse => true,
        OutputMatcher::IntGe(_) => true,
        OutputMatcher::IntLe(_) => true,
        OutputMatcher::Any => true,
    }
}

/// Check if an OutputMatcher is a "weak" assertion (matches almost anything).
///
/// Weak matchers like `Any`, `IsRequest`, and `IsResponse` pass regardless
/// of the actual value, so they provide minimal observability. They're still
/// valid for chain discovery, but an all-weak terminal observer is flagged
/// in the coverage report.
fn is_weak_matcher(matcher: &OutputMatcher) -> bool {
    matches!(
        matcher,
        OutputMatcher::Any | OutputMatcher::IsRequest | OutputMatcher::IsResponse
    )
}

// ============================================================================
// Discovery
// ============================================================================

/// Extract probes from a MockSpec.
fn extract_probes(spec: &MockSpec, _analysis: &DagAnalysis) -> Vec<Probe> {
    let mut probes = Vec::new();
    let mut seen = HashSet::new();

    // Transport mocks → probes at boundary nodes
    for tm in &spec.transport_mocks {
        if seen.insert(tm.node.clone()) {
            probes.push(Probe {
                node_id: tm.node.clone(),
                source: ProbeSource::TransportMock,
            });
        }
    }

    // Boundary mocks → probes at boundary nodes
    for bm in &spec.boundary_mocks {
        if seen.insert(bm.node.clone()) {
            probes.push(Probe {
                node_id: bm.node.clone(),
                source: ProbeSource::BoundaryMock,
            });
        }
    }

    // NodeExamples with inputs → probes at pure nodes
    for ex in &spec.node_examples {
        if !ex.inputs.is_empty() && seen.insert(ex.node_id.clone()) {
            probes.push(Probe {
                node_id: ex.node_id.clone(),
                source: ProbeSource::NodeExampleInputs,
            });
        }
    }

    // Input mocks → probes at entry nodes (dangling input ports)
    for im in &spec.input_mocks {
        if seen.insert(im.node.clone()) {
            probes.push(Probe {
                node_id: im.node.clone(),
                source: ProbeSource::InputMock,
            });
        }
    }

    probes
}

/// Extract observers from a MockSpec.
fn extract_observers<T>(spec: &MockSpec, dag: &Dag<T>) -> Vec<Observer> {
    let terminal_nodes = find_terminal_nodes(dag);
    let mut observers = Vec::new();

    // Merge matchers from NodeExamples and LiveExpectedOutputs by node_id.
    // This avoids a bug where NodeExamples with only input-dependent matchers
    // (Exact/Contains/Satisfies) would mark a node as "seen" without creating
    // an observer, suppressing valid chain-safe matchers from live_expected_outputs.
    let mut matchers_by_node: BTreeMap<String, BTreeMap<String, MatcherDescription>> =
        BTreeMap::new();

    // Collect chain-safe matchers from NodeExamples.
    for ex in &spec.node_examples {
        for (port, matcher) in &ex.outputs {
            if is_input_independent(matcher) {
                matchers_by_node
                    .entry(ex.node_id.clone())
                    .or_default()
                    .insert(
                        port.clone(),
                        MatcherDescription {
                            description: format!("{:?}", matcher),
                            has_concrete_value: matches!(matcher, OutputMatcher::Exact(_)),
                            is_input_independent: true,
                            is_weak: is_weak_matcher(matcher),
                        },
                    );
            }
        }
    }

    // Merge chain-safe matchers from LiveExpectedOutputs (union of ports).
    for leo in &spec.live_expected_outputs {
        if is_input_independent(&leo.matcher) {
            matchers_by_node
                .entry(leo.node.clone())
                .or_default()
                .entry(leo.port.clone())
                .or_insert_with(|| MatcherDescription {
                    description: format!("{:?}", leo.matcher),
                    has_concrete_value: matches!(leo.matcher, OutputMatcher::Exact(_)),
                    is_input_independent: true,
                    is_weak: is_weak_matcher(&leo.matcher),
                });
        }
    }

    // Build observers from merged matchers.
    for (node_id, matchers) in matchers_by_node {
        if !matchers.is_empty() {
            observers.push(Observer {
                node_id: node_id.clone(),
                matchers,
                is_terminal: terminal_nodes.contains(&node_id),
            });
        }
    }

    observers
}

/// Find terminal nodes: nodes with no outgoing edges in the DAG.
fn find_terminal_nodes<T>(dag: &Dag<T>) -> HashSet<String> {
    let all_nodes: HashSet<String> = dag.nodes.iter().map(|n| n.id.0.clone()).collect();
    let has_outgoing: HashSet<String> = dag.edges.iter().map(|e| e.from_node.0.clone()).collect();
    all_nodes.difference(&has_outgoing).cloned().collect()
}

/// Compute the set of nodes reachable from a given node via forward edges.
fn reachable_from<T>(dag: &Dag<T>, start: &str) -> HashSet<String> {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &dag.edges {
        adjacency
            .entry(edge.from_node.0.as_str())
            .or_default()
            .push(edge.to_node.0.as_str());
    }

    let mut visited = HashSet::new();
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        if visited.insert(node.to_string()) {
            if let Some(neighbors) = adjacency.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        stack.push(next);
                    }
                }
            }
        }
    }

    // Remove the start node itself — we want downstream nodes only.
    visited.remove(start);
    visited
}

/// Find all nodes on any path from `start` to `end` in the DAG.
fn nodes_on_paths<T>(dag: &Dag<T>, start: &str, end: &str) -> Vec<String> {
    // Forward reachability from start.
    let forward = {
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &dag.edges {
            adjacency
                .entry(edge.from_node.0.as_str())
                .or_default()
                .push(edge.to_node.0.as_str());
        }
        let mut visited = HashSet::new();
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            if visited.insert(node.to_string()) {
                if let Some(neighbors) = adjacency.get(node) {
                    for &next in neighbors {
                        if !visited.contains(next) {
                            stack.push(next);
                        }
                    }
                }
            }
        }
        visited
    };

    // Backward reachability from end.
    let backward = {
        let mut rev_adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &dag.edges {
            rev_adjacency
                .entry(edge.to_node.0.as_str())
                .or_default()
                .push(edge.from_node.0.as_str());
        }
        let mut visited = HashSet::new();
        let mut stack = vec![end];
        while let Some(node) = stack.pop() {
            if visited.insert(node.to_string()) {
                if let Some(neighbors) = rev_adjacency.get(node) {
                    for &next in neighbors {
                        if !visited.contains(next) {
                            stack.push(next);
                        }
                    }
                }
            }
        }
        visited
    };

    // Intersection: nodes on some path from start to end.
    let mut result: Vec<String> = forward.intersection(&backward).cloned().collect();
    result.sort(); // deterministic order
    result
}

/// Find the nearest observers reachable from a probe.
///
/// "Nearest" means: for each path from the probe, find the first observer.
/// This implements the segmentation rule: intermediate observers split chains.
fn nearest_observers<T>(dag: &Dag<T>, probe_node: &str, observers: &[Observer]) -> Vec<String> {
    let observer_set: HashSet<&str> = observers.iter().map(|o| o.node_id.as_str()).collect();

    // BFS from probe, stopping at observers.
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &dag.edges {
        adjacency
            .entry(edge.from_node.0.as_str())
            .or_default()
            .push(edge.to_node.0.as_str());
    }

    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    let mut found_observers = Vec::new();

    // Seed with immediate neighbors (not the probe itself).
    if let Some(neighbors) = adjacency.get(probe_node) {
        for &next in neighbors {
            queue.push_back(next);
        }
    }

    while let Some(node) = queue.pop_front() {
        if !visited.insert(node.to_string()) {
            continue;
        }

        if observer_set.contains(node) {
            // Found a nearest observer — don't continue past it on this path.
            found_observers.push(node.to_string());
            // Don't add neighbors: this observer will become a new probe.
        } else {
            // Continue searching through non-observer nodes.
            if let Some(neighbors) = adjacency.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        queue.push_back(next);
                    }
                }
            }
        }
    }

    found_observers
}

/// Run the full probe-observer analysis.
pub fn analyze_probe_observers<T: Clone>(
    dag: &Dag<T>,
    spec: &MockSpec,
    analysis: &DagAnalysis,
) -> ProbeObserverAnalysis {
    let probes = extract_probes(spec, analysis);
    let observers = extract_observers(spec, dag);
    let terminal_nodes = find_terminal_nodes(dag);

    let mut tests = Vec::new();
    let mut gaps = Vec::new();

    // Build observer lookup.
    let observer_map: HashMap<&str, &Observer> =
        observers.iter().map(|o| (o.node_id.as_str(), o)).collect();

    // For each probe, find nearest observers and generate tests.
    // Then promote intermediate observers to probes and recurse.
    let mut processed_pairs: HashSet<(String, String)> = HashSet::new();
    let mut all_probes = probes.clone();
    let mut probe_queue: Vec<Probe> = probes.clone();

    while let Some(probe) = probe_queue.pop() {
        let reachable = reachable_from(dag, &probe.node_id);

        // Find nearest observers (BFS stopping at observer nodes).
        let nearest = nearest_observers(dag, &probe.node_id, &observers);

        for obs_node_id in &nearest {
            let pair_key = (probe.node_id.clone(), obs_node_id.clone());
            if processed_pairs.contains(&pair_key) {
                continue;
            }
            processed_pairs.insert(pair_key);

            if let Some(observer) = observer_map.get(obs_node_id.as_str()) {
                let subgraph = nodes_on_paths(dag, &probe.node_id, obs_node_id);
                let depth = subgraph.len();

                tests.push(ProbeObserverTest {
                    probe: probe.clone(),
                    observer: (*observer).clone(),
                    subgraph_nodes: subgraph,
                    depth,
                });

                // If this is an intermediate observer (not terminal), promote to probe.
                // All intermediate observers are promoted because test execution
                // seeds from a full baseline DryRun — concrete Exact values at the
                // observer are not needed to derive downstream window inputs.
                if !observer.is_terminal {
                    let promoted = Probe {
                        node_id: obs_node_id.clone(),
                        source: ProbeSource::IntermediateObserver {
                            upstream_probe: probe.node_id.clone(),
                        },
                    };
                    all_probes.push(promoted.clone());
                    probe_queue.push(promoted);
                }
            }
        }

        // Check for coverage gaps: terminal nodes reachable from this probe
        // that have no observer (directly or indirectly via intermediate observers).
        for terminal in &terminal_nodes {
            if reachable.contains(terminal) && !observer_map.contains_key(terminal.as_str()) {
                // Check if there's an observer on ANY path between probe and terminal.
                let path_nodes = nodes_on_paths(dag, &probe.node_id, terminal);
                let has_observer_on_path = path_nodes
                    .iter()
                    .any(|n| observer_map.contains_key(n.as_str()));

                if !has_observer_on_path {
                    gaps.push(CoverageGap {
                        probe_node: probe.node_id.clone(),
                        terminal_node: terminal.clone(),
                    });
                }
            }
        }
    }

    // Sort tests for deterministic output.
    tests.sort_by(|a, b| {
        a.probe
            .node_id
            .cmp(&b.probe.node_id)
            .then(a.observer.node_id.cmp(&b.observer.node_id))
    });

    // Deduplicate gaps.
    gaps.sort_by(|a, b| {
        a.probe_node
            .cmp(&b.probe_node)
            .then(a.terminal_node.cmp(&b.terminal_node))
    });
    gaps.dedup_by(|a, b| a.probe_node == b.probe_node && a.terminal_node == b.terminal_node);

    ProbeObserverAnalysis {
        probes: all_probes,
        observers,
        tests,
        gaps,
    }
}

/// Generate a human-readable observability report.
pub fn observability_report(analysis: &ProbeObserverAnalysis) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Probes: {}", analysis.probes.len()));
    for probe in &analysis.probes {
        let source = match &probe.source {
            ProbeSource::TransportMock => "transport mock",
            ProbeSource::BoundaryMock => "boundary mock",
            ProbeSource::NodeExampleInputs => "node example inputs",
            ProbeSource::InputMock => "input mock",
            ProbeSource::IntermediateObserver { upstream_probe } => {
                // Use a temporary string for this case
                &format!("intermediate (from {})", upstream_probe)
            }
        };
        lines.push(format!("  {} ({})", probe.node_id, source));
    }

    lines.push(String::new());
    lines.push(format!("Observers: {}", analysis.observers.len()));
    for obs in &analysis.observers {
        let terminal_tag = if obs.is_terminal { " [terminal]" } else { "" };
        let all_weak = obs.matchers.values().all(|m| m.is_weak);
        let weak_tag = if all_weak { " [WEAK]" } else { "" };
        let matcher_desc: Vec<String> = obs
            .matchers
            .iter()
            .map(|(port, m)| format!("{}: {}", port, m.description))
            .collect();
        lines.push(format!(
            "  {}{}{} ({})",
            obs.node_id,
            terminal_tag,
            weak_tag,
            matcher_desc.join(", ")
        ));
    }

    // Flag weak terminal observers as a coverage concern.
    let weak_terminals: Vec<&Observer> = analysis
        .observers
        .iter()
        .filter(|o| o.is_terminal && o.matchers.values().all(|m| m.is_weak))
        .collect();
    if !weak_terminals.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "Weak terminal observers: {} (all matchers are Any/IsRequest/IsResponse)",
            weak_terminals.len()
        ));
        for obs in &weak_terminals {
            lines.push(format!(
                "  {} — consider adding stronger assertions (NonEmpty, IsString, IntGe, etc.)",
                obs.node_id
            ));
        }
    }

    lines.push(String::new());
    lines.push(format!("Integration tests: {}", analysis.tests.len()));
    for test in &analysis.tests {
        lines.push(format!(
            "  {} -> {} (depth {})",
            test.probe.node_id, test.observer.node_id, test.depth
        ));
    }

    if !analysis.gaps.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "Coverage gaps (unobserved terminals): {}",
            analysis.gaps.len()
        ));
        for gap in &analysis.gaps {
            lines.push(format!(
                "  {} reachable from probe {} but has no OutputMatcher",
                gap.terminal_node, gap.probe_node
            ));
        }
    }

    lines.join("\n")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::node::Node;
    use gunbc_ir::{Dag, Edge};

    use gunbc_ir::Port;
    use gunbc_test::NodeExample;

    fn edge(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Edge {
        Edge::new(from_node, from_port, to_node, to_port)
    }

    /// Helper to build a simple linear DAG: A -> B -> C
    fn linear_dag() -> Dag<()> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![Port::new("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::new("in", "String")],
            vec![Port::new("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "c",
            vec![Port::new("in", "String")],
            vec![Port::new("out", "String")],
            (),
        ));
        dag.add_edge(edge("a", "out", "b", "in"));
        dag.add_edge(edge("b", "out", "c", "in"));
        dag
    }

    fn simple_analysis() -> DagAnalysis {
        DagAnalysis {
            boundaries: gunbc_ir::BoundaryInfo::default(),
            edge_types: vec![],
            port_cardinalities: vec![],
            node_count: 3,
            edge_count: 2,
            transport_executors: vec!["a".to_string()],
            tool_env_nodes: vec![],
            guarded_nodes: vec![],
            pure_nodes: vec!["b".to_string(), "c".to_string()],
            credential_nodes: vec![],
        }
    }

    #[test]
    fn test_extract_probes_from_transport_mocks() {
        let spec = MockSpec::new("test").transport_mock(
            "a",
            "response",
            gunbc_ir::Value::Str("hello".into()),
        );
        let analysis = simple_analysis();
        let probes = extract_probes(&spec, &analysis);
        assert_eq!(probes.len(), 1);
        assert_eq!(probes[0].node_id, "a");
    }

    #[test]
    fn test_extract_observers_from_node_examples() {
        let dag = linear_dag();
        let spec = MockSpec::new("test").node_example(
            NodeExample::new("c")
                .input("in", gunbc_ir::Value::Str("hello".into()))
                .output("out", OutputMatcher::non_empty()),
        );
        let observers = extract_observers(&spec, &dag);
        assert_eq!(observers.len(), 1);
        assert_eq!(observers[0].node_id, "c");
        assert!(observers[0].is_terminal);
    }

    #[test]
    fn test_terminal_nodes() {
        let dag = linear_dag();
        let terminals = find_terminal_nodes(&dag);
        assert!(terminals.contains("c"));
        assert!(!terminals.contains("a"));
        assert!(!terminals.contains("b"));
    }

    #[test]
    fn test_reachable_from() {
        let dag = linear_dag();
        let reachable = reachable_from(&dag, "a");
        assert!(reachable.contains("b"));
        assert!(reachable.contains("c"));
        assert!(!reachable.contains("a"));
    }

    #[test]
    fn test_probe_observer_pairing() {
        let dag = linear_dag();
        let analysis = simple_analysis();
        let spec = MockSpec::new("test")
            .transport_mock("a", "response", gunbc_ir::Value::Str("hello".into()))
            .node_example(
                NodeExample::new("c")
                    .input("in", gunbc_ir::Value::Str("hello".into()))
                    .output("out", OutputMatcher::non_empty()),
            );

        let result = analyze_probe_observers(&dag, &spec, &analysis);
        assert_eq!(result.probes.len(), 2); // a (transport) + c (example inputs)
        assert_eq!(result.observers.len(), 1); // c
        assert!(!result.tests.is_empty());

        // Should have a test from probe a to observer c
        let test = result
            .tests
            .iter()
            .find(|t| t.probe.node_id == "a" && t.observer.node_id == "c")
            .expect("should find a->c test");
        assert!(test.subgraph_nodes.contains(&"a".to_string()));
        assert!(test.subgraph_nodes.contains(&"b".to_string()));
        assert!(test.subgraph_nodes.contains(&"c".to_string()));
    }

    #[test]
    fn test_coverage_gap_detection() {
        let dag = linear_dag();
        let analysis = simple_analysis();
        // Probe at a, but NO observer at terminal c
        let spec = MockSpec::new("test").transport_mock(
            "a",
            "response",
            gunbc_ir::Value::Str("hello".into()),
        );

        let result = analyze_probe_observers(&dag, &spec, &analysis);
        assert!(!result.gaps.is_empty());
        assert!(result
            .gaps
            .iter()
            .any(|g| g.probe_node == "a" && g.terminal_node == "c"));
    }

    #[test]
    fn test_intermediate_observer_splits_chain() {
        let dag = linear_dag();
        let analysis = simple_analysis();

        // Probe at a, intermediate observer at b (with IsString — input-independent),
        // terminal observer at c (with NonEmpty — input-independent).
        let spec = MockSpec::new("test")
            .transport_mock("a", "response", gunbc_ir::Value::Str("hello".into()))
            .node_example(
                NodeExample::new("b")
                    .input("in", gunbc_ir::Value::Str("hello".into()))
                    .output("out", OutputMatcher::IsString),
            )
            .node_example(
                NodeExample::new("c")
                    .input("in", gunbc_ir::Value::Str("world".into()))
                    .output("out", OutputMatcher::non_empty()),
            );

        let result = analyze_probe_observers(&dag, &spec, &analysis);

        // b is an intermediate observer that splits the chain:
        //   a->b (probe a reaches observer b)
        //   b->c (promoted observer b reaches terminal c)
        // a->c is NOT generated because BFS from a stops at observer b.
        let a_to_b = result
            .tests
            .iter()
            .find(|t| t.probe.node_id == "a" && t.observer.node_id == "b");
        let a_to_c = result
            .tests
            .iter()
            .find(|t| t.probe.node_id == "a" && t.observer.node_id == "c");

        assert!(a_to_b.is_some(), "should have a->b test");
        assert!(
            a_to_c.is_none(),
            "a->c should be segmented: a hits observer b first"
        );

        // b is promoted to a probe (intermediate observer), so b->c exists.
        // b is also a probe from NodeExample inputs, so there are two sources.
        let b_to_c = result
            .tests
            .iter()
            .find(|t| t.probe.node_id == "b" && t.observer.node_id == "c");
        assert!(
            b_to_c.is_some(),
            "should have b->c test (b promoted + example inputs)"
        );

        // Verify the promoted probe appears in the analysis.
        let promoted = result.probes.iter().any(|p| {
            p.node_id == "b" && matches!(p.source, ProbeSource::IntermediateObserver { .. })
        });
        assert!(
            promoted,
            "b should be promoted to probe from intermediate observer"
        );

        assert!(result.gaps.is_empty(), "no coverage gaps");
    }

    #[test]
    fn test_live_expected_outputs_not_suppressed_by_exact_node_example() {
        // Regression test: a NodeExample with only Exact outputs (input-dependent)
        // should not prevent live_expected_outputs for the same node from becoming
        // observers.
        let dag = linear_dag();

        let spec = MockSpec::new("test")
            .transport_mock("a", "response", gunbc_ir::Value::Str("hello".into()))
            // Node c has a NodeExample with only Exact (input-dependent) output...
            .node_example(
                NodeExample::new("c")
                    .input("in", gunbc_ir::Value::Str("hello".into()))
                    .output(
                        "out",
                        OutputMatcher::Exact(Box::new(gunbc_ir::Value::Str("world".into()))),
                    ),
            )
            // ...and a live_expected_output with chain-safe NonEmpty matcher.
            .live_expected_output("c", "out", OutputMatcher::non_empty());

        let observers = extract_observers(&spec, &dag);

        // The live_expected_output's NonEmpty matcher should create an observer
        // at c, even though the NodeExample had only Exact matchers.
        assert_eq!(observers.len(), 1, "should have observer at c");
        assert_eq!(observers[0].node_id, "c");
        assert!(
            observers[0].matchers.contains_key("out"),
            "observer should have 'out' matcher"
        );
    }

    #[test]
    fn test_intermediate_observer_promoted_without_exact() {
        // Intermediate observers are promoted to probes even without Exact matchers.
        // Tests are seeded from baseline DryRun, so concrete values aren't needed.
        let dag = linear_dag();
        let analysis = simple_analysis();

        // Probe at a, intermediate observer at b with only NonEmpty (no Exact).
        let spec = MockSpec::new("test")
            .transport_mock("a", "response", gunbc_ir::Value::Str("hello".into()))
            .node_example(NodeExample::new("b").output("out", OutputMatcher::non_empty()))
            .node_example(NodeExample::new("c").output("out", OutputMatcher::non_empty()));

        let result = analyze_probe_observers(&dag, &spec, &analysis);

        // b should be promoted as an intermediate observer → probe.
        let promoted = result.probes.iter().any(|p| {
            p.node_id == "b" && matches!(p.source, ProbeSource::IntermediateObserver { .. })
        });
        assert!(promoted, "b should be promoted even without Exact matchers");

        // Should have: a->b, b->c (b is promoted probe)
        let a_to_b = result
            .tests
            .iter()
            .any(|t| t.probe.node_id == "a" && t.observer.node_id == "b");
        let b_to_c = result
            .tests
            .iter()
            .any(|t| t.probe.node_id == "b" && t.observer.node_id == "c");
        assert!(a_to_b, "should have a->b test");
        assert!(b_to_c, "should have b->c test from promoted probe");
    }
}
