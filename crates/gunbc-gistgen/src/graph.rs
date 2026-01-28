use gunbc_ir::*;
use gunbc_ir::types::{BehaviorKind, Idempotency, PatternDecision};

use crate::generated;
use crate::ops::GistgenOp;

pub fn build_gistgen_dag(repo_path: &str, glob: &str, dry_run: bool) -> Dag<GistgenOp> {
    let upload_behavior = if dry_run {
        BehaviorKind::Observe
    } else {
        BehaviorKind::WritesWorld(Idempotency::NotIdempotent)
    };

    // Auth SubDAG — built from generated builder (ports, edges, export_node correct by construction)
    let auth_node = generated::build_auth_subdag(
        GistgenOp::AuthCheck,
        GistgenOp::AuthCreate,
        GistgenOp::AuthResolve,
    );

    let nodes = vec![
        Node {
            id: NodeId("context".into()),
            inputs: vec![],
            outputs: vec![port("repo", "String"), port("selection_spec", "String")],
            metadata: node_meta("gistgen", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::Context {
                repo_path: repo_path.into(),
                glob_pattern: glob.into(),
            }),
        },
        auth_node,
        Node {
            id: NodeId("enumerate_files".into()),
            inputs: vec![port("repo", "String")],
            outputs: vec![port("files", "StrList")],
            metadata: node_meta("gistgen", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::EnumerateFiles),
        },
        Node {
            id: NodeId("filter_files".into()),
            inputs: vec![port("files", "StrList"), port("selection_spec", "String")],
            outputs: vec![port("files", "StrList")],
            metadata: node_meta("gistgen", BehaviorKind::Pure),
            body: NodeBody::Opaque(GistgenOp::FilterFiles),
        },
        Node {
            id: NodeId("read_files".into()),
            inputs: vec![port("files", "StrList")],
            outputs: vec![port("contents", "MapStrStr")],
            metadata: node_meta("gistgen", BehaviorKind::Observe),
            body: NodeBody::Opaque(GistgenOp::ReadFiles),
        },
        Node {
            id: NodeId("compose_snapshot".into()),
            inputs: vec![port("contents", "MapStrStr")],
            outputs: vec![port("snapshot", "String")],
            metadata: node_meta("gistgen", BehaviorKind::Pure),
            body: NodeBody::Opaque(GistgenOp::ComposeSnapshot),
        },
        Node {
            id: NodeId("upload_gist".into()),
            inputs: vec![port("snapshot", "String"), port("token", "Secret")],
            outputs: vec![port("gist_url", "String")],
            metadata: node_meta("gistgen", upload_behavior),
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
        export_node: None,
    };

    Dag { nodes, edges, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::Value;

    #[test]
    fn dag_structure_correct() {
        let dag = build_gistgen_dag(".", "**/*.rs", true);
        let auth = dag.nodes.iter().find(|n| n.id.0 == "auth").unwrap();
        match &auth.body {
            NodeBody::SubDag(sub) => {
                assert!(sub.metadata.export_node.is_some());
                assert_eq!(sub.metadata.export_node.as_ref().unwrap().0, "auth_resolve");
                assert_eq!(sub.nodes.len(), 3);
                assert_eq!(sub.edges.len(), 3);
            }
            _ => panic!("auth should be SubDag"),
        }
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
    fn auth_subdag_has_upsert_topology() {
        let dag = build_gistgen_dag(".", "**/*.rs", true);
        let auth = dag.nodes.iter().find(|n| n.id.0 == "auth").unwrap();
        match &auth.body {
            NodeBody::SubDag(sub) => {
                gunbc_test::assert_upsert_topology(sub, "auth_check", "auth_create", "auth_resolve");
            }
            _ => panic!("auth should be SubDag"),
        }
    }

    #[test]
    fn auth_create_skipped_when_guard_false() {
        let dag = build_gistgen_dag(".", "**/*.rs", true);
        let log = gunbc_exec::execute(&dag).unwrap();
        // auth_create is lowered to auth/auth_create
        gunbc_test::assert_upsert_skip_semantics(&log, "auth/auth_create");
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
