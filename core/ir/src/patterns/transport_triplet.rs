//! Transport triplet helper for the common prepare → execute → parse pattern.
//!
//! Many DAG graphs use a three-node pattern for transport operations:
//!
//! 1. **Prepare** — pure node that builds a `TransportRequest`
//! 2. **Execute** — transport boundary that runs the request
//! 3. **Parse** — pure node that interprets the `TransportResponse`
//!
//! This module provides helpers that stamp out the full triplet (nodes + internal
//! wiring) in one call, eliminating ~40 lines of boilerplate per triplet.
//!
//! Two variants exist:
//!
//! - [`add_skippable_transport_triplet`]: The prepare node may decide to skip,
//!   propagating `skip` and `skip_reason` through execute to parse.
//! - [`add_transport_triplet`]: Every request is executed unconditionally.

use crate::build::{optional, port};
use crate::builder::{BuilderError, DagBuilder, NodeRef};
use crate::dag::Port;
use crate::node::Node;

/// References to the three nodes created by a transport triplet.
pub struct TransportTriplet<T> {
    pub prepare: NodeRef<T>,
    pub execute: NodeRef<T>,
    pub parse: NodeRef<T>,
}

/// Add a skippable transport triplet: prepare → execute → parse.
///
/// The execute node has the standard skippable shape:
///   inputs:  `[optional("request", "TransportRequest"), port("skip", "Bool")]`
///   outputs: `[optional("response", "TransportResponse"), port("skip", "Bool"),
///             optional("skip_reason", "String")]`
///
/// Standard wiring included:
///   - `prepare.request → execute.request`
///   - `prepare.skip → execute.skip`
///   - `execute.response → parse.response`
///   - `execute.skip → parse.skip`
///   - `prepare.skip_reason → parse.skip_reason` (bypasses execute)
///
/// `prepare_inputs` are the *extra* inputs for the prepare node — the helper
/// automatically appends the standard outputs (`request`, `skip`, `skip_reason`).
///
/// `parse_outputs` are the *extra* outputs for the parse node — the helper
/// automatically prepends the standard inputs (`response`, `skip`, `skip_reason`).
#[allow(clippy::too_many_arguments)]
pub fn add_skippable_transport_triplet<T>(
    builder: &mut DagBuilder<T>,
    name: &str,
    prepare_inputs: Vec<Port>,
    parse_outputs: Vec<Port>,
    prepare_op: T,
    parse_op: T,
    transport_op: T,
    after: &NodeRef<T>,
) -> Result<TransportTriplet<T>, BuilderError> {
    let prepare_name = format!("prepare_{name}");
    let execute_name = format!("execute_{name}");
    let parse_name = format!("parse_{name}");

    // Prepare node: caller inputs + standard transport outputs
    let prepare = builder.add_node_after(
        Node::opaque(
            prepare_name.as_str(),
            prepare_inputs,
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            prepare_op,
        ),
        after,
    )?;

    // Execute node: standard skippable transport shape
    let execute = builder.add_node_after(
        Node::opaque(
            execute_name.as_str(),
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            transport_op,
        ),
        &prepare,
    )?;

    // Parse node: standard transport inputs + caller outputs
    let parse_inputs = vec![
        optional("response", "TransportResponse"),
        port("skip", "Bool"),
        optional("skip_reason", "String"),
    ];

    let parse = builder.add_node_after(
        Node::opaque(parse_name.as_str(), parse_inputs, parse_outputs, parse_op),
        &execute,
    )?;

    // Wire up internal edges
    builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
    builder.add_edge(prepare.out("skip"), execute.in_port("skip"))?;
    builder.add_edge(execute.out("response"), parse.in_port("response"))?;
    builder.add_edge(execute.out("skip"), parse.in_port("skip"))?;
    builder.add_edge(prepare.out("skip_reason"), parse.in_port("skip_reason"))?;

    Ok(TransportTriplet {
        prepare,
        execute,
        parse,
    })
}

/// Add a non-skippable transport triplet: prepare → execute → parse.
///
/// The execute node has the standard shape:
///   inputs:  `[port("request", "TransportRequest"), port("skip", "Bool")]`
///   outputs: `[port("response", "TransportResponse")]`
///
/// Standard wiring: `prepare.request → execute.request`,
/// `prepare.skip → execute.skip`,
/// `execute.response → parse.response`.
///
/// `prepare_inputs` are the *extra* inputs for the prepare node — the helper
/// automatically appends `port("request", "TransportRequest")` and
/// `port("skip", "Bool")` to outputs.
///
/// `parse_outputs` are the *extra* outputs for the parse node — the helper
/// automatically prepends `port("response", "TransportResponse")` to inputs.
pub fn add_transport_triplet<T>(
    builder: &mut DagBuilder<T>,
    name: &str,
    prepare_inputs: Vec<Port>,
    parse_outputs: Vec<Port>,
    prepare_op: T,
    parse_op: T,
    transport_op: T,
) -> Result<TransportTriplet<T>, BuilderError> {
    let prepare_name = format!("prepare_{name}");
    let execute_name = format!("execute_{name}");
    let parse_name = format!("parse_{name}");

    // Prepare node: caller inputs + request + skip output
    let prepare = builder.add_root_node(Node::opaque(
        prepare_name.as_str(),
        prepare_inputs,
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        prepare_op,
    ))?;

    // Execute node: standard transport shape (skip wired to false upstream)
    let execute = builder.add_node_after(
        Node::opaque(
            execute_name.as_str(),
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            transport_op,
        ),
        &prepare,
    )?;

    // Parse node: response input + caller outputs
    let parse = builder.add_node_after(
        Node::opaque(
            parse_name.as_str(),
            vec![port("response", "TransportResponse")],
            parse_outputs,
            parse_op,
        ),
        &execute,
    )?;

    // Wire up internal edges
    builder.add_edge(prepare.out("request"), execute.in_port("request"))?;
    builder.add_edge(prepare.out("skip"), execute.in_port("skip"))?;
    builder.add_edge(execute.out("response"), parse.in_port("response"))?;

    Ok(TransportTriplet {
        prepare,
        execute,
        parse,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum TestOp {
        Prepare,
        Execute,
        Parse,
    }

    #[test]
    fn test_skippable_triplet() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        // Need a root node to hang the triplet off of
        let root = builder
            .add_root_node(Node::opaque(
                "root",
                vec![],
                vec![port("success", "Bool")],
                TestOp::Prepare,
            ))
            .unwrap();

        let triplet = add_skippable_transport_triplet(
            &mut builder,
            "my_step",
            vec![port("success", "Bool")],
            vec![port("step_ok", "Bool")],
            TestOp::Prepare,
            TestOp::Parse,
            TestOp::Execute,
            &root,
        )
        .unwrap();

        // Wire the root output to the prepare input
        builder
            .add_edge(root.out("success"), triplet.prepare.in_port("success"))
            .unwrap();

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 4); // root + prepare + execute + parse
        // 5 internal edges + 1 root→prepare = 6
        assert_eq!(dag.edges.len(), 6);

        // Verify node names
        assert!(dag.get_node(&"prepare_my_step".into()).is_some());
        assert!(dag.get_node(&"execute_my_step".into()).is_some());
        assert!(dag.get_node(&"parse_my_step".into()).is_some());
    }

    #[test]
    fn test_non_skippable_triplet() {
        let mut builder: DagBuilder<TestOp> = DagBuilder::new();

        let _triplet = add_transport_triplet(
            &mut builder,
            "deps_exists",
            vec![], // no extra prepare inputs
            vec![port("exists", "Bool")],
            TestOp::Prepare,
            TestOp::Parse,
            TestOp::Execute,
        )
        .unwrap();

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 3); // prepare→execute (request + skip), execute→parse

        assert!(dag.get_node(&"prepare_deps_exists".into()).is_some());
        assert!(dag.get_node(&"execute_deps_exists".into()).is_some());
        assert!(dag.get_node(&"parse_deps_exists".into()).is_some());
    }
}
