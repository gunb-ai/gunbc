//! Branch pattern: Conditional execution based on boolean condition.
//!
//! The branch pattern implements if/else logic using guarded ports:
//!
//! 1. **Condition**: A boolean value that determines which branch executes
//! 2. **True Branch**: A DAG that executes when condition is true
//! 3. **False Branch**: A DAG that executes when condition is false
//! 4. **Merge**: Combines the results from whichever branch executed
//!
//! ```text
//! ┌────────────────────────────────────────────────────┐
//! │                      Branch                         │
//! │                  ┌───────────┐                      │
//! │          ┌──────▶│ True DAG  │──────┐              │
//! │          │       └───────────┘      │              │
//! │  condition       (guard: true)      ▼              │
//! │     │                           ┌───────┐          │
//! │     │            ┌───────────┐  │ Merge │─▶ output │
//! │     └───────────▶│ False DAG │──┘       │          │
//! │                  └───────────┘          │          │
//! │                  (guard: false)         │          │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! Only one branch executes based on the condition value.

use crate::dag::{Dag, Edge, Guard, Port};
use crate::node::Node;
use crate::patterns::PatternOp;
use crate::patterns::{validate_resource_inputs, ResourceInput};
use crate::types::Cardinality;
use crate::value::Value;

/// Builder for the branch pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```text
/// let branch_node = BranchBuilder::new("check_and_process")
///     .with_condition("is_valid", "Bool")
///     .with_true_branch(valid_dag)
///     .with_false_branch(invalid_dag)
///     .with_output("result", "String")
///     .build();
/// ```
pub struct BranchBuilder<T> {
    name: String,
    true_dag: Option<Dag<T>>,
    false_dag: Option<Dag<T>>,
    resource_inputs: Vec<ResourceInput>,
    // Port configurations
    condition_port_name: String,
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> BranchBuilder<T> {
    /// Create a new branch builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            true_dag: None,
            false_dag: None,
            resource_inputs: Vec::new(),
            condition_port_name: "condition".to_string(),
            input_port_name: "input".to_string(),
            input_port_type: "String".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "String".to_string(),
        }
    }

    /// Set the DAG to execute when condition is true.
    pub fn with_true_branch(mut self, dag: Dag<T>) -> Self {
        self.true_dag = Some(dag);
        self
    }

    /// Set the DAG to execute when condition is false.
    pub fn with_false_branch(mut self, dag: Dag<T>) -> Self {
        self.false_dag = Some(dag);
        self
    }

    /// Configure the condition port name.
    pub fn with_condition(mut self, name: impl Into<String>) -> Self {
        self.condition_port_name = name.into();
        self
    }

    /// Configure the input port (data passed to both branches).
    pub fn with_input(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.input_port_name = name.into();
        self.input_port_type = type_id.into();
        self
    }

    /// Configure the output port (result from whichever branch executes).
    pub fn with_output(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.output_port_name = name.into();
        self.output_port_type = type_id.into();
        self
    }

    /// Declare a resource input that both branch DAGs require.
    ///
    /// At build time, validates that both true and false branch DAGs have matching entrypoints.
    pub fn with_resource_input(mut self, ri: ResourceInput) -> Self {
        self.resource_inputs.push(ri);
        self
    }

    /// Build the branch pattern as a SubDag node.
    ///
    /// # Panics
    ///
    /// Panics if both branches are not set.
    pub fn build(self) -> Node<T>
    where
        T: From<PatternOp>,
    {
        let true_dag = self.true_dag.expect("true branch DAG is required");
        let false_dag = self.false_dag.expect("false branch DAG is required");
        validate_resource_inputs(
            &format!("{}:true_branch", self.name),
            &self.resource_inputs,
            &true_dag,
        );
        validate_resource_inputs(
            &format!("{}:false_branch", self.name),
            &self.resource_inputs,
            &false_dag,
        );

        let mut dag = Dag::new();

        // True branch: guarded by condition == true
        dag.add_node(Node::subdag("true_branch", true_dag).with_input_guard(
            self.condition_port_name.as_str(),
            Guard::Eq(Value::Bool(true)),
        ));

        // False branch: guarded by condition == false
        dag.add_node(Node::subdag("false_branch", false_dag).with_input_guard(
            self.condition_port_name.as_str(),
            Guard::Eq(Value::Bool(false)),
        ));

        // Merge node: collects result from whichever branch executed
        dag.add_node(Node::opaque(
            "merge",
            vec![
                Port::optional("true_result", self.output_port_type.as_str()),
                Port::optional("false_result", self.output_port_type.as_str()),
            ],
            vec![
                Port::scalar(
                    self.output_port_name.as_str(),
                    self.output_port_type.as_str(),
                ),
                Port::scalar("branch_taken", "String"),
            ],
            T::from(PatternOp::BranchMerge {
                output_port: self.output_port_name.clone(),
            }),
        ));

        // Wire branches to merge
        dag.add_edge(Edge::new("true_branch", "result", "merge", "true_result"));
        dag.add_edge(Edge::new("false_branch", "result", "merge", "false_result"));

        // Create outer node
        Node::subdag(self.name.as_str(), dag)
    }
}

/// Convenience builder for simple if-then (no else) pattern.
pub struct IfBuilder<T> {
    name: String,
    then_dag: Option<Dag<T>>,
    resource_inputs: Vec<ResourceInput>,
    condition_port_name: String,
    input_port_name: String,
    input_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> IfBuilder<T> {
    /// Create a new if builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            then_dag: None,
            resource_inputs: Vec::new(),
            condition_port_name: "condition".to_string(),
            input_port_name: "input".to_string(),
            input_port_type: "String".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "String".to_string(),
        }
    }

    /// Set the DAG to execute when condition is true.
    pub fn with_then(mut self, dag: Dag<T>) -> Self {
        self.then_dag = Some(dag);
        self
    }

    /// Configure the condition port name.
    pub fn with_condition(mut self, name: impl Into<String>) -> Self {
        self.condition_port_name = name.into();
        self
    }

    /// Configure the input port.
    pub fn with_input(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.input_port_name = name.into();
        self.input_port_type = type_id.into();
        self
    }

    /// Configure the output port.
    pub fn with_output(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.output_port_name = name.into();
        self.output_port_type = type_id.into();
        self
    }

    /// Declare a resource input that the then DAG requires.
    pub fn with_resource_input(mut self, ri: ResourceInput) -> Self {
        self.resource_inputs.push(ri);
        self
    }

    /// Build the if pattern as a SubDag node.
    ///
    /// If condition is false, the output will be `Value::Skipped`.
    pub fn build(self) -> Node<T> {
        let then_dag = self.then_dag.expect("then DAG is required");
        validate_resource_inputs(&self.name, &self.resource_inputs, &then_dag);

        let mut dag = Dag::new();

        // Then branch: guarded by condition == true
        dag.add_node(Node::subdag("then_branch", then_dag).with_input_guard(
            self.condition_port_name.as_str(),
            Guard::Eq(Value::Bool(true)),
        ));

        // Create outer node — output is optional because condition may be false
        Node::subdag(self.name.as_str(), dag)
            .with_output_cardinality(self.output_port_name.as_str(), Cardinality::ZERO_OR_ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    type TestOp = PatternOp;

    fn make_branch_body() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "op",
            vec![
                Port::scalar("input", "String"),
                Port::scalar("condition", "Bool"),
            ],
            vec![Port::scalar("result", "String")],
            PatternOp::BranchMerge {
                output_port: "result".into(),
            },
        ));
        dag
    }

    fn make_if_body() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "op",
            vec![
                Port::scalar("input", "String"),
                Port::scalar("condition", "Bool"),
            ],
            vec![Port::scalar("output", "String")],
            PatternOp::BranchMerge {
                output_port: "output".into(),
            },
        ));
        dag
    }

    #[test]
    fn test_branch_builder_creates_subdag() {
        let node = BranchBuilder::new("test_branch")
            .with_true_branch(make_branch_body())
            .with_false_branch(make_branch_body())
            .build();

        assert_eq!(node.id.0, "test_branch");
        assert!(node.is_subdag());

        // Inputs are sorted alphabetically: condition, input
        assert!(node.inputs.iter().any(|p| p.name.0 == "condition"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "input"));

        // Outputs: branch_taken, output (sorted)
        assert!(node.outputs.iter().any(|p| p.name.0 == "output"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "branch_taken"));
    }

    #[test]
    fn test_branch_subdag_structure() {
        let node = BranchBuilder::new("test")
            .with_true_branch(make_branch_body())
            .with_false_branch(make_branch_body())
            .build();

        match &node.body {
            NodeBody::SubDag(dag, _) => {
                assert_eq!(dag.nodes.len(), 3);
                assert_eq!(dag.edges.len(), 2);

                // Check node names
                let node_names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_names.contains(&"true_branch"));
                assert!(node_names.contains(&"false_branch"));
                assert!(node_names.contains(&"merge"));

                // Check guards on branches
                let true_node = dag.get_node(&"true_branch".into()).unwrap();
                let cond_port = true_node
                    .inputs
                    .iter()
                    .find(|p| p.name.0 == "condition")
                    .unwrap();
                assert!(cond_port.guard.is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_if_builder() {
        let node = IfBuilder::new("test_if").with_then(make_if_body()).build();

        assert_eq!(node.id.0, "test_if");
        assert!(node.is_subdag());

        // Output should be optional (may be skipped)
        let output = node.outputs.iter().find(|p| p.name.0 == "output").unwrap();
        assert_eq!(output.cardinality, Cardinality::ZERO_OR_ONE);
    }

    #[test]
    fn test_branch_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = BranchBuilder::new("branch")
            .with_true_branch(make_branch_body())
            .with_false_branch(make_branch_body())
            .build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "branch interface errors: {:?}", errors);
    }

    #[test]
    fn test_if_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let mut then_dag: Dag<TestOp> = Dag::new();
        then_dag.add_node(Node::opaque(
            "op",
            vec![
                Port::scalar("input", "String"),
                Port::scalar("condition", "Bool"),
            ],
            vec![Port::scalar("output", "String")],
            PatternOp::BranchMerge {
                output_port: "output".into(),
            },
        ));

        let node = IfBuilder::new("test_if").with_then(then_dag).build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "if interface errors: {:?}", errors);
    }
}
