#![allow(clippy::disallowed_methods)]

use std::collections::HashMap;

use daglang_driver::{compile_from_context_with_options, CompileOptions, DriverContext};
use daglang_lower::LoweredOp;
use gunbc_dag::resolve_lowered_dag;
use gunbc_exec::Executable;
use gunbc_ir::node::NodeBody;
use gunbc_ir::{Dag, Value, WorkspaceLayout};

#[test]
fn compiled_sdlc_pipeline_emits_ordered_stage_progression_metadata() {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .expect("resolve workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("pipelines/sdlc.dag")),
    };
    let output = compile_from_context_with_options(
        &context,
        CompileOptions {
            profile: Some("unit_test".to_string()),
            ..CompileOptions::default()
        },
    )
    .expect("compile sdlc pipeline with unit_test profile");
    let lowered_pipeline_node = output
        .lowered_dag
        .get_node(&"pipelines.sdlc::sdlc".into())
        .expect("lowered sdlc pipeline node present")
        .clone();
    let stage_names = match &lowered_pipeline_node.body {
        NodeBody::Opaque(LoweredOp::Pipeline { stage_names, .. }) => stage_names.clone(),
        other => panic!("expected lowered pipeline op, got {other:?}"),
    };
    let stage_count = stage_names.len() as i64;
    assert!(
        stage_count >= 8,
        "compiled sdlc pipeline should expose at least 8 stages, got {stage_count}"
    );
    assert!(
        stage_names.contains(&"fetch".to_string()),
        "stage order should contain fetch stage"
    );

    let mut pipeline_only = Dag::new();
    pipeline_only.add_node(lowered_pipeline_node);
    let resolved_pipeline_only =
        resolve_lowered_dag(&pipeline_only).expect("resolve pipeline-only dag");
    let pipeline_node = resolved_pipeline_only
        .get_node(&"pipelines.sdlc::sdlc".into())
        .expect("resolved pipeline-only node present");
    let NodeBody::Opaque(op) = &pipeline_node.body else {
        panic!("pipeline-only node should resolve to opaque operation")
    };
    for (index, current_stage) in stage_names.iter().enumerate() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "current_stage".to_string(),
            Value::Str(current_stage.clone()),
        );
        let outputs = op
            .execute(inputs)
            .expect("pipeline dispatch operation should execute");
        let expected_next = stage_names
            .get(index + 1)
            .unwrap_or(current_stage)
            .as_str();
        assert_eq!(
            outputs.get("next_stage").and_then(Value::as_str),
            Some(expected_next),
            "stage `{current_stage}` should progress to `{expected_next}`"
        );
    }
}

#[test]
fn compiled_sdlc_pipeline_resolves_full_dag() {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .expect("resolve workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("pipelines/sdlc.dag")),
    };
    let output = compile_from_context_with_options(
        &context,
        CompileOptions {
            profile: Some("unit_test".to_string()),
            ..CompileOptions::default()
        },
    )
    .expect("compile sdlc pipeline with unit_test profile");

    let node_count = output.lowered_dag.nodes.len();
    assert!(
        node_count > 20,
        "compiled sdlc pipeline should have >20 nodes, got {node_count}"
    );

    // Diagnostic: dump all node IDs and their body types for inspection
    let mut node_summary: Vec<String> = Vec::new();
    for node in &output.lowered_dag.nodes {
        let body_type = match &node.body {
            NodeBody::Opaque(op) => match op {
                LoweredOp::Callable { module, name, service_metadata, .. } => {
                    let has_meta = service_metadata.is_some();
                    format!("Callable({module}::{name}, meta={has_meta})")
                }
                LoweredOp::Primitive { kind, .. } => format!("Primitive({kind:?})"),
                LoweredOp::Pipeline { stage_names, .. } => format!("Pipeline({} stages)", stage_names.len()),
                LoweredOp::Collection { kind, .. } => format!("Collection({kind:?})"),
                LoweredOp::LoopUnpack { .. } => "LoopUnpack".to_string(),
                LoweredOp::LoopPack { .. } => "LoopPack".to_string(),
                LoweredOp::BranchMerge { .. } => "BranchMerge".to_string(),
            },
            NodeBody::SubDag(inner) => format!("SubDag({} nodes)", inner.nodes.len()),
        };
        node_summary.push(format!("  {} -> {}", node.id.0, body_type));
    }
    node_summary.sort();
    eprintln!("=== Lowered DAG nodes ({}) ===", output.lowered_dag.nodes.len());
    for line in &node_summary {
        eprintln!("{line}");
    }

    let resolved = resolve_lowered_dag(&output.lowered_dag);
    match &resolved {
        Ok(dag) => {
            assert!(
                dag.nodes.len() > 20,
                "resolved dag should preserve node count, got {}",
                dag.nodes.len()
            );
        }
        Err(e) => {
            panic!("resolve_lowered_dag failed: {e}");
        }
    }
}

#[test]
fn compiled_reconciler_pipeline_emits_ordered_stage_progression_metadata() {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .expect("resolve workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("pipelines/reconciler.dag")),
    };
    let output = compile_from_context_with_options(
        &context,
        CompileOptions {
            profile: Some("unit_test".to_string()),
            ..CompileOptions::default()
        },
    )
    .expect("compile reconciler pipeline with unit_test profile");
    let lowered_pipeline_node = output
        .lowered_dag
        .get_node(&"pipelines.reconciler::reconciler".into())
        .expect("lowered reconciler pipeline node present")
        .clone();
    let stage_names = match &lowered_pipeline_node.body {
        NodeBody::Opaque(LoweredOp::Pipeline { stage_names, .. }) => stage_names.clone(),
        other => panic!("expected lowered pipeline op, got {other:?}"),
    };
    assert_eq!(
        stage_names,
        vec![
            "discover".to_string(),
            "check_convergence".to_string(),
            "complete".to_string()
        ],
        "compiled reconciler pipeline should expose deterministic 3-stage order"
    );

    let mut pipeline_only = Dag::new();
    pipeline_only.add_node(lowered_pipeline_node);
    let resolved_pipeline_only =
        resolve_lowered_dag(&pipeline_only).expect("resolve pipeline-only dag");
    let pipeline_node = resolved_pipeline_only
        .get_node(&"pipelines.reconciler::reconciler".into())
        .expect("resolved pipeline-only node present");
    let NodeBody::Opaque(op) = &pipeline_node.body else {
        panic!("pipeline-only node should resolve to opaque operation")
    };
    let mut inputs = HashMap::new();
    inputs.insert("current_stage".to_string(), Value::Str("discover".to_string()));
    let outputs = op
        .execute(inputs)
        .expect("pipeline dispatch operation should execute");
    assert_eq!(
        outputs.get("next_stage").and_then(Value::as_str),
        Some("check_convergence"),
        "discover should progress to check_convergence"
    );
}
