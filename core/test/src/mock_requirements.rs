//! Typed mock requirements extracted from DAG structure.
//!
//! This module provides "impossible by construction" mock specification.
//! Instead of validating MockSpec against DAG at testgen time, the DAG
//! produces typed mock slots that must be filled before building a MockSpec.
//!
//! # Example
//!
//! ```ignore
//! let dag = build_my_dag();
//!
//! // DAG knows what mocks it needs
//! let spec = dag.mock_requirements()
//!     .boundary_str("execute", "result", "mock result")
//!     .unwrap()
//!     .transport_response("fetch", "response", mock_response())
//!     .unwrap()
//!     .build()  // Fails if required slots unfilled
//!     .expect("all mocks provided");
//! ```

use crate::mock_spec::{BoundaryMock, MockSpec, NodeExample, TransportMock};
use gunbc_ir::transport::TransportResponse;
use gunbc_ir::{
    parse_map_type_id, value_backing_for_type_id, Cardinality, NodeId, PortName, TypeId, Value,
    ValueKind,
};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

/// A slot that requires a mock value.
#[derive(Debug, Clone)]
pub struct MockSlot {
    /// Node that owns this port
    pub node_id: NodeId,
    /// Port name
    pub port_name: PortName,
    /// Expected type
    pub type_id: TypeId,
    /// Expected cardinality
    pub cardinality: Cardinality,
    /// Whether this mock is required (vs optional with default)
    pub required: bool,
    /// Kind of mock (boundary, transport, resource)
    pub kind: MockSlotKind,
}

/// What kind of mock slot this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockSlotKind {
    /// Boundary node output (world write)
    Boundary,
    /// Transport executor output
    Transport,
    /// Resource/environment output
    Resource,
}

/// Error when setting a mock value.
#[derive(Debug)]
pub enum MockTypeError {
    /// Unknown node/port combination
    UnknownSlot { node: String, port: String },

    /// Value type doesn't match port type
    TypeMismatch {
        node: String,
        port: String,
        expected: String,
        actual: String,
    },

    /// Value cardinality doesn't match port cardinality.
    CardinalityMismatch {
        node: String,
        port: String,
        expected: Cardinality,
        actual: usize,
    },
}

impl fmt::Display for MockTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockTypeError::UnknownSlot { node, port } => {
                write!(f, "unknown mock slot {}.{}", node, port)
            }
            MockTypeError::TypeMismatch {
                node,
                port,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "type mismatch for {}.{}: expected {}, got {}",
                    node, port, expected, actual
                )
            }
            MockTypeError::CardinalityMismatch {
                node,
                port,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "cardinality mismatch for {}.{}: expected {:?}, got {} value(s)",
                    node, port, expected, actual
                )
            }
        }
    }
}

impl Error for MockTypeError {}

/// Error when building a MockSpec with missing required mocks.
#[derive(Debug)]
pub struct MockIncompleteError {
    /// DAG name
    pub dag_name: String,
    /// Structured info about each missing slot.
    pub missing: Vec<MissingSlot>,
}

/// A missing mock slot with enough info for an actionable error message.
#[derive(Debug, Clone)]
pub struct MissingSlot {
    /// Node ID
    pub node: String,
    /// Port name
    pub port: String,
    /// Expected type
    pub type_id: String,
    /// Slot kind
    pub kind: MockSlotKind,
}

impl fmt::Display for MockIncompleteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "incomplete mock spec for '{}': {} unfilled slot(s)",
            self.dag_name,
            self.missing.len()
        )?;
        writeln!(f)?;
        writeln!(f, "  Missing boundary mocks:")?;
        for slot in &self.missing {
            writeln!(
                f,
                "    .boundary(\"{}\", \"{}\", <{}>)",
                slot.node, slot.port, slot.type_id
            )?;
        }
        writeln!(f)?;
        write!(
            f,
            "  Add these to the mock spec in the graph_mock.rs for '{}'.",
            self.dag_name
        )
    }
}

impl Error for MockIncompleteError {}

/// Builder for constructing a complete MockSpec from DAG requirements.
///
/// Extracted from a DAG via `dag.mock_requirements()`, this builder knows
/// exactly what mocks are needed and validates types at the point of
/// mock construction.
#[derive(Debug, Clone)]
pub struct MockRequirements {
    dag_name: String,
    slots: Vec<MockSlot>,
    filled: HashSet<(NodeId, PortName)>,
    boundary_mocks: Vec<BoundaryMock>,
    transport_mocks: Vec<TransportMock>,
    node_examples: Vec<NodeExample>,
    skipped_examples: Vec<String>,
    /// Node ID prefixes whose slots are delegated to runtime composition
    /// (e.g. via `MockSpec::include_prefixed_runtime_mocks`).
    /// Slots matching these prefixes are not required at build time.
    excluded_prefixes: Vec<String>,
}

impl MockRequirements {
    /// Create empty requirements for a DAG.
    pub fn new(dag_name: impl Into<String>) -> Self {
        Self {
            dag_name: dag_name.into(),
            slots: Vec::new(),
            filled: HashSet::new(),
            boundary_mocks: Vec::new(),
            transport_mocks: Vec::new(),
            node_examples: Vec::new(),
            skipped_examples: Vec::new(),
            excluded_prefixes: Vec::new(),
        }
    }

    /// Exclude slots whose node IDs start with the given prefix from completeness checks.
    ///
    /// Use this when mocks for a SubDag prefix will be provided later via
    /// `MockSpec::include_prefixed_runtime_mocks`. The slots are still created
    /// (so they can be filled if desired) but they won't cause `build()` to fail
    /// if left unfilled.
    pub fn exclude_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.excluded_prefixes.push(prefix.into());
        self
    }

    /// Add a required mock slot.
    pub fn add_slot(mut self, slot: MockSlot) -> Self {
        self.slots.push(slot);
        self
    }

    /// Add multiple slots.
    pub fn with_slots(mut self, slots: impl IntoIterator<Item = MockSlot>) -> Self {
        self.slots.extend(slots);
        self
    }

    /// Find a slot by node and port name.
    fn find_slot(&self, node: &str, port: &str) -> Result<&MockSlot, MockTypeError> {
        self.slots
            .iter()
            .find(|s| s.node_id.0 == node && s.port_name.0 == port)
            .ok_or_else(|| MockTypeError::UnknownSlot {
                node: node.to_string(),
                port: port.to_string(),
            })
    }

    /// Returns the canonical type-name string for a Value's kind.
    fn value_kind_name(value: &Value) -> &'static str {
        value.kind().type_name()
    }

    /// Check if a value type is compatible with an expected type.
    fn types_compatible(expected: &str, value: &Value) -> bool {
        let actual_kind = value.kind();
        let actual_name = actual_kind.type_name();

        // Exact match
        if expected == actual_name {
            return true;
        }

        // Any matches anything
        if expected == "Any" {
            return true;
        }

        // Optional types accept the inner type or Unit (none)
        if let Some(inner) = expected.strip_prefix("Optional") {
            if actual_name == inner || actual_kind == ValueKind::Unit {
                return true;
            }
        }

        // Skipped is compatible with any type
        if actual_kind == ValueKind::Skipped {
            return true;
        }

        // Json is flexible
        if expected == "Json" || actual_kind == ValueKind::Json {
            return true;
        }

        // Parametric map types: Map<String, T>
        if let Some((key_type, value_type)) = parse_map_type_id(expected) {
            if key_type != "String" {
                return false;
            }
            if let Value::Map(entries) = value {
                return entries
                    .values()
                    .all(|entry| Self::types_compatible(&value_type, entry));
            }
            return false;
        }

        // Platform has dual backing (String or Map)
        if expected == "Platform"
            && (actual_kind == ValueKind::String || actual_kind == ValueKind::Map)
        {
            return true;
        }

        // Delegate to centralized ValueBacking for all other type→value compatibility
        let backing = value_backing_for_type_id(expected);
        backing.accepts_value_kind(actual_kind)
    }

    /// Validate a value against a slot's type.
    fn validate_type(&self, slot: &MockSlot, value: &Value) -> Result<(), MockTypeError> {
        if !Self::types_compatible(&slot.type_id.0, value) {
            return Err(MockTypeError::TypeMismatch {
                node: slot.node_id.0.clone(),
                port: slot.port_name.0.clone(),
                expected: slot.type_id.0.clone(),
                actual: Self::value_kind_name(value).to_string(),
            });
        }
        Ok(())
    }

    /// Compute a count for cardinality validation.
    fn value_count(value: &Value) -> u32 {
        match value {
            Value::Unit | Value::Skipped => 0,
            Value::List(values) | Value::Set(values) => {
                u32::try_from(values.len()).unwrap_or(u32::MAX)
            }
            _ => 1,
        }
    }

    /// Validate a value against a slot's cardinality.
    fn validate_cardinality(&self, slot: &MockSlot, value: &Value) -> Result<(), MockTypeError> {
        let count = Self::value_count(value);
        if !slot.cardinality.allows_count(count) {
            return Err(MockTypeError::CardinalityMismatch {
                node: slot.node_id.0.clone(),
                port: slot.port_name.0.clone(),
                expected: slot.cardinality,
                actual: count as usize,
            });
        }
        Ok(())
    }

    /// Set a boundary mock value.
    ///
    /// Type is validated at call time, not at testgen time.
    pub fn boundary(
        mut self,
        node: &str,
        port: &str,
        value: impl Into<Value>,
    ) -> Result<Self, MockTypeError> {
        let value = value.into();

        // Find slot and extract what we need before mutating self
        let slot = self.find_slot(node, port)?;
        self.validate_type(slot, &value)?;
        self.validate_cardinality(slot, &value)?;
        let slot_kind = slot.kind;

        let key = (NodeId(node.to_string()), PortName(port.to_string()));

        // Replace existing mock if present (don't allow duplicates)
        // Remove any existing mock for this slot first
        match slot_kind {
            MockSlotKind::Transport => {
                self.transport_mocks
                    .retain(|m| !(m.node == node && m.port == port));
                self.transport_mocks.push(TransportMock {
                    node: node.to_string(),
                    port: port.to_string(),
                    value,
                });
            }
            MockSlotKind::Boundary | MockSlotKind::Resource => {
                self.boundary_mocks
                    .retain(|m| !(m.node == node && m.port == port));
                self.boundary_mocks.push(BoundaryMock {
                    node: node.to_string(),
                    port: port.to_string(),
                    value,
                    sequence: None,
                });
            }
        }

        self.filled.insert(key);

        Ok(self)
    }

    /// Set a string boundary mock.
    pub fn boundary_str(self, node: &str, port: &str, value: &str) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Str(value.to_string()))
    }

    /// Set a bool boundary mock.
    pub fn boundary_bool(self, node: &str, port: &str, value: bool) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Bool(value))
    }

    /// Set an int boundary mock.
    pub fn boundary_int(self, node: &str, port: &str, value: i64) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Int(value))
    }

    /// Set a JSON boundary mock.
    pub fn boundary_json(
        self,
        node: &str,
        port: &str,
        value: serde_json::Value,
    ) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Json(value))
    }

    /// Set a transport response mock.
    ///
    /// Only valid for transport executor output ports.
    pub fn transport_response(
        self,
        node: &str,
        port: &str,
        response: TransportResponse,
    ) -> Result<Self, MockTypeError> {
        self.boundary(node, port, Value::Response(response))
    }

    /// Add a node example.
    pub fn node_example(mut self, example: NodeExample) -> Self {
        self.node_examples.push(example);
        self
    }

    /// Skip node example requirement for a node.
    pub fn skip_node_example(mut self, node_id: &str) -> Self {
        self.skipped_examples.push(node_id.to_string());
        self
    }

    /// Get list of unfilled required slots.
    pub fn missing_slots(&self) -> Vec<&MockSlot> {
        self.slots
            .iter()
            .filter(|s| {
                s.required
                    && !self
                        .filled
                        .contains(&(s.node_id.clone(), s.port_name.clone()))
                    && !self
                        .excluded_prefixes
                        .iter()
                        .any(|prefix| s.node_id.0.starts_with(prefix.as_str()))
            })
            .collect()
    }

    /// Check if all required slots are filled.
    pub fn is_complete(&self) -> bool {
        self.missing_slots().is_empty()
    }

    /// Build the MockSpec.
    ///
    /// Fails if required slots are unfilled.
    pub fn build(self) -> Result<MockSpec, MockIncompleteError> {
        let missing = self.missing_slots();

        if !missing.is_empty() {
            return Err(MockIncompleteError {
                dag_name: self.dag_name.clone(),
                missing: missing
                    .iter()
                    .map(|s| MissingSlot {
                        node: s.node_id.0.clone(),
                        port: s.port_name.0.clone(),
                        type_id: s.type_id.0.clone(),
                        kind: s.kind,
                    })
                    .collect(),
            });
        }

        let mut spec = MockSpec::new(&self.dag_name);

        for bm in self.boundary_mocks {
            spec = spec.boundary(&bm.node, &bm.port, bm.value);
        }

        for tm in self.transport_mocks {
            spec = spec.transport_mock(&tm.node, &tm.port, tm.value);
        }

        for example in self.node_examples {
            spec = spec.node_example(example);
        }

        for skip in self.skipped_examples {
            spec = spec.skip_node_example(skip);
        }

        Ok(spec)
    }

    /// Build the MockSpec, panicking on incomplete.
    ///
    /// Use this when you're confident all mocks are provided.
    pub fn build_unchecked(self) -> MockSpec {
        match self.build() {
            Ok(spec) => spec,
            Err(e) => panic!("{e}"),
        }
    }
}

/// Extract mock requirements from a DAG.
///
/// This analyzes the DAG structure to determine:
/// - Boundary ports (unconnected outputs) that need mocks
/// - Transport executor outputs that need transport mocks
/// - Resource/environment outputs that need resource mocks
///
/// # Example
///
/// ```ignore
/// let dag = build_my_dag();
/// let requirements = extract_mock_requirements(&dag, "my_dag");
///
/// let spec = requirements
///     .boundary_str("output_node", "result", "mock value")
///     .unwrap()
///     .build()
///     .expect("all mocks provided");
/// ```
pub fn extract_mock_requirements<T: Clone>(dag: &gunbc_ir::Dag<T>, name: &str) -> MockRequirements {
    use gunbc_ir::detect_boundaries;
    use std::collections::HashSet;

    // Lower the DAG to flatten SubDags so transport executors inside SubDags
    // are visible and boundary analysis uses lowered node IDs.
    let lowered = gunbc_exec::lower(dag)
        .unwrap_or_else(|e| panic!("extract_mock_requirements: lower failed: {e}"));
    let dag = &lowered.dag;

    let boundaries = detect_boundaries(dag);

    // Find transport executor nodes (consume TransportRequest)
    let transport_nodes: HashSet<&str> = dag
        .nodes
        .iter()
        .filter(|n| n.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"))
        .map(|n| n.id.0.as_str())
        .collect();

    // Find resource/environment nodes (emit ToolHandle, Credential, etc.)
    let resource_types = [
        "ToolHandle",
        "Credential",
        "FilesystemHandle",
        "NetworkHandle",
        "Timestamp",
        "Platform",
        "CloudSecretConfig",
    ];
    let resource_nodes: HashSet<&str> = dag
        .nodes
        .iter()
        .filter(|n| {
            n.outputs
                .iter()
                .any(|p| resource_types.contains(&p.type_id.0.as_str()))
        })
        .map(|n| n.id.0.as_str())
        .collect();

    // Find CLI tool nodes (consume ToolHandle but are not resource providers)
    // These nodes execute external tools and need mocks for their outputs during DryRun
    let cli_tool_nodes: HashSet<&str> = dag
        .nodes
        .iter()
        .filter(|n| {
            // Has ToolHandle input
            let has_tool_input = n.inputs.iter().any(|p| p.type_id.0 == "ToolHandle");
            // Is not a resource node (doesn't emit resource types)
            let is_not_resource = !resource_nodes.contains(n.id.0.as_str());
            // Is not a transport node (doesn't consume TransportRequest)
            let is_not_transport = !transport_nodes.contains(n.id.0.as_str());
            has_tool_input && is_not_resource && is_not_transport
        })
        .map(|n| n.id.0.as_str())
        .collect();

    let mut requirements = MockRequirements::new(name);

    // Add slots for boundary ports from transport and resource nodes only
    // Pure node terminal outputs are COMPUTED, not mocked, so they're not required
    for (node_id, port_name) in &boundaries.boundary_ports {
        let node = dag.get_node(node_id).unwrap();
        let port = node.outputs.iter().find(|p| &p.name == port_name).unwrap();

        // Only require mocks for transport, resource, and CLI tool nodes
        // Pure node terminal outputs are computed during execution
        let (kind, required) = if transport_nodes.contains(node_id.0.as_str()) {
            (MockSlotKind::Transport, true)
        } else if resource_nodes.contains(node_id.0.as_str()) {
            (MockSlotKind::Resource, true)
        } else if cli_tool_nodes.contains(node_id.0.as_str()) {
            // CLI tool nodes (like clippy_lint) need mocks for DryRun interception
            (MockSlotKind::Transport, true)
        } else {
            // Pure node terminal outputs - optional (for expected output verification)
            (MockSlotKind::Boundary, false)
        };

        requirements = requirements.add_slot(MockSlot {
            node_id: node_id.clone(),
            port_name: port_name.clone(),
            type_id: port.type_id.clone(),
            cardinality: port.cardinality,
            required,
            kind,
        });
    }

    // Also add slots for ALL transport node outputs (even unconnected ones)
    // because DryRun interception requires mocks for every output port
    for node in &dag.nodes {
        if !transport_nodes.contains(node.id.0.as_str()) {
            continue;
        }

        for port in &node.outputs {
            // Skip if already added as boundary
            let is_boundary = boundaries
                .boundary_ports
                .iter()
                .any(|(nid, pn)| nid == &node.id && pn == &port.name);
            if is_boundary {
                continue;
            }

            // Transport nodes require mocks for ALL outputs (connected or not)
            // because DryRun interception returns mocked values for the entire node
            requirements = requirements.add_slot(MockSlot {
                node_id: node.id.clone(),
                port_name: port.name.clone(),
                type_id: port.type_id.clone(),
                cardinality: port.cardinality,
                required: true,
                kind: MockSlotKind::Transport,
            });
        }
    }

    // Also add slots for resource/env node outputs that ARE connected downstream
    // (they provide capability tokens that need mocks for DryRun)
    for node in &dag.nodes {
        if !resource_nodes.contains(node.id.0.as_str()) {
            continue;
        }

        for port in &node.outputs {
            // Skip if already added as boundary
            let is_boundary = boundaries
                .boundary_ports
                .iter()
                .any(|(nid, pn)| nid == &node.id && pn == &port.name);
            if is_boundary {
                continue;
            }

            // Check if this output is connected downstream
            let is_connected = dag
                .edges
                .iter()
                .any(|e| e.from_node == node.id && e.from_port == port.name);

            if is_connected {
                requirements = requirements.add_slot(MockSlot {
                    node_id: node.id.clone(),
                    port_name: port.name.clone(),
                    type_id: port.type_id.clone(),
                    cardinality: port.cardinality,
                    required: true,
                    kind: MockSlotKind::Resource,
                });
            }
        }
    }

    // Also add slots for ALL CLI tool node outputs (even connected ones)
    // because DryRun interception requires mocks for every output port
    for node in &dag.nodes {
        if !cli_tool_nodes.contains(node.id.0.as_str()) {
            continue;
        }

        for port in &node.outputs {
            // Skip if already added as boundary
            let is_boundary = boundaries
                .boundary_ports
                .iter()
                .any(|(nid, pn)| nid == &node.id && pn == &port.name);
            if is_boundary {
                continue;
            }

            // CLI tool nodes require mocks for ALL outputs (connected or not)
            // because DryRun interception returns mocked values for the entire node
            requirements = requirements.add_slot(MockSlot {
                node_id: node.id.clone(),
                port_name: port.name.clone(),
                type_id: port.type_id.clone(),
                cardinality: port.cardinality,
                required: true,
                kind: MockSlotKind::Transport, // Treat CLI tools like transport for interception
            });
        }
    }

    requirements
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_slot(node: &str, port: &str, type_id: &str) -> MockSlot {
        MockSlot {
            node_id: NodeId(node.to_string()),
            port_name: PortName(port.to_string()),
            type_id: TypeId(type_id.to_string()),
            cardinality: Cardinality::ONE,
            required: true,
            kind: MockSlotKind::Boundary,
        }
    }

    #[test]
    fn test_complete_build_succeeds() {
        let reqs = MockRequirements::new("test").add_slot(test_slot("node", "port", "String"));

        let result = reqs.boundary_str("node", "port", "value").unwrap().build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_incomplete_build_fails() {
        let reqs = MockRequirements::new("test").add_slot(test_slot("node", "port", "String"));

        let result = reqs.build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .missing
            .iter()
            .any(|s| s.node == "node" && s.port == "port"));
    }

    #[test]
    fn test_type_mismatch_detected() {
        let reqs = MockRequirements::new("test").add_slot(test_slot("node", "port", "Int"));

        let result = reqs.boundary_str("node", "port", "not an int");

        assert!(result.is_err());
        match result.unwrap_err() {
            MockTypeError::TypeMismatch {
                expected, actual, ..
            } => {
                assert_eq!(expected, "Int");
                assert_eq!(actual, "String");
            }
            _ => panic!("expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_cardinality_mismatch_detected() {
        let slot = MockSlot {
            node_id: NodeId("node".to_string()),
            port_name: PortName("port".to_string()),
            type_id: TypeId("Any".to_string()),
            cardinality: Cardinality::ONE,
            required: true,
            kind: MockSlotKind::Boundary,
        };

        let reqs = MockRequirements::new("test").add_slot(slot);
        let value = Value::List(vec![Value::Int(1), Value::Int(2)]);

        let result = reqs.boundary("node", "port", value);

        assert!(result.is_err());
        match result.unwrap_err() {
            MockTypeError::CardinalityMismatch {
                expected, actual, ..
            } => {
                assert_eq!(expected, Cardinality::ONE);
                assert_eq!(actual, 2);
            }
            _ => panic!("expected CardinalityMismatch error"),
        }
    }

    #[test]
    fn test_unknown_slot_detected() {
        let reqs = MockRequirements::new("test");

        let result = reqs.boundary_str("unknown", "port", "value");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MockTypeError::UnknownSlot { .. }
        ));
    }

    #[test]
    fn test_transport_slot() {
        let slot = MockSlot {
            node_id: NodeId("execute".to_string()),
            port_name: PortName("response".to_string()),
            type_id: TypeId("TransportResponse".to_string()),
            cardinality: Cardinality::ONE,
            required: true,
            kind: MockSlotKind::Transport,
        };

        let reqs = MockRequirements::new("test").add_slot(slot);

        let response = TransportResponse::Shell(gunbc_ir::transport::ShellResponse::ok("test"));
        let result = reqs.transport_response("execute", "response", response);

        assert!(result.is_ok());
    }

    #[test]
    fn test_map_backed_types_compatible() {
        let reqs = MockRequirements::new("test").add_slot(test_slot("env", "handle", "ToolHandle"));

        // ToolHandle is represented as Map
        let mut map = std::collections::BTreeMap::new();
        map.insert("type".to_string(), Value::Str("tool_handle".to_string()));
        map.insert("id".to_string(), Value::Str("test".to_string()));

        let result = reqs.boundary("env", "handle", Value::Map(map));

        assert!(result.is_ok());
    }

    #[test]
    fn test_parametric_map_types_compatible() {
        let reqs = MockRequirements::new("test").add_slot(test_slot(
            "render",
            "meta",
            "Map<String,String>",
        ));
        let mut map = std::collections::BTreeMap::new();
        map.insert("name".to_string(), Value::Str("gunbc".to_string()));

        let result = reqs.boundary("render", "meta", Value::Map(map));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parametric_map_types_reject_wrong_value_type() {
        let reqs = MockRequirements::new("test").add_slot(test_slot(
            "render",
            "meta",
            "Map<String,String>",
        ));
        let mut map = std::collections::BTreeMap::new();
        map.insert("count".to_string(), Value::Int(7));

        let result = reqs.boundary("render", "meta", Value::Map(map));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MockTypeError::TypeMismatch { .. }
        ));
    }

    #[test]
    fn test_extract_mock_requirements_from_dag() {
        use gunbc_ir::build::{edge, port};
        use gunbc_ir::{Dag, Node};

        // Build a simple DAG with a transport node
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare",
            vec![],
            vec![port("request", "TransportRequest")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            (),
        ));
        dag.add_node(Node::opaque(
            "parse",
            vec![port("response", "TransportResponse")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("prepare", "request", "execute", "request"));
        dag.add_edge(edge("execute", "response", "parse", "response"));

        // Extract requirements
        let reqs = extract_mock_requirements(&dag, "test");

        // Should have:
        // 1. execute.response (transport output, connected downstream)
        // 2. parse.result (boundary, terminal output)
        assert_eq!(reqs.slots.len(), 2);

        // Check transport slot
        let transport_slot = reqs
            .slots
            .iter()
            .find(|s| s.node_id.0 == "execute" && s.port_name.0 == "response");
        assert!(transport_slot.is_some());
        assert_eq!(transport_slot.unwrap().kind, MockSlotKind::Transport);

        // Check boundary slot
        let boundary_slot = reqs
            .slots
            .iter()
            .find(|s| s.node_id.0 == "parse" && s.port_name.0 == "result");
        assert!(boundary_slot.is_some());
        assert_eq!(boundary_slot.unwrap().kind, MockSlotKind::Boundary);

        // Building without filling slots should fail
        assert!(reqs.build().is_err());
    }

    #[test]
    fn test_extract_and_build_complete_spec() {
        use gunbc_ir::build::port;
        use gunbc_ir::{Dag, Node};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("output", "String")],
            (),
        ));

        let reqs = extract_mock_requirements(&dag, "test");

        // Should have one boundary slot
        assert_eq!(reqs.slots.len(), 1);

        // Fill it and build
        let spec = reqs
            .boundary_str("source", "output", "test value")
            .unwrap()
            .build()
            .unwrap();

        // MockSpec should have the boundary mock
        assert!(spec.get_boundary_mock("source", "output").is_some());
    }

    #[test]
    fn test_rejects_unknown_slot() {
        use gunbc_ir::build::port;
        use gunbc_ir::{Dag, Node};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("output", "String")],
            (),
        ));

        crate::assert_typed_builder_rejects_invalid_slot(&dag, "test");
    }
}
