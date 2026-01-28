use gunbc_ir::*;
use gunbc_ir::metadata::NodeMetadata;
use gunbc_ir::types::{BehaviorKind, Idempotency, PatternDecision, ToolId};

use crate::ops::GistgenOp;

fn port(name: &str, ty: &str) -> Port {
    Port {
        name: PortName(name.into()),
        type_id: TypeId(ty.into()),
        guard: None,
    }
}

fn guarded_port(name: &str, ty: &str, guard: &str) -> Port {
    Port {
        name: PortName(name.into()),
        type_id: TypeId(ty.into()),
        guard: Some(guard.into()),
    }
}

fn edge(from: &str, from_port: &str, to: &str, to_port: &str) -> Edge {
    Edge {
        from_node: NodeId(from.into()),
        from_port: PortName(from_port.into()),
        to_node: NodeId(to.into()),
        to_port: PortName(to_port.into()),
    }
}

fn meta(tool: &str, behavior: BehaviorKind) -> NodeMetadata {
    NodeMetadata {
        tool: ToolId(tool.into()),
        behavior,
    }
}

/// Build the auth sub-DAG with check → create → resolve diamond.
fn build_auth_subdag() -> Dag<GistgenOp> {
    let nodes = vec![
        Node {
            id: NodeId("auth_check".into()),
            inputs: vec![],
            outputs: vec![port("token", "Secret"), port("needs_create", "Bool")],
            metadata: meta("auth", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::AuthCheck),
        },
        Node {
            id: NodeId("auth_create".into()),
            inputs: vec![guarded_port("needs_create", "Bool", "needs_create == true")],
            outputs: vec![port("token", "Secret")],
            metadata: meta("auth", BehaviorKind::WritesWorld(Idempotency::Idempotent)),
            body: NodeBody::Opaque(GistgenOp::AuthCreate),
        },
        Node {
            id: NodeId("auth_resolve".into()),
            inputs: vec![port("check_token", "Secret"), port("create_token", "Secret")],
            outputs: vec![port("token", "Secret")],
            metadata: meta("auth", BehaviorKind::Pure),
            body: NodeBody::Opaque(GistgenOp::AuthResolve),
        },
    ];

    let edges = vec![
        edge("auth_check", "token", "auth_resolve", "check_token"),
        edge("auth_check", "needs_create", "auth_create", "needs_create"),
        edge("auth_create", "token", "auth_resolve", "create_token"),
    ];

    let metadata = DagMetadata {
        pattern_decisions: vec![PatternDecisionEntry {
            tool: ToolId("auth".into()),
            pattern: "upsert".into(),
            decision: PatternDecision::Instantiated,
        }],
    };

    Dag { nodes, edges, metadata }
}

pub fn build_gistgen_dag(repo_path: &str, glob: &str, dry_run: bool) -> Dag<GistgenOp> {
    let upload_behavior = if dry_run {
        BehaviorKind::Observe
    } else {
        BehaviorKind::WritesWorld(Idempotency::NotIdempotent)
    };

    let nodes = vec![
        Node {
            id: NodeId("context".into()),
            inputs: vec![],
            outputs: vec![port("repo", "String"), port("selection_spec", "String")],
            metadata: meta("gistgen", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::Context {
                repo_path: repo_path.into(),
                glob_pattern: glob.into(),
            }),
        },
        Node {
            id: NodeId("auth".into()),
            inputs: vec![],
            outputs: vec![port("token", "Secret")],
            metadata: meta("auth", BehaviorKind::Observe),
            body: NodeBody::SubDag(build_auth_subdag()),
        },
        Node {
            id: NodeId("enumerate_files".into()),
            inputs: vec![port("repo", "String")],
            outputs: vec![port("files", "StrList")],
            metadata: meta("gistgen", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::EnumerateFiles),
        },
        Node {
            id: NodeId("filter_files".into()),
            inputs: vec![port("files", "StrList"), port("selection_spec", "String")],
            outputs: vec![port("files", "StrList")],
            metadata: meta("gistgen", BehaviorKind::Pure),
            body: NodeBody::Opaque(GistgenOp::FilterFiles),
        },
        Node {
            id: NodeId("read_files".into()),
            inputs: vec![port("files", "StrList")],
            outputs: vec![port("contents", "MapStrStr")],
            metadata: meta("gistgen", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::ReadFiles),
        },
        Node {
            id: NodeId("compose_snapshot".into()),
            inputs: vec![port("contents", "MapStrStr")],
            outputs: vec![port("snapshot", "String")],
            metadata: meta("gistgen", BehaviorKind::Pure),
            body: NodeBody::Opaque(GistgenOp::ComposeSnapshot),
        },
        Node {
            id: NodeId("upload_gist".into()),
            inputs: vec![port("snapshot", "String"), port("token", "Secret")],
            outputs: vec![port("gist_url", "String")],
            metadata: meta("gistgen", upload_behavior),
            body: NodeBody::Opaque(GistgenOp::UploadGist { dry_run }),
        },
    ];

    let edges = vec![
        edge("context", "repo", "enumerate_files", "repo"),
        edge("context", "selection_spec", "filter_files", "selection_spec"),
        edge("auth", "token", "upload_gist", "token"),
        edge("enumerate_files", "files", "filter_files", "files"),
        edge("filter_files", "files", "read_files", "files"),
        edge("read_files", "contents", "compose_snapshot", "contents"),
        edge("compose_snapshot", "snapshot", "upload_gist", "snapshot"),
    ];

    let metadata = DagMetadata {
        pattern_decisions: vec![
            PatternDecisionEntry {
                tool: ToolId("auth".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::Instantiated,
            },
            PatternDecisionEntry {
                tool: ToolId("gistgen".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::NotApplicable {
                    reason: "gistgen is an Emit tool, not Upsert".into(),
                },
            },
        ],
    };

    Dag { nodes, edges, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::Value;

    #[test]
    fn dag_validates() {
        let dag = build_gistgen_dag(".", "**/*.rs", true);
        assert!(gunbc_validate::validate(&dag).is_ok());
    }

    #[test]
    fn dag_executes_dry_run() {
        let dag = build_gistgen_dag(".", "**/*.rs", true);
        let log = gunbc_exec::execute(&dag).unwrap();
        assert!(!log.entries.is_empty());
        let last = log.entries.last().unwrap();
        assert_eq!(last.node_id, "upload_gist");
        if let Some(Value::Str(url)) = last.outputs.get("gist_url") {
            assert!(url.contains("dry-run"));
        } else {
            panic!("expected gist_url in upload_gist outputs");
        }
    }

    #[test]
    fn dry_run_upload_is_observe() {
        let dag = build_gistgen_dag(".", "**/*", true);
        let upload = dag.nodes.iter().find(|n| n.id.0 == "upload_gist").unwrap();
        assert_eq!(upload.metadata.behavior, BehaviorKind::Observe);
    }

    #[test]
    fn real_upload_is_writes_world() {
        let dag = build_gistgen_dag(".", "**/*", false);
        let upload = dag.nodes.iter().find(|n| n.id.0 == "upload_gist").unwrap();
        assert_eq!(upload.metadata.behavior, BehaviorKind::WritesWorld(Idempotency::NotIdempotent));
    }
}
