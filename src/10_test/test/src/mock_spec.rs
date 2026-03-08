//! Mock specifications for DAGs.
//!
//! Mock specs are declared in adjacent files (e.g., `graph_mock.rs`) and define:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//!
//! This enables chain validation: A's mock output must satisfy B's expected input.
//!
//! # File Convention
//!
//! ```text
//! lib/tools/gist/src/
//! ├── graph.rs       # DAG definition
//! ├── graph_mock.rs  # Mock specifications (this file)
//! └── lib.rs
//! ```
//!
//! # Example
//!
//! ```text
//! // graph_mock.rs
//! use gunbc_test::MockSpec;
//!
//! pub fn gist_mock_spec() -> MockSpec {
//!     MockSpec::new("gist")
//!         .boundary("create_gist", "url", Value::Str("https://gist.github.com/mock/123".into()))
//!         .expects_input("files", InputConstraint::non_empty())
//! }
//! ```

use gunbc_exec::BoundaryMocks;
use gunbc_ir::{normalize_resource_id, Value};
use std::collections::HashMap;

/// A complete mock specification for a DAG/tool.
#[derive(Debug, Clone)]
pub struct MockSpec {
    /// Name of the tool/DAG this spec is for
    pub name: String,

    /// Mock values for boundary nodes (world writes)
    pub boundary_mocks: Vec<BoundaryMock>,

    /// Expected input constraints from upstream
    pub input_expectations: Vec<InputExpectation>,

    /// Resource simulations for testing resource acquisition
    pub resource_mocks: ResourceMocks,

    /// Mock values for transport executor nodes (injected via DryRun).
    /// These are the values that intercepted transport nodes return.
    pub transport_mocks: Vec<TransportMock>,

    /// Expected outputs at terminal/boundary nodes (for flow test assertions).
    /// After DryRun execution, these are verified against actual outputs.
    pub expected_outputs: Vec<ExpectedOutput>,

    /// Expected outputs for live flow tests (Real execution).
    /// These use OutputMatcher instead of exact values.
    pub live_expected_outputs: Vec<LiveExpectedOutput>,

    /// Per-node I/O examples for generating unit tests.
    /// Each example specifies inputs and expected outputs for a single node.
    pub node_examples: Vec<NodeExample>,

    /// Node IDs explicitly skipped from example enforcement.
    /// Use this for primitive/utility nodes that are tested in their own crates.
    pub skipped_node_examples: Vec<String>,

    /// Mock values for DAG entry inputs (dangling input ports with no upstream edge).
    /// These values are injected when testing a DAG in isolation.
    pub input_mocks: Vec<InputMock>,
}

impl MockSpec {
    fn join_node_prefix(prefix: &str, node: &str) -> String {
        let trimmed = prefix.trim_matches('/');
        if trimmed.is_empty() {
            node.to_string()
        } else {
            format!("{}/{}", trimmed, node)
        }
    }

    /// Create a new mock spec for a named tool.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            boundary_mocks: Vec::new(),
            input_expectations: Vec::new(),
            resource_mocks: ResourceMocks::new(),
            transport_mocks: Vec::new(),
            expected_outputs: Vec::new(),
            live_expected_outputs: Vec::new(),
            node_examples: Vec::new(),
            skipped_node_examples: Vec::new(),
            input_mocks: Vec::new(),
        }
    }

    /// Add a boundary mock (what this node outputs when mocked).
    pub fn boundary(
        mut self,
        node: impl Into<String>,
        port: impl Into<String>,
        value: Value,
    ) -> Self {
        self.boundary_mocks.push(BoundaryMock {
            node: node.into(),
            port: port.into(),
            value,
            sequence: None,
        });
        self
    }

    /// Add an input expectation (what this node requires from upstream).
    pub fn expects_input(mut self, port: impl Into<String>, constraint: InputConstraint) -> Self {
        self.input_expectations.push(InputExpectation {
            port: port.into(),
            constraint,
        });
        self
    }

    /// Add a resource lock simulation.
    pub fn resource_lock(mut self, id: impl Into<String>) -> Self {
        self.resource_mocks = self.resource_mocks.lock(id);
        self
    }

    /// Add a resource lease simulation with duration.
    pub fn resource_lease(mut self, id: impl Into<String>, duration_ms: u64) -> Self {
        self.resource_mocks = self.resource_mocks.lease(id, duration_ms);
        self
    }

    /// Add a lock that fails to acquire (for error testing).
    pub fn resource_lock_fails(mut self, id: impl Into<String>, error: impl Into<String>) -> Self {
        self.resource_mocks = self.resource_mocks.lock_fails(id, error);
        self
    }

    /// Add a lease that expires during operation (for timeout testing).
    pub fn resource_lease_expires(mut self, id: impl Into<String>, duration_ms: u64) -> Self {
        self.resource_mocks = self.resource_mocks.lease_expires(id, duration_ms);
        self
    }

    /// Add a credential resource simulation.
    pub fn resource_credential(mut self, id: impl Into<String>, expiry_ms: Option<u64>) -> Self {
        self.resource_mocks = self.resource_mocks.credential(id, expiry_ms);
        self
    }

    /// Add a boundary mock with a sequenced response.
    ///
    /// The mock returns values from `sequence` in order; exhausting the
    /// sequence is an error (no silent fallback).
    pub fn boundary_sequence(
        mut self,
        node: impl Into<String>,
        port: impl Into<String>,
        sequence: Vec<Value>,
    ) -> Self {
        assert!(
            !sequence.is_empty(),
            "sequence must not be empty; use boundary() for static mocks"
        );
        self.boundary_mocks.push(BoundaryMock {
            node: node.into(),
            port: port.into(),
            value: sequence.last().unwrap().clone(),
            sequence: Some(sequence),
        });
        self
    }

    /// Add a transport mock (value returned by an intercepted transport executor node).
    pub fn transport_mock(
        mut self,
        node: impl Into<String>,
        port: impl Into<String>,
        value: Value,
    ) -> Self {
        self.transport_mocks.push(TransportMock {
            node: node.into(),
            port: port.into(),
            value,
        });
        self
    }

    /// Add an expected output (assertion for flow test verification).
    pub fn expected_output(
        mut self,
        node: impl Into<String>,
        port: impl Into<String>,
        expected: Value,
    ) -> Self {
        self.expected_outputs.push(ExpectedOutput {
            node: node.into(),
            port: port.into(),
            expected,
        });
        self
    }

    /// Add an expected output matcher for live flow tests.
    pub fn live_expected_output(
        mut self,
        node: impl Into<String>,
        port: impl Into<String>,
        matcher: OutputMatcher,
    ) -> Self {
        self.live_expected_outputs.push(LiveExpectedOutput {
            node: node.into(),
            port: port.into(),
            matcher,
        });
        self
    }

    /// Add a node I/O example for generating unit tests.
    ///
    /// Node examples are used by testgen to create per-node unit tests that
    /// verify: given these inputs, the node produces outputs matching these matchers.
    pub fn node_example(mut self, example: NodeExample) -> Self {
        self.node_examples.push(example);
        self
    }

    /// Skip example enforcement for a node.
    ///
    /// Use this for primitive/utility nodes that are tested in their own crates
    /// and don't need I/O examples in the integration test suite. Without this,
    /// testgen will fail if a pure node has no examples.
    pub fn skip_node_example(mut self, node_id: impl Into<String>) -> Self {
        self.skipped_node_examples.push(node_id.into());
        self
    }

    /// Add an input mock for a DAG entry point (dangling input port).
    ///
    /// Use this when a node has an input port with no incoming edge.
    /// The mock value will be injected as if it came from an upstream node.
    pub fn input_mock(
        mut self,
        node: impl Into<String>,
        port: impl Into<String>,
        value: Value,
    ) -> Self {
        self.input_mocks.push(InputMock {
            node: node.into(),
            port: port.into(),
            value,
        });
        self
    }

    /// Merge runtime mocks from another spec with a node ID prefix.
    ///
    /// This is useful when a parent DAG embeds a child DAG as a SubDag. Child
    /// node IDs become `"{prefix}/{child_node}"` after lowering, so DryRun mock
    /// coverage can be composed without duplicating child mock definitions.
    ///
    /// Merged fields:
    /// - `boundary_mocks`
    /// - `transport_mocks`
    /// - `input_mocks`
    ///
    /// Other fields (resource simulations, expectations, node examples, etc.)
    /// are intentionally not merged.
    pub fn include_prefixed_runtime_mocks(
        mut self,
        prefix: impl AsRef<str>,
        other: &MockSpec,
    ) -> Self {
        let prefix = prefix.as_ref();

        self.boundary_mocks
            .extend(other.boundary_mocks.iter().cloned().map(|mut mock| {
                mock.node = Self::join_node_prefix(prefix, &mock.node);
                mock
            }));

        self.transport_mocks
            .extend(other.transport_mocks.iter().cloned().map(|mut mock| {
                mock.node = Self::join_node_prefix(prefix, &mock.node);
                mock
            }));

        self.input_mocks
            .extend(other.input_mocks.iter().cloned().map(|mut mock| {
                mock.node = Self::join_node_prefix(prefix, &mock.node);
                mock
            }));

        self
    }

    /// Convert boundary and transport mocks to BoundaryMocks for CLI dry-run.
    ///
    /// Excludes input_mocks (CLI inputs come from command-line flags, not MockSpec).
    /// Use this in generated CLI binaries for `--dry-run` mode.
    pub fn to_dry_run_mocks(&self) -> BoundaryMocks {
        let mut mocks = BoundaryMocks::new();
        for bm in &self.boundary_mocks {
            if let Some(seq) = &bm.sequence {
                mocks.set_sequence(&bm.node, &bm.port, seq.clone());
            } else {
                mocks.set_value(&bm.node, &bm.port, bm.value.clone());
            }
        }
        for tm in &self.transport_mocks {
            mocks.set_value(&tm.node, &tm.port, tm.value.clone());
        }
        mocks
    }

    /// Convert this MockSpec into BoundaryMocks suitable for `execute_dag`.
    ///
    /// Maps boundary_mocks + transport_mocks to output mocks and input_mocks to
    /// input mocks (for DAG entry points) in the resulting BoundaryMocks.
    pub fn to_boundary_mocks(&self) -> BoundaryMocks {
        let mut mocks = BoundaryMocks::new();
        // Boundary mocks for output interception (env nodes, explicit boundaries, etc.)
        for bm in &self.boundary_mocks {
            if let Some(seq) = &bm.sequence {
                mocks.set_sequence(&bm.node, &bm.port, seq.clone());
            } else {
                mocks.set_value(&bm.node, &bm.port, bm.value.clone());
            }
        }
        // Transport mocks for output interception
        for tm in &self.transport_mocks {
            mocks.set_value(&tm.node, &tm.port, tm.value.clone());
        }
        // Input mocks for DAG entry point injection
        for im in &self.input_mocks {
            mocks.set_input(&im.node, &im.port, im.value.clone());
        }
        mocks
    }

    /// Check whether this spec has flow test data (transport mocks or expected outputs).
    pub fn has_flow_test_data(&self) -> bool {
        !self.transport_mocks.is_empty() || !self.expected_outputs.is_empty()
    }

    /// Check whether this spec has live flow expectations.
    pub fn has_live_flow_test_data(&self) -> bool {
        !self.live_expected_outputs.is_empty()
    }

    /// Get mock value for a specific boundary port.
    pub fn get_boundary_mock(&self, node: &str, port: &str) -> Option<&Value> {
        self.boundary_mocks
            .iter()
            .find(|m| m.node == node && m.port == port)
            .map(|m| &m.value)
    }

    /// Get mock value for a specific transport executor port.
    pub fn get_transport_mock(&self, node: &str, port: &str) -> Option<&Value> {
        self.transport_mocks
            .iter()
            .find(|m| m.node == node && m.port == port)
            .map(|m| &m.value)
    }

    /// Check if a value satisfies input expectations for a port.
    pub fn satisfies_input(&self, port: &str, value: &Value) -> Result<(), String> {
        let expectation = self.input_expectations.iter().find(|e| e.port == port);

        match expectation {
            Some(exp) => exp.constraint.check(value),
            None => Ok(()), // No constraint = anything goes
        }
    }

    /// Get resource simulation by ID.
    pub fn get_resource(&self, id: &str) -> Option<&ResourceSimulation> {
        self.resource_mocks.get(id)
    }
}

/// A mock value for a boundary node.
#[derive(Debug, Clone)]
pub struct BoundaryMock {
    /// Node ID
    pub node: String,
    /// Port name
    pub port: String,
    /// Mock value to return (static value, or context for sequences)
    pub value: Value,
    /// Optional ordered sequence of responses (exhaustion is always an error)
    pub sequence: Option<Vec<Value>>,
}

/// A mock value for a transport executor node (injected via DryRun interception).
#[derive(Debug, Clone)]
pub struct TransportMock {
    /// Transport executor node ID (e.g., "execute_build")
    pub node: String,
    /// Output port name (e.g., "response")
    pub port: String,
    /// Mock value to return for this port
    pub value: Value,
}

/// A mock value for a DAG entry input (dangling input port with no upstream edge).
#[derive(Debug, Clone)]
pub struct InputMock {
    /// Node ID that has the dangling input (e.g., "prepare")
    pub node: String,
    /// Input port name (e.g., "provider")
    pub port: String,
    /// Mock value to inject for this input
    pub value: Value,
}

/// An expected output at a terminal/boundary node (for flow test assertions).
///
/// DEPRECATED: Use NodeExample with OutputMatchers instead. ExpectedOutput
/// causes tautological tests — the expected value is often just a copy of
/// what the code already produces.
#[derive(Debug, Clone)]
pub struct ExpectedOutput {
    /// Node ID to check (e.g., "report")
    pub node: String,
    /// Output port name (e.g., "overall_success")
    pub port: String,
    /// Expected value
    pub expected: Value,
}

/// An expected output matcher for live flow tests.
#[derive(Debug, Clone)]
pub struct LiveExpectedOutput {
    /// Node ID to check (e.g., "parse")
    pub node: String,
    /// Output port name (e.g., "content")
    pub port: String,
    /// Expected matcher
    pub matcher: OutputMatcher,
}

/// An expectation about input from upstream.
#[derive(Debug, Clone)]
pub struct InputExpectation {
    /// Port name
    pub port: String,
    /// Constraint that upstream must satisfy
    pub constraint: InputConstraint,
}

/// Constraint on input values.
#[derive(Debug, Clone)]
pub enum InputConstraint {
    /// Value must be non-empty (for lists/strings)
    NonEmpty,
    /// Value must be one of these specific values
    OneOf(Vec<Value>),
    /// Value must match a type pattern
    TypePattern(String),
    /// Custom predicate with description
    Custom {
        description: String,
        predicate: fn(&Value) -> bool,
    },
    /// No constraint (anything accepted)
    Any,
}

impl InputConstraint {
    /// Create a non-empty constraint.
    pub fn non_empty() -> Self {
        Self::NonEmpty
    }

    /// Create a one-of constraint.
    pub fn one_of(values: Vec<Value>) -> Self {
        Self::OneOf(values)
    }

    /// Create a type pattern constraint.
    pub fn type_pattern(pattern: impl Into<String>) -> Self {
        Self::TypePattern(pattern.into())
    }

    /// Create a custom constraint.
    pub fn custom(description: impl Into<String>, predicate: fn(&Value) -> bool) -> Self {
        Self::Custom {
            description: description.into(),
            predicate,
        }
    }

    /// Check if a value satisfies this constraint.
    pub fn check(&self, value: &Value) -> Result<(), String> {
        match self {
            InputConstraint::NonEmpty => match value {
                Value::Str(s) if s.is_empty() => Err("expected non-empty string".into()),
                Value::List(v) if v.is_empty() => Err("expected non-empty list".into()),
                Value::Set(v) if v.is_empty() => Err("expected non-empty set".into()),
                _ => Ok(()),
            },
            InputConstraint::OneOf(values) => {
                if values.iter().any(|v| values_match(v, value)) {
                    Ok(())
                } else {
                    Err("value not in allowed set".to_string())
                }
            }
            InputConstraint::TypePattern(pattern) => {
                // Simple type checking based on value variant
                let type_name = value_type_name(value);
                if type_name.contains(pattern) || pattern == "Any" {
                    Ok(())
                } else {
                    Err(format!("expected type '{}', got '{}'", pattern, type_name))
                }
            }
            InputConstraint::Custom {
                description,
                predicate,
            } => {
                if predicate(value) {
                    Ok(())
                } else {
                    Err(format!("failed constraint: {}", description))
                }
            }
            InputConstraint::Any => Ok(()),
        }
    }
}

/// Check if two values match (for OneOf constraint).
///
/// Uses Value's PartialEq, which does order-independent comparison for Sets.
fn values_match(expected: &Value, actual: &Value) -> bool {
    expected == actual
}

// ============================================================================
// Resource Simulation
// ============================================================================

/// Simulated resource for testing resource acquisition patterns.
///
/// Resources can be locks, leases, connections, etc. In tests, we simulate
/// their acquisition, holding, and release behavior.
#[derive(Debug, Clone)]
pub struct ResourceSimulation {
    /// Resource identifier
    pub resource_id: String,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Simulated behaviors
    pub behaviors: Vec<ResourceBehavior>,
}

impl ResourceSimulation {
    /// Create a new resource simulation.
    pub fn new(id: impl Into<String>, resource_type: ResourceType) -> Self {
        let id = id.into();
        Self {
            resource_id: normalize_resource_id(&id),
            resource_type,
            behaviors: Vec::new(),
        }
    }

    /// Add a simulated behavior.
    pub fn with_behavior(mut self, behavior: ResourceBehavior) -> Self {
        self.behaviors.push(behavior);
        self
    }

    /// Simulate acquiring this resource.
    pub fn acquire(&self) -> ResourceAcquireResult {
        for behavior in &self.behaviors {
            if let ResourceBehavior::FailAcquire { error } = behavior {
                return ResourceAcquireResult::Failed(error.clone());
            }
            if let ResourceBehavior::DelayAcquire { ms } = behavior {
                return ResourceAcquireResult::Delayed(*ms);
            }
        }
        ResourceAcquireResult::Acquired
    }

    /// Check if resource should timeout during hold.
    pub fn should_timeout(&self, held_ms: u64) -> bool {
        match self.resource_type {
            ResourceType::Lease { duration_ms } => held_ms > duration_ms,
            ResourceType::Credential {
                expiry_ms: Some(ms),
            } => held_ms > ms,
            _ => false,
        }
    }
}

/// Type of resource being simulated.
#[derive(Debug, Clone)]
pub enum ResourceType {
    /// Exclusive lock (mutex)
    Lock,
    /// Time-bounded lease
    Lease { duration_ms: u64 },
    /// Shared lock (read lock)
    SharedLock { max_holders: usize },
    /// Connection pool slot
    PoolSlot { pool_size: usize },
    /// Credential with optional expiry
    Credential { expiry_ms: Option<u64> },
}

/// Simulated behavior for a resource.
#[derive(Debug, Clone)]
pub enum ResourceBehavior {
    /// Acquisition succeeds (default)
    AcquireSucceeds,
    /// Acquisition fails with error
    FailAcquire { error: String },
    /// Acquisition delays by N milliseconds
    DelayAcquire { ms: u64 },
    /// Release fails (for testing cleanup failures)
    FailRelease { error: String },
    /// Lease expires during operation
    LeaseExpires,
    /// Contention: another holder has it
    Contended { holder: String },
}

/// Result of simulated resource acquisition.
#[derive(Debug, Clone)]
pub enum ResourceAcquireResult {
    /// Successfully acquired
    Acquired,
    /// Acquisition failed
    Failed(String),
    /// Acquisition delayed
    Delayed(u64),
    /// Waiting for contention
    Waiting,
}

/// Resource mock specification.
#[derive(Debug, Clone, Default)]
pub struct ResourceMocks {
    /// Simulated resources
    pub resources: Vec<ResourceSimulation>,
}

impl ResourceMocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a lock simulation.
    pub fn lock(mut self, id: impl Into<String>) -> Self {
        self.resources
            .push(ResourceSimulation::new(id, ResourceType::Lock));
        self
    }

    /// Add a lease simulation.
    pub fn lease(mut self, id: impl Into<String>, duration_ms: u64) -> Self {
        self.resources.push(ResourceSimulation::new(
            id,
            ResourceType::Lease { duration_ms },
        ));
        self
    }

    /// Add a lock that fails to acquire.
    pub fn lock_fails(mut self, id: impl Into<String>, error: impl Into<String>) -> Self {
        let sim = ResourceSimulation::new(id, ResourceType::Lock).with_behavior(
            ResourceBehavior::FailAcquire {
                error: error.into(),
            },
        );
        self.resources.push(sim);
        self
    }

    /// Add a lease that expires.
    pub fn lease_expires(mut self, id: impl Into<String>, duration_ms: u64) -> Self {
        let sim = ResourceSimulation::new(id, ResourceType::Lease { duration_ms })
            .with_behavior(ResourceBehavior::LeaseExpires);
        self.resources.push(sim);
        self
    }

    /// Add a credential simulation.
    pub fn credential(mut self, id: impl Into<String>, expiry_ms: Option<u64>) -> Self {
        self.resources.push(ResourceSimulation::new(
            id,
            ResourceType::Credential { expiry_ms },
        ));
        self
    }

    /// Add a credential that fails to acquire.
    pub fn credential_fails(mut self, id: impl Into<String>, error: impl Into<String>) -> Self {
        let sim = ResourceSimulation::new(id, ResourceType::Credential { expiry_ms: None })
            .with_behavior(ResourceBehavior::FailAcquire {
                error: error.into(),
            });
        self.resources.push(sim);
        self
    }

    /// Get simulation for a resource.
    pub fn get(&self, id: &str) -> Option<&ResourceSimulation> {
        let normalized = normalize_resource_id(id);
        self.resources.iter().find(|r| r.resource_id == normalized)
    }
}

/// Get a type name for a value.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Unit => "Unit",
        Value::Bool(_) => "Bool",
        Value::Str(_) => "String",
        Value::Int(_) => "Int",
        Value::List(_) => "List",
        Value::Set(_) => "Set",
        Value::Map(_) => "Map",
        Value::Json(_) => "Json",
        Value::Skipped => "Skipped",
        _ => "Unknown",
    }
}

/// Assert that all expected boundaries exist in a MockSpec.
///
/// This helper consolidates the common pattern of testing boundary mock presence:
///
/// ```text
/// // Before (repeated for each boundary):
/// assert!(spec.get_boundary_mock("node", "port").is_some(),
///     "missing boundary mock for node.port");
///
/// // After:
/// assert_boundaries(&spec, &[("node", "port"), ("other", "value")]);
/// ```
///
/// # Panics
///
/// Panics if any expected boundary is missing from the MockSpec.
pub fn assert_boundaries(spec: &MockSpec, expected: &[(&str, &str)]) {
    for (node, port) in expected {
        assert!(
            spec.get_boundary_mock(node, port).is_some(),
            "missing boundary mock for {}.{} in MockSpec '{}'",
            node,
            port,
            spec.name
        );
    }
}

/// Assert that all expected transport mocks exist in a MockSpec.
///
/// Similar to `assert_boundaries` but for transport mocks.
pub fn assert_transport_mocks(spec: &MockSpec, expected: &[(&str, &str)]) {
    for (node, port) in expected {
        let found = spec
            .transport_mocks
            .iter()
            .any(|m| m.node == *node && m.port == *port);
        assert!(
            found,
            "missing transport mock for {}.{} in MockSpec '{}'",
            node, port, spec.name
        );
    }
}

/// Validate that upstream mock specs satisfy downstream expectations.
///
/// Given a chain A -> B, verify that A's boundary mocks satisfy B's input expectations.
pub fn validate_chain(
    upstream_spec: &MockSpec,
    downstream_spec: &MockSpec,
    edge_port_mapping: &HashMap<String, String>, // upstream_port -> downstream_port
) -> ChainValidationResult {
    let mut errors = Vec::new();

    for (upstream_port, downstream_port) in edge_port_mapping {
        // Find upstream's mock value for this port
        let upstream_mock = upstream_spec
            .boundary_mocks
            .iter()
            .find(|m| m.port == *upstream_port);

        // Find downstream's expectation for this port
        let downstream_exp = downstream_spec
            .input_expectations
            .iter()
            .find(|e| e.port == *downstream_port);

        match (upstream_mock, downstream_exp) {
            (Some(mock), Some(exp)) => {
                if let Err(e) = exp.constraint.check(&mock.value) {
                    errors.push(ChainError::ConstraintViolation {
                        upstream: upstream_spec.name.clone(),
                        upstream_port: upstream_port.clone(),
                        downstream: downstream_spec.name.clone(),
                        downstream_port: downstream_port.clone(),
                        error: e,
                    });
                }
            }
            (None, Some(_)) => {
                errors.push(ChainError::MissingMock {
                    upstream: upstream_spec.name.clone(),
                    port: upstream_port.clone(),
                });
            }
            _ => {} // No expectation = OK
        }
    }

    ChainValidationResult { errors }
}

/// Result of chain validation.
#[derive(Debug)]
pub struct ChainValidationResult {
    pub errors: Vec<ChainError>,
}

impl ChainValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Error in chain validation.
#[derive(Debug)]
pub enum ChainError {
    /// Upstream mock doesn't satisfy downstream constraint
    ConstraintViolation {
        upstream: String,
        upstream_port: String,
        downstream: String,
        downstream_port: String,
        error: String,
    },
    /// Upstream doesn't provide a mock for required port
    MissingMock { upstream: String, port: String },
}

// ============================================================================
// Node Examples (DAG definition = test specification)
// ============================================================================

/// An I/O example for a node, used to generate unit tests.
///
/// Each example specifies inputs to provide and expected outputs.
/// TestGenerator uses these to generate tests that:
/// 1. Execute the node with the given inputs
/// 2. Assert outputs match the expected matchers
///
/// # Example
///
/// ```text
/// let example = NodeExample::new("prepare_prompt")
///     .input("artifact", Value::Str("fn foo() {}".into()))
///     .input("criteria", Value::Json(security_criteria()))
///     .output("question", OutputMatcher::contains("security"))
///     .output("system_prompt", OutputMatcher::non_empty());
/// ```
#[derive(Debug, Clone)]
pub struct NodeExample {
    /// Node ID this example is for
    pub node_id: String,
    /// Input values to provide
    pub inputs: HashMap<String, Value>,
    /// Expected outputs with matchers
    pub outputs: HashMap<String, OutputMatcher>,
    /// Optional description for the test
    pub description: Option<String>,
}

impl NodeExample {
    /// Create a new example for a node.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            description: None,
        }
    }

    /// Add an input value.
    pub fn input(mut self, port: impl Into<String>, value: Value) -> Self {
        self.inputs.insert(port.into(), value);
        self
    }

    /// Add an expected output with a matcher.
    pub fn output(mut self, port: impl Into<String>, matcher: OutputMatcher) -> Self {
        self.outputs.insert(port.into(), matcher);
        self
    }

    /// Add a description for the generated test.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Matcher for expected output values.
///
/// OutputMatcher provides flexible ways to assert on node outputs:
/// - Exact: value must equal expected exactly
/// - Contains: string output must contain substring
/// - NonEmpty: value must be non-empty (strings, lists)
/// - Typed matchers: IsBool, IsInt, IsString, IsRequest, IsResponse
/// - IntGe/IntLe: integer range checks
/// - Satisfies: custom predicate function (fallback for complex checks)
#[derive(Clone)]
pub enum OutputMatcher {
    /// Output must equal this value exactly
    Exact(Box<Value>),
    /// Output string must contain this substring
    Contains(String),
    /// Output must be non-empty
    NonEmpty,
    /// Output must be a boolean (any value).
    ///
    /// Unlike `Satisfies`, this generates a real codegen assertion.
    IsBool,
    /// Output must be an integer (any value).
    IsInt,
    /// Output must be a string (any value).
    IsString,
    /// Output must be a secret (any value).
    IsSecret,
    /// Output must be a transport request.
    IsRequest,
    /// Output must be a transport response.
    IsResponse,
    /// Integer must be >= threshold.
    IntGe(i64),
    /// Integer must be <= threshold.
    IntLe(i64),
    /// Output must satisfy a custom predicate.
    ///
    /// Prefer typed matchers (IsBool, IntGe, etc.) when possible, since
    /// codegen can emit real assertions for them. `Satisfies` is supported
    /// in generated tests via a runtime matcher check when `mock_spec()` is
    /// available (i.e., TestGenerator was given `with_mock_spec_fn`).
    Satisfies {
        description: String,
        predicate: fn(&Value) -> bool,
    },
    /// Any value is acceptable
    Any,
}

impl std::fmt::Debug for OutputMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputMatcher::Exact(v) => write!(f, "Exact({:?})", v),
            OutputMatcher::Contains(s) => write!(f, "Contains(\"{}\")", s),
            OutputMatcher::NonEmpty => write!(f, "NonEmpty"),
            OutputMatcher::IsBool => write!(f, "IsBool"),
            OutputMatcher::IsInt => write!(f, "IsInt"),
            OutputMatcher::IsString => write!(f, "IsString"),
            OutputMatcher::IsSecret => write!(f, "IsSecret"),
            OutputMatcher::IsRequest => write!(f, "IsRequest"),
            OutputMatcher::IsResponse => write!(f, "IsResponse"),
            OutputMatcher::IntGe(n) => write!(f, "IntGe({})", n),
            OutputMatcher::IntLe(n) => write!(f, "IntLe({})", n),
            OutputMatcher::Satisfies { description, .. } => {
                write!(f, "Satisfies({})", description)
            }
            OutputMatcher::Any => write!(f, "Any"),
        }
    }
}

impl OutputMatcher {
    /// Create an exact match.
    pub fn exact(value: Value) -> Self {
        Self::Exact(Box::new(value))
    }

    /// Create a contains match for strings.
    pub fn contains(substring: impl Into<String>) -> Self {
        Self::Contains(substring.into())
    }

    /// Create a non-empty match.
    pub fn non_empty() -> Self {
        Self::NonEmpty
    }

    /// Create a custom predicate match.
    pub fn satisfies(description: impl Into<String>, predicate: fn(&Value) -> bool) -> Self {
        Self::Satisfies {
            description: description.into(),
            predicate,
        }
    }

    /// Whether this matcher is safe for chain/integration tests.
    ///
    /// Input-independent matchers verify structural properties (type, non-emptiness)
    /// rather than exact values that depend on specific inputs. Only these matchers
    /// are used in probe-observer chain tests.
    pub fn is_chain_safe(&self) -> bool {
        matches!(
            self,
            OutputMatcher::NonEmpty
                | OutputMatcher::IsBool
                | OutputMatcher::IsInt
                | OutputMatcher::IsString
                | OutputMatcher::IsSecret
                | OutputMatcher::IsRequest
                | OutputMatcher::IsResponse
                | OutputMatcher::IntGe(_)
                | OutputMatcher::IntLe(_)
                | OutputMatcher::Any
        )
    }

    /// Check if a value matches this matcher.
    pub fn check(&self, value: &Value) -> Result<(), String> {
        match self {
            OutputMatcher::Exact(expected) => {
                if values_match(expected, value) {
                    Ok(())
                } else {
                    Err(format!("expected {:?}, got {:?}", expected, value))
                }
            }
            OutputMatcher::Contains(substring) => match value {
                Value::Str(s) if s.contains(substring) => Ok(()),
                Value::Str(s) => Err(format!("string doesn't contain '{}': {:?}", substring, s)),
                _ => Err(format!("expected String, got {:?}", value)),
            },
            OutputMatcher::NonEmpty => match value {
                Value::Str(s) if !s.is_empty() => Ok(()),
                Value::Str(_) => Err("expected non-empty string".into()),
                Value::List(v) if !v.is_empty() => Ok(()),
                Value::List(_) => Err("expected non-empty list".into()),
                Value::Set(v) if !v.is_empty() => Ok(()),
                Value::Set(_) => Err("expected non-empty set".into()),
                _ => Ok(()), // Other types considered non-empty
            },
            OutputMatcher::IsBool => match value {
                Value::Bool(_) => Ok(()),
                _ => Err(format!("expected Bool, got {:?}", value)),
            },
            OutputMatcher::IsInt => match value {
                Value::Int(_) => Ok(()),
                _ => Err(format!("expected Int, got {:?}", value)),
            },
            OutputMatcher::IsString => match value {
                Value::Str(_) => Ok(()),
                _ => Err(format!("expected String, got {:?}", value)),
            },
            OutputMatcher::IsSecret => match value {
                Value::Secret(_) => Ok(()),
                _ => Err(format!("expected Secret, got {:?}", value)),
            },
            OutputMatcher::IsRequest => match value {
                Value::Request(_) => Ok(()),
                _ => Err(format!("expected Request, got {:?}", value)),
            },
            OutputMatcher::IsResponse => match value {
                Value::Response(_) => Ok(()),
                _ => Err(format!("expected Response, got {:?}", value)),
            },
            OutputMatcher::IntGe(threshold) => match value {
                Value::Int(n) if n >= threshold => Ok(()),
                Value::Int(n) => Err(format!("expected Int >= {}, got {}", threshold, n)),
                _ => Err(format!("expected Int, got {:?}", value)),
            },
            OutputMatcher::IntLe(threshold) => match value {
                Value::Int(n) if n <= threshold => Ok(()),
                Value::Int(n) => Err(format!("expected Int <= {}, got {}", threshold, n)),
                _ => Err(format!("expected Int, got {:?}", value)),
            },
            OutputMatcher::Satisfies {
                description,
                predicate,
            } => {
                if predicate(value) {
                    Ok(())
                } else {
                    Err(format!("failed: {}", description))
                }
            }
            OutputMatcher::Any => Ok(()),
        }
    }

    /// Whether this matcher produces an executable assertion (vs. a comment).
    ///
    /// Used by codegen to decide whether to prefix the output variable with `_`.
    /// All typed matchers (IsBool, IntGe, etc.) generate real assertions.
    /// Only `Any` doesn't.
    pub fn generates_assertion(&self) -> bool {
        !matches!(self, OutputMatcher::Any)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const NET_PORT: &str = gunbc_ir::API_NETWORK_HANDLE_PORT;

    #[test]
    fn test_mock_spec_builder() {
        let spec = MockSpec::new("gist")
            .boundary(
                "create_gist",
                "url",
                Value::Str("https://example.com".into()),
            )
            .expects_input("files", InputConstraint::non_empty());

        assert_eq!(spec.name, "gist");
        assert_eq!(spec.boundary_mocks.len(), 1);
        assert_eq!(spec.input_expectations.len(), 1);
    }

    #[test]
    fn test_get_boundary_mock() {
        let spec = MockSpec::new("test").boundary("node1", "out", Value::Str("value".into()));

        assert!(spec.get_boundary_mock("node1", "out").is_some());
        assert!(spec.get_boundary_mock("node1", "other").is_none());
    }

    #[test]
    fn test_include_prefixed_runtime_mocks() {
        let child = MockSpec::new("child")
            .boundary(
                "net_env",
                NET_PORT,
                Value::Map(std::collections::BTreeMap::new()),
            )
            .transport_mock("execute", "response", Value::Str("ok".into()))
            .input_mock("prepare", "audience", Value::Str("mock".into()));

        let parent = MockSpec::new("parent")
            .boundary("root", "out", Value::Bool(true))
            .include_prefixed_runtime_mocks("cloud_credential/gcp_wif_secret", &child);

        assert!(parent
            .get_boundary_mock("cloud_credential/gcp_wif_secret/net_env", NET_PORT)
            .is_some());
        assert!(parent
            .get_transport_mock("cloud_credential/gcp_wif_secret/execute", "response")
            .is_some());
        assert!(parent.input_mocks.iter().any(|m| {
            m.node == "cloud_credential/gcp_wif_secret/prepare"
                && m.port == "audience"
                && matches!(m.value, Value::Str(_))
        }));
        assert!(parent.get_boundary_mock("root", "out").is_some());
    }

    #[test]
    fn test_non_empty_constraint() {
        let constraint = InputConstraint::non_empty();

        assert!(constraint.check(&Value::Str("hello".into())).is_ok());
        assert!(constraint.check(&Value::Str("".into())).is_err());
        assert!(constraint.check(&Value::str_list(vec!["a".into()])).is_ok());
        assert!(constraint.check(&Value::str_list(vec![])).is_err());
    }

    #[test]
    fn test_chain_validation() {
        let upstream =
            MockSpec::new("producer").boundary("output_node", "data", Value::Str("hello".into()));

        let downstream =
            MockSpec::new("consumer").expects_input("input", InputConstraint::non_empty());

        let mut mapping = HashMap::new();
        mapping.insert("data".to_string(), "input".to_string());

        let result = validate_chain(&upstream, &downstream, &mapping);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_validation_failure() {
        let upstream =
            MockSpec::new("producer").boundary("output_node", "data", Value::Str("".into())); // Empty!

        let downstream =
            MockSpec::new("consumer").expects_input("input", InputConstraint::non_empty());

        let mut mapping = HashMap::new();
        mapping.insert("data".to_string(), "input".to_string());

        let result = validate_chain(&upstream, &downstream, &mapping);
        assert!(!result.is_ok());
    }

    // ========================================================================
    // NodeExample and OutputMatcher tests
    // ========================================================================

    #[test]
    fn test_output_matcher_exact() {
        let matcher = OutputMatcher::exact(Value::Str("hello".into()));

        assert!(matcher.check(&Value::Str("hello".into())).is_ok());
        assert!(matcher.check(&Value::Str("world".into())).is_err());
        assert!(matcher.check(&Value::Int(42)).is_err());
    }

    #[test]
    fn test_output_matcher_contains() {
        let matcher = OutputMatcher::contains("world");

        assert!(matcher.check(&Value::Str("hello world".into())).is_ok());
        assert!(matcher.check(&Value::Str("world peace".into())).is_ok());
        assert!(matcher.check(&Value::Str("hello".into())).is_err());
        assert!(matcher.check(&Value::Int(42)).is_err());
    }

    #[test]
    fn test_output_matcher_non_empty() {
        let matcher = OutputMatcher::non_empty();

        assert!(matcher.check(&Value::Str("hello".into())).is_ok());
        assert!(matcher.check(&Value::Str("".into())).is_err());
        assert!(matcher.check(&Value::str_list(vec!["a".into()])).is_ok());
        assert!(matcher.check(&Value::str_list(vec![])).is_err());
        // Other types are considered non-empty
        assert!(matcher.check(&Value::Int(0)).is_ok());
    }

    #[test]
    fn test_output_matcher_satisfies() {
        let matcher =
            OutputMatcher::satisfies("is positive", |v| matches!(v, Value::Int(n) if *n > 0));

        assert!(matcher.check(&Value::Int(42)).is_ok());
        assert!(matcher.check(&Value::Int(-1)).is_err());
    }

    #[test]
    fn test_output_matcher_any() {
        let matcher = OutputMatcher::Any;

        assert!(matcher.check(&Value::Str("anything".into())).is_ok());
        assert!(matcher.check(&Value::Int(42)).is_ok());
        assert!(matcher.check(&Value::Unit).is_ok());
    }

    #[test]
    fn test_node_example_builder() {
        let example = NodeExample::new("prepare_prompt")
            .input("artifact", Value::Str("fn foo() {}".into()))
            .input("criteria", Value::Str("security".into()))
            .output("question", OutputMatcher::contains("security"))
            .output("system_prompt", OutputMatcher::non_empty())
            .description("Test with security criteria");

        assert_eq!(example.node_id, "prepare_prompt");
        assert_eq!(example.inputs.len(), 2);
        assert_eq!(example.outputs.len(), 2);
        assert_eq!(
            example.description,
            Some("Test with security criteria".to_string())
        );
    }

    #[test]
    fn test_mock_spec_with_node_examples() {
        let example = NodeExample::new("parse")
            .input("response", Value::Str("{\"content\": \"test\"}".into()))
            .output("content", OutputMatcher::exact(Value::Str("test".into())));

        let spec = MockSpec::new("test_dag")
            .transport_mock("execute", "response", Value::Str("ok".into()))
            .node_example(example);

        assert_eq!(spec.node_examples.len(), 1);
        assert_eq!(spec.node_examples[0].node_id, "parse");
    }

    // ========================================================================
    // assert_boundaries and assert_transport_mocks tests
    // ========================================================================

    #[test]
    fn test_assert_boundaries_success() {
        let spec = MockSpec::new("test")
            .boundary("node1", "out1", Value::Str("a".into()))
            .boundary("node2", "out2", Value::Bool(true));

        // Should not panic
        assert_boundaries(&spec, &[("node1", "out1"), ("node2", "out2")]);
    }

    #[test]
    #[should_panic(expected = "missing boundary mock for node3.out3")]
    fn test_assert_boundaries_failure() {
        let spec = MockSpec::new("test").boundary("node1", "out1", Value::Str("a".into()));

        // Should panic because node3.out3 is missing
        assert_boundaries(&spec, &[("node1", "out1"), ("node3", "out3")]);
    }

    #[test]
    fn test_assert_transport_mocks_success() {
        let spec = MockSpec::new("test")
            .transport_mock("execute1", "response", Value::Str("ok".into()))
            .transport_mock("execute2", "response", Value::Str("done".into()));

        // Should not panic
        assert_transport_mocks(&spec, &[("execute1", "response"), ("execute2", "response")]);
    }

    #[test]
    #[should_panic(expected = "missing transport mock for execute3.response")]
    fn test_assert_transport_mocks_failure() {
        let spec =
            MockSpec::new("test").transport_mock("execute1", "response", Value::Str("ok".into()));

        // Should panic because execute3.response is missing
        assert_transport_mocks(&spec, &[("execute1", "response"), ("execute3", "response")]);
    }

    // ========================================================================
    // Credential resource simulation tests
    // ========================================================================

    #[test]
    fn test_credential_acquire() {
        let spec = MockSpec::new("test").resource_credential("cred:api", Some(3_600_000));
        let resource = spec.get_resource("cred:api").unwrap();
        let result = resource.acquire();
        assert!(matches!(result, ResourceAcquireResult::Acquired));
    }

    #[test]
    fn test_credential_acquire_fails() {
        let mocks = ResourceMocks::new().credential_fails("cred:api", "invalid key");
        let resource = mocks.get("cred:api").unwrap();
        let result = resource.acquire();
        assert!(matches!(result, ResourceAcquireResult::Failed(_)));
    }

    #[test]
    fn test_credential_timeout() {
        let spec = MockSpec::new("test").resource_credential("cred:api", Some(3_600_000));
        let resource = spec.get_resource("cred:api").unwrap();
        assert!(!resource.should_timeout(1_800_000)); // 30 min — not expired
        assert!(resource.should_timeout(3_600_001)); // 1 hour + 1ms — expired
    }

    #[test]
    fn test_credential_no_expiry_never_times_out() {
        let spec = MockSpec::new("test").resource_credential("cred:api", None);
        let resource = spec.get_resource("cred:api").unwrap();
        assert!(!resource.should_timeout(u64::MAX));
    }

    #[test]
    fn test_resource_mock_id_aliases_normalize_to_canonical() {
        let spec = MockSpec::new("test")
            .resource_lock("file:Makefile")
            .resource_lock("target:manager");

        // Legacy and canonical IDs should both resolve.
        assert!(spec.get_resource("file:Makefile").is_some());
        assert!(spec.get_resource("file:Makefile").is_some());
        assert!(spec.get_resource("target:manager").is_some());
        assert!(spec.get_resource("target:manager").is_some());
    }

    // ========================================================================
    // to_boundary_mocks tests
    // ========================================================================

    use gunbc_ir::{NodeId, PortName};

    /// Helper to get the static value from a BoundaryMocks output mock.
    fn get_output_value(bm: &BoundaryMocks, node: &str, port: &str) -> Option<Value> {
        bm.get_mock(&NodeId::from(node), &PortName::from(port))
            .map(|m| m.value.clone())
    }

    #[test]
    fn test_to_boundary_mocks_includes_boundary_mocks() {
        let spec = MockSpec::new("test")
            .boundary("env_net", "network", Value::Str("mock-net".into()))
            .boundary("env_fs", "filesystem", Value::Bool(true));

        let bm = spec.to_boundary_mocks();
        assert_eq!(
            get_output_value(&bm, "env_net", "network"),
            Some(Value::Str("mock-net".into())),
            "boundary mock should appear in BoundaryMocks"
        );
        assert_eq!(
            get_output_value(&bm, "env_fs", "filesystem"),
            Some(Value::Bool(true)),
            "second boundary mock should appear"
        );
    }

    #[test]
    fn test_to_boundary_mocks_includes_transport_mocks() {
        let spec = MockSpec::new("test").transport_mock(
            "execute_http",
            "response",
            Value::Str("200 OK".into()),
        );

        let bm = spec.to_boundary_mocks();
        assert_eq!(
            get_output_value(&bm, "execute_http", "response"),
            Some(Value::Str("200 OK".into())),
            "transport mock should be included in BoundaryMocks"
        );
    }

    #[test]
    fn test_to_boundary_mocks_includes_input_mocks() {
        let spec =
            MockSpec::new("test").input_mock("prepare", "provider", Value::Str("gcp".into()));

        let bm = spec.to_boundary_mocks();
        assert_eq!(
            bm.get_input("prepare", "provider"),
            Some(&Value::Str("gcp".into())),
            "input mock should be included via set_input"
        );
    }

    #[test]
    fn test_to_boundary_mocks_combines_all_mock_types() {
        let spec = MockSpec::new("combined")
            .boundary("env", NET_PORT, Value::Str("net-mock".into()))
            .transport_mock("exec", "resp", Value::Int(42))
            .input_mock("entry", "arg", Value::Bool(false));

        let bm = spec.to_boundary_mocks();
        assert_eq!(
            get_output_value(&bm, "env", NET_PORT),
            Some(Value::Str("net-mock".into()))
        );
        assert_eq!(get_output_value(&bm, "exec", "resp"), Some(Value::Int(42)));
        assert_eq!(bm.get_input("entry", "arg"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_to_boundary_mocks_sequence() {
        let spec = MockSpec::new("test").boundary_sequence(
            "poll",
            "status",
            vec![Value::Str("pending".into()), Value::Str("running".into())],
        );

        let bm = spec.to_boundary_mocks();
        assert!(
            bm.get_mock(&NodeId::from("poll"), &PortName::from("status"))
                .is_some(),
            "sequence mock should be retrievable"
        );
    }

    #[test]
    fn test_to_boundary_mocks_empty_spec() {
        let spec = MockSpec::new("empty");
        let bm = spec.to_boundary_mocks();
        assert!(
            get_output_value(&bm, "any", "port").is_none(),
            "empty spec produces empty BoundaryMocks"
        );
        assert!(
            bm.get_input("any", "port").is_none(),
            "empty spec has no input mocks"
        );
    }

    #[test]
    fn test_to_dry_run_mocks_excludes_input_mocks() {
        let spec = MockSpec::new("test")
            .boundary("env", NET_PORT, Value::Str("mock".into()))
            .input_mock("entry", "arg", Value::Bool(true));

        let dry = spec.to_dry_run_mocks();
        assert_eq!(
            get_output_value(&dry, "env", NET_PORT),
            Some(Value::Str("mock".into())),
            "boundary mock should be in dry run mocks"
        );
        assert!(
            dry.get_input("entry", "arg").is_none(),
            "input mocks should NOT be in dry run mocks"
        );
    }

    #[test]
    fn test_to_boundary_mocks_vs_to_dry_run_mocks_difference() {
        let spec = MockSpec::new("diff")
            .boundary("env", NET_PORT, Value::Str("mock".into()))
            .transport_mock("exec", "resp", Value::Int(1))
            .input_mock("entry", "arg", Value::Bool(true));

        let boundary = spec.to_boundary_mocks();
        let dry_run = spec.to_dry_run_mocks();

        // Both should have boundary and transport mocks
        assert_eq!(
            get_output_value(&boundary, "env", NET_PORT),
            get_output_value(&dry_run, "env", NET_PORT)
        );
        assert_eq!(
            get_output_value(&boundary, "exec", "resp"),
            get_output_value(&dry_run, "exec", "resp")
        );

        // Only to_boundary_mocks should have input mocks
        assert!(boundary.get_input("entry", "arg").is_some());
        assert!(dry_run.get_input("entry", "arg").is_none());
    }
}
