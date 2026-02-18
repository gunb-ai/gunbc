//! Transport triplet helper for the common prepare → execute → parse pattern.
//!
//! Many DAG graphs use a three-node pattern for transport operations:
//!
//! 1. **Prepare** — pure node that builds a `TransportRequest`
//! 2. **Execute** — transport boundary that runs the request
//! 3. **Parse** — pure node that interprets the `TransportResponse`
//!
//! Each helper creates a **SubDag** node wrapping the three internal nodes,
//! so that the triplet appears as a single expandable unit in the DAG
//! topology viewer.
//!
//! Variants:
//!
//! - [`add_transport_triplet`]: Every request is executed unconditionally.
//! - [`add_skippable_transport_triplet`]: The prepare node may decide to skip,
//!   propagating `skip` and `skip_reason` through execute to parse.
//! - [`add_transport_triplet_named_with_passthrough`]: Explicit node names with
//!   passthrough ports from prepare to parse.

use crate::build::{optional, port};
use crate::builder::{BuilderError, DagBuilder, NodeRef};
use crate::dag::{Dag, Edge, Port};
use crate::node::Node;

/// Add a non-skippable transport triplet as a **SubDag**: prepare → execute → parse.
///
/// Creates an internal DAG with three opaque nodes wired together, then wraps
/// the entire thing in a `Node::subdag(name, ...)`.  The SubDag's interface is
/// auto-inferred:
///
///   - **Inputs**: `prepare_inputs` ∪ `execute_resource_inputs`
///   - **Outputs**: `parse_outputs`
///
/// Returns a [`NodeRef`] to the SubDag node.
#[allow(clippy::too_many_arguments)]
pub fn add_transport_triplet<T>(
    builder: &mut DagBuilder<T>,
    name: &str,
    prepare_inputs: Vec<Port>,
    execute_resource_inputs: Vec<Port>,
    parse_outputs: Vec<Port>,
    prepare_op: T,
    parse_op: T,
    transport_op: T,
    after: Option<&NodeRef<T>>,
) -> Result<NodeRef<T>, BuilderError> {
    let prepare_name = format!("prepare_{name}");
    let execute_name = format!("execute_{name}");
    let parse_name = format!("parse_{name}");

    // Build internal DAG ---------------------------------------------------
    let mut inner = Dag::new();

    inner.add_node(Node::opaque(
        prepare_name.as_str(),
        prepare_inputs,
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        prepare_op,
    ));

    let mut exec_inputs = vec![port("request", "TransportRequest"), port("skip", "Bool")];
    exec_inputs.extend(execute_resource_inputs);
    inner.add_node(Node::opaque(
        execute_name.as_str(),
        exec_inputs,
        vec![port("response", "TransportResponse")],
        transport_op,
    ));

    inner.add_node(Node::opaque(
        parse_name.as_str(),
        vec![port("response", "TransportResponse")],
        parse_outputs,
        parse_op,
    ));

    inner.add_edge(Edge::new(
        prepare_name.as_str(),
        "request",
        execute_name.as_str(),
        "request",
    ));
    inner.add_edge(Edge::new(
        prepare_name.as_str(),
        "skip",
        execute_name.as_str(),
        "skip",
    ));
    inner.add_edge(Edge::new(
        execute_name.as_str(),
        "response",
        parse_name.as_str(),
        "response",
    ));

    // Wrap & insert ---------------------------------------------------------
    let subdag = Node::subdag(name, inner);
    match after {
        None => builder.add_root_node(subdag),
        Some(dep) => builder.add_node_after(subdag, dep),
    }
}

/// Add a skippable transport triplet as a **SubDag**: prepare → execute → parse.
///
/// The execute node has the standard skippable shape:
///   inputs:  `[optional("request", …), port("skip", …)]`
///   outputs: `[optional("response", …), port("skip", …), optional("skip_reason", …)]`
///
/// Internal wiring:
///   - `prepare.request → execute.request`
///   - `prepare.skip → execute.skip`
///   - `execute.response → parse.response`
///   - `execute.skip → parse.skip`
///   - `prepare.skip_reason → parse.skip_reason` (bypasses execute)
///
/// Returns a [`NodeRef`] to the SubDag node.
#[allow(clippy::too_many_arguments)]
pub fn add_skippable_transport_triplet<T>(
    builder: &mut DagBuilder<T>,
    name: &str,
    prepare_inputs: Vec<Port>,
    execute_resource_inputs: Vec<Port>,
    parse_outputs: Vec<Port>,
    prepare_op: T,
    parse_op: T,
    transport_op: T,
    after: &NodeRef<T>,
) -> Result<NodeRef<T>, BuilderError> {
    let prepare_name = format!("prepare_{name}");
    let execute_name = format!("execute_{name}");
    let parse_name = format!("parse_{name}");

    // Build internal DAG ---------------------------------------------------
    let mut inner = Dag::new();

    inner.add_node(Node::opaque(
        prepare_name.as_str(),
        prepare_inputs,
        vec![
            optional("request", "TransportRequest"),
            port("skip", "Bool"),
            optional("skip_reason", "OptionalString"),
        ],
        prepare_op,
    ));

    let mut exec_inputs = vec![
        optional("request", "TransportRequest"),
        port("skip", "Bool"),
    ];
    exec_inputs.extend(execute_resource_inputs);
    inner.add_node(Node::opaque(
        execute_name.as_str(),
        exec_inputs,
        vec![
            optional("response", "TransportResponse"),
            port("skip", "Bool"),
            optional("skip_reason", "OptionalString"),
        ],
        transport_op,
    ));

    inner.add_node(Node::opaque(
        parse_name.as_str(),
        vec![
            optional("response", "TransportResponse"),
            port("skip", "Bool"),
            optional("skip_reason", "OptionalString"),
        ],
        parse_outputs,
        parse_op,
    ));

    inner.add_edge(Edge::new(
        prepare_name.as_str(),
        "request",
        execute_name.as_str(),
        "request",
    ));
    inner.add_edge(Edge::new(
        prepare_name.as_str(),
        "skip",
        execute_name.as_str(),
        "skip",
    ));
    inner.add_edge(Edge::new(
        execute_name.as_str(),
        "response",
        parse_name.as_str(),
        "response",
    ));
    inner.add_edge(Edge::new(
        execute_name.as_str(),
        "skip",
        parse_name.as_str(),
        "skip",
    ));
    inner.add_edge(Edge::new(
        prepare_name.as_str(),
        "skip_reason",
        parse_name.as_str(),
        "skip_reason",
    ));

    // Wrap & insert ---------------------------------------------------------
    let subdag = Node::subdag(name, inner);
    builder.add_node_after(subdag, after)
}

/// Add a non-skippable transport triplet with explicit node names and passthrough
/// ports, wrapped as a **SubDag**.
///
/// `passthrough` ports are added to prepare outputs and parse inputs, and are
/// wired internally (`prepare.<port> → parse.<port>`).
///
/// Returns a [`NodeRef`] to the SubDag node.
#[allow(clippy::too_many_arguments)]
pub fn add_transport_triplet_named_with_passthrough<T>(
    builder: &mut DagBuilder<T>,
    name: &str,
    prepare_name: &str,
    execute_name: &str,
    parse_name: &str,
    prepare_inputs: Vec<Port>,
    execute_resource_inputs: Vec<Port>,
    passthrough: Vec<Port>,
    parse_outputs: Vec<Port>,
    prepare_op: T,
    parse_op: T,
    transport_op: T,
    after: Option<&NodeRef<T>>,
) -> Result<NodeRef<T>, BuilderError> {
    // Build internal DAG ---------------------------------------------------
    let mut inner = Dag::new();

    let mut prepare_outputs = vec![port("request", "TransportRequest"), port("skip", "Bool")];
    prepare_outputs.extend(passthrough.clone());
    inner.add_node(Node::opaque(
        prepare_name,
        prepare_inputs,
        prepare_outputs,
        prepare_op,
    ));

    let mut exec_inputs = vec![port("request", "TransportRequest"), port("skip", "Bool")];
    exec_inputs.extend(execute_resource_inputs);
    inner.add_node(Node::opaque(
        execute_name,
        exec_inputs,
        vec![port("response", "TransportResponse")],
        transport_op,
    ));

    let mut parse_inputs = vec![port("response", "TransportResponse")];
    parse_inputs.extend(passthrough.clone());
    inner.add_node(Node::opaque(
        parse_name,
        parse_inputs,
        parse_outputs,
        parse_op,
    ));

    inner.add_edge(Edge::new(prepare_name, "request", execute_name, "request"));
    inner.add_edge(Edge::new(prepare_name, "skip", execute_name, "skip"));
    inner.add_edge(Edge::new(execute_name, "response", parse_name, "response"));
    for pt in &passthrough {
        inner.add_edge(Edge::new(
            prepare_name,
            pt.name.0.as_str(),
            parse_name,
            pt.name.0.as_str(),
        ));
    }

    // Wrap & insert ---------------------------------------------------------
    let subdag = Node::subdag(name, inner);
    match after {
        None => builder.add_root_node(subdag),
        Some(dep) => builder.add_node_after(subdag, dep),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[derive(Debug, Clone)]
    enum TestOp {
        Prepare,
        Execute,
        Parse,
    }

    #[test]
    fn test_non_skippable_triplet_creates_subdag() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        let trip = add_transport_triplet(
            &mut builder,
            "fetch",
            vec![port("url", "String")],
            vec![port("res:file", "FilesystemHandle")],
            vec![port("data", "String")],
            TestOp::Prepare,
            TestOp::Parse,
            TestOp::Execute,
            None,
        )
        .unwrap();

        // SubDag interface: inputs = [url, res:file], outputs = [data]
        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 1); // single SubDag node
        assert_eq!(dag.nodes[0].id.0, "fetch");
        assert!(dag.nodes[0].is_subdag());

        if let NodeBody::SubDag(ref inner) = dag.nodes[0].body {
            assert_eq!(inner.nodes.len(), 3);
            assert_eq!(inner.edges.len(), 3);
            assert!(inner.get_node(&"prepare_fetch".into()).is_some());
            assert!(inner.get_node(&"execute_fetch".into()).is_some());
            assert!(inner.get_node(&"parse_fetch".into()).is_some());
        } else {
            panic!("Expected SubDag");
        }

        // Check that the SubDag node's inferred ports are correct.
        let node = &dag.nodes[0];
        assert!(
            node.inputs.iter().any(|p| p.name.0 == "url"),
            "SubDag should expose prepare's 'url' input"
        );
        assert!(
            node.inputs.iter().any(|p| p.name.0 == "res:file"),
            "SubDag should expose execute's 'res:file' input"
        );
        assert!(
            node.outputs.iter().any(|p| p.name.0 == "data"),
            "SubDag should expose parse's 'data' output"
        );

        // NodeRef should support in_port / out for wiring.
        let _ = trip.in_port("url");
        let _ = trip.out("data");
    }

    #[test]
    fn test_skippable_triplet_creates_subdag() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        let root = builder
            .add_root_node(Node::opaque(
                "root",
                vec![],
                vec![port("ok", "Bool")],
                TestOp::Prepare,
            ))
            .unwrap();

        let _trip = add_skippable_transport_triplet(
            &mut builder,
            "step",
            vec![port("ok", "Bool")],
            vec![],
            vec![port("result", "Bool")],
            TestOp::Prepare,
            TestOp::Parse,
            TestOp::Execute,
            &root,
        )
        .unwrap();

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 2); // root + SubDag

        let step = dag.get_node(&"step".into()).unwrap();
        assert!(step.is_subdag());

        if let NodeBody::SubDag(ref inner) = step.body {
            assert_eq!(inner.nodes.len(), 3);
            assert_eq!(inner.edges.len(), 5); // request, skip, response, skip(2), skip_reason
        } else {
            panic!("Expected SubDag");
        }
    }

    #[test]
    fn test_named_with_passthrough_creates_subdag() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        let trip = add_transport_triplet_named_with_passthrough(
            &mut builder,
            "manifest",
            "prepare_manifest",
            "execute_manifest",
            "parse_manifest",
            vec![port("path", "String")],
            vec![],
            vec![port("path", "String")], // passthrough
            vec![port("ok", "Bool")],
            TestOp::Prepare,
            TestOp::Parse,
            TestOp::Execute,
            None,
        )
        .unwrap();

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.nodes[0].id.0, "manifest");
        assert!(dag.nodes[0].is_subdag());

        if let NodeBody::SubDag(ref inner) = dag.nodes[0].body {
            assert_eq!(inner.nodes.len(), 3);
            // request + skip + response + passthrough = 4 edges
            assert_eq!(inner.edges.len(), 4);
        } else {
            panic!("Expected SubDag");
        }

        // Passthrough is internal; SubDag interface should NOT expose it.
        // Inputs: [path], Outputs: [ok]
        let node = &dag.nodes[0];
        assert_eq!(
            node.inputs.iter().filter(|p| p.name.0 == "path").count(),
            1,
            "Only one 'path' input (prepare's)"
        );
        assert!(
            node.outputs.iter().any(|p| p.name.0 == "ok"),
            "Should expose parse's 'ok' output"
        );

        let _ = trip.in_port("path");
        let _ = trip.out("ok");
    }
}
