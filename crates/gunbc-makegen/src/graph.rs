use gunbc_ir::types::PatternDecision;
use gunbc_ir::*;

use crate::ops::MakegenOp;
use crate::types::MakegenConfig;

/// Build the main DAG for makegen.
///
/// Structure:
///   context → check → compose → resolve → sink
///
/// The sink node operation is swapped at build time based on dry_run flag:
///   - dry_run=false: WriteFile
///   - dry_run=true:  PrintStdout
pub fn build_makegen_dag(config: &MakegenConfig, dry_run: bool) -> Dag<MakegenOp> {
    let sink_op = if dry_run {
        MakegenOp::PrintStdout
    } else {
        MakegenOp::WriteFile
    };

    let nodes = vec![
        // Context - produces initial configuration
        Node {
            id: NodeId("context".into()),
            inputs: vec![],
            outputs: vec![
                port("workspace_path", "String"),
                port("output_path", "String"),
                port("force", "Bool"),
            ],
            body: NodeBody::Opaque(MakegenOp::Context {
                config: config.clone(),
            }),
        },
        // Check - determines if generation is needed
        Node {
            id: NodeId("check".into()),
            inputs: vec![
                port("workspace_path", "String"),
                port("output_path", "String"),
                port("force", "Bool"),
            ],
            outputs: vec![
                port("input_hash", "String"),
                port("makefile_path", "String"),
                port("needs_generate", "Bool"),
                port("file_exists", "Bool"),
            ],
            body: NodeBody::Opaque(MakegenOp::Check),
        },
        // Compose - generates Makefile content
        Node {
            id: NodeId("compose".into()),
            inputs: vec![port("input_hash", "String")],
            outputs: vec![port("content", "String")],
            body: NodeBody::Opaque(MakegenOp::ComposeMakefile),
        },
        // Resolve - determines what to output
        Node {
            id: NodeId("resolve".into()),
            inputs: vec![
                port("content", "String"),
                port("input_hash", "String"),
                port("makefile_path", "String"),
                port("needs_generate", "Bool"),
                port("file_exists", "Bool"),
            ],
            outputs: vec![
                port("content", "String"),
                port("hash", "String"),
                port("needs_write", "Bool"),
                port("makefile_path", "String"),
                port("file_existed", "Bool"),
            ],
            body: NodeBody::Opaque(MakegenOp::Resolve),
        },
        // Sink - WriteFile or PrintStdout based on dry_run
        Node {
            id: NodeId("sink".into()),
            inputs: vec![
                port("content", "String"),
                port("needs_write", "Bool"),
                port("makefile_path", "String"),
                port("file_existed", "Bool"),
            ],
            outputs: vec![port("status", "String")],
            body: NodeBody::Opaque(sink_op),
        },
    ];

    let edges = vec![
        // Context to check
        edge("context", "workspace_path", "check", "workspace_path"),
        edge("context", "output_path", "check", "output_path"),
        edge("context", "force", "check", "force"),
        // Check to compose
        edge("check", "input_hash", "compose", "input_hash"),
        // Check + compose to resolve
        edge("compose", "content", "resolve", "content"),
        edge("check", "input_hash", "resolve", "input_hash"),
        edge("check", "makefile_path", "resolve", "makefile_path"),
        edge("check", "needs_generate", "resolve", "needs_generate"),
        edge("check", "file_exists", "resolve", "file_exists"),
        // Resolve to sink
        edge("resolve", "content", "sink", "content"),
        edge("resolve", "needs_write", "sink", "needs_write"),
        edge("resolve", "makefile_path", "sink", "makefile_path"),
        edge("resolve", "file_existed", "sink", "file_existed"),
    ];

    let metadata = DagMetadata {
        pattern_decisions: vec![PatternDecisionEntry {
            node: NodeId("makegen".into()),
            pattern: "upsert".into(),
            decision: PatternDecision::Instantiated,
        }],
        export_node: None,
        boundary_declarations: vec![],
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
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, true);
        let log = gunbc_exec::execute(&dag).unwrap();
        assert!(!log.entries.is_empty());
        let sink_entry = log.entries.iter().find(|e| e.node_id == "sink").unwrap();
        assert!(matches!(sink_entry.outputs.get("status"), Some(Value::Str(_))));
    }

    #[test]
    fn dag_has_pattern_decision() {
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, true);
        assert_eq!(dag.metadata.pattern_decisions.len(), 1);
        assert_eq!(dag.metadata.pattern_decisions[0].pattern, "upsert");
        assert!(matches!(
            dag.metadata.pattern_decisions[0].decision,
            PatternDecision::Instantiated
        ));
    }
}
