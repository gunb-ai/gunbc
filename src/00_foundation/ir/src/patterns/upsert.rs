//! Upsert pattern: Check → Create → Resolve.
//!
//! The upsert pattern is used for idempotent resource creation:
//!
//! 1. **Check**: Determine if the resource already exists (read-only)
//! 2. **Create**: If not exists, create the resource (guarded by check result)
//! 3. **Resolve**: Verify the resource exists and return its handle (read-only)
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │                   Upsert                     │
//! │  ┌───────┐    ┌────────┐    ┌─────────┐    │
//! │  │ Check │───▶│ Create │───▶│ Resolve │    │
//! │  └───────┘    └────────┘    └─────────┘    │
//! │      │            │              │          │
//! │      └── guard ───┘              │          │
//! │      (exists=false)              ▼          │
//! │                              (boundary)     │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! The Create node has a guard that only executes if Check returns `false`.

use crate::dag::{Dag, Edge, Port};
use crate::type_op::{Predicate, PredicateValue};
use crate::node::Node;
use crate::types::Cardinality;

/// Builder for the upsert pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```text
/// let upsert = UpsertBuilder::new("install_tool")
///     .with_check(DepsOp::CheckInstalled)
///     .with_create(DepsOp::Install)
///     .with_resolve(DepsOp::Verify)
///     .build();
/// ```
pub struct UpsertBuilder<T> {
    name: String,
    check_op: Option<T>,
    create_op: Option<T>,
    resolve_op: Option<T>,
    // Port configurations
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> UpsertBuilder<T> {
    /// Create a new upsert builder with the given name.
    ///
    /// The name is used as the node ID for the upsert subgraph.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            check_op: None,
            create_op: None,
            resolve_op: None,
            input_port_name: "resource_id".to_string(),
            input_port_type: "String".to_string(),
            output_port_name: "handle".to_string(),
            output_port_type: "String".to_string(),
        }
    }

    /// Set the check operation.
    ///
    /// The check operation should output a `Bool` on port `"exists"`.
    pub fn with_check(mut self, op: T) -> Self {
        self.check_op = Some(op);
        self
    }

    /// Set the create operation.
    ///
    /// The create operation is guarded by `exists == false`.
    pub fn with_create(mut self, op: T) -> Self {
        self.create_op = Some(op);
        self
    }

    /// Set the resolve operation.
    ///
    /// The resolve operation always runs and outputs the final handle.
    pub fn with_resolve(mut self, op: T) -> Self {
        self.resolve_op = Some(op);
        self
    }

    /// Configure the input port.
    pub fn with_input_port(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.input_port_name = name.into();
        self.input_port_type = type_id.into();
        self
    }

    /// Configure the output port.
    pub fn with_output_port(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.output_port_name = name.into();
        self.output_port_type = type_id.into();
        self
    }

    /// Build the upsert pattern as a SubDag node.
    ///
    /// # Panics
    ///
    /// Panics if any of the operations (check, create, resolve) are not set.
    pub fn build(self) -> Node<T> {
        let check_op = self.check_op.expect("check operation is required");
        let create_op = self.create_op.expect("create operation is required");
        let resolve_op = self.resolve_op.expect("resolve operation is required");

        let mut dag = Dag::new();

        // Check node: determines if resource exists
        dag.add_node(Node::opaque(
            "check",
            vec![Port::scalar(
                self.input_port_name.as_str(),
                self.input_port_type.as_str(),
            )],
            vec![Port::scalar("exists", "Bool")],
            check_op,
        ));

        // Create node: guarded by exists == false
        dag.add_node(Node::opaque(
            "create",
            vec![
                Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str()),
                Port::guarded_with_cardinality(
                    "exists",
                    "Bool",
                    Cardinality::ONE,
                    Predicate::Equals(PredicateValue::Bool(false)),
                ),
            ],
            vec![],
            create_op,
        ));

        // Resolve node: always runs, verifies final state
        dag.add_node(Node::opaque(
            "resolve",
            vec![Port::scalar(
                self.input_port_name.as_str(),
                self.input_port_type.as_str(),
            )],
            vec![Port::scalar(
                self.output_port_name.as_str(),
                self.output_port_type.as_str(),
            )],
            resolve_op,
        ));

        // Wire: check.exists -> create.exists (for guard)
        dag.add_edge(Edge::new("check", "exists", "create", "exists"));

        // Create the outer node with the subdag
        Node::subdag(self.name.as_str(), dag)
    }
}

/// Phase of an upsert operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertPhase {
    /// Check if resource exists (read-only)
    Check,
    /// Create resource if missing (idempotent, guarded)
    Create,
    /// Verify and return resolved handle (read-only)
    Resolve,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[derive(Debug, Clone)]
    enum TestOp {
        Check,
        Create,
        Resolve,
    }

    #[test]
    fn test_upsert_builder_creates_subdag() {
        let node = UpsertBuilder::new("test_upsert")
            .with_check(TestOp::Check)
            .with_create(TestOp::Create)
            .with_resolve(TestOp::Resolve)
            .build();

        assert_eq!(node.id.0, "test_upsert");
        assert!(node.is_subdag());

        // Check inputs/outputs
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.inputs[0].name.0, "resource_id");

        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.outputs[0].name.0, "handle");
    }

    #[test]
    fn test_upsert_subdag_structure() {
        let node = UpsertBuilder::new("test")
            .with_check(TestOp::Check)
            .with_create(TestOp::Create)
            .with_resolve(TestOp::Resolve)
            .build();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
                assert_eq!(dag.nodes.len(), 3);
                assert_eq!(dag.edges.len(), 1);

                // Check node names
                let node_names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_names.contains(&"check"));
                assert!(node_names.contains(&"create"));
                assert!(node_names.contains(&"resolve"));

                // Check create has guard
                let create_node = dag.get_node(&"create".into()).unwrap();
                let exists_port = create_node
                    .inputs
                    .iter()
                    .find(|p| p.name.0 == "exists")
                    .unwrap();
                assert!(exists_port.guard.is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_upsert_custom_ports() {
        let node = UpsertBuilder::new("custom")
            .with_check(TestOp::Check)
            .with_create(TestOp::Create)
            .with_resolve(TestOp::Resolve)
            .with_input_port("tool_name", "ToolId")
            .with_output_port("install_path", "Path")
            .build();

        assert_eq!(node.inputs[0].name.0, "tool_name");
        assert_eq!(node.inputs[0].type_id.0, "ToolId");
        assert_eq!(node.outputs[0].name.0, "install_path");
        assert_eq!(node.outputs[0].type_id.0, "Path");
    }

    // ============ Interface Validation Tests ============

    #[test]
    fn test_upsert_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = UpsertBuilder::new("upsert")
            .with_check(TestOp::Check)
            .with_create(TestOp::Create)
            .with_resolve(TestOp::Resolve)
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "upsert interface errors: {:?}", errors);
    }
}
