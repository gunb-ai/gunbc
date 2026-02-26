//! Cross-workflow mock corpus types for black-box node testing.
//!
//! Every node is a black box: it should be tested against inputs accumulated
//! from **all** workflows it appears in, not just one. The corpus types model
//! this cross-workflow accumulation.

use gunbc_ir::{NodeId, Value};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Node Identity
// ---------------------------------------------------------------------------

/// Identity of a reusable node across workflows.
///
/// Two nodes in different workflows that represent the same callable
/// (same module + callable name) share a `NodeIdentity`.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodeIdentity {
    pub module: String,
    pub callable: String,
}

impl fmt::Display for NodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.module, self.callable)
    }
}

impl NodeIdentity {
    pub fn new(module: impl Into<String>, callable: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            callable: callable.into(),
        }
    }

    /// Parse a node identity from a node ID string.
    ///
    /// Node IDs follow the convention `module::callable` or
    /// `module::callable::sub_id`. Returns `None` if the ID doesn't
    /// contain at least one `::` separator.
    pub fn from_node_id(node_id: &str) -> Option<Self> {
        let parts: Vec<&str> = node_id.splitn(3, "::").collect();
        if parts.len() >= 2 {
            Some(Self {
                module: parts[0].to_string(),
                callable: parts[1].to_string(),
            })
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

/// How an example was obtained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedKind {
    /// Observed during a workflow DryRun.
    WorkflowObserved,
    /// Provided via an explicit MockSpec.
    ExplicitMockSpec,
    /// Derived from the type contract (boundary witnesses).
    TypeDerived,
    /// Generated via property-based methods (future).
    PropertyBased,
}

/// Tracks where a corpus example came from.
#[derive(Clone, Debug)]
pub struct Provenance {
    /// Which workflow produced this example.
    pub workflow: String,
    /// Optional profile (e.g., "unit_test", "local").
    pub profile: Option<String>,
    /// The specific node instance in the DAG.
    pub node_instance: NodeId,
    /// Path through nested SubDags (empty for top-level nodes).
    pub subdag_path: Vec<NodeId>,
    /// How this example was seeded.
    pub seed_kind: SeedKind,
}

// ---------------------------------------------------------------------------
// Expectation
// ---------------------------------------------------------------------------

/// What a corpus example expects from node execution.
#[derive(Clone, Debug)]
pub enum Expectation {
    /// Node must produce these exact output values.
    ExactOutputs(HashMap<String, Value>),
    /// Node outputs must satisfy these matchers.
    OutputMatchers(HashMap<String, crate::OutputMatcher>),
    /// Only check that outputs have the correct types (no value assertion).
    TypeContractOnly,
    /// Expect the node to produce a validation error.
    ExpectValidationError,
}

// ---------------------------------------------------------------------------
// CorpusExample
// ---------------------------------------------------------------------------

/// A single test example for a node: inputs + expected behavior.
#[derive(Clone, Debug)]
pub struct CorpusExample {
    /// Where this example came from.
    pub provenance: Provenance,
    /// Input values keyed by port name.
    pub inputs: HashMap<String, Value>,
    /// What we expect from execution.
    pub expectation: Expectation,
}

impl CorpusExample {
    /// Compute a dedup key: (workflow, hash of sorted input entries).
    fn dedup_key(&self) -> (String, u64) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let mut keys: Vec<&String> = self.inputs.keys().collect();
        keys.sort();
        for k in keys {
            k.hash(&mut hasher);
            // Hash the debug representation as a stable proxy for Value
            format!("{:?}", self.inputs[k]).hash(&mut hasher);
        }
        (self.provenance.workflow.clone(), hasher.finish())
    }
}

// ---------------------------------------------------------------------------
// EdgeExample
// ---------------------------------------------------------------------------

/// A test example for an edge between two nodes: exercises real wiring.
#[derive(Clone, Debug)]
pub struct EdgeExample {
    /// Where this example came from.
    pub provenance: Provenance,
    /// Source node of the edge.
    pub from_node: NodeIdentity,
    /// Target node of the edge.
    pub to_node: NodeIdentity,
    /// Port mapping: from_port → to_port.
    pub edge_port_map: HashMap<String, String>,
    /// Inputs to feed the source node.
    pub a_inputs: HashMap<String, Value>,
    /// Additional inputs for the target node (ports not covered by the edge).
    pub b_other_inputs: HashMap<String, Value>,
}

// ---------------------------------------------------------------------------
// MockCorpus
// ---------------------------------------------------------------------------

/// Accumulated test examples for a single node identity across all workflows.
#[derive(Clone, Debug, Default)]
pub struct MockCorpus {
    pub examples: Vec<CorpusExample>,
}

impl MockCorpus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an example to this corpus.
    pub fn add(&mut self, example: CorpusExample) {
        self.examples.push(example);
    }

    /// Merge another corpus into this one (union).
    pub fn merge(&mut self, other: MockCorpus) {
        self.examples.extend(other.examples);
        self.dedup();
    }

    /// Remove duplicate examples (same workflow + same input hash).
    pub fn dedup(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.examples.retain(|ex| seen.insert(ex.dedup_key()));
    }

    /// Get all examples originating from a specific workflow.
    pub fn examples_for_workflow(&self, workflow: &str) -> Vec<&CorpusExample> {
        self.examples
            .iter()
            .filter(|ex| ex.provenance.workflow == workflow)
            .collect()
    }

    /// Number of examples.
    pub fn len(&self) -> usize {
        self.examples.len()
    }

    /// True if no examples.
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    /// Distinct workflow names represented in this corpus.
    pub fn workflow_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .examples
            .iter()
            .map(|ex| ex.provenance.workflow.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        names.sort();
        names
    }
}

// ---------------------------------------------------------------------------
// Normalization
// ---------------------------------------------------------------------------

/// Normalize a value for stable comparison across environments.
///
/// - Sorts map keys recursively (BTreeMap is already sorted, but nested
///   values within lists may contain maps)
/// - Replaces tempdir prefixes with `<TMP>`
/// - Replaces home directory with `<HOME>`
pub fn normalize_value(value: &Value) -> Value {
    let home = std::env::var("HOME").unwrap_or_default();
    let tmp = std::env::temp_dir()
        .to_string_lossy()
        .trim_end_matches('/')
        .to_string();
    normalize_value_inner(value, &home, &tmp)
}

fn normalize_value_inner(value: &Value, home: &str, tmp: &str) -> Value {
    match value {
        Value::Str(s) => {
            let mut normalized = s.clone();
            if !tmp.is_empty() {
                normalized = normalized.replace(tmp, "<TMP>");
            }
            if !home.is_empty() {
                normalized = normalized.replace(home, "<HOME>");
            }
            Value::Str(normalized)
        }
        Value::List(items) => Value::List(
            items
                .iter()
                .map(|v| normalize_value_inner(v, home, tmp))
                .collect(),
        ),
        Value::Map(map) => {
            let normalized: std::collections::BTreeMap<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), normalize_value_inner(v, home, tmp)))
                .collect();
            Value::Map(normalized)
        }
        // Other value types pass through unchanged
        other => other.clone(),
    }
}

/// Check whether a type ID represents a redacted/sensitive type.
///
/// Types like Secret, Credential, and various Handle types should not
/// have their values included in corpus examples or test assertions.
pub fn is_redacted_type(type_id: &str) -> bool {
    use gunbc_ir::semantic_carrier_kind_for_type_id;
    use gunbc_ir::SemanticCarrierKind;

    matches!(
        semantic_carrier_kind_for_type_id(type_id),
        SemanticCarrierKind::Secret
            | SemanticCarrierKind::Credential
            | SemanticCarrierKind::ToolHandle
            | SemanticCarrierKind::FilesystemHandle
            | SemanticCarrierKind::NetworkHandle
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provenance(workflow: &str) -> Provenance {
        Provenance {
            workflow: workflow.to_string(),
            profile: None,
            node_instance: NodeId("test::node".into()),
            subdag_path: vec![],
            seed_kind: SeedKind::WorkflowObserved,
        }
    }

    fn make_example(workflow: &str, input_val: &str) -> CorpusExample {
        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), Value::Str(input_val.to_string()));
        CorpusExample {
            provenance: make_provenance(workflow),
            inputs,
            expectation: Expectation::TypeContractOnly,
        }
    }

    #[test]
    fn node_identity_display() {
        let id = NodeIdentity::new("std.render", "format_heading");
        assert_eq!(id.to_string(), "std.render::format_heading");
    }

    #[test]
    fn node_identity_from_node_id() {
        let id = NodeIdentity::from_node_id("std.render::format_heading::sub1").unwrap();
        assert_eq!(id.module, "std.render");
        assert_eq!(id.callable, "format_heading");

        let id2 = NodeIdentity::from_node_id("std.render::format_heading").unwrap();
        assert_eq!(id2.module, "std.render");
        assert_eq!(id2.callable, "format_heading");

        assert!(NodeIdentity::from_node_id("no_separator").is_none());
    }

    #[test]
    fn corpus_dedup_removes_same_workflow_same_inputs() {
        let mut corpus = MockCorpus::new();
        corpus.add(make_example("wf1", "hello"));
        corpus.add(make_example("wf1", "hello")); // duplicate
        corpus.add(make_example("wf1", "world")); // different input
        corpus.dedup();
        assert_eq!(corpus.len(), 2);
    }

    #[test]
    fn corpus_dedup_keeps_different_workflows() {
        let mut corpus = MockCorpus::new();
        corpus.add(make_example("wf1", "hello"));
        corpus.add(make_example("wf2", "hello")); // same input, different workflow
        corpus.dedup();
        assert_eq!(corpus.len(), 2);
    }

    #[test]
    fn corpus_merge_unions_and_dedup() {
        let mut c1 = MockCorpus::new();
        c1.add(make_example("wf1", "a"));
        c1.add(make_example("wf1", "b"));

        let mut c2 = MockCorpus::new();
        c2.add(make_example("wf1", "a")); // duplicate of c1
        c2.add(make_example("wf2", "c"));

        c1.merge(c2);
        assert_eq!(c1.len(), 3); // a(wf1), b(wf1), c(wf2)
    }

    #[test]
    fn corpus_examples_for_workflow() {
        let mut corpus = MockCorpus::new();
        corpus.add(make_example("wf1", "a"));
        corpus.add(make_example("wf2", "b"));
        corpus.add(make_example("wf1", "c"));

        let wf1 = corpus.examples_for_workflow("wf1");
        assert_eq!(wf1.len(), 2);

        let wf2 = corpus.examples_for_workflow("wf2");
        assert_eq!(wf2.len(), 1);
    }

    #[test]
    fn corpus_workflow_names() {
        let mut corpus = MockCorpus::new();
        corpus.add(make_example("beta", "x"));
        corpus.add(make_example("alpha", "y"));
        corpus.add(make_example("beta", "z"));

        let names = corpus.workflow_names();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn normalize_value_replaces_paths() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/test".to_string());
        let input = Value::Str(format!("{}/project/file.rs", home));
        let normalized = normalize_value(&input);
        if let Value::Str(s) = normalized {
            assert!(
                s.contains("<HOME>"),
                "expected <HOME> placeholder, got: {}",
                s
            );
            assert!(!s.contains(&home));
        } else {
            panic!("expected Str");
        }
    }

    #[test]
    fn normalize_value_recurses_into_collections() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/test".to_string());
        let input = Value::List(vec![
            Value::Str(format!("{}/a", home)),
            Value::Map({
                let mut m = std::collections::BTreeMap::new();
                m.insert("path".to_string(), Value::Str(format!("{}/b", home)));
                m
            }),
        ]);
        let normalized = normalize_value(&input);
        let debug = format!("{:?}", normalized);
        assert!(!debug.contains(&home), "home dir should be replaced");
    }

    #[test]
    fn is_redacted_type_identifies_sensitive_types() {
        assert!(is_redacted_type("Secret"));
        assert!(is_redacted_type("Credential"));
        assert!(is_redacted_type("ToolHandle"));
        assert!(is_redacted_type("FilesystemHandle"));
        assert!(is_redacted_type("NetworkHandle"));
    }

    #[test]
    fn is_redacted_type_allows_structural_types() {
        assert!(!is_redacted_type("String"));
        assert!(!is_redacted_type("Int"));
        assert!(!is_redacted_type("Bool"));
        assert!(!is_redacted_type("List<String>"));
    }

    #[test]
    fn empty_corpus() {
        let corpus = MockCorpus::new();
        assert!(corpus.is_empty());
        assert_eq!(corpus.len(), 0);
        assert!(corpus.workflow_names().is_empty());
    }
}
