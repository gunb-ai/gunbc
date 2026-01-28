// GENERATED — this file would be produced by gunbc-codegen from contracts.
// Currently hand-written to validate the design; structure matches codegen output.

use gunbc_exec::Executable;
use gunbc_ir::algebra::{Predicate, Value};
use gunbc_ir::types::PatternDecision;
use gunbc_ir::*;

/// Constructs the auth SubDAG wrapper node with correct wiring by construction.
pub fn build_auth_subdag<T: Executable>(
    auth_check: T,
    auth_create: T,
    auth_resolve: T,
) -> Node<T> {
    let inner_nodes = vec![
        Node {
            id: NodeId("auth_check".into()),
            inputs: vec![],
            outputs: vec![
                port("token", "Secret"),
                port("needs_create", "Bool"),
            ],
            body: NodeBody::Opaque(auth_check),
        },
        Node {
            id: NodeId("auth_create".into()),
            inputs: vec![
                guarded_port("needs_create", "Bool", Predicate::Eq(Value::Bool(true))),
            ],
            outputs: vec![
                port("token", "Secret"),
            ],
            body: NodeBody::Opaque(auth_create),
        },
        Node {
            id: NodeId("auth_resolve".into()),
            inputs: vec![
                port("check_token", "Secret"),
                port("create_token", "Secret"),
            ],
            outputs: vec![
                port("token", "Secret"),
            ],
            body: NodeBody::Opaque(auth_resolve),
        },
    ];

    let inner_edges = vec![
        edge("auth_check", "token", "auth_resolve", "check_token"),
        edge("auth_check", "needs_create", "auth_create", "needs_create"),
        edge("auth_create", "token", "auth_resolve", "create_token"),
    ];

    let inner_metadata = DagMetadata {
        pattern_decisions: vec![PatternDecisionEntry {
            node: NodeId("auth".into()),
            pattern: "upsert".into(),
            decision: PatternDecision::Instantiated,
        }],
        export_node: Some(NodeId("auth_resolve".into())),
        boundary_declarations: vec![],
    };

    let inner_dag = Dag {
        nodes: inner_nodes,
        edges: inner_edges,
        metadata: inner_metadata,
    };

    Node {
        id: NodeId("auth".into()),
        inputs: vec![],
        outputs: vec![port("token", "Secret")],
        body: NodeBody::SubDag(inner_dag),
    }
}
