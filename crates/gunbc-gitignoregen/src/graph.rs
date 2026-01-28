use gunbc_ir::types::PatternDecision;
use gunbc_ir::*;

use crate::ops::GitignoreOp;
use crate::types::GitignoreConfig;
use gunbc_ir::transport::external_types;

/// Build the main DAG for gitignoregen.
///
/// Structure:
///   context → check → compose → sink → resolve
///
/// The sink node operation is swapped at build time based on dry_run flag:
///   - dry_run=false: FileOp::WriteFile
///   - dry_run=true:  FileOp::PrintStdout
pub fn build_gitignoregen_dag(config: &GitignoreConfig, dry_run: bool) -> Dag<GitignoreOp> {
    let sink_op = if dry_run {
        GitignoreOp::File(gunbc_ir::transport::file::FileOp::PrintStdout)
    } else {
        GitignoreOp::File(gunbc_ir::transport::file::FileOp::WriteFile)
    };

    let nodes = vec![
        // Context - produces initial configuration
        Node {
            id: NodeId("context".into()),
            inputs: vec![],
            outputs: vec![
                port("file_path", "String"),
                port("force", "Bool"),
                port("input_hash", "String"),
            ],
            body: NodeBody::Opaque(GitignoreOp::Context {
                config: config.clone(),
            }),
        },
        // Check - determines if generation is needed
        Node {
            id: NodeId("check".into()),
            inputs: vec![
                port("file_path", "String"),
                port("force", "Bool"),
                port("input_hash", "String"),
            ],
            outputs: vec![
                port("input_hash", "String"),
                port("file_path", "String"),
                port("needs_write", "Bool"),
                port("file_existed", "Bool"),
            ],
            body: NodeBody::Opaque(GitignoreOp::File(
                gunbc_ir::transport::file::FileOp::CheckExisting,
            )),
        },
        // Compose - generates .gitignore content
        Node {
            id: NodeId("compose".into()),
            inputs: vec![port("input_hash", "String")],
            outputs: vec![port("content", "String")],
            body: NodeBody::Opaque(GitignoreOp::ComposeGitignore),
        },
        // Sink - WriteFile or PrintStdout based on dry_run
        Node {
            id: NodeId("sink".into()),
            inputs: vec![
                port("content", "String"),
                eq_guarded_port("needs_write", "Bool", Value::Bool(true)),
                port("file_path", "String"),
                port("file_existed", "Bool"),
            ],
            outputs: vec![port("write_status", "String")],
            body: NodeBody::Opaque(sink_op),
        },
        // Resolve - determines final status
        Node {
            id: NodeId("resolve".into()),
            inputs: vec![port("needs_write", "Bool"), port("write_status", "String")],
            outputs: vec![port("status", "String")],
            body: NodeBody::Opaque(GitignoreOp::File(
                gunbc_ir::transport::file::FileOp::ResolveUpsert,
            )),
        },
    ];

    let edges = vec![
        // Context to check
        edge("context", "file_path", "check", "file_path"),
        edge("context", "force", "check", "force"),
        edge("context", "input_hash", "check", "input_hash"),
        // Check to compose
        edge("check", "input_hash", "compose", "input_hash"),
        // Compose + check to sink
        edge("compose", "content", "sink", "content"),
        edge("check", "needs_write", "sink", "needs_write"),
        edge("check", "file_path", "sink", "file_path"),
        edge("check", "file_existed", "sink", "file_existed"),
        // Check + sink to resolve
        edge("check", "needs_write", "resolve", "needs_write"),
        edge("sink", "write_status", "resolve", "write_status"),
    ];

    let mut boundary_declarations = Vec::new();
    if !dry_run {
        boundary_declarations.push(BoundaryDeclaration {
            node: NodeId("sink".into()),
            port: PortName("write_status".into()),
            external_type: external_types::fs_write(),
        });
    }

    let metadata = DagMetadata {
        pattern_decisions: vec![PatternDecisionEntry {
            node: NodeId("gitignoregen".into()),
            pattern: "upsert".into(),
            decision: PatternDecision::Instantiated,
        }],
        export_node: Some(NodeId("resolve".into())),
        boundary_declarations,
    };

    Dag {
        nodes,
        edges,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::Value;

    #[test]
    fn dag_executes_dry_run() {
        let config = GitignoreConfig::default();
        let dag = build_gitignoregen_dag(&config, true);
        let log = gunbc_exec::execute(&dag).unwrap();
        assert!(!log.entries.is_empty());
        let resolve_entry = log.entries.iter().find(|e| e.node_id == "resolve").unwrap();
        assert!(matches!(resolve_entry.outputs.get("status"), Some(Value::Str(_))));
    }

    #[test]
    fn dag_has_pattern_decision() {
        let config = GitignoreConfig::default();
        let dag = build_gitignoregen_dag(&config, true);
        assert_eq!(dag.metadata.pattern_decisions.len(), 1);
        assert_eq!(dag.metadata.pattern_decisions[0].pattern, "upsert");
        assert!(matches!(
            dag.metadata.pattern_decisions[0].decision,
            PatternDecision::Instantiated
        ));
    }
}
