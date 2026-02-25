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
use crate::patterns::{validate_resource_inputs, ResourceInput};
use crate::types::Cardinality;

/// Builder for the loop pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```text
/// let loop_node = LoopBuilder::new("process_files")
///     .with_input("files", "String", Cardinality::ZERO_OR_MORE)
///     .with_body(body_dag)
///     .with_output("processed", "String")
///     .build();
/// ```
pub struct LoopBuilder<T> {
    name: String,
    body_dag: Option<Dag<T>>,
    resource_inputs: Vec<ResourceInput>,
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
            resource_inputs: Vec::new(),
            input_port_name: "input".to_string(),
            // Type is the element type; cardinality handles the collection semantics.
            // Previously this was "List" (dual encoding), but the port's cardinality
            // already expresses that this is a multi-valued port.
            input_port_type: "String".to_string(),
            input_cardinality: Cardinality::ZERO_OR_MORE,
            element_port_name: "element".to_string(),
            element_port_type: "String".to_string(),
            output_port_name: "output".to_string(),
            output_port_type: "String".to_string(),
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

    /// Declare a resource input that the body DAG requires.
    ///
    /// At build time, validates that the body DAG has a matching entrypoint.
    pub fn with_resource_input(mut self, ri: ResourceInput) -> Self {
        self.resource_inputs.push(ri);
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
        validate_resource_inputs(&self.name, &self.resource_inputs, &body_dag);

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
                Port::scalar("count", "Int"),
            ],
            T::from(PatternOp::LoopUnpack {
                input_port: self.input_port_name.clone(),
                element_port: self.element_port_name.clone(),
            }),
        ));

        // Body subdag: processes each element
        dag.add_node(Node::subdag("body", body_dag));

        // Pack node: collects results back into a list
        dag.add_node(Node::opaque(
            "pack",
            vec![
                Port::with_cardinality(
                    "result",
                    self.element_port_type.as_str(),
                    self.input_cardinality,
                ),
                Port::scalar("count", "Int"),
            ],
            vec![Port::with_cardinality(
                self.output_port_name.as_str(),
                self.output_port_type.as_str(),
                self.input_cardinality,
            )],
            T::from(PatternOp::LoopPack {
                output_port: self.output_port_name.clone(),
            }),
        ));

        // Wire the internal nodes
        dag.add_edge(Edge::new(
            "unpack",
            self.element_port_name.as_str(),
            "body",
            self.element_port_name.as_str(),
        ));
        dag.add_edge(Edge::new("body", "result", "pack", "result"));
        dag.add_edge(Edge::new("unpack", "count", "pack", "count"));

        // Create the outer node
        Node::subdag(self.name.as_str(), dag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;
    use crate::resource::AccessMode;

    type TestOp = PatternOp;

    fn make_loop_body() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![Port::scalar("element", "String")],
            vec![Port::scalar("result", "String")],
            PatternOp::LoopPack {
                output_port: "result".into(),
            },
        ));
        dag
    }

    #[test]
    fn test_loop_builder_creates_subdag() {
        let node = LoopBuilder::new("test_loop")
            .with_body(make_loop_body())
            .build();

        assert_eq!(node.id.0, "test_loop");
        assert!(node.is_subdag());

        // Check inputs/outputs by name (sorted alphabetically)
        assert!(node.inputs.iter().any(|p| p.name.0 == "input"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "output"));
    }

    #[test]
    fn test_loop_subdag_structure() {
        let node = LoopBuilder::new("test").with_body(make_loop_body()).build();

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
        let node = LoopBuilder::new("test")
            .with_input("items", "String", Cardinality::ONE_OR_MORE)
            .with_body(make_loop_body())
            .build();

        // OneOrMore should be preserved
        let items = node.inputs.iter().find(|p| p.name.0 == "items").unwrap();
        assert_eq!(items.cardinality, Cardinality::ONE_OR_MORE);
    }

    // ============ Interface Validation Tests ============

    #[test]
    fn test_loop_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = LoopBuilder::new("loop").with_body(make_loop_body()).build();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "loop interface errors: {:?}", errors);
    }

    // ============ Resource Input Tests ============

    fn make_loop_body_with_resource() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "transform",
            vec![
                Port::scalar("element", "String"),
                Port::resource("platform", "Platform", AccessMode::Read),
            ],
            vec![Port::scalar("result", "String")],
            PatternOp::LoopPack {
                output_port: "result".into(),
            },
        ));
        dag
    }

    #[test]
    fn test_loop_with_resource_input_validates() {
        use crate::patterns::ResourceInput;

        let node = LoopBuilder::new("test_loop")
            .with_body(make_loop_body_with_resource())
            .with_resource_input(ResourceInput::new("res:platform", "Platform"))
            .build();

        assert!(node.is_subdag());
        // The res:platform port should bubble up through auto-inference
        assert!(node.inputs.iter().any(|p| p.name.0 == "res:platform"));
    }

    #[test]
    fn test_loop_without_resource_input_still_works() {
        // Backward compat: body has res:platform but no with_resource_input() call
        let node = LoopBuilder::new("test_loop")
            .with_body(make_loop_body_with_resource())
            .build();

        assert!(node.is_subdag());
        // Still works via auto-inference
        assert!(node.inputs.iter().any(|p| p.name.0 == "res:platform"));
    }

    #[test]
    #[should_panic(expected = "has no matching entrypoint")]
    fn test_loop_resource_input_mismatch_panics() {
        use crate::patterns::ResourceInput;

        // Body doesn't have res:credential, so this should panic
        let _node = LoopBuilder::new("test_loop")
            .with_body(make_loop_body())
            .with_resource_input(ResourceInput::new("res:credential", "Credential"))
            .build();
    }
}
