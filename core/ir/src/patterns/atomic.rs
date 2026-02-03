//! Atomic pattern: Precondition → Operation → Postcondition.
//!
//! The atomic pattern ensures an operation is only executed when preconditions
//! are met, and postconditions are verified after execution:
//!
//! 1. **Precondition**: Verify operation can proceed (optional, guarded)
//! 2. **Operation**: Execute the main operation
//! 3. **Postcondition**: Verify operation succeeded (optional)
//!
//! ```text
//! ┌────────────────────────────────────────────────────┐
//! │                     Atomic                          │
//! │  ┌──────────────┐    ┌───────────┐    ┌─────────┐ │
//! │  │ Precondition │───▶│ Operation │───▶│ Postcon │ │
//! │  └──────────────┘    └───────────┘    └─────────┘ │
//! │         │                  │               │       │
//! │         └──── guard ───────┘               │       │
//! │            (pre_ok=true)                   ▼       │
//! │                                       (verify)     │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! Unlike upsert, the atomic pattern is more general and doesn't assume
//! check-then-create semantics.

use crate::dag::{Dag, Edge, Guard, Port};
use crate::node::Node;
use crate::types::Cardinality;
use crate::value::Value;

/// Builder for the atomic operation pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```ignore
/// let atomic = AtomicBuilder::new("safe_delete")
///     .with_precondition(FileOp::CheckEmpty)
///     .with_operation(FileOp::Delete)
///     .with_postcondition(FileOp::VerifyDeleted)
///     .build();
/// ```
pub struct AtomicBuilder<T> {
    name: String,
    precondition_op: Option<T>,
    operation_op: Option<T>,
    postcondition_op: Option<T>,
    // Port configurations
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> AtomicBuilder<T> {
    /// Create a new atomic builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            precondition_op: None,
            operation_op: None,
            postcondition_op: None,
            input_port_name: "input".to_string(),
            input_port_type: "Any".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "Any".to_string(),
        }
    }

    /// Set the precondition operation (optional).
    ///
    /// The precondition should output `"pre_ok": Bool`.
    /// If pre_ok is false, the operation is skipped.
    pub fn with_precondition(mut self, op: T) -> Self {
        self.precondition_op = Some(op);
        self
    }

    /// Set the main operation (required).
    pub fn with_operation(mut self, op: T) -> Self {
        self.operation_op = Some(op);
        self
    }

    /// Set the postcondition operation (optional).
    ///
    /// The postcondition verifies the operation succeeded.
    pub fn with_postcondition(mut self, op: T) -> Self {
        self.postcondition_op = Some(op);
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

    /// Build the atomic pattern as a SubDag node.
    ///
    /// # Panics
    ///
    /// Panics if the operation is not set.
    pub fn build(self) -> Node<T> {
        let operation_op = self.operation_op.expect("operation is required");

        let mut dag = Dag::new();

        // Precondition node (if provided)
        let has_precondition = self.precondition_op.is_some();
        if let Some(precondition_op) = self.precondition_op {
            dag.add_node(Node::opaque(
                "precondition",
                vec![Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str())],
                vec![
                    Port::scalar("pre_ok", "Bool"),
                    Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str()),
                ],
                precondition_op,
            ));
        }

        // Operation node
        let operation_inputs = if has_precondition {
            vec![
                Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str()),
                Port::guarded_with_cardinality(
                    "pre_ok",
                    "Bool",
                    Cardinality::One,
                    Guard::Eq(Value::Bool(true)),
                ),
            ]
        } else {
            vec![Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str())]
        };

        dag.add_node(Node::opaque(
            "operation",
            operation_inputs,
            vec![
                Port::scalar(self.output_port_name.as_str(), self.output_port_type.as_str()),
                Port::scalar("op_ok", "Bool"),
            ],
            operation_op,
        ));

        // Postcondition node (if provided)
        let has_postcondition = self.postcondition_op.is_some();
        if let Some(postcondition_op) = self.postcondition_op {
            dag.add_node(Node::opaque(
                "postcondition",
                vec![
                    Port::scalar(self.output_port_name.as_str(), self.output_port_type.as_str()),
                    Port::scalar("op_ok", "Bool"),
                ],
                vec![
                    Port::scalar(self.output_port_name.as_str(), self.output_port_type.as_str()),
                    Port::scalar("verified", "Bool"),
                ],
                postcondition_op,
            ));
        }

        // Wire edges
        if has_precondition {
            dag.add_edge(Edge::new(
                "precondition",
                self.input_port_name.as_str(),
                "operation",
                self.input_port_name.as_str(),
            ));
            dag.add_edge(Edge::new("precondition", "pre_ok", "operation", "pre_ok"));
        }

        if has_postcondition {
            dag.add_edge(Edge::new(
                "operation",
                self.output_port_name.as_str(),
                "postcondition",
                self.output_port_name.as_str(),
            ));
            dag.add_edge(Edge::new("operation", "op_ok", "postcondition", "op_ok"));
        }

        Node::subdag(self.name.as_str(), dag)
    }
}

/// Phase of an atomic operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicPhase {
    /// Verify operation can proceed
    Precondition,
    /// Execute the main operation
    Operation,
    /// Verify operation succeeded
    Postcondition,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[derive(Debug, Clone)]
    enum TestOp {
        Precondition,
        Operation,
        Postcondition,
    }

    #[test]
    fn test_atomic_builder_operation_only() {
        let node = AtomicBuilder::new("simple_op")
            .with_operation(TestOp::Operation)
            .build();

        assert_eq!(node.id.0, "simple_op");
        assert!(node.is_subdag());

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 1);
                assert_eq!(dag.edges.len(), 0);
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_atomic_builder_full() {
        let node = AtomicBuilder::new("guarded_op")
            .with_precondition(TestOp::Precondition)
            .with_operation(TestOp::Operation)
            .with_postcondition(TestOp::Postcondition)
            .build();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 3);
                assert_eq!(dag.edges.len(), 4);

                // Check operation has guard
                let op_node = dag.get_node(&"operation".into()).unwrap();
                let pre_ok_port = op_node.inputs.iter().find(|p| p.name.0 == "pre_ok").unwrap();
                assert!(pre_ok_port.guard.is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_atomic_custom_ports() {
        let node = AtomicBuilder::new("custom")
            .with_operation(TestOp::Operation)
            .with_input_port("file_path", "Path")
            .with_output_port("result", "Bool")
            .build();

        let file_path_input = node.inputs.iter().find(|p| p.name.0 == "file_path").unwrap();
        assert_eq!(file_path_input.type_id.0, "Path");

        let result_output = node.outputs.iter().find(|p| p.name.0 == "result").unwrap();
        assert_eq!(result_output.type_id.0, "Bool");
    }

    // ============ Interface Validation Tests ============

    #[test]
    fn test_atomic_operation_only_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = AtomicBuilder::new("atomic")
            .with_operation(TestOp::Operation)
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "atomic (op-only) interface errors: {:?}", errors);
    }

    #[test]
    fn test_atomic_full_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = AtomicBuilder::new("atomic")
            .with_precondition(TestOp::Precondition)
            .with_operation(TestOp::Operation)
            .with_postcondition(TestOp::Postcondition)
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "atomic (full) interface errors: {:?}", errors);
    }
}
