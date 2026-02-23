#![allow(clippy::disallowed_methods)]

use std::collections::{HashMap, HashSet};

use daglang_driver::{compile_from_context_with_options, CompileOptions, DriverContext};
use daglang_lower::LoweredOp;
use gunbc_dag::resolve_lowered_dag;
use gunbc_exec::{execute_with_mode, BoundaryMocks, DynOp, Executable, ExecutionMode};
use gunbc_ir::node::NodeBody;
use gunbc_ir::transport::{
    FileOp, FileResponse, ShellRequest, ShellResponse, TransportRequest, TransportResponse,
};
use gunbc_ir::{Dag, Node, Value, WorkspaceLayout};

fn compile_sdlc_pipeline() -> daglang_driver::CompileOutput {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .expect("resolve workspace layout");
    let dsl_root = layout.workspace_root.join("dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("pipelines/sdlc.dag")),
    };
    compile_from_context_with_options(
        &context,
        CompileOptions {
            profile: Some("unit_test".to_string()),
            ..CompileOptions::default()
        },
    )
    .expect("compile sdlc pipeline with unit_test profile")
}

/// Build an executable DAG from the resolved SDLC DAG, stripping the Pipeline
/// metadata node and deduplicating scalar input edges.
///
/// The compiled DAG includes all service transport nodes as singletons. When
/// multiple DSL functions call the same service operation, the compiler wires
/// all callers to the same transport node, creating multiple edges to scalar
/// inputs. In a real pipeline, stages execute sequentially so only one set of
/// edges is active. For dry-run, we keep the first edge per (target, port)
/// pair, which is safe since transport nodes are intercepted anyway.
/// Non-stub SDLC providers and non-unit_test profiles. Nodes from these
/// modules exist in the compiled DAG but are not wired into the active
/// profile — exclude them to avoid missing-credential errors in dry-run.
const EXCLUDED_PREFIXES: &[&str] = &[
    "services_sdlc_providers_file_",
    "services_sdlc_providers_github_",
    "services_sdlc_providers_gcs_",
    "services_sdlc_providers_codex_",
    "services_sdlc_providers_inline_",
    "profiles_local",
    "profiles_cloud_run",
];

fn is_excluded_node(node_id: &str) -> bool {
    node_id == "pipelines.sdlc::sdlc"
        || EXCLUDED_PREFIXES
            .iter()
            .any(|prefix| node_id.contains(prefix))
}

fn build_executable_dag(resolved: &Dag<DynOp>) -> Dag<DynOp> {
    let mut dag = Dag::new();
    let included: HashSet<&str> = resolved
        .nodes
        .iter()
        .filter(|n| !is_excluded_node(&n.id.0))
        .map(|n| n.id.0.as_str())
        .collect();

    for node in &resolved.nodes {
        if included.contains(node.id.0.as_str()) {
            dag.add_node(node.clone());
        }
    }

    // Deduplicate scalar inputs (shared service transport nodes may have
    // multiple upstream edges from different DSL function callsites).
    let mut seen_inputs: HashSet<(String, String)> = HashSet::new();
    for edge in &resolved.edges {
        if !included.contains(edge.from_node.0.as_str())
            || !included.contains(edge.to_node.0.as_str())
        {
            continue;
        }
        let key = (edge.to_node.0.clone(), edge.to_port.0.clone());
        if seen_inputs.insert(key) {
            dag.edges.push(edge.clone());
        }
    }
    dag
}

/// Auto-mock all boundary nodes that the executor would intercept in DryRun.
///
/// This covers:
/// - Transport prepare nodes (emit TransportRequest) → dummy ShellRequest
/// - Transport execute nodes (consume TransportRequest) → ShellResponse::ok("") or FileResponse
/// - Resource env nodes (emit FilesystemHandle, Timestamp, Credential, etc.)
/// - Tool env nodes (emit ToolHandle)
fn auto_mock_all_boundaries<T>(dag: &Dag<T>, mocks: &mut BoundaryMocks) {
    use gunbc_primitives::filename;

    for node in &dag.nodes {
        // Transport prepare nodes (produce TransportRequest outputs).
        // These nodes try to interpolate credentials at runtime; in dry-run
        // their outputs feed only into already-mocked execute nodes.
        if is_transport_prepare(node) {
            for port in &node.outputs {
                if !mocks.has_mock(&node.id, &port.name) {
                    let value = if port.type_id.0 == "TransportRequest" {
                        Value::Request(TransportRequest::Shell(ShellRequest::new("mock")))
                    } else if port.type_id.0 == "Bool" {
                        Value::Bool(false)
                    } else {
                        Value::Str(String::new())
                    };
                    mocks.set_value(&node.id.0, &port.name.0, value);
                }
            }
        }
        // Transport execute nodes
        if is_transport_execute(node) {
            for port in &node.outputs {
                if !mocks.has_mock(&node.id, &port.name) {
                    let is_file = node
                        .inputs
                        .iter()
                        .any(|p| p.type_id.0 == "FilesystemHandle");
                    let value = if is_file {
                        Value::Response(TransportResponse::File(FileResponse {
                            path: String::new(),
                            operation: FileOp::Read,
                            success: true,
                            content: Some(String::new()),
                            bytes: None,
                            exists: None,
                            error: None,
                        }))
                    } else {
                        Value::Response(TransportResponse::Shell(ShellResponse::ok("")))
                    };
                    mocks.set_value(&node.id.0, &port.name.0, value);
                }
            }
        }
        // Transport parse nodes (consume TransportResponse). Mock their
        // outputs directly so they're intercepted — avoids response type
        // mismatches between Shell/REST/File execute mocks and parse ops.
        if is_transport_parse(node) {
            for port in &node.outputs {
                if !mocks.has_mock(&node.id, &port.name) {
                    let value = match port.type_id.0.as_str() {
                        "Bool" => Value::Bool(true),
                        "Int" => Value::Int(200),
                        "List" => Value::List(Vec::new()),
                        _ => Value::Str(String::new()),
                    };
                    mocks.set_value(&node.id.0, &port.name.0, value);
                }
            }
        }
        // Resource env nodes (FilesystemHandle, Timestamp, Credential, etc.)
        for port in &node.outputs {
            let needs_mock = matches!(
                port.type_id.0.as_str(),
                "FilesystemHandle"
                    | "NetworkHandle"
                    | "Timestamp"
                    | "Credential"
                    | "Platform"
                    | "CloudSecretConfig"
            );
            if needs_mock && !mocks.has_mock(&node.id, &port.name) {
                let value = match port.type_id.0.as_str() {
                    "FilesystemHandle" => {
                        filename::FilesystemHandle::cross_platform(filename::Scope::Write).into()
                    }
                    "Timestamp" => Value::Str("2026-01-01T00:00:00Z".to_string()),
                    _ => Value::Str(format!("mock-{}", port.type_id.0)),
                };
                mocks.set_value(&node.id.0, &port.name.0, value);
            }
        }
        // Tool env nodes (emit ToolHandle)
        if node
            .outputs
            .iter()
            .any(|p| p.type_id.0 == "ToolHandle")
        {
            for port in &node.outputs {
                if port.type_id.0 == "ToolHandle" && !mocks.has_mock(&node.id, &port.name) {
                    mocks.set_value(
                        &node.id.0,
                        &port.name.0,
                        Value::Str("mock-tool-handle".to_string()),
                    );
                }
            }
        }
        // Tool consumers (consume ToolHandle) — need full output mocks
        if node.inputs.iter().any(|p| p.type_id.0 == "ToolHandle") {
            for port in &node.outputs {
                if !mocks.has_mock(&node.id, &port.name) {
                    mocks.set_value(
                        &node.id.0,
                        &port.name.0,
                        Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
                    );
                }
            }
        }
    }
}

fn is_transport_prepare<T>(node: &Node<T>) -> bool {
    node.outputs
        .iter()
        .any(|port| port.type_id.0 == "TransportRequest")
}

fn is_transport_execute<T>(node: &Node<T>) -> bool {
    node.inputs
        .iter()
        .any(|port| port.type_id.0 == "TransportRequest")
}

fn is_transport_parse<T>(node: &Node<T>) -> bool {
    node.inputs
        .iter()
        .any(|port| port.type_id.0 == "TransportResponse")
}

#[test]
fn compiled_sdlc_pipeline_emits_ordered_stage_progression_metadata() {
    let output = compile_sdlc_pipeline();
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
    let output = compile_sdlc_pipeline();

    let node_count = output.lowered_dag.nodes.len();
    assert!(
        node_count > 20,
        "compiled sdlc pipeline should have >20 nodes, got {node_count}"
    );

    let resolved =
        resolve_lowered_dag(&output.lowered_dag).expect("resolve_lowered_dag on full SDLC DAG");
    assert!(
        resolved.nodes.len() > 20,
        "resolved dag should preserve node count, got {}",
        resolved.nodes.len()
    );
}

#[test]
fn compiled_sdlc_pipeline_dry_run_execution() {
    let output = compile_sdlc_pipeline();
    let resolved =
        resolve_lowered_dag(&output.lowered_dag).expect("resolve_lowered_dag on full SDLC DAG");
    let executable_dag = build_executable_dag(&resolved);

    let mut mocks = BoundaryMocks::new();
    auto_mock_all_boundaries(&executable_dag, &mut mocks);
    let mode = ExecutionMode::DryRun(mocks);
    let log = execute_with_mode(&executable_dag, mode)
        .expect("dry-run execution of compiled SDLC pipeline should succeed");

    // Verify execution touched a meaningful number of nodes
    assert!(
        log.entries.len() > 10,
        "dry-run should execute >10 nodes, got {}",
        log.entries.len()
    );
}

#[test]
fn compiled_sdlc_pipeline_e2e_stage_progression() {
    let output = compile_sdlc_pipeline();

    // 1. Verify compilation produces pipeline with all expected stages
    let lowered_pipeline_node = output
        .lowered_dag
        .get_node(&"pipelines.sdlc::sdlc".into())
        .expect("lowered sdlc pipeline node present")
        .clone();
    let stage_names = match &lowered_pipeline_node.body {
        NodeBody::Opaque(LoweredOp::Pipeline { stage_names, .. }) => stage_names.clone(),
        other => panic!("expected lowered pipeline op, got {other:?}"),
    };

    // 2. Resolve full DAG
    let resolved =
        resolve_lowered_dag(&output.lowered_dag).expect("resolve_lowered_dag on full SDLC DAG");

    // 3. Verify pipeline dispatch handles all stages correctly
    let mut pipeline_only = Dag::new();
    pipeline_only.add_node(lowered_pipeline_node);
    let resolved_pipeline =
        resolve_lowered_dag(&pipeline_only).expect("resolve pipeline-only dag");
    let pipeline_node = resolved_pipeline
        .get_node(&"pipelines.sdlc::sdlc".into())
        .expect("resolved pipeline node");
    let NodeBody::Opaque(op) = &pipeline_node.body else {
        panic!("pipeline node should be opaque")
    };

    let mut stages_processed = Vec::new();
    for current_stage in &stage_names {
        let mut inputs = HashMap::new();
        inputs.insert(
            "current_stage".to_string(),
            Value::Str(current_stage.clone()),
        );
        let outputs = op
            .execute(inputs)
            .expect("pipeline dispatch should succeed");
        assert!(
            outputs.contains_key("next_stage"),
            "stage `{current_stage}` should produce next_stage output"
        );
        stages_processed.push(current_stage.clone());
    }

    // 4. All stages were processed
    assert_eq!(
        stages_processed, stage_names,
        "all compiled stages should be processed"
    );
    assert!(
        stage_names.len() >= 8,
        "SDLC pipeline should have at least 8 stages, got {}",
        stage_names.len()
    );

    // 5. Dry-run execution completes without errors
    let executable_dag = build_executable_dag(&resolved);

    let mut mocks = BoundaryMocks::new();
    auto_mock_all_boundaries(&executable_dag, &mut mocks);
    let mode = ExecutionMode::DryRun(mocks);
    let log = execute_with_mode(&executable_dag, mode)
        .expect("E2E dry-run of compiled SDLC pipeline should succeed");

    assert!(
        log.entries.len() > 10,
        "E2E execution should touch >10 nodes, got {}",
        log.entries.len()
    );

    // 6. Verify DSL stage/worker function nodes are present —
    //    proving logic comes from compiled DSL, not hand-written Rust
    let node_ids: Vec<&str> = resolved.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let has_dsl_func_nodes = node_ids.iter().any(|id| {
        id.contains("sdlc_stages")
            || id.contains("sdlc_worker")
            || id.contains("tools.design")
    });
    assert!(
        has_dsl_func_nodes,
        "resolved DAG should contain SDLC function nodes from DSL"
    );
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
    inputs.insert(
        "current_stage".to_string(),
        Value::Str("discover".to_string()),
    );
    let outputs = op
        .execute(inputs)
        .expect("pipeline dispatch operation should execute");
    assert_eq!(
        outputs.get("next_stage").and_then(Value::as_str),
        Some("check_convergence"),
        "discover should progress to check_convergence"
    );
}
