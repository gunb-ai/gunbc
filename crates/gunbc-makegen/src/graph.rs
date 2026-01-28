use gunbc_ir::algebra::{Predicate, Value};
use gunbc_ir::types::PatternDecision;
use gunbc_ir::*;

use crate::ops::MakegenOp;
use crate::types::MakegenConfig;

/// Build the main DAG for makegen.
///
/// Structure:
///   context → check → [guarded generation pipeline] → resolve → sink
///
/// The sink node operation is swapped at build time based on dry_run flag:
///   - dry_run=false: WriteFile
///   - dry_run=true:  PrintStdout
///
/// All nodes are pure transformations. Effects happen at the terminal sink
/// where data leaves the system.
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
                port("per_crate_targets", "Bool"),
                port("lint_targets", "Bool"),
                port("output_path", "String"),
                port("force", "Bool"),
            ],
            body: NodeBody::Opaque(MakegenOp::Context { config: config.clone() }),
        },
        // Check - determines if generation is needed
        Node {
            id: NodeId("check".into()),
            inputs: vec![
                port("workspace_path", "String"),
                port("output_path", "String"),
                port("force", "Bool"),
                port("per_crate_targets", "Bool"),
                port("lint_targets", "Bool"),
            ],
            outputs: vec![
                port("input_hash", "String"),
                port("makefile_path", "String"),
                port("needs_generate", "Bool"),
                port("file_exists", "Bool"),
                // Pass through for generation pipeline
                port("workspace_path", "String"),
                port("per_crate_targets", "Bool"),
                port("lint_targets", "Bool"),
            ],
            body: NodeBody::Opaque(MakegenOp::Check),
        },
        // Generation pipeline (guarded by needs_generate)
        Node {
            id: NodeId("parse_workspace".into()),
            inputs: vec![
                guarded_port("needs_generate", "Bool", Predicate::Eq(Value::Bool(true))),
                port("workspace_path", "String"),
            ],
            outputs: vec![
                port("crate_names", "StrList"),
                port("crate_paths", "StrList"),
                port("crate_is_bin", "StrList"),
                port("crate_is_lib", "StrList"),
            ],
            body: NodeBody::Opaque(MakegenOp::ParseWorkspace),
        },
        Node {
            id: NodeId("generate_targets".into()),
            inputs: vec![
                port("crate_names", "StrList"),
                port("per_crate_targets", "Bool"),
                port("lint_targets", "Bool"),
            ],
            outputs: vec![port("targets", "StrList")],
            body: NodeBody::Opaque(MakegenOp::GenerateTargets),
        },
        Node {
            id: NodeId("generate_rules".into()),
            inputs: vec![
                port("targets", "StrList"),
                port("crate_names", "StrList"),
            ],
            outputs: vec![port("rules", "StrList")],
            body: NodeBody::Opaque(MakegenOp::GenerateRules),
        },
        Node {
            id: NodeId("compose_makefile".into()),
            inputs: vec![
                port("rules", "StrList"),
                port("input_hash", "String"),
            ],
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
        edge("context", "per_crate_targets", "check", "per_crate_targets"),
        edge("context", "lint_targets", "check", "lint_targets"),
        // Check to generation pipeline
        edge("check", "needs_generate", "parse_workspace", "needs_generate"),
        edge("check", "workspace_path", "parse_workspace", "workspace_path"),
        edge("check", "per_crate_targets", "generate_targets", "per_crate_targets"),
        edge("check", "lint_targets", "generate_targets", "lint_targets"),
        edge("check", "input_hash", "compose_makefile", "input_hash"),
        // Generation pipeline internal flow
        edge("parse_workspace", "crate_names", "generate_targets", "crate_names"),
        edge("generate_targets", "targets", "generate_rules", "targets"),
        edge("parse_workspace", "crate_names", "generate_rules", "crate_names"),
        edge("generate_rules", "rules", "compose_makefile", "rules"),
        // To resolve
        edge("compose_makefile", "content", "resolve", "content"),
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
        pattern_decisions: vec![
            PatternDecisionEntry {
                node: NodeId("makegen".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::Instantiated,
            },
        ],
        export_node: None,
        boundary_declarations: vec![],
    };

    Dag { nodes, edges, metadata }
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
