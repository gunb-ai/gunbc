use gunbc_ir::*;
use gunbc_ir::metadata::NodeMetadata;
use gunbc_ir::types::{BehaviorKind, Idempotency, PatternDecision, ToolId};

use crate::ops::MakegenOp;
use crate::types::MakegenConfig;

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

/// Build the main DAG for makegen.
///
/// Structure:
///   context → check → [guarded generation pipeline] → resolve → sink
///
/// The sink node is swapped at build time based on dry_run flag:
///   - dry_run=false: WriteFile (WritesWorld)
///   - dry_run=true:  PrintStdout (Observe)
///
/// This demonstrates the "Pure generation + Effect sink" pattern where all
/// transformation logic is pure and only the terminal sink performs effects.
pub fn build_makegen_dag(config: &MakegenConfig, dry_run: bool) -> Dag<MakegenOp> {
    let sink_behavior = if dry_run {
        BehaviorKind::Observe
    } else {
        BehaviorKind::WritesWorld(Idempotency::Idempotent)
    };

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
            metadata: meta("makegen", BehaviorKind::Pure),
            body: NodeBody::Opaque(MakegenOp::Context { config: config.clone() }),
        },
        // Check - determines if generation is needed (Observe)
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
            metadata: meta("makegen", BehaviorKind::Observe),
            body: NodeBody::Opaque(MakegenOp::Check),
        },
        // Generation pipeline (guarded by needs_generate)
        Node {
            id: NodeId("parse_workspace".into()),
            inputs: vec![
                guarded_port("needs_generate", "Bool", "needs_generate == true"),
                port("workspace_path", "String"),
            ],
            outputs: vec![
                port("crate_names", "StrList"),
                port("crate_paths", "StrList"),
                port("crate_is_bin", "StrList"),
                port("crate_is_lib", "StrList"),
            ],
            metadata: meta("makegen", BehaviorKind::Pure),
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
            metadata: meta("makegen", BehaviorKind::Pure),
            body: NodeBody::Opaque(MakegenOp::GenerateTargets),
        },
        Node {
            id: NodeId("generate_rules".into()),
            inputs: vec![
                port("targets", "StrList"),
                port("crate_names", "StrList"),
            ],
            outputs: vec![port("rules", "StrList")],
            metadata: meta("makegen", BehaviorKind::Pure),
            body: NodeBody::Opaque(MakegenOp::GenerateRules),
        },
        Node {
            id: NodeId("compose_makefile".into()),
            inputs: vec![
                port("rules", "StrList"),
                port("input_hash", "String"),
            ],
            outputs: vec![port("content", "String")],
            metadata: meta("makegen", BehaviorKind::Pure),
            body: NodeBody::Opaque(MakegenOp::ComposeMakefile),
        },
        // Resolve - determines what to output (Pure)
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
            metadata: meta("makegen", BehaviorKind::Pure),
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
            metadata: meta("makegen", sink_behavior),
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
                tool: ToolId("makegen".into()),
                pattern: "upsert".into(),
                decision: PatternDecision::Instantiated,
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
    fn dag_validates() {
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, true);
        assert!(gunbc_validate::validate(&dag).is_ok());
    }

    #[test]
    fn dry_run_dag_has_observe_sink() {
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, true);
        let sink = dag.nodes.iter().find(|n| n.id.0 == "sink").unwrap();
        assert_eq!(sink.metadata.behavior, BehaviorKind::Observe);
    }

    #[test]
    fn real_dag_has_writes_world_sink() {
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, false);
        let sink = dag.nodes.iter().find(|n| n.id.0 == "sink").unwrap();
        assert_eq!(sink.metadata.behavior, BehaviorKind::WritesWorld(Idempotency::Idempotent));
    }

    #[test]
    fn dag_executes_dry_run() {
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, true);
        let log = gunbc_exec::execute(&dag).unwrap();
        assert!(!log.entries.is_empty());
        // Find the final sink entry
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

    #[test]
    fn generation_nodes_are_pure() {
        let config = MakegenConfig::default();
        let dag = build_makegen_dag(&config, true);
        let gen_nodes = ["parse_workspace", "generate_targets", "generate_rules", "compose_makefile"];
        for node_name in &gen_nodes {
            let node = dag.nodes.iter().find(|n| n.id.0 == *node_name).unwrap();
            assert_eq!(node.metadata.behavior, BehaviorKind::Pure, "Node {} should be Pure", node_name);
        }
    }
}
