use gunbc_ir::transport::external_types;
use gunbc_ir::types::PatternDecision;
use gunbc_ir::*;

use crate::ops::ToolsgenOp;
use crate::types::ToolsgenConfig;

/// Build the DAG for toolsgen.
///
/// Structure:
///   context → check → compose → sink → resolve
pub fn build_toolsgen_dag(config: &ToolsgenConfig, dry_run: bool) -> Dag<ToolsgenOp> {
    let sink_op = if dry_run {
        ToolsgenOp::PrintStdout
    } else {
        ToolsgenOp::WriteFile
    };

    let nodes = vec![
        Node {
            id: NodeId("context".into()),
            inputs: vec![],
            outputs: vec![
                port("workspace_path", "String"),
                port("output_path", "String"),
                port("force", "Bool"),
            ],
            body: NodeBody::Opaque(ToolsgenOp::Context {
                config: config.clone(),
            }),
        },
        Node {
            id: NodeId("check".into()),
            inputs: vec![
                port("workspace_path", "String"),
                port("output_path", "String"),
                port("force", "Bool"),
            ],
            outputs: vec![
                port("input_hash", "String"),
                port("file_path", "String"),
                port("needs_write", "Bool"),
                port("file_existed", "Bool"),
            ],
            body: NodeBody::Opaque(ToolsgenOp::Check),
        },
        Node {
            id: NodeId("compose".into()),
            inputs: vec![port("input_hash", "String")],
            outputs: vec![port("content", "String")],
            body: NodeBody::Opaque(ToolsgenOp::ComposeCargoWrapper),
        },
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
        Node {
            id: NodeId("resolve".into()),
            inputs: vec![port("needs_write", "Bool"), port("write_status", "String")],
            outputs: vec![port("status", "String")],
            body: NodeBody::Opaque(ToolsgenOp::Resolve),
        },
    ];

    let edges = vec![
        edge("context", "workspace_path", "check", "workspace_path"),
        edge("context", "output_path", "check", "output_path"),
        edge("context", "force", "check", "force"),
        edge("check", "input_hash", "compose", "input_hash"),
        edge("compose", "content", "sink", "content"),
        edge("check", "needs_write", "sink", "needs_write"),
        edge("check", "file_path", "sink", "file_path"),
        edge("check", "file_existed", "sink", "file_existed"),
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
            node: NodeId("toolsgen".into()),
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
        let config = ToolsgenConfig::default();
        let dag = build_toolsgen_dag(&config, true);
        let log = gunbc_exec::execute(&dag).unwrap();
        let resolve_entry = log
            .entries
            .iter()
            .find(|e| e.node_id == "resolve")
            .unwrap();
        assert!(matches!(resolve_entry.outputs.get("status"), Some(Value::Str(_))));
    }

    #[test]
    fn dag_has_pattern_decision() {
        let config = ToolsgenConfig::default();
        let dag = build_toolsgen_dag(&config, true);
        assert_eq!(dag.metadata.pattern_decisions.len(), 1);
        assert_eq!(dag.metadata.pattern_decisions[0].pattern, "upsert");
        assert!(matches!(
            dag.metadata.pattern_decisions[0].decision,
            PatternDecision::Instantiated
        ));
    }
}
