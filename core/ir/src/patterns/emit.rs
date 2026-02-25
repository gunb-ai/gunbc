//! Emit pattern: Prepare → Format → Hash → Compare → Write → Record.
//!
//! The emit pattern models the full emission pipeline as a SubDag:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                          Emit                                   │
//! │  ┌─────────┐  ┌────────┐  ┌──────┐  ┌─────────┐  ┌─────────┐ │
//! │  │ Prepare │─▶│ Format │─▶│ Hash │─▶│ Compare │─▶│  Write  │ │
//! │  └─────────┘  └────────┘  └──────┘  └─────────┘  └─────────┘ │
//! │       │                                   │            │       │
//! │       │                                   │  (guard)   │       │
//! │       │                                   │ changed=T  │       │
//! │       │                                   ▼            ▼       │
//! │       │                              ┌─────────┐               │
//! │       └─────────────────────────────▶│ Record  │  (boundary)   │
//! │                                      └─────────┘               │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! This makes rendering a first-class DAG citizen with:
//! - Skip-if-unchanged avoiding spurious mtime bumps
//! - Content hash enabling staleness detection
//! - Manifest recording (path, content_hash) for CI verification
//!
//! The Write node has a guard that only executes if Compare returns `changed=true`.

use crate::dag::{Dag, Edge, Guard, Port};
use crate::node::Node;
use crate::types::Cardinality;
use crate::value::Value;

/// Builder for the emit pattern.
///
/// # Type Parameters
///
/// - `T`: The operation type used in the DAG
///
/// # Example
///
/// ```text
/// let emit = EmitBuilder::new("makefile")
///     .with_prepare(MyOp::PrepareMakefile)
///     .with_format(MyOp::FormatMakefile)
///     .with_hash(MyOp::HashContent)
///     .with_compare(MyOp::CompareFile)
///     .with_write(MyOp::WriteFile)
///     .with_record(MyOp::RecordManifest)
///     .build();
/// ```
pub struct EmitBuilder<T> {
    name: String,
    prepare_op: Option<T>,
    format_op: Option<T>,
    hash_op: Option<T>,
    compare_op: Option<T>,
    write_op: Option<T>,
    record_op: Option<T>,
    // Port configurations
    content_port_name: String,
    content_port_type: String,
    path_port_name: String,
    path_port_type: String,
}

impl<T: Clone> EmitBuilder<T> {
    /// Create a new emit builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prepare_op: None,
            format_op: None,
            hash_op: None,
            compare_op: None,
            write_op: None,
            record_op: None,
            content_port_name: "content".to_string(),
            content_port_type: "String".to_string(),
            path_port_name: "path".to_string(),
            path_port_type: "String".to_string(),
        }
    }

    /// Set the prepare operation (takes input, produces content IR).
    pub fn with_prepare(mut self, op: T) -> Self {
        self.prepare_op = Some(op);
        self
    }

    /// Set the format operation (takes content IR, produces rendered string).
    pub fn with_format(mut self, op: T) -> Self {
        self.format_op = Some(op);
        self
    }

    /// Set the hash operation (takes rendered string, produces ContentHash).
    pub fn with_hash(mut self, op: T) -> Self {
        self.hash_op = Some(op);
        self
    }

    /// Set the compare operation (takes hash + path, reads existing file hash).
    pub fn with_compare(mut self, op: T) -> Self {
        self.compare_op = Some(op);
        self
    }

    /// Set the write operation (guarded — only writes if hash differs).
    pub fn with_write(mut self, op: T) -> Self {
        self.write_op = Some(op);
        self
    }

    /// Set the record operation (records path + content_hash to manifest).
    pub fn with_record(mut self, op: T) -> Self {
        self.record_op = Some(op);
        self
    }

    /// Configure the content port name and type.
    pub fn with_content_port(
        mut self,
        name: impl Into<String>,
        type_id: impl Into<String>,
    ) -> Self {
        self.content_port_name = name.into();
        self.content_port_type = type_id.into();
        self
    }

    /// Configure the path port name and type.
    pub fn with_path_port(mut self, name: impl Into<String>, type_id: impl Into<String>) -> Self {
        self.path_port_name = name.into();
        self.path_port_type = type_id.into();
        self
    }

    /// Build the emit pattern as a SubDag node.
    ///
    /// Creates 6 internal nodes:
    /// 1. **prepare**: Takes input, produces content IR (pure)
    /// 2. **format**: Takes content IR, produces rendered string (pure)
    /// 3. **hash**: Takes rendered string, computes content hash (pure)
    /// 4. **compare**: Takes hash + path, reads existing file hash (boundary read)
    /// 5. **write**: Guarded — only writes if changed=true (boundary write)
    /// 6. **record**: Records (path, content_hash) to manifest output port
    ///
    /// # Panics
    ///
    /// Panics if any of the six operations are not set.
    pub fn build(self) -> Node<T> {
        let prepare_op = self.prepare_op.expect("prepare operation is required");
        let format_op = self.format_op.expect("format operation is required");
        let hash_op = self.hash_op.expect("hash operation is required");
        let compare_op = self.compare_op.expect("compare operation is required");
        let write_op = self.write_op.expect("write operation is required");
        let record_op = self.record_op.expect("record operation is required");

        let mut dag = Dag::new();

        // 1. Prepare: input → content IR
        dag.add_node(Node::opaque(
            "prepare",
            vec![Port::scalar(
                self.content_port_name.as_str(),
                self.content_port_type.as_str(),
            )],
            vec![Port::scalar("content_ir", "String")],
            prepare_op,
        ));

        // 2. Format: content IR → rendered string
        dag.add_node(Node::opaque(
            "format",
            vec![Port::scalar("content_ir", "String")],
            vec![Port::scalar("rendered", "String")],
            format_op,
        ));

        // 3. Hash: rendered string → content hash
        dag.add_node(Node::opaque(
            "hash",
            vec![Port::scalar("rendered", "String")],
            vec![Port::scalar("content_hash", "String")],
            hash_op,
        ));

        // 4. Compare: hash + path → changed (bool)
        dag.add_node(Node::opaque(
            "compare",
            vec![
                Port::scalar("content_hash", "String"),
                Port::scalar(self.path_port_name.as_str(), self.path_port_type.as_str()),
            ],
            vec![Port::scalar("changed", "Bool")],
            compare_op,
        ));

        // 5. Write: guarded by changed=true, writes rendered content to path
        dag.add_node(Node::opaque(
            "write",
            vec![
                Port::scalar("rendered", "String"),
                Port::scalar(self.path_port_name.as_str(), self.path_port_type.as_str()),
                Port::guarded_with_cardinality(
                    "changed",
                    "Bool",
                    Cardinality::ONE,
                    Guard::Eq(Value::Bool(true)),
                ),
            ],
            vec![],
            write_op,
        ));

        // 6. Record: records path + content_hash to manifest (always runs)
        dag.add_node(Node::opaque(
            "record",
            vec![
                Port::scalar(self.path_port_name.as_str(), self.path_port_type.as_str()),
                Port::scalar("content_hash", "String"),
            ],
            vec![
                Port::scalar("manifest_path", "String"),
                Port::scalar("manifest_hash", "String"),
            ],
            record_op,
        ));

        // Wire: prepare.content_ir → format.content_ir
        dag.add_edge(Edge::new("prepare", "content_ir", "format", "content_ir"));
        // Wire: format.rendered → hash.rendered
        dag.add_edge(Edge::new("format", "rendered", "hash", "rendered"));
        // Wire: hash.content_hash → compare.content_hash
        dag.add_edge(Edge::new("hash", "content_hash", "compare", "content_hash"));
        // Wire: compare.changed → write.changed (for guard)
        dag.add_edge(Edge::new("compare", "changed", "write", "changed"));
        // Wire: format.rendered → write.rendered
        dag.add_edge(Edge::new("format", "rendered", "write", "rendered"));
        // Wire: hash.content_hash → record.content_hash
        dag.add_edge(Edge::new("hash", "content_hash", "record", "content_hash"));

        Node::subdag(self.name.as_str(), dag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[derive(Debug, Clone)]
    enum TestOp {
        Prepare,
        Format,
        Hash,
        Compare,
        Write,
        Record,
    }

    fn build_test_emit() -> Node<TestOp> {
        EmitBuilder::new("test_emit")
            .with_prepare(TestOp::Prepare)
            .with_format(TestOp::Format)
            .with_hash(TestOp::Hash)
            .with_compare(TestOp::Compare)
            .with_write(TestOp::Write)
            .with_record(TestOp::Record)
            .build()
    }

    #[test]
    fn test_emit_builder_creates_subdag() {
        let node = build_test_emit();

        assert_eq!(node.id.0, "test_emit");
        assert!(node.is_subdag());

        // Check that the outer node has the expected interface:
        // Inputs: content (from prepare) + path (from compare/write/record)
        assert!(
            !node.inputs.is_empty(),
            "emit node should have inputs inferred from inner DAG"
        );
        // Outputs: manifest_path + manifest_hash (from record)
        assert!(
            !node.outputs.is_empty(),
            "emit node should have outputs inferred from inner DAG"
        );
    }

    #[test]
    fn test_emit_subdag_structure() {
        let node = build_test_emit();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 6);
                assert_eq!(dag.edges.len(), 6);

                let node_names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_names.contains(&"prepare"));
                assert!(node_names.contains(&"format"));
                assert!(node_names.contains(&"hash"));
                assert!(node_names.contains(&"compare"));
                assert!(node_names.contains(&"write"));
                assert!(node_names.contains(&"record"));

                // Check write has guard on changed
                let write_node = dag.get_node(&"write".into()).unwrap();
                let changed_port = write_node
                    .inputs
                    .iter()
                    .find(|p| p.name.0 == "changed")
                    .unwrap();
                assert!(changed_port.guard.is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_emit_interface_validates() {
        use crate::validate::validate_subdag_interfaces;

        let node = build_test_emit();

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(node);

        let errors = validate_subdag_interfaces(&dag);
        assert!(errors.is_empty(), "emit interface errors: {:?}", errors);
    }

    #[test]
    fn test_emit_custom_ports() {
        let node = EmitBuilder::new("custom")
            .with_prepare(TestOp::Prepare)
            .with_format(TestOp::Format)
            .with_hash(TestOp::Hash)
            .with_compare(TestOp::Compare)
            .with_write(TestOp::Write)
            .with_record(TestOp::Record)
            .with_content_port("source_code", "SourceIR")
            .with_path_port("output_file", "FilePath")
            .build();

        // The outer node should have the custom port names inferred
        let has_source = node.inputs.iter().any(|p| p.name.0 == "source_code");
        let has_output_file = node.inputs.iter().any(|p| p.name.0 == "output_file");
        assert!(has_source, "should have custom content port");
        assert!(has_output_file, "should have custom path port");
    }
}
