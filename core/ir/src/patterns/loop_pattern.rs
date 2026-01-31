//! Loop pattern: Iterate over a collection, applying a body DAG to each element.
//!
//! The loop pattern processes collections by applying a transformation to each element:
//!
//! 1. **Input**: A collection (list) to iterate over
//! 2. **Body**: A DAG that processes each element
//! 3. **Output**: A transformed collection
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │                    Loop                      │
//! │  ┌──────────┐    ┌──────────┐    ┌──────┐  │
//! │  │ Unpack   │───▶│   Body   │───▶│ Pack │  │
//! │  └──────────┘    └──────────┘    └──────┘  │
//! │      │               │ (n)           │      │
//! │  input: List     per element    output: List│
//! └─────────────────────────────────────────────┘
//! ```
//!
//! The Loop pattern preserves cardinality:
//! - `ZeroOrMore` input → `ZeroOrMore` output
//! - `OneOrMore` input → `OneOrMore` output

use crate::dag::{Dag, Edge, Port};
use crate::node::Node;
use crate::patterns::PatternOp;
use crate::types::Cardinality;

/// Builder for the loop pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```ignore
/// let loop_node = LoopBuilder::new("process_files")
///     .with_input("files", "StrList")
///     .with_body(body_dag)
///     .with_output("processed", "StrList")
///     .build();
/// ```
pub struct LoopBuilder<T> {
    name: String,
    body_dag: Option<Dag<T>>,
    // Port configurations
    input_port_name: String,
    input_port_type: String,
    input_cardinality: Cardinality,
    element_port_name: String,
    element_port_type: String,
    output_port_name: String,
    output_port_type: String,
}

impl<T: Clone> LoopBuilder<T> {
    /// Create a new loop builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            body_dag: None,
            input_port_name: "input".to_string(),
            input_port_type: "StrList".to_string(),
            input_cardinality: Cardinality::ZeroOrMore,
            element_port_name: "element".to_string(),
            element_port_type: "String".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "StrList".to_string(),
        }
    }

    /// Set the body DAG that processes each element.
    ///
    /// The body DAG should have:
    /// - An input port matching `element_port_name` with type `element_port_type`
    /// - An output port named "result"
    pub fn with_body(mut self, dag: Dag<T>) -> Self {
        self.body_dag = Some(dag);
        self
    }

    /// Configure the input port (the collection to iterate).
    pub fn with_input(
        mut self,
        name: impl Into<String>,
        type_id: impl Into<String>,
        cardinality: Cardinality,
    ) -> Self {
        self.input_port_name = name.into();
        self.input_port_type = type_id.into();
        self.input_cardinality = cardinality;
        self
    }

    /// Configure the element port (what each iteration receives).
    pub fn with_element(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.element_port_name = name.into();
        self.element_port_type = type_id.into();
        self
    }

    /// Configure the output port (the collected results).
    pub fn with_output(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.output_port_name = name.into();
        self.output_port_type = type_id.into();
        self
    }

    /// Build the loop pattern as a SubDag node.
    ///
    /// Note: The actual iteration is handled at execution time by the executor.
    /// This pattern creates a structural representation that the executor
    /// interprets as "apply body to each element".
    pub fn build(self) -> Node<T>
    where
        T: From<PatternOp>,
    {
        let body_dag = self.body_dag.expect("body DAG is required");

        let mut dag = Dag::new();

        // Unpack node: receives the list, provides iteration control
        dag.add_node(Node::opaque(
            "unpack",
            vec![Port::with_cardinality(
                self.input_port_name.as_str(),
                self.input_port_type.as_str(),
                self.input_cardinality,
            )],
            vec![
                Port::with_cardinality(
                    self.element_port_name.as_str(),
                    self.element_port_type.as_str(),
                    self.input_cardinality,
                ),
                Port::scalar("index", "Int"),
                Port::scalar("count", "Int"),
            ],
            T::from(PatternOp::LoopUnpack {
                input_port: self.input_port_name.clone(),
                element_port: self.element_port_name.clone(),
            }),
        ));

        // Body subdag: processes each element
        let body_inputs = vec![Port::scalar(self.element_port_name.as_str(), self.element_port_type.as_str())];
        let body_outputs = vec![Port::scalar("result", self.element_port_type.as_str())];
        dag.add_node(Node::subdag("body", body_inputs, body_outputs, body_dag));

        // Pack node: collects results back into a list
        dag.add_node(Node::opaque(
            "pack",
            vec![
                Port::with_cardinality("result", self.element_port_type.as_str(), self.input_cardinality),
                Port::scalar("count", "Int"),
            ],
            vec![
                Port::with_cardinality(
                    self.output_port_name.as_str(),
                    self.output_port_type.as_str(),
                    self.input_cardinality,
                ),
                Port::scalar("iterations", "Int"),
            ],
            T::from(PatternOp::LoopPack {
                output_port: self.output_port_name.clone(),
            }),
        ));

        // Wire the internal nodes
        dag.add_edge(Edge::new("unpack", self.element_port_name.as_str(), "body", self.element_port_name.as_str()));
        dag.add_edge(Edge::new("body", "result", "pack", "result"));
        dag.add_edge(Edge::new("unpack", "count", "pack", "count"));

        // Create the outer node
        Node::subdag(
            self.name.as_str(),
            vec![Port::with_cardinality(
                self.input_port_name.as_str(),
                self.input_port_type.as_str(),
                self.input_cardinality,
            )],
            vec![
                Port::with_cardinality(
                    self.output_port_name.as_str(),
                    self.output_port_type.as_str(),
                    self.input_cardinality,
                ),
                Port::scalar("iterations", "Int"),
            ],
            dag,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    type TestOp = PatternOp;

    #[test]
    fn test_loop_builder_creates_subdag() {
        let body_dag: Dag<TestOp> = Dag::new();

        let node = LoopBuilder::new("test_loop")
            .with_body(body_dag)
            .build();

        assert_eq!(node.id.0, "test_loop");
        assert!(node.is_subdag());

        // Check inputs/outputs
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.inputs[0].name.0, "input");
        assert_eq!(node.inputs[0].cardinality, Cardinality::ZeroOrMore);

        assert_eq!(node.outputs.len(), 2);
        assert_eq!(node.outputs[0].name.0, "output");
        assert_eq!(node.outputs[1].name.0, "iterations");
    }

    #[test]
    fn test_loop_subdag_structure() {
        let body_dag: Dag<TestOp> = Dag::new();

        let node = LoopBuilder::new("test")
            .with_body(body_dag)
            .build();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 3);
                assert_eq!(dag.edges.len(), 3);

                // Check node names
                let node_names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_names.contains(&"unpack"));
                assert!(node_names.contains(&"body"));
                assert!(node_names.contains(&"pack"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_loop_preserves_cardinality() {
        let body_dag: Dag<TestOp> = Dag::new();

        let node = LoopBuilder::new("test")
            .with_input("items", "StrList", Cardinality::OneOrMore)
            .with_body(body_dag)
            .build();

        // OneOrMore should be preserved
        assert_eq!(node.inputs[0].cardinality, Cardinality::OneOrMore);
        assert_eq!(node.outputs[0].cardinality, Cardinality::OneOrMore);
    }
}
