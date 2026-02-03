//! Transaction pattern: Begin → Body → Commit/Rollback.
//!
//! The transaction pattern provides transactional semantics for operations:
//!
//! 1. **Begin**: Initialize transaction state (acquire locks, etc.)
//! 2. **Body**: Execute the main operations (a subdag)
//! 3. **Commit**: Finalize on success (guarded by body success)
//! 4. **Rollback**: Cleanup on failure (guarded by body failure)
//!
//! ```text
//! ┌──────────────────────────────────────────────────┐
//! │                  Transaction                      │
//! │  ┌───────┐    ┌──────┐    ┌────────┬──────────┐ │
//! │  │ Begin │───▶│ Body │───▶│ Commit │ Rollback │ │
//! │  └───────┘    └──────┘    └────────┴──────────┘ │
//! │                   │              ▲       ▲       │
//! │                   │              │       │       │
//! │                   └──── guard ───┴───────┘       │
//! │                       (success/failure)          │
//! └──────────────────────────────────────────────────┘
//! ```

use crate::dag::{Dag, Edge, Guard, Port};
use crate::node::Node;
use crate::types::Cardinality;
use crate::value::Value;

/// Builder for the transaction pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```ignore
/// let txn = TransactionBuilder::new("database_update")
///     .with_begin(DbOp::BeginTransaction)
///     .with_body(update_dag)
///     .with_commit(DbOp::Commit)
///     .with_rollback(DbOp::Rollback)
///     .build();
/// ```
pub struct TransactionBuilder<T> {
    name: String,
    begin_op: Option<T>,
    body_dag: Option<Dag<T>>,
    commit_op: Option<T>,
    rollback_op: Option<T>,
    // Port configurations
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> TransactionBuilder<T> {
    /// Create a new transaction builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            begin_op: None,
            body_dag: None,
            commit_op: None,
            rollback_op: None,
            input_port_name: "input".to_string(),
            input_port_type: "Any".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "Any".to_string(),
        }
    }

    /// Set the begin operation.
    ///
    /// The begin operation initializes the transaction context.
    /// It should output a `txn_id` on port `"txn_id"`.
    pub fn with_begin(mut self, op: T) -> Self {
        self.begin_op = Some(op);
        self
    }

    /// Set the body subgraph.
    ///
    /// The body contains the main operations to execute within the transaction.
    /// It should output `"success": Bool` to indicate if the transaction should commit.
    pub fn with_body(mut self, dag: Dag<T>) -> Self {
        self.body_dag = Some(dag);
        self
    }

    /// Set the commit operation.
    ///
    /// The commit operation finalizes the transaction on success.
    /// It is guarded by `success == true`.
    pub fn with_commit(mut self, op: T) -> Self {
        self.commit_op = Some(op);
        self
    }

    /// Set the rollback operation.
    ///
    /// The rollback operation cleans up the transaction on failure.
    /// It is guarded by `success == false`.
    pub fn with_rollback(mut self, op: T) -> Self {
        self.rollback_op = Some(op);
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

    /// Build the transaction pattern as a SubDag node.
    ///
    /// # Panics
    ///
    /// Panics if any of the required operations (begin, body, commit, rollback) are not set.
    pub fn build(self) -> Node<T> {
        let begin_op = self.begin_op.expect("begin operation is required");
        let body_dag = self.body_dag.expect("body dag is required");
        let commit_op = self.commit_op.expect("commit operation is required");
        let rollback_op = self.rollback_op.expect("rollback operation is required");

        let mut dag = Dag::new();

        // Begin node: initialize transaction
        dag.add_node(Node::opaque(
            "begin",
            vec![Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str())],
            vec![
                Port::scalar("txn_id", "String"),
                Port::scalar(self.input_port_name.as_str(), self.input_port_type.as_str()),
            ],
            begin_op,
        ));

        // Body node: the main operations (as a nested subdag)
        dag.add_node(Node::subdag("body", body_dag));

        // Commit node: guarded by success == true
        dag.add_node(Node::opaque(
            "commit",
            vec![
                Port::scalar("txn_id", "String"),
                Port::guarded_with_cardinality(
                    "success",
                    "Bool",
                    Cardinality::One,
                    Guard::Eq(Value::Bool(true)),
                ),
            ],
            vec![Port::scalar("committed", "Bool")],
            commit_op,
        ));

        // Rollback node: guarded by success == false
        dag.add_node(Node::opaque(
            "rollback",
            vec![
                Port::scalar("txn_id", "String"),
                Port::guarded_with_cardinality(
                    "success",
                    "Bool",
                    Cardinality::One,
                    Guard::Eq(Value::Bool(false)),
                ),
            ],
            vec![Port::scalar("rolled_back", "Bool")],
            rollback_op,
        ));

        // Wire: begin -> body
        dag.add_edge(Edge::new("begin", "txn_id", "body", "txn_id"));
        dag.add_edge(Edge::new(
            "begin",
            self.input_port_name.as_str(),
            "body",
            self.input_port_name.as_str(),
        ));

        // Wire: body.success -> commit.success and rollback.success
        dag.add_edge(Edge::new("body", "success", "commit", "success"));
        dag.add_edge(Edge::new("body", "success", "rollback", "success"));

        // Wire: begin.txn_id -> commit.txn_id and rollback.txn_id
        dag.add_edge(Edge::new("begin", "txn_id", "commit", "txn_id"));
        dag.add_edge(Edge::new("begin", "txn_id", "rollback", "txn_id"));

        // Create the outer node with the subdag
        Node::subdag(self.name.as_str(), dag)
    }
}

/// Phase of a transaction operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionPhase {
    /// Initialize transaction (acquire locks, etc.)
    Begin,
    /// Execute main operations
    Body,
    /// Finalize on success
    Commit,
    /// Cleanup on failure
    Rollback,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[derive(Debug, Clone)]
    enum TestOp {
        Begin,
        Commit,
        Rollback,
        BodyOp,
    }

    fn empty_body_dag() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "op",
            vec![Port::scalar("txn_id", "String"), Port::scalar("input", "Any")],
            vec![Port::scalar("success", "Bool"), Port::scalar("output", "Any")],
            TestOp::BodyOp,
        ));
        dag
    }

    #[test]
    fn test_transaction_builder_creates_subdag() {
        let node = TransactionBuilder::new("test_txn")
            .with_begin(TestOp::Begin)
            .with_body(empty_body_dag())
            .with_commit(TestOp::Commit)
            .with_rollback(TestOp::Rollback)
            .build();

        assert_eq!(node.id.0, "test_txn");
        assert!(node.is_subdag());

        // Check inputs/outputs
        assert_eq!(node.inputs.len(), 1);
        assert!(node.inputs.iter().any(|p| p.name.0 == "input"));
        // Outputs: committed, output, rolled_back (all inner boundaries)
        assert!(node.outputs.iter().any(|p| p.name.0 == "output"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "committed"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "rolled_back"));
    }

    #[test]
    fn test_transaction_subdag_structure() {
        let node = TransactionBuilder::new("test")
            .with_begin(TestOp::Begin)
            .with_body(empty_body_dag())
            .with_commit(TestOp::Commit)
            .with_rollback(TestOp::Rollback)
            .build();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 4);

                // Check node names
                let node_names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_names.contains(&"begin"));
                assert!(node_names.contains(&"body"));
                assert!(node_names.contains(&"commit"));
                assert!(node_names.contains(&"rollback"));

                // Check commit has guard
                let commit_node = dag.get_node(&"commit".into()).unwrap();
                let success_port = commit_node.inputs.iter().find(|p| p.name.0 == "success").unwrap();
                assert!(success_port.guard.is_some());

                // Check rollback has guard
                let rollback_node = dag.get_node(&"rollback".into()).unwrap();
                let success_port = rollback_node.inputs.iter().find(|p| p.name.0 == "success").unwrap();
                assert!(success_port.guard.is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }

    // ============ Interface Validation Tests ============

    #[test]
    fn test_transaction_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = TransactionBuilder::new("txn")
            .with_begin(TestOp::Begin)
            .with_body(empty_body_dag())
            .with_commit(TestOp::Commit)
            .with_rollback(TestOp::Rollback)
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "transaction interface errors: {:?}", errors);
    }
}
