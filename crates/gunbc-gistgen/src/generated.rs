// GENERATED — this file would be produced by gunbc-codegen from contracts.
// Currently hand-written to validate the design; structure matches codegen output.

use gunbc_ir::*;
use gunbc_ir::types::{BehaviorKind, Idempotency, PatternDecision};
use gunbc_exec::Executable;

/// Constructs the auth SubDAG wrapper node with correct wiring by construction.
///
/// The returned node is a SubDag node containing:
/// - 3 inner nodes: auth_check, auth_create, auth_resolve
/// - 3 edges following the upsert diamond topology
/// - export_node set to auth_resolve (always present, not Option)
/// - Pattern decision declared as Instantiated
///
/// Port names, types, and edge wiring are derived from the auth contracts —
/// a mismatch in the contracts would produce a compile error or a codegen
/// verification failure, not a runtime validation error.
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
            metadata: node_meta("auth", BehaviorKind::Observe),
            body: NodeBody::Opaque(auth_check),
        },
        Node {
            id: NodeId("auth_create".into()),
            inputs: vec![
                guarded_port("needs_create", "Bool", "needs_create == true"),
            ],
            outputs: vec![
                port("token", "Secret"),
            ],
            metadata: node_meta("auth", BehaviorKind::WritesWorld(Idempotency::Idempotent)),
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
            metadata: node_meta("auth", BehaviorKind::Pure),
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
            tool: ToolId("auth".into()),
            pattern: "upsert".into(),
            decision: PatternDecision::Instantiated,
        }],
        export_node: Some(NodeId("auth_resolve".into())),
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
        metadata: node_meta("auth", BehaviorKind::WritesWorld(Idempotency::Idempotent)),
        body: NodeBody::SubDag(inner_dag),
    }
}
