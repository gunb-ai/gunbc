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
//! ```ignore
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
use gunbc_ir::Value;
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
}

impl MockSpec {
    /// Create a new mock spec for a named tool.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            boundary_mocks: Vec::new(),
            input_expectations: Vec::new(),
            resource_mocks: ResourceMocks::new(),
            transport_mocks: Vec::new(),
            expected_outputs: Vec::new(),
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
        });
        self
    }

    /// Add an input expectation (what this node requires from upstream).
    pub fn expects_input(
        mut self,
        port: impl Into<String>,
        constraint: InputConstraint,
    ) -> Self {
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

    /// Convert this MockSpec into BoundaryMocks suitable for `execute_with_mode`.
    ///
    /// Maps transport_mocks to port-level mocks in the resulting BoundaryMocks.
    pub fn to_boundary_mocks(&self) -> BoundaryMocks {
        let mut mocks = BoundaryMocks::new();
        for tm in &self.transport_mocks {
            mocks.set_value(&tm.node, &tm.port, tm.value.clone());
        }
        mocks
    }

    /// Check whether this spec has flow test data (transport mocks or expected outputs).
    pub fn has_flow_test_data(&self) -> bool {
        !self.transport_mocks.is_empty() || !self.expected_outputs.is_empty()
    }

    /// Get mock value for a specific boundary port.
    pub fn get_boundary_mock(&self, node: &str, port: &str) -> Option<&Value> {
        self.boundary_mocks
            .iter()
            .find(|m| m.node == node && m.port == port)
            .map(|m| &m.value)
    }

    /// Check if a value satisfies input expectations for a port.
    pub fn satisfies_input(&self, port: &str, value: &Value) -> Result<(), String> {
        let expectation = self
            .input_expectations
            .iter()
            .find(|e| e.port == port);

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
    /// Mock value to return
    pub value: Value,
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

/// An expected output at a terminal/boundary node (for flow test assertions).
#[derive(Debug, Clone)]
pub struct ExpectedOutput {
    /// Node ID to check (e.g., "report")
    pub node: String,
    /// Output port name (e.g., "overall_success")
    pub port: String,
    /// Expected value
    pub expected: Value,
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
                Value::StrList(v) if v.is_empty() => Err("expected non-empty list".into()),
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
            InputConstraint::Custom { description, predicate } => {
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
fn values_match(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Unit, Value::Unit) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        _ => false,
    }
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
        Self {
            resource_id: id.into(),
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
        if let ResourceType::Lease { duration_ms } = self.resource_type {
            held_ms > duration_ms
        } else {
            false
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
        self.resources.push(ResourceSimulation::new(id, ResourceType::Lock));
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
        let sim = ResourceSimulation::new(id, ResourceType::Lock)
            .with_behavior(ResourceBehavior::FailAcquire { error: error.into() });
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

    /// Get simulation for a resource.
    pub fn get(&self, id: &str) -> Option<&ResourceSimulation> {
        self.resources.iter().find(|r| r.resource_id == id)
    }
}

/// Get a type name for a value.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Unit => "Unit",
        Value::Bool(_) => "Bool",
        Value::Str(_) => "String",
        Value::Int(_) => "Int",
        Value::StrList(_) => "StrList",
        Value::MapStrStr(_) => "MapStrStr",
        Value::Json(_) => "Json",
        Value::Skipped => "Skipped",
        _ => "Unknown",
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
    MissingMock {
        upstream: String,
        port: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_spec_builder() {
        let spec = MockSpec::new("gist")
            .boundary("create_gist", "url", Value::Str("https://example.com".into()))
            .expects_input("files", InputConstraint::non_empty());

        assert_eq!(spec.name, "gist");
        assert_eq!(spec.boundary_mocks.len(), 1);
        assert_eq!(spec.input_expectations.len(), 1);
    }

    #[test]
    fn test_get_boundary_mock() {
        let spec = MockSpec::new("test")
            .boundary("node1", "out", Value::Str("value".into()));

        assert!(spec.get_boundary_mock("node1", "out").is_some());
        assert!(spec.get_boundary_mock("node1", "other").is_none());
    }

    #[test]
    fn test_non_empty_constraint() {
        let constraint = InputConstraint::non_empty();

        assert!(constraint.check(&Value::Str("hello".into())).is_ok());
        assert!(constraint.check(&Value::Str("".into())).is_err());
        assert!(constraint.check(&Value::StrList(vec!["a".into()])).is_ok());
        assert!(constraint.check(&Value::StrList(vec![])).is_err());
    }

    #[test]
    fn test_chain_validation() {
        let upstream = MockSpec::new("producer")
            .boundary("output_node", "data", Value::Str("hello".into()));

        let downstream = MockSpec::new("consumer")
            .expects_input("input", InputConstraint::non_empty());

        let mut mapping = HashMap::new();
        mapping.insert("data".to_string(), "input".to_string());

        let result = validate_chain(&upstream, &downstream, &mapping);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chain_validation_failure() {
        let upstream = MockSpec::new("producer")
            .boundary("output_node", "data", Value::Str("".into())); // Empty!

        let downstream = MockSpec::new("consumer")
            .expects_input("input", InputConstraint::non_empty());

        let mut mapping = HashMap::new();
        mapping.insert("data".to_string(), "input".to_string());

        let result = validate_chain(&upstream, &downstream, &mapping);
        assert!(!result.is_ok());
    }
}
