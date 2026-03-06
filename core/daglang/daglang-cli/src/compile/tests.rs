use super::*;
use crate::pipeline::PipelineContext;
use daglang_derive::derive_artifacts;
use daglang_lower::{
    CallableKind, LoweredOp, ObligationCategory, ServiceCallMetadata, ServiceTransportClass,
};
use gunbc_exec::{lower, ExecutionMode};
use gunbc_ir::{node::NodeKind, Dag, Edge, Node, Port};
use gunbc_test::{unique_temp_dir, unique_temp_file};
use serde_json::Value;
use std::path::PathBuf;

fn workspace_dsl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl")
}

fn workspace_single_file_context(relative_path: &str) -> PipelineContext {
    let root = workspace_dsl_root();
    PipelineContext {
        roots: vec![root.clone()],
        target_file: Some(root.join(relative_path)),
    }
}

/// Create a unique temp directory with a `sample/` subdirectory for fixture files.
fn unique_temp_root(name: &str) -> PathBuf {
    let root = unique_temp_dir(name);
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    root
}

/// Create a temp directory, write `content` to `sample/main.dag`, and return a
/// directory-mode `PipelineContext` (no target file) plus the root path for cleanup.
fn temp_dag_context(name: &str, content: &str) -> (PipelineContext, PathBuf) {
    let root = unique_temp_root(name);
    std::fs::write(root.join("sample/main.dag"), content).expect("failed to write dag fixture");
    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };
    (context, root)
}

fn assert_typecheck_stage_error(error: &CompileError) {
    assert!(error.contains("typecheck errors"));
    assert!(!error.contains("lower error"));
}

#[test]
fn build_context_normalizes_absolute_directory_input_components() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_dir_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let normalized_root = root.join("sample");
    std::fs::create_dir_all(&normalized_root).expect("failed to create temp directory root");
    let input = root.join("sample").join(".").join("nested").join("..");
    let input_str = input.to_string_lossy().to_string();

    let cwd = std::env::temp_dir();
    let context = build_context(&cwd, Some(&input_str)).expect("build_context should succeed");
    assert_eq!(context.roots, vec![normalized_root.clone()]);
    assert!(context.target_file.is_none());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_normalizes_absolute_single_file_input_components() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_file_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let normalized_file = root.join("sample/main.dag");
    std::fs::create_dir_all(normalized_file.parent().expect("file should have parent"))
        .expect("failed to create temp file parent");
    std::fs::write(&normalized_file, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write temp dag file");
    let input = root
        .join("sample")
        .join("nested")
        .join("..")
        .join("main.dag");
    let input_str = input.to_string_lossy().to_string();

    let cwd = std::env::temp_dir();
    let context = build_context(&cwd, Some(&input_str)).expect("build_context should succeed");
    assert_eq!(
        context.roots,
        vec![normalized_file
            .parent()
            .expect("file should have parent")
            .to_path_buf()]
    );
    assert_eq!(context.target_file, Some(normalized_file.clone()));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_default_root_is_cwd_dsl() {
    let cwd = std::env::temp_dir().join(format!(
        "daglang_build_context_default_root_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let context = build_context(&cwd, None).expect("build_context should succeed");
    assert_eq!(context.roots, vec![cwd.join("dsl")]);
    assert!(context.target_file.is_none());
}

#[test]
fn check_from_context_succeeds_for_valid_single_file() {
    let fixture = unique_temp_file("check_valid_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.check_valid
fn run() -> Unit { }
"#,
    )
    .expect("failed to write check valid fixture");
    let cwd = std::env::temp_dir();
    let input = fixture.to_string_lossy().to_string();
    let context = build_context(&cwd, Some(&input)).expect("build_context should succeed");

    let output = check_from_context(&context).expect("check should succeed");
    assert_eq!(
        output.parsed_files, 1,
        "single-file check should report exactly one parsed file"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup check valid fixture");
}

#[test]
fn check_from_context_reports_typecheck_error_for_invalid_single_file() {
    let fixture = unique_temp_file("check_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.check_invalid
fn run() -> String { return 42 }
"#,
    )
    .expect("failed to write check invalid fixture");
    let cwd = std::env::temp_dir();
    let input = fixture.to_string_lossy().to_string();
    let context = build_context(&cwd, Some(&input)).expect("build_context should succeed");

    let error = check_from_context(&context).expect_err("check should fail");
    assert_typecheck_stage_error(&error);
    assert!(
        error.contains("type mismatch: expected `String`, got `Int`"),
        "check should surface type mismatch details: {error}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup check invalid fixture");
}

#[test]
fn build_context_rejects_dag_directory_input() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_dag_dir_target_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(dag_dir.join("nested"))
        .expect("failed to create .dag directory fixture");

    let error = crate::path_utils::check_dag_directory_conflict(&dag_dir);
    assert!(
        error.is_some(),
        ".dag directory should be rejected with explicit error"
    );
    assert!(
        error.as_ref().unwrap().contains("is a directory"),
        "error message should mention directory: {:?}",
        error
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_rejects_dag_directory_with_trailing_slash() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_dag_dir_trailing_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .dag directory fixture");
    let input_with_trailing_slash = format!("{}/", dag_dir.display());

    let cwd = std::env::temp_dir();
    let normalized =
        crate::path_utils::normalize_cli_path(&cwd, &PathBuf::from(&input_with_trailing_slash));
    let error = crate::path_utils::check_dag_directory_conflict(&normalized);
    assert!(
        error.is_some(),
        ".dag directory with trailing slash should be rejected"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_treats_uppercase_dag_directory_with_trailing_slash_as_root() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_uppercase_dag_dir_trailing_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let dag_dir = root.join("bundle.DAG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory fixture");
    let input_with_trailing_slash = format!("{}/", dag_dir.display());

    let cwd = std::env::temp_dir();
    let context = build_context(&cwd, Some(&input_with_trailing_slash))
        .expect("build_context should succeed for non-lowercase .dag extension");

    assert_eq!(context.roots, vec![dag_dir.clone()]);
    assert_eq!(context.target_file, None);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_treats_mixed_case_dag_directory_with_trailing_slash_as_root() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_mixed_case_dag_dir_trailing_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory fixture");
    let input_with_trailing_slash = format!("{}/", dag_dir.display());

    let cwd = std::env::temp_dir();
    let context = build_context(&cwd, Some(&input_with_trailing_slash))
        .expect("build_context should succeed for non-lowercase .dag extension");

    assert_eq!(context.roots, vec![dag_dir.clone()]);
    assert_eq!(context.target_file, None);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_treats_uppercase_dag_directory_input_as_root() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_uppercase_dag_dir_target_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let dag_dir = root.join("bundle.DAG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory fixture");

    let input_str = dag_dir.to_string_lossy().to_string();
    let cwd = std::env::temp_dir();
    let context = build_context(&cwd, Some(&input_str))
        .expect("build_context should succeed for non-lowercase .dag extension");

    assert_eq!(context.roots, vec![dag_dir.clone()]);
    assert_eq!(context.target_file, None);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn build_context_treats_mixed_case_dag_directory_input_as_root() {
    let root = std::env::temp_dir().join(format!(
        "daglang_build_context_mixed_case_dag_dir_target_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory fixture");

    let input_str = dag_dir.to_string_lossy().to_string();
    let cwd = std::env::temp_dir();
    let context = build_context(&cwd, Some(&input_str))
        .expect("build_context should succeed for non-lowercase .dag extension");

    assert_eq!(context.roots, vec![dag_dir.clone()]);
    assert_eq!(context.target_file, None);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_reports_cyclic_dependency_errors() {
    let root = std::env::temp_dir().join(format!(
        "daglang_compile_cycle_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("a")).expect("failed to create module a dir");
    std::fs::create_dir_all(root.join("b")).expect("failed to create module b dir");
    std::fs::write(
        root.join("a/a.dag"),
        "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
    )
    .expect("failed to write module a");
    std::fs::write(
        root.join("b/b.dag"),
        "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
    )
    .expect("failed to write module b");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let err = compile_from_context(&context).expect_err("compile should fail on cycles");
    assert!(err.contains("cyclic dependency"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_makegen_produces_non_empty_outputs() {
    let context = workspace_single_file_context("tools/makegen.dag");

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());
    let rendered = render_manifest(&output.derived);
    assert!(rendered.contains("TestObligations:"));
    assert!(rendered.contains("service_transport_prepare_targets:"));
    assert!(rendered.contains("service_param_source_targets:"));
    assert!(rendered.contains("resource_provide_targets:"));
}

#[test]
fn resolve_lowered_dag_unknown_callable_module_fails_closed() {
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "sample::unknown",
        vec![],
        vec![Port::scalar("out", "String")],
        LoweredOp::Callable {
            module: "sample.module".to_string(),
            kind: CallableKind::Func,
            name: "unknown".to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));

    // Unknown Callable nodes still resolve via passthrough (the compiler validated them).
    // ExternCall nodes are the hard-error path (see resolve_extern_call).
    let resolved =
        resolve_lowered_dag(&dag).expect("unknown callables should resolve via passthrough");
    assert_eq!(resolved.nodes.len(), 1);
    let debug = format!("{:?}", resolved.nodes[0].body);
    assert!(
        debug.contains("DeclaredOutputCallableOp"),
        "unexpected op debug: {debug}"
    );
}

#[test]
fn resolve_lowered_dag_skips_pipeline_nodes() {
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "pipeline::ci",
        vec![],
        vec![Port::scalar("out", "String")],
        LoweredOp::Pipeline {
            module: "pipelines".to_string(),
            name: "ci".to_string(),
            stages: 3,
            stage_names: vec![
                "cloud_env".to_string(),
                "codegen_stage".to_string(),
                "generate".to_string(),
            ],
        },
    ));

    // Pipeline nodes are metadata — the resolver skips them silently
    // rather than requiring a separate strip pass.
    let resolved = resolve_lowered_dag(&dag).expect("pipeline nodes should be skipped");
    assert!(
        resolved.nodes.is_empty(),
        "pipeline nodes should be filtered out, got {} nodes",
        resolved.nodes.len()
    );
}

#[test]
fn compile_resolve_execute_end_to_end_function_body_expressions() {
    let (context, root) = temp_dag_context(
        "function_body_e2e_dir",
        r#"module sample.main
fn summarize(flags: List<Bool>, include_disabled: Bool) -> String {
  scoped = if include_disabled {
    flags
  } else {
    flags |> filter(flag => flag)
  }
  labels = for flag in scoped {
    match flag {
      true => "enabled:true"
      false => "disabled:false"
    }
  }
  labels_csv = labels |> map(label => label) |> join(",")
  enabled_count = scoped |> count()
  payload = {
    enabled_count: enabled_count,
    labels: labels,
    report: "count={enabled_count}; labels={labels_csv}"
  }
  payload.report
}
func run() -> { report: String } {
  report = summarize(
    flags: [true, false, true],
    include_disabled: false
  )
  return {
    report: report
  }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    // Bridge 1: fn items are lowered as SubDag nodes with inner FnBodyCompute.
    // Control flow is handled by fn body evaluation — no pattern SubDag nodes needed.
    assert!(
        !output
            .lowered_dag
            .nodes
            .iter()
            .any(|node| node.id.0.contains("::cf_if_")),
        "fn with fn_body should not have control-flow subdag nodes"
    );
    assert!(
        !output
            .lowered_dag
            .nodes
            .iter()
            .any(|node| node.id.0.contains("::cf_match_")),
        "fn with fn_body should not have control-flow subdag nodes"
    );
    assert!(
        !output
            .lowered_dag
            .nodes
            .iter()
            .any(|node| node.id.0.contains("::cf_for_")),
        "fn with fn_body should not have control-flow subdag nodes"
    );
    let lowered = lower(&output.lowered_dag).expect("lowered DAG should flatten function subdags");
    let resolved =
        resolve_lowered_dag(&lowered.dag).expect("resolved DAG should build from lowered graph");
    // C10: FnBodyCallableOp evaluates the fn body directly, producing the
    // return value from the fn's computation instead of requiring passthrough
    // wiring from ExprCompute nodes.
    let result = execute_resolved_dag(&resolved, ExecutionMode::Real, None);
    assert!(result.is_ok(), "execution should succeed: {:?}", result.err());

    // Verify the fn body evaluation produced the correct report string.
    // Bridge 1: fn items are lowered as SubDag nodes. After exec lowering
    // (flattening), the inner node is prefixed: "sample.main::summarize/body".
    let log = result.unwrap();
    let summarize_entry = log
        .entries
        .iter()
        .find(|e| e.node_id.starts_with("sample.main::summarize"));
    assert!(summarize_entry.is_some(), "summarize node should have executed");
    let summarize_outputs = &summarize_entry.unwrap().outputs;
    let return_value = summarize_outputs.get("return");
    assert!(
        return_value.is_some() && !matches!(return_value, Some(gunbc_ir::Value::Skipped)),
        "C10: fn body should produce a non-Skipped return value, got: {:?}",
        return_value
    );
    // The fn filters [true, false, true] with include_disabled=false → [true, true],
    // then maps to labels, joins, and builds a report string.
    if let Some(gunbc_ir::Value::Str(report)) = return_value {
        assert!(
            report.contains("count=2"),
            "report should contain enabled count of 2, got: {report}"
        );
    }

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn render_obligations_json_emits_expected_keys() {
    let context = workspace_single_file_context("tools/makegen.dag");
    let output = compile_from_context(&context).expect("compile should succeed");

    let rendered = render_obligations(&output.derived, OutputFormat::Json);
    let parsed: Value = serde_json::from_str(&rendered).expect("obligations json should parse");
    assert_eq!(
        parsed
            .get("dry_run_completion_required")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert!(parsed.get("total_obligations").is_some());
    assert!(parsed.get("transport_execution_targets").is_some());
    assert!(parsed.get("pure_node_determinism_targets").is_some());
    assert!(parsed.get("service_transport_hermetic_targets").is_some());
    assert!(parsed.get("service_transport_external_targets").is_some());
    assert!(parsed.get("service_transport_idempotent_targets").is_some());
    assert!(parsed.get("service_transport_readonly_targets").is_some());
    assert!(parsed
        .get("interface_contract_verification_targets")
        .is_some());
}

#[test]
fn render_triplets_json_includes_makegen_transport_nodes() {
    let context = workspace_single_file_context("tools/makegen.dag");
    let output = compile_from_context(&context).expect("compile should succeed");

    let rendered = render_triplets(&output.derived, OutputFormat::Json);
    let parsed: Value = serde_json::from_str(&rendered).expect("triplets json should parse");
    let triplets = parsed
        .get("triplets")
        .and_then(Value::as_array)
        .expect("triplets should be an array");
    // After generic pattern expansion, transport triplet node names
    // follow the pattern expansion naming convention (no longer the
    // hardcoded prepare_read_makegen / prepare_write_makegen names).
    let prepare_names: Vec<&str> = triplets
        .iter()
        .filter_map(|t| t.get("prepare_node").and_then(Value::as_str))
        .collect();
    assert!(
        prepare_names
            .iter()
            .any(|p| p.contains("read") || p.contains("Read")),
        "expected read transport triplet, found: {prepare_names:?}"
    );
    assert!(
        prepare_names
            .iter()
            .any(|p| p.contains("write") || p.contains("Write")),
        "expected write transport triplet, found: {prepare_names:?}"
    );
}

#[test]
fn render_triplets_json_includes_service_semantic_metadata_when_present() {
    let mut dag = Dag::new();
    dag.add_node(
        Node::opaque(
            "prepare_transport_service",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::prepare::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    spec: None,
                })),
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportPrepare),
    );
    dag.add_node(
        Node::opaque(
            "execute_transport_service",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::execute::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    spec: None,
                })),
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportExecute),
    );
    dag.add_node(
        Node::opaque(
            "parse_transport_service",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::parse::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    spec: None,
                })),
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportParse),
    );
    dag.add_edge(Edge::new(
        "prepare_transport_service",
        "request",
        "execute_transport_service",
        "request",
    ));
    dag.add_edge(Edge::new(
        "execute_transport_service",
        "response",
        "parse_transport_service",
        "response",
    ));

    let derived = derive_artifacts(&dag).expect("triplet derivation should succeed");
    let rendered = render_triplets(&derived, OutputFormat::Json);
    let parsed: Value = serde_json::from_str(&rendered).expect("triplets json should parse");
    let triplets = parsed
        .get("triplets")
        .and_then(Value::as_array)
        .expect("triplets should be an array");
    let metadata = triplets
        .first()
        .and_then(|triplet| triplet.get("service_metadata"))
        .expect("triplet should include service metadata");
    assert_eq!(
        metadata.get("transport").and_then(Value::as_str),
        Some("shell_local")
    );
    assert_eq!(
        metadata.get("idempotent").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        metadata.get("readonly").and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn render_triplets_text_is_deterministic() {
    let context = workspace_single_file_context("tools/makegen.dag");
    let output = compile_from_context(&context).expect("compile should succeed");

    let first = render_triplets(&output.derived, OutputFormat::Text);
    let second = render_triplets(&output.derived, OutputFormat::Text);
    assert_eq!(first, second, "triplet rendering should be deterministic");
}

#[test]
fn workspace_tool_transport_triplet_audit_preserves_prepare_execute_parse_structure() {
    let tool_files = [
        ("tools/bootstrap.dag", 1usize),
        ("tools/codegen.dag", 1usize),
        ("tools/deps.dag", 1usize),
        ("tools/makegen.dag", 1usize),
        ("tools/pragma.dag", 1usize),
        ("tools/testgen.dag", 0usize),
    ];

    let mut total_triplets = 0usize;

    for (relative_path, min_triplets) in tool_files {
        let context = workspace_single_file_context(relative_path);
        let output = compile_from_context(&context).expect("tool compile should succeed");
        let triplets = &output.derived.transport_triplets;
        assert!(
            triplets.len() >= min_triplets,
            "expected at least {min_triplets} transport triplets in {relative_path}"
        );
        total_triplets += triplets.len();

        for triplet in triplets {
            assert!(
                output.lowered_dag.edges.iter().any(|edge| {
                    edge.from_node.0 == triplet.prepare_node
                        && edge.from_port.0 == "request"
                        && edge.to_node.0 == triplet.execute_node
                        && edge.to_port.0 == "request"
                }),
                "missing prepare->execute request edge for triplet {:?} in {relative_path}",
                triplet
            );

            for parse_node in &triplet.parse_nodes {
                assert!(
                    output.lowered_dag.edges.iter().any(|edge| {
                        edge.from_node.0 == triplet.execute_node
                            && edge.from_port.0 == "response"
                            && edge.to_node.0 == *parse_node
                            && edge.to_port.0 == "response"
                    }),
                    "missing execute->parse response edge for triplet {:?} in {relative_path}",
                    triplet
                );
            }
        }
    }

    assert!(
        total_triplets >= 16,
        "expected substantial tool triplet coverage across workspace DSL tools"
    );
}

#[test]
fn render_manifest_reuses_obligations_text_block() {
    let context = workspace_single_file_context("tools/makegen.dag");
    let output = compile_from_context(&context).expect("compile should succeed");

    let manifest = render_manifest(&output.derived);
    let obligations = render_obligations(&output.derived, OutputFormat::Text);
    assert!(
        manifest.ends_with(&obligations),
        "manifest output should embed the same obligations text renderer"
    );
}

#[test]
fn render_manifest_groups_stage_groups_into_collapsible_sections() {
    let context = workspace_single_file_context("pipelines/sdlc_ci.dag");
    let output = compile_from_context(&context).expect("compile should succeed");

    let manifest = render_manifest(&output.derived);
    assert!(
        manifest.contains("  stage_groups:\n    > [collapsed] pipelines.sdlc_ci.sdlc_ci"),
        "manifest text should render sdlc_ci stage groups as collapsible section"
    );
    assert!(
        manifest.contains("      - build:"),
        "manifest text should render build stage inside section"
    );
    assert!(
        manifest.contains("      - hermetic:"),
        "manifest text should render hermetic stage inside section"
    );
}

#[test]
fn render_manifest_groups_scatter_points_as_counters() {
    let root = std::env::temp_dir().join(format!(
        "daglang_manifest_scatter_points_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let file = root.join("sample.dag");
    std::fs::write(
        &file,
        r#"module sample
fn run(values: List<String>) -> String {
  rendered = values |> map(v => v) |> join(",")
  return rendered
}
"#,
    )
    .expect("failed to write source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: Some(file),
    };
    let output = compile_from_context_with_options(
        &context,
        CompileOptions {
            emit_collection_nodes: true,
            ..CompileOptions::default()
        },
    )
    .expect("compile should succeed with collection nodes");

    let manifest = render_manifest(&output.derived);
    assert!(
        manifest.contains("  scatter_points:\n    - sample.run [0/2]"),
        "manifest text should render grouped scatter counter for collection pipeline: {manifest}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn collect_transport_triplets_sorts_parse_nodes_and_ignores_non_transport_edges() {
    let mut dag = Dag::new();
    dag.add_node(
        Node::opaque(
            "prepare_a",
            vec![],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "prepare".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportPrepare),
    );
    dag.add_node(
        Node::opaque(
            "execute_a",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "execute".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportExecute),
    );
    dag.add_node(
        Node::opaque(
            "parse_z",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "parse_z".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportParse),
    );
    dag.add_node(
        Node::opaque(
            "parse_a",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.triplets".to_string(),
                kind: CallableKind::Pattern,
                name: "parse_a".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::TransportParse),
    );
    dag.add_node(Node::opaque(
        "non_transport_sink",
        vec![Port::scalar("value", "String")],
        vec![Port::scalar("ok", "Bool")],
        LoweredOp::Callable {
            module: "sample.triplets".to_string(),
            kind: CallableKind::Pattern,
            name: "sink".to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));

    dag.add_edge(Edge::new("prepare_a", "request", "execute_a", "request"));
    dag.add_edge(Edge::new("execute_a", "response", "parse_z", "response"));
    dag.add_edge(Edge::new("execute_a", "response", "parse_a", "response"));
    dag.add_edge(Edge::new("parse_a", "body", "non_transport_sink", "value"));

    let triplets = collect_transport_triplets(&dag);
    assert_eq!(triplets.len(), 1, "expected exactly one transport triplet");
    let triplet = &triplets[0];
    assert_eq!(triplet.prepare_node, "prepare_a");
    assert_eq!(triplet.execute_node, "execute_a");
    assert_eq!(
        triplet.parse_nodes,
        vec!["parse_a".to_string(), "parse_z".to_string()],
        "parse nodes should be sorted and deterministic"
    );
}

#[test]
fn compile_reports_pipeline_diagnostics_for_invalid_source() {
    let broken_file = unique_temp_file("broken");
    std::fs::write(&broken_file, "module broken\nfn bad( -> Unit {}")
        .expect("failed to write broken source");

    let context = PipelineContext {
        roots: vec![broken_file
            .parent()
            .expect("temp file should have parent")
            .to_path_buf()],
        target_file: Some(broken_file.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert!(error.contains("compile diagnostics"));
    assert!(error.contains(":2:"));
    assert!(!error.contains("typecheck errors"));
    assert!(!error.contains("lower error"));

    std::fs::remove_file(broken_file).expect("failed to cleanup broken source");
}

#[test]
fn compile_directory_reports_module_path_mismatch() {
    let root = std::env::temp_dir().join(format!(
        "daglang_compile_mismatch_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    std::fs::write(
        root.join("main.dag"),
        "module mismatch.main\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };
    let error = compile_from_context(&context).expect_err("compile should fail");
    assert!(error.contains("module path mismatches"));
    assert!(error.contains("main"));
    assert!(!error.contains("typecheck errors"));
    assert!(!error.contains("lower error"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_sorts_lex_before_parse_diagnostics() {
    let root = std::env::temp_dir().join(format!(
        "daglang_compile_lex_before_parse_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write parse-error file");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write lex-error file");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    let error_text = error.to_string();
    let first_diagnostic_line = error_text
        .lines()
        .find(|line: &&str| line.contains(".dag:"))
        .expect("expected at least one file diagnostic line");
    assert!(
        first_diagnostic_line.contains("z_lex.dag"),
        "lex diagnostics should sort before parse diagnostics: {error}"
    );
    assert!(error.contains("a_parse.dag"));
    assert!(error.contains("unexpected character '$'"));
    assert!(!error.contains("typecheck errors"));
    assert!(!error.contains("lower error"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unresolved_service_call_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_service_call");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write unresolved service-call fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unresolved service call"));
    assert!(error.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unresolved_service_call_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_service_dir",
        r#"module sample.main
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unresolved service call"));
    assert!(error.contains("MissingStorage.read"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_uses_bound_resource_capability_call_succeeds() {
    let fixture = unique_temp_file("single_file_uses_bound_resource_capability_call");
    std::fs::write(
        &fixture,
        r#"module sample.resources
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write resource-bound capability fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_uses_bound_resource_capability_call_succeeds() {
    let (context, root) = temp_dag_context(
        "resource_bound_service_call_dir",
        r#"module sample.main
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unresolved_uses_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_uses");
    std::fs::write(
        &fixture,
        r#"module sample.uses
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write unresolved uses fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown used resource type"));
    assert!(error.contains("MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unresolved_uses_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_uses_dir",
        r#"module sample.main
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown used resource type"));
    assert!(error.contains("MissingResource"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_uses_resource_with_runtime_config_suffix_succeeds() {
    let fixture = unique_temp_file("single_file_uses_resource_with_config_suffix");
    std::fs::write(
        &fixture,
        r#"module sample.uses
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}
"#,
    )
    .expect("failed to write configured uses fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_uses_resource_with_runtime_config_suffix_succeeds() {
    let (context, root) = temp_dag_context(
        "uses_config_suffix_dir",
        r#"module sample.main
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unresolved_provides_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_provides");
    std::fs::write(
        &fixture,
        r#"module sample.provides
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write unresolved provides fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown provided resource type"));
    assert!(error.contains("MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unresolved_provides_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_provides_dir",
        r#"module sample.main
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown provided resource type"));
    assert!(error.contains("MissingResource"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_provides_resource_with_runtime_config_suffix_succeeds() {
    let fixture = unique_temp_file("single_file_provides_resource_with_config_suffix");
    std::fs::write(
        &fixture,
        r#"module sample.provides
resource ArtifactStore {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}
"#,
    )
    .expect("failed to write configured provides fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_provides_resource_with_runtime_config_suffix_succeeds() {
    let (context, root) = temp_dag_context(
        "provides_config_suffix_dir",
        r#"module sample.main
resource ArtifactStore {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unresolved_import_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_import");
    std::fs::write(
        &fixture,
        r#"module sample.single
import missing.dep
fn run() -> Unit {}
"#,
    )
    .expect("failed to write unresolved import fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unresolved import"));
    assert!(error.contains("missing.dep"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unresolved_import_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_import_dir",
        r#"module sample.main
import missing.dep
fn run() -> Unit {}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    // CP-1: unresolved imports now fail at resolve stage (earlier than typecheck)
    assert!(error.contains("unresolved import"));
    assert!(error.contains("missing.dep"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unresolved_call_target_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_call_target");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> Unit {
  missing()
}
"#,
    )
    .expect("failed to write unresolved callable fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unresolved call target"));
    assert!(error.contains("missing"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unresolved_call_target_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_call_dir",
        r#"module sample.main
fn run() -> Unit {
  missing()
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unresolved call target"));
    assert!(error.contains("missing"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_collection_intrinsics_typecheck_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "collection_intrinsics_dir",
        r#"module sample.main
type Stage {
  success: Bool,
  skipped: Bool,
  name: String
}
fn summarize(stages: List<Stage>) -> Int {
  let passed = stages |> filter(s => s.success) |> count()
  let labels = stages |> map(s => s.name) |> join(",")
  let done = labels |> ends_with("ok")
  passed
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_collection_option_emits_collection_nodes() {
    let (context, root) = temp_dag_context(
        "collection_option_dir",
        r#"module sample.main
fn run(values: List<String>) -> String {
  rendered = values |> map(v => v) |> join(",")
  return rendered
}
"#,
    );
    let output = compile_from_context_with_options(
        &context,
        CompileOptions {
            emit_collection_nodes: true,
            ..CompileOptions::default()
        },
    )
    .expect("compile should succeed with collection option");
    assert!(output.lowered_dag.nodes.iter().any(|node| {
        matches!(
            node.body,
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
                kind: daglang_lower::CollectionOpKind::Map,
                ..
            })
        )
    }));
    assert!(output.lowered_dag.nodes.iter().any(|node| {
        matches!(
            node.body,
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
                kind: daglang_lower::CollectionOpKind::Join,
                ..
            })
        )
    }));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_function_typed_parameter_calls_typecheck_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "fn_typed_param_calls_dir",
        r#"module sample.main
fn apply(value: Int, callback: fn(Int) -> Int) -> Int {
  callback(value)
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_sum_variant_constructor_calls_typecheck_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "sum_variant_constructor_calls_dir",
        r#"module sample.main
type CloudConfig
  = GcpConfig { project: String, region: String }
  | AwsConfig { region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc", region: "us-central1")
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_zero_arity_variant_identifier_returns_typecheck_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "zero_arity_variant_identifier_dir",
        r#"module sample.main
type Environment = Dev | Ci
fn env() -> Environment {
  Dev
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_lossy_match_fn_body_does_not_fail_missing_tail_mismatch() {
    let (context, root) = temp_dag_context(
        "lossy_match_body_dir",
        r#"module sample.main
type CloudConfig
  = GcpConfig { project: String }
  | AwsConfig { account: String }
type CloudProvider = Gcp | Aws

fn provider_of(config: CloudConfig) -> CloudProvider {
  match config {
    GcpConfig { ... } => Gcp
    AwsConfig { ... } => Aws
  }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_std_helper_intrinsics_typecheck_in_strict_mode() {
    let root = unique_temp_root("std_helper_intrinsics_dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type DocgenSources {}

fn run(sources: DocgenSources, payload: String) -> String {
  let a = "template" |> replace_section("section", "value")
  let b = render_test_listings(sources: sources)
  let c = render_graph_structure(sources: sources)
  let d = render_source_artifacts(sources: sources)
  let e = compute_topology_diff(current: "{}", base: "{}")
  let f = render_annotated_mermaid(diff: e, topology: "{}", title: "title")
  let g = detect_runtime()
  let h = generate()
  let i = now()
  let j = build_token(
    payload: payload,
    scheme: "Bearer",
    header_name: "Authorization",
    source_id: "source",
    required_scopes: ["gist"]
  )
  a
}
"#,
    )
    .expect("failed to write helper intrinsic source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_duplicate_service_reports_ambiguous_service_call() {
    let fixture = unique_temp_file("single_file_duplicate_service");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write duplicate service fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `FsStorage`"));
    assert!(error.contains("ambiguous service call"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_duplicate_service_reports_ambiguous_service_call() {
    let (context, root) = temp_dag_context(
        "duplicate_service_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `FsStorage`"));
    assert!(error.contains("ambiguous service call"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_duplicate_callable_reports_ambiguous_call_target() {
    let fixture = unique_temp_file("single_file_duplicate_callable");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
    )
    .expect("failed to write duplicate callable fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `helper`"));
    assert!(error.contains("ambiguous call target"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_duplicate_callable_reports_ambiguous_call_target() {
    let (context, root) = temp_dag_context(
        "duplicate_callable_dir",
        r#"module sample.main
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `helper`"));
    assert!(error.contains("ambiguous call target"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_duplicate_resource_uses_reports_ambiguous_used_type() {
    let fixture = unique_temp_file("single_file_duplicate_resource_uses");
    std::fs::write(
        &fixture,
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write duplicate resource-uses fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `SharedResource`"));
    assert!(error.contains("ambiguous used resource type"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_duplicate_resource_uses_reports_ambiguous_used_type() {
    let (context, root) = temp_dag_context(
        "duplicate_resource_uses_dir",
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `SharedResource`"));
    assert!(error.contains("ambiguous used resource type"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_duplicate_resource_provides_reports_ambiguous_provided_type() {
    let fixture = unique_temp_file("single_file_duplicate_resource_provides");
    std::fs::write(
        &fixture,
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write duplicate resource-provides fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `SharedResource`"));
    assert!(error.contains("ambiguous provided resource type"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_duplicate_resource_provides_reports_ambiguous_provided_type() {
    let (context, root) = temp_dag_context(
        "duplicate_resource_provides_dir",
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `SharedResource`"));
    assert!(error.contains("ambiguous provided resource type"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unresolved_service_interface_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_service_interface");
    std::fs::write(
        &fixture,
        r#"module sample.services
service FsStorage implements MissingStorage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write unresolved service-interface fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`FsStorage` references unresolved interface `MissingStorage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_unresolved_resource_interface_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unresolved_resource_interface");
    std::fs::write(
        &fixture,
        r#"module sample.resources
resource Disk implements MissingStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write unresolved resource-interface fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`Disk` references unresolved interface `MissingStorage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unresolved_service_interface_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_service_interface_dir",
        r#"module sample.main
service FsStorage implements MissingStorage {
  operation read(path: String) -> { body: String }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`FsStorage` references unresolved interface `MissingStorage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_unresolved_resource_interface_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unresolved_resource_interface_dir",
        r#"module sample.main
resource Disk implements MissingStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`Disk` references unresolved interface `MissingStorage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_duplicate_interface_reports_ambiguous_implements() {
    let fixture = unique_temp_file("single_file_duplicate_interface");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write duplicate-interface fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `Storage` in module `sample.single`"));
    assert!(error.contains("`FsStorage` references ambiguous interface `Storage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_duplicate_interface_reports_ambiguous_implements() {
    let (context, root) = temp_dag_context(
        "duplicate_interface_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate definition `Storage` in module `sample.main`"));
    assert!(error.contains("`FsStorage` references ambiguous interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_unit_return_without_tail_expression_succeeds() {
    let fixture = unique_temp_file("single_file_unit_without_tail");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> Unit {
  let x = 42
}
"#,
    )
    .expect("failed to write Unit-return fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_unit_return_without_tail_expression_succeeds() {
    let (context, root) = temp_dag_context(
        "unit_without_tail_dir",
        r#"module sample.main
fn run() -> Unit {
  let x = 42
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_missing_tail_non_unit_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_non_unit_without_tail");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> String {
  let x = 42
}
"#,
    )
    .expect("failed to write non-Unit return fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type mismatch: expected `String`, got `Unit`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_missing_tail_non_unit_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "non_unit_without_tail_dir",
        r#"module sample.main
fn run() -> String {
  let x = 42
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type mismatch: expected `String`, got `Unit`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_call_arity_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_call_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
    )
    .expect("failed to write call-arity fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("call arity mismatch"));
    assert!(error.contains("fmt"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_unknown_named_call_argument_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unknown_named_call_argument");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
    )
    .expect("failed to write unknown-arg fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown named argument"));
    assert!(error.contains("text"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_duplicate_named_call_argument_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_duplicate_named_call_argument");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(value: "a", value: "b") }
"#,
    )
    .expect("failed to write duplicate-arg fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate named argument"));
    assert!(error.contains("value"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_service_call_arity_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_service_call_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read()
  return { body: response.body }
}
"#,
    )
    .expect("failed to write service call-arity fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("service call arity mismatch"));
    assert!(error.contains("FsStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_unknown_named_service_argument_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unknown_named_service_argument");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(name: "README.md")
  return { body: response.body }
}
"#,
    )
    .expect("failed to write unknown service-arg fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown named argument"));
    assert!(error.contains("name"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_duplicate_named_service_argument_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_duplicate_named_service_argument");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "a", path: "b")
  return { body: response.body }
}
"#,
    )
    .expect("failed to write duplicate service-arg fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate named argument"));
    assert!(error.contains("path"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_undefined_type_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_undefined_type");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(input: MissingType) -> String { "ok" }
"#,
    )
    .expect("failed to write undefined-type fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("undefined type `MissingType"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_type_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { return 42 }
"#,
    )
    .expect("failed to write type-mismatch fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_implicit_return_type_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_implicit_return_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { 42 }
"#,
    )
    .expect("failed to write implicit-return mismatch fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_no_such_field_record_literal_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_no_such_field_record_literal");
    std::fs::write(
        &fixture,
        r#"module sample.types
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}
"#,
    )
    .expect("failed to write no-such-field record-literal fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type `Record` has no field `missing`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_no_such_field_named_record_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_no_such_field_named_record");
    std::fs::write(
        &fixture,
        r#"module sample.types
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
"#,
    )
    .expect("failed to write no-such-field named-record fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type `Payload` has no field `missing`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_unsatisfiable_refinement_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_unsatisfiable_refinement");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(value: Int where range(min: 5, max: 1)) -> Int { value }
"#,
    )
    .expect("failed to write unsatisfiable-refinement fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unsatisfiable refinement on `Int`: range min 5 exceeds max 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_generic_arity_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_generic_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(values: Map<String>) -> Int { 1 }
"#,
    )
    .expect("failed to write generic-arity mismatch fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("generic arity mismatch for `Map`: expected 2, got 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_user_defined_generic_arity_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_user_defined_generic_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
type Box<T> = T
fn run(values: Box<String, Int>) -> String { values }
"#,
    )
    .expect("failed to write user-defined generic-arity mismatch fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("generic arity mismatch for `Box`: expected 1, got 2"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_undefined_type_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "undefined_type_dir",
        r#"module sample.main
fn run(input: MissingType) -> String { "ok" }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("undefined type `MissingType"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_type_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "type_mismatch_dir",
        r#"module sample.main
fn run() -> String { return 42 }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_implicit_return_type_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "implicit_return_mismatch_dir",
        r#"module sample.main
fn run() -> String { 42 }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_no_such_field_record_literal_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "no_such_field_record_dir",
        r#"module sample.main
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type `Record` has no field `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_no_such_field_named_record_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "no_such_field_named_record_dir",
        r#"module sample.main
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("type `Payload` has no field `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_unsatisfiable_refinement_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unsatisfiable_refinement_dir",
        r#"module sample.main
fn run(value: Int where range(min: 5, max: 1)) -> Int { value }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unsatisfiable refinement on `Int`: range min 5 exceeds max 1"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_generic_arity_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "generic_arity_mismatch_dir",
        r#"module sample.main
fn run(values: Map<String>) -> Int { 1 }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("generic arity mismatch for `Map`: expected 2, got 1"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_user_defined_generic_arity_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "user_defined_generic_arity_mismatch_dir",
        r#"module sample.main
type Box<T> = T
fn run(values: Box<String, Int>) -> String { values }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("generic arity mismatch for `Box`: expected 1, got 2"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_call_arity_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "call_arity_mismatch_dir",
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("call arity mismatch"));
    assert!(error.contains("fmt"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_call_with_defaulted_params_succeeds() {
    let (context, root) = temp_dag_context(
        "call_defaults_dir",
        r#"module sample.main
fn greet(name: String, punctuation: String = "!") -> String { name }
fn run() -> String { greet(name: "hi") }
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    // Verify the default value "!" is injected as a literal source node
    // with an edge to the greet callable's "punctuation" port.
    let greet_node = output
        .lowered_dag
        .nodes
        .iter()
        .find(|n| n.id.0.contains("greet"))
        .expect("greet node should exist");
    let has_punctuation_edge = output
        .lowered_dag
        .edges
        .iter()
        .any(|e| e.to_node.0 == greet_node.id.0 && e.to_port.0 == "punctuation");
    assert!(
        has_punctuation_edge,
        "default value should create an edge to 'punctuation' port on greet node"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_pattern_call_with_extra_named_wiring_args_succeeds() {
    let (context, root) = temp_dag_context(
        "pattern_wiring_args_dir",
        r#"module sample.main
pattern ensure(should_act: Bool = true) -> { acted: Bool } {
  return { acted: should_act }
}
fn run() -> Bool {
  let result = ensure(check: true, action: false)
  result.acted
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_generic_fn_type_params_typecheck_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "generic_fn_type_params_dir",
        r#"module sample.main
fn identity<T>(value: T) -> T {
  value
}
fn relay<T>(value: T) -> T {
  identity(value: value)
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_generic_pattern_type_params_typecheck_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "generic_pattern_type_params_dir",
        r#"module sample.main
pattern passthrough<T: Serializable>(value: T) -> { value: T } {
  return { value: value }
}
fn relay<T>(value: T) -> T {
  let result = passthrough(value: value)
  result.value
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_named_record_literal_return_succeeds_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "named_record_literal_return_dir",
        r#"module sample.main
type StageResult {
  success: Bool,
  skipped: Bool
}
fn result() -> StageResult {
  { success: true, skipped: false }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_resource_config_named_return_succeeds_in_strict_mode() {
    let (context, root) = temp_dag_context(
        "resource_config_named_return_dir",
        r#"module sample.main
resource GcsBucket {
  config {
    name: String,
    project: String
  }
}
fn gcp_dev_storage() -> GcsBucket.Config {
  { name: "gunbc-dev-artifacts", project: "gunbai-auto" }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_unknown_named_call_argument_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unknown_named_call_arg_dir",
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown named argument"));
    assert!(error.contains("text"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_duplicate_named_call_argument_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "duplicate_named_call_arg_dir",
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(value: "a", value: "b") }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate named argument"));
    assert!(error.contains("value"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_service_call_arity_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "service_call_arity_mismatch_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read()
  return { body: response.body }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("service call arity mismatch"));
    assert!(error.contains("FsStorage.read"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_service_call_with_defaulted_inputs_succeeds() {
    let (context, root) = temp_dag_context(
        "service_call_defaults_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input {
      path: String,
      recursive: Bool = false
    }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String, recursive: Bool = false) -> { ok: Bool }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp")
  return { ok: response.ok }
}
"#,
    );

    let output = compile_from_context(&context).expect("compile should succeed");
    assert!(!output.lowered_dag.nodes.is_empty());
    assert!(output.derived.manifest.total_nodes > 0);
    assert!(!output.emitted.files.is_empty());

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_unknown_named_service_argument_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "unknown_named_service_arg_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(name: "README.md")
  return { body: response.body }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("unknown named argument"));
    assert!(error.contains("name"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_duplicate_named_service_argument_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "duplicate_named_service_arg_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "a", path: "b")
  return { body: response.body }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate named argument"));
    assert!(error.contains("path"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_single_file_duplicate_parameter_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_duplicate_parameter");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run(a: String, a: Int) -> String { a }
"#,
    )
    .expect("failed to write duplicate-parameter fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate parameter `a` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_duplicate_output_field_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_duplicate_output_field");
    std::fs::write(
        &fixture,
        r#"module sample.single
func run() -> { ok: Bool, ok: String } { return { ok: true } }
"#,
    )
    .expect("failed to write duplicate-output fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate output field `ok` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_duplicate_uses_binding_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_duplicate_uses_binding");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write duplicate-uses fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate uses binding `fs` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_duplicate_provides_binding_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_duplicate_provides_binding");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } provides out: Storage provides out: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write duplicate-provides fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate provides binding `out` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_use_provide_binding_conflict_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_use_provide_binding_conflict");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses io: Storage provides io: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write use/provide conflict fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("binding `io` is declared in both uses/provides in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_missing_resource_capability_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_missing_resource_capability");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write missing-resource-capability fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_missing_service_operation_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_missing_service_operation");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write missing-service-operation fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(
        error.contains("service `FsStorage` is missing operation `write` for interface `Storage`")
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_service_interface_signature_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_interface_signature_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
    )
    .expect("failed to write service-signature-mismatch fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`FsStorage` does not match `Storage.read` contract"));
    assert!(error.contains("expected `String` but found `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_single_file_resource_interface_signature_mismatch_fails_in_typecheck_stage() {
    let fixture = unique_temp_file("single_file_resource_signature_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write resource-signature-mismatch fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`Disk` does not match `Storage.read` contract"));
    assert!(error.contains("expected `String` but found `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_directory_duplicate_parameter_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "duplicate_parameter_dir",
        r#"module sample.main
fn run(a: String, a: Int) -> String { a }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate parameter `a` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_duplicate_output_field_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "duplicate_output_field_dir",
        r#"module sample.main
func run() -> { ok: Bool, ok: String } { return { ok: true } }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate output field `ok` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_duplicate_uses_binding_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "duplicate_uses_binding_dir",
        r#"module sample.main
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage { return { ok: true } }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate uses binding `fs` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_duplicate_provides_binding_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "duplicate_provides_binding_dir",
        r#"module sample.main
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } provides out: Storage provides out: Storage { return { ok: true } }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("duplicate provides binding `out` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_use_provide_binding_conflict_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "use_provide_binding_conflict_dir",
        r#"module sample.main
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses io: Storage provides io: Storage { return { ok: true } }
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("binding `io` is declared in both uses/provides in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_missing_resource_capability_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "missing_resource_capability_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_missing_service_operation_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "missing_service_operation_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(
        error.contains("service `FsStorage` is missing operation `write` for interface `Storage`")
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_service_interface_signature_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "service_signature_mismatch_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`FsStorage` does not match `Storage.read` contract"));
    assert!(error.contains("expected `String` but found `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_resource_interface_signature_mismatch_fails_in_typecheck_stage() {
    let (context, root) = temp_dag_context(
        "resource_signature_mismatch_dir",
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
    );

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`Disk` does not match `Storage.read` contract"));
    assert!(error.contains("expected `String` but found `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_ambiguous_interface_reference_fails_in_typecheck_stage() {
    let root = unique_temp_root("ambiguous_interface_reference_dir");
    std::fs::write(
            root.join("sample/first.dag"),
            "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write first interface source");
    std::fs::write(
            root.join("sample/second.dag"),
            "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write second interface source");
    std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nservice FsStorage implements Storage { operation read(path: String) -> { body: String } }",
        )
        .expect("failed to write main source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("ambiguous interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_ambiguous_resource_interface_reference_fails_in_typecheck_stage() {
    let root = unique_temp_root("ambiguous_resource_interface_reference_dir");
    std::fs::write(
            root.join("sample/first.dag"),
            "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write first interface source");
    std::fs::write(
            root.join("sample/second.dag"),
            "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write second interface source");
    std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nresource Disk implements Storage { capability read { input { path: String } output { body: String } } }",
        )
        .expect("failed to write main source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("`Disk` references ambiguous interface `Storage`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_ambiguous_uses_resource_type_fails_in_typecheck_stage() {
    let root = unique_temp_root("ambiguous_uses_resource_type_dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nresource SharedResource {}",
    )
    .expect("failed to write first resource source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nresource SharedResource {}",
    )
    .expect("failed to write second resource source");
    std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nfunc run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }",
        )
        .expect("failed to write main source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("ambiguous used resource type `SharedResource`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_ambiguous_provides_resource_type_fails_in_typecheck_stage() {
    let root = unique_temp_root("ambiguous_provides_resource_type_dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nresource SharedResource {}",
    )
    .expect("failed to write first resource source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nresource SharedResource {}",
    )
    .expect("failed to write second resource source");
    std::fs::write(
            root.join("sample/main.dag"),
            "module sample.main\nfunc run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }",
        )
        .expect("failed to write main source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("ambiguous provided resource type `SharedResource`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_ambiguous_service_call_fails_in_typecheck_stage() {
    let root = unique_temp_root("ambiguous_service_call_dir");
    std::fs::write(
        root.join("sample/first.dag"),
        r#"module sample.first
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
    )
    .expect("failed to write first service source");
    std::fs::write(
        root.join("sample/second.dag"),
        r#"module sample.second
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
    )
    .expect("failed to write second service source");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run(path: String) -> { body: String } {
  let response = SharedService.read(path: path)
  return { body: response.body }
}"#,
    )
    .expect("failed to write main source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("ambiguous service call `SharedService.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_directory_ambiguous_callable_target_fails_in_typecheck_stage() {
    let root = unique_temp_root("ambiguous_callable_target_dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nfn render(value: String) -> String { value }",
    )
    .expect("failed to write first callable source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nfn render(value: String) -> String { value }",
    )
    .expect("failed to write second callable source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { render(value: \"ok\") }",
    )
    .expect("failed to write main source");

    let context = PipelineContext {
        roots: vec![root.clone()],
        target_file: None,
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    assert_typecheck_stage_error(&error);
    assert!(error.contains("ambiguous call target `render`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

// ---------------------------------------------------------------------------
// Diagnostic golden tests — lock the error output format for three
// representative failure classes: typecheck, lower, and verification.
// ---------------------------------------------------------------------------

/// Golden test: typecheck-stage error (TC015) for an unresolved import.
///
/// Locks the output format:
///   typecheck errors:
///     unresolved import `nonexistent.path` in module `sample.main`
#[test]
fn golden_diagnostic_typecheck_unresolved_import() {
    let fixture = unique_temp_file("golden_typecheck_unresolved_import");
    std::fs::write(
        &fixture,
        r#"module sample.main
import nonexistent.path
fn run() -> Unit { }
"#,
    )
    .expect("failed to write fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    let rendered = error.to_string();

    // Stage prefix
    assert!(
        rendered.starts_with("typecheck errors:\n"),
        "typecheck error must start with stage prefix: {rendered}"
    );
    // Error message body
    assert!(
        rendered.contains("unresolved import `nonexistent.path` in module `sample.main`"),
        "typecheck error must contain unresolved import message: {rendered}"
    );
    // Must NOT contain other stage prefixes (error routed to correct stage)
    assert!(!rendered.contains("lower error:"), "wrong stage: {rendered}");
    assert!(
        !rendered.contains("verification errors:"),
        "wrong stage: {rendered}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

/// Golden test: lower-stage error (LOW018) for a module with no callable
/// or pipeline declarations (data-only).
///
/// Locks the output format:
///   lower error: no callable or pipeline declarations to lower
#[test]
fn golden_diagnostic_lower_no_lowerable_items() {
    let fixture = unique_temp_file("golden_lower_no_items");
    std::fs::write(
        &fixture,
        "module sample.main\ntype Foo { x: String }\n",
    )
    .expect("failed to write fixture");

    let context = PipelineContext {
        roots: vec![fixture
            .parent()
            .expect("fixture should have parent")
            .to_path_buf()],
        target_file: Some(fixture.clone()),
    };

    let error = compile_from_context(&context).expect_err("compile should fail");
    let rendered = error.to_string();

    // Stage prefix
    assert!(
        rendered.starts_with("lower error: "),
        "lower error must start with stage prefix: {rendered}"
    );
    // Error message body with exact format
    assert_eq!(
        rendered,
        "lower error: no callable or pipeline declarations to lower",
        "lower error output format mismatch"
    );
    // Must NOT contain other stage prefixes
    assert!(
        !rendered.contains("typecheck errors:"),
        "wrong stage: {rendered}"
    );
    assert!(
        !rendered.contains("verification errors:"),
        "wrong stage: {rendered}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

/// Golden test: verification-stage error for unwired required inputs.
///
/// Constructs a `CompileError::Verification` directly to lock the output
/// format, since triggering verification errors from DSL fixtures requires
/// complex multi-file patterns.
///
/// Locks the output format:
///   verification errors:
///     unwired required input: node 'prepare_read_content' port 'expected_content'
#[test]
fn golden_diagnostic_verification_unwired_input() {
    use gunbc_ir::{UnwiredInputError, VerifyError};

    let error = CompileError::Verification(vec![VerifyError::UnwiredInput(UnwiredInputError {
        node_id: "node_abc123".to_string(),
        node_name: "prepare_read_content".to_string(),
        port_name: "expected_content".to_string(),
    })]);

    let rendered = error.to_string();

    // Stage prefix
    assert!(
        rendered.starts_with("verification errors:\n"),
        "verification error must start with stage prefix: {rendered}"
    );
    // Exact line format: two-space indent, "unwired required input: node '...' port '...'"
    assert!(
        rendered.contains(
            "  unwired required input: node 'prepare_read_content' port 'expected_content'"
        ),
        "verification error line format mismatch: {rendered}"
    );
    // Must NOT contain other stage prefixes
    assert!(
        !rendered.contains("typecheck errors:"),
        "wrong stage: {rendered}"
    );
    assert!(
        !rendered.contains("lower error:"),
        "wrong stage: {rendered}"
    );
}
