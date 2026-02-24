//! Tool dispatch: in-process tool execution for workflow units.
//!
//! Each tool's execution logic is extracted from its former standalone binary
//! into a callable function. The workflow executor calls [`dispatch_tool()`]
//! instead of spawning subprocesses.
//!
//! This eliminates 8 standalone binaries and makes the DSL the single source
//! of truth (Lane A: "one representation").

#![allow(clippy::disallowed_methods)] // Build-time file generation uses std::fs

use crate::tool_runner::{
    freshness_steps_planned, print_tool_header, run_tool_result, update_freshness_manifest_if_needed,
    RunToolOptions,
};
use crate::wire_fs_env_write_mock;
use gunbc_exec::{
    compose_with_freshness, execute_and_display_with_result, print_attention, AttentionLevel,
    BoundaryMocks, ExecutionMode,
};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use std::fmt;
use std::io::IsTerminal;

// ============================================================================
// Error type
// ============================================================================

#[derive(Debug)]
pub enum ToolError {
    GraphBuild(String),
    Execution(String),
    DriftDetected(Vec<String>),
    UnknownTool(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphBuild(msg) => write!(f, "graph build failed: {msg}"),
            Self::Execution(msg) => write!(f, "execution failed: {msg}"),
            Self::DriftDetected(files) => {
                write!(f, "drift detected in: {}", files.join(", "))
            }
            Self::UnknownTool(name) => write!(f, "unknown tool: {name}"),
        }
    }
}

// ============================================================================
// Main dispatch entry point
// ============================================================================

/// Dispatch a tool by name, executing it in-process.
///
/// This replaces what each standalone binary's `main()` did: parse args,
/// build DAG, wire inputs, execute. Called by the workflow executor for
/// `UnitCommand::ToolDispatch` entries.
pub fn dispatch_tool(name: &str, mode: ExecMode, dry_run: bool) -> Result<(), ToolError> {
    match name {
        "build" => run_build(dry_run),
        "bootstrap" => run_bootstrap(mode, dry_run),
        "testgen" => run_testgen(mode, dry_run),
        "makegen" => run_makegen(mode, dry_run),
        "pragma" => run_pragma(mode, dry_run),
        "docgen" => run_docgen(dry_run),
        "codegen-dag" => run_codegen_dag(mode, dry_run),
        "review" => run_review(dry_run),
        _ => Err(ToolError::UnknownTool(name.to_string())),
    }
}

// ============================================================================
// build
// ============================================================================

fn run_build(dry_run: bool) -> Result<(), ToolError> {
    let dag = crate::build::build_build_graph().map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
        wire_fs_env_write_mock(&dag, &mut mocks);

        mocks.set_value("execute_build", "response", ok_shell());
        mocks.set_value("execute_test", "response", ok_shell());
        mocks.set_value("execute_test", "skip", Value::Bool(false));
        mocks.set_value("execute_test", "skip_reason", Value::Str(String::new()));
        mocks.set_value("execute_clippy", "response", ok_shell());
        mocks.set_value("execute_clippy", "skip", Value::Bool(false));
        mocks.set_value("execute_clippy", "skip_reason", Value::Str(String::new()));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    print_tool_header(
        "build",
        &[(
            "mode",
            if dry_run { "dry-run" } else { "real" }.to_string(),
        )],
    );
    run_tool_result(
        dag,
        mode,
        RunToolOptions {
            success_port: Some("overall_success"),
            with_freshness: true,
            ..RunToolOptions::default()
        },
    )
    .map_err(ToolError::Execution)
}

// ============================================================================
// bootstrap
// ============================================================================

fn bootstrap_path_for_node(node_id: &str) -> Option<&'static str> {
    if node_id.contains("gitignore") || node_id.contains("bootstrap_2") {
        Some(".gitignore")
    } else if node_id.contains("makefile") || node_id.contains("bootstrap") {
        Some("Makefile")
    } else {
        None
    }
}

fn run_bootstrap(mode: ExecMode, dry_run: bool) -> Result<(), ToolError> {
    use crate::resources::{GITIGNORE_OUTPUT_PATH, MAKEFILE_OUTPUT_PATH};
    use gunbc_ir::resource::{
        update_resource_manifest, ManagedResource, ManifestEntry, ResourceDef, ResourceError,
        ResourceIo, ResourceManifest,
    };
    use gunbc_lib_transport::TransportIo;
    use std::fmt::Write;
    use std::path::PathBuf;

    let dag = crate::bootstrap::build_bootstrap_graph()
        .map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    // Wire entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(mode == ExecMode::Verify),
                );
            }
            "path" => {
                let path = if node_id.0.contains("Find_ListDirs") {
                    "crates"
                } else if let Some(path) = bootstrap_path_for_node(&node_id.0) {
                    path
                } else {
                    continue;
                };
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(path.to_string()),
                );
            }
            "max_depth" if node_id.0.contains("Find_ListDirs") => {
                input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Int(1));
            }
            "min_depth" if node_id.0.contains("Find_ListDirs") => {
                input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Int(1));
            }
            _ => {}
        }
    }

    // Set up execution mode
    let exec_mode = if dry_run && mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
        wire_fs_env_write_mock(&dag, &mut mocks);

        mocks.set_value(
            "execute_scan_workspace",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok(
                "crates/example\n",
            ))),
        );

        // Makefile read
        mocks.set_value(
            "execute_read_makefile",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<DRY-RUN>".into()),
                bytes: None,
                exists: None,
                error: None,
            })),
        );

        // Makefile write transport
        mocks.set_value("execute_makefile_transport", "makefile_response", ok_shell());
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value("execute_makefile_transport", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_makefile_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        // Gitignore read
        mocks.set_value(
            "execute_read_gitignore",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<DRY-RUN>".into()),
                bytes: None,
                exists: None,
                error: None,
            })),
        );

        // Gitignore write transport
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_response",
            ok_shell(),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value("execute_gitignore_transport", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_gitignore_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    let steps = gunbc_lib_transport::check_and_plan_freshness();
    let ran_freshness_steps = freshness_steps_planned(steps.as_deref());
    let dag = compose_with_freshness(dag, steps);

    if mode == ExecMode::Verify {
        let animated = std::io::stdout().is_terminal();
        match execute_and_display_with_result(&dag, exec_mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                update_freshness_manifest_if_needed(ran_freshness_steps);
                let log = result.log;

                let makefile_fresh = log
                    .entries
                    .iter()
                    .find(|e| e.node_id == "compare_makefile_content")
                    .and_then(|e| e.outputs.get("fresh"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let gitignore_fresh = log
                    .entries
                    .iter()
                    .find(|e| e.node_id == "compare_gitignore_content")
                    .and_then(|e| e.outputs.get("fresh"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut drifted = Vec::new();
                if !makefile_fresh {
                    drifted.push("Makefile".to_string());
                }
                if !gitignore_fresh {
                    drifted.push(".gitignore".to_string());
                }

                if drifted.is_empty() {
                    let ok_count = 2;
                    println!("bootstrap --mode=verify: {ok_count} files up to date");
                    Ok(())
                } else {
                    let mut body = String::new();
                    for path in &drifted {
                        writeln!(body, "DRIFT  {path}").unwrap();
                    }
                    print_attention(
                        AttentionLevel::Error,
                        "bootstrap --mode=verify: drift detected",
                        body.trim_end(),
                    );
                    Err(ToolError::DriftDetected(drifted))
                }
            }
            Err(e) => Err(ToolError::Execution(e.to_string())),
        }
    } else {
        print_tool_header(
            "bootstrap",
            &[
                (
                    "mode",
                    if dry_run { "dry-run" } else { "real" }.to_string(),
                ),
                ("resource_mode", mode.to_string()),
            ],
        );
        run_tool_result(
            dag,
            exec_mode,
            RunToolOptions {
                input_mocks: Some(&input_mocks),
                ..RunToolOptions::default()
            },
        )
        .map_err(ToolError::Execution)?;
        update_freshness_manifest_if_needed(ran_freshness_steps);

        if !dry_run && mode == ExecMode::Ensure {
            #[derive(Clone)]
            struct BootstrapResource {
                def: ResourceDef,
                outputs: Vec<PathBuf>,
            }

            impl ManagedResource for BootstrapResource {
                fn definition(&self) -> &ResourceDef {
                    &self.def
                }

                fn create(
                    &self,
                    manifest: &ResourceManifest,
                    io: &dyn ResourceIo,
                ) -> Result<ManifestEntry, ResourceError> {
                    let (key, file_count, input_files) =
                        self.compute_key_with_file_list(manifest, io)?;
                    Ok(ManifestEntry::new(key, file_count)
                        .with_outputs(self.outputs.clone())
                        .with_input_files(input_files))
                }
            }

            println!();
            println!("Updating resource manifest...");
            let io = TransportIo::new();
            let resources = [
                BootstrapResource {
                    def: crate::makefile_resource_def(),
                    outputs: vec![PathBuf::from(MAKEFILE_OUTPUT_PATH)],
                },
                BootstrapResource {
                    def: crate::gitignore_resource_def(),
                    outputs: vec![PathBuf::from(GITIGNORE_OUTPUT_PATH)],
                },
            ];

            for resource in &resources {
                if let Err(e) = update_resource_manifest(resource, &io) {
                    eprintln!("Failed to update manifest: {e}");
                }
            }
            println!("Resource manifest updated.");
        }

        Ok(())
    }
}

// ============================================================================
// testgen
// ============================================================================

fn run_testgen(mode: ExecMode, dry_run: bool) -> Result<(), ToolError> {
    use crate::testgen_dag::{
        build_mock_spec_from_test, build_testgen_target_def, compile_dag_for_test,
        dag_builder_call_for_module, discover_dag_tests,
    };
    use gunbc_codegen::FileWriter;
    use gunbc_ir::resource::{
        update_resource_manifest, ManagedResource, ManifestEntry, ResourceDef, ResourceError,
        ResourceIo, ResourceManifest,
    };
    use gunbc_ir::WorkspaceLayout;
    use gunbc_lib_transport::TransportIo;
    use gunbc_testgen_registry::generate_target;
    use std::fmt::Write;
    use std::path::PathBuf;

    let output_dir = PathBuf::from(".");

    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|e| ToolError::Execution(format!("Failed to resolve workspace layout: {e}")))?;
    let dsl_root = layout.workspace_root.join("dsl");

    let targets = discover_dag_tests(&dsl_root);
    if targets.is_empty() {
        println!("No .dag test blocks found; skipping test generation.");
        return Ok(());
    }

    let is_verify = mode == ExecMode::Verify;

    print_tool_header(
        "testgen",
        &[
            ("output_dir", output_dir.display().to_string()),
            (
                "mode",
                if is_verify {
                    "verify"
                } else if dry_run {
                    "dry-run"
                } else {
                    "real"
                }
                .to_string(),
            ),
            ("targets", targets.len().to_string()),
        ],
    );

    let io = TransportIo::new();
    let writer = FileWriter::new(dry_run || is_verify, &io);

    let mut ok_count = 0;
    let mut written_count = 0;
    let mut stale = Vec::new();
    let mut errors = Vec::new();

    for target in &targets {
        let dag = match compile_dag_for_test(&target.dsl_module) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!(
                    "{}: failed to compile DAG: {e}",
                    target.test_name
                ));
                continue;
            }
        };

        let spec = build_mock_spec_from_test(&dag, target);
        let dag_builder_call = dag_builder_call_for_module(&target.dsl_module);
        let config = build_testgen_target_def(target, &output_dir, &dag_builder_call);

        let test_code = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            generate_target(&config, dag, spec)
        })) {
            Ok(code) => code,
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| e.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown panic");
                errors.push(format!("{}: {msg}", target.test_name));
                continue;
            }
        };

        let output_path = config.output_path.as_ref();
        match writer.write_if_changed(output_path, &test_code) {
            Ok(result) => {
                if result.changed {
                    if is_verify {
                        println!("[{}] STALE - needs regeneration", target.test_name);
                        stale.push(output_path.to_string());
                    } else if dry_run {
                        println!("[{}] would write {}", target.test_name, output_path);
                        written_count += 1;
                    } else {
                        println!("[{}] updated {}", target.test_name, output_path);
                        written_count += 1;
                    }
                } else {
                    println!("[{}] up to date", target.test_name);
                    ok_count += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{}: write failed: {e}", target.test_name));
            }
        }
    }

    // Summary
    println!();
    if !errors.is_empty() {
        let mut body = String::new();
        for err in &errors {
            writeln!(body, "  {err}").unwrap();
        }
        print_attention(AttentionLevel::Error, "testgen errors", body.trim_end());
    }

    if is_verify {
        println!("check complete: {} ok, {} stale", ok_count, stale.len());
        if !stale.is_empty() {
            return Err(ToolError::DriftDetected(stale));
        }
    } else {
        println!(
            "testgen complete: {} up to date, {} {}",
            ok_count,
            written_count,
            if dry_run { "would write" } else { "written" }
        );
    }

    if !errors.is_empty() {
        return Err(ToolError::Execution(format!(
            "{} testgen errors",
            errors.len()
        )));
    }

    if !dry_run && !is_verify {
        #[derive(Clone)]
        struct TestgenResource {
            def: ResourceDef,
            outputs: Vec<PathBuf>,
        }

        impl ManagedResource for TestgenResource {
            fn definition(&self) -> &ResourceDef {
                &self.def
            }

            fn create(
                &self,
                manifest: &ResourceManifest,
                io: &dyn ResourceIo,
            ) -> Result<ManifestEntry, ResourceError> {
                let (key, file_count, input_files) =
                    self.compute_key_with_file_list(manifest, io)?;
                Ok(ManifestEntry::new(key, file_count)
                    .with_outputs(self.outputs.clone())
                    .with_input_files(input_files))
            }
        }

        println!();
        println!("Updating resource manifest...");
        let def = crate::testgen_resource_def();
        let resource = TestgenResource {
            def,
            outputs: Vec::new(),
        };
        let io_for_manifest = TransportIo::new();
        match update_resource_manifest(&resource, &io_for_manifest) {
            Ok(()) => println!("Resource manifest updated."),
            Err(e) => eprintln!("Failed to update manifest: {e}"),
        }
    }

    Ok(())
}

// ============================================================================
// makegen
// ============================================================================

fn run_makegen(mode: ExecMode, dry_run: bool) -> Result<(), ToolError> {
    use crate::resources::MAKEFILE_OUTPUT_PATH;
    use gunbc_ir::resource::{
        update_resource_manifest, ManagedResource, ManifestEntry, ResourceDef, ResourceError,
        ResourceIo, ResourceManifest,
    };
    use gunbc_lib_transport::TransportIo;
    use std::path::PathBuf;

    let path = "Makefile".to_string();

    let dag = crate::makegen::build_makegen_graph()
        .map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    // Wire entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "path" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(path.clone()),
                );
            }
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(mode == ExecMode::Verify),
                );
            }
            _ => {}
        }
    }

    let exec_mode = if dry_run && mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        wire_fs_env_write_mock(&dag, &mut mocks);
        mocks.set_value(
            "execute_read_makegen",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: path.clone(),
                operation: FileOp::Read,
                success: true,
                content: Some("<DRY-RUN>".to_string()),
                bytes: None,
                exists: None,
                error: None,
            })),
        );
        mocks.set_value(
            "execute_makegen_transport",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: path.clone(),
                operation: FileOp::Write,
                success: true,
                content: Some("<DRY-RUN>".to_string()),
                bytes: None,
                exists: None,
                error: None,
            })),
        );
        mocks.set_value(
            "execute_makegen_transport",
            "makegen_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_makegen_transport",
            "makegen_content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value("execute_makegen_transport", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_makegen_transport",
            "skip_reason",
            Value::Str(String::new()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    let animated = std::io::stdout().is_terminal();
    let steps = gunbc_lib_transport::check_and_plan_freshness();
    let ran_freshness_steps = freshness_steps_planned(steps.as_deref());
    let dag = compose_with_freshness(dag, steps);

    if mode == ExecMode::Verify {
        match execute_and_display_with_result(&dag, exec_mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                update_freshness_manifest_if_needed(ran_freshness_steps);
                let log = result.log;
                let fresh = log
                    .entries
                    .iter()
                    .find(|e| e.node_id == "compare_makegen_content")
                    .or_else(|| log.entries.iter().find(|e| e.outputs.contains_key("fresh")))
                    .and_then(|e| e.outputs.get("fresh"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if fresh {
                    println!("makegen --mode=verify: 1 file up to date");
                    Ok(())
                } else {
                    print_attention(
                        AttentionLevel::Error,
                        "makegen --mode=verify: drift detected",
                        &format!("DRIFT  {path}"),
                    );
                    Err(ToolError::DriftDetected(vec![path]))
                }
            }
            Err(e) => Err(ToolError::Execution(e.to_string())),
        }
    } else {
        print_tool_header(
            "makegen",
            &[
                ("path", path.to_string()),
                ("mode", if dry_run { "dry-run" } else { "real" }.to_string()),
                ("resource_mode", mode.to_string()),
            ],
        );
        run_tool_result(
            dag,
            exec_mode,
            RunToolOptions {
                input_mocks: Some(&input_mocks),
                ..RunToolOptions::default()
            },
        )
        .map_err(ToolError::Execution)?;
        update_freshness_manifest_if_needed(ran_freshness_steps);

        if !dry_run && mode == ExecMode::Ensure {
            #[derive(Clone)]
            struct MakefileResource {
                def: ResourceDef,
                outputs: Vec<PathBuf>,
            }

            impl ManagedResource for MakefileResource {
                fn definition(&self) -> &ResourceDef {
                    &self.def
                }

                fn create(
                    &self,
                    manifest: &ResourceManifest,
                    io: &dyn ResourceIo,
                ) -> Result<ManifestEntry, ResourceError> {
                    let (key, file_count, input_files) =
                        self.compute_key_with_file_list(manifest, io)?;
                    Ok(ManifestEntry::new(key, file_count)
                        .with_outputs(self.outputs.clone())
                        .with_input_files(input_files))
                }
            }

            println!();
            println!("Updating resource manifest...");
            let resource = MakefileResource {
                def: crate::makefile_resource_def(),
                outputs: vec![PathBuf::from(MAKEFILE_OUTPUT_PATH)],
            };
            let io = TransportIo::new();
            match update_resource_manifest(&resource, &io) {
                Ok(()) => println!("Resource manifest updated."),
                Err(e) => eprintln!("Failed to update manifest: {e}"),
            }
        }

        Ok(())
    }
}

// ============================================================================
// pragma
// ============================================================================

fn run_pragma(mode: ExecMode, dry_run: bool) -> Result<(), ToolError> {
    use std::fmt::Write;

    let dag =
        crate::pragma::build_pragma_graph().map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    let file_paths: &[(&str, &str)] = &[
        ("pragma_3", "tools/pragma-lint-policy.txt"),
        ("pragma_2", "tools/disallowed-methods-allowlist.txt"),
        ("pragma", "clippy.toml"),
    ];

    // Wire entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(mode == ExecMode::Verify),
                );
            }
            "path" => {
                let path = file_paths
                    .iter()
                    .find(|(key, _)| node_id.0.contains(key))
                    .map(|(_, path)| *path);
                if let Some(path) = path {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(path.to_string()),
                    );
                }
            }
            _ => {}
        }
    }

    let exec_mode = if dry_run && mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        wire_fs_env_write_mock(&dag, &mut mocks);

        for (key, path) in file_paths {
            let read_node = format!("execute_read_{}", key);
            let write_node = format!("execute_{}_transport", key);

            mocks.set_value(
                &read_node,
                "response",
                Value::Response(TransportResponse::File(FileResponse {
                    path: (*path).into(),
                    operation: FileOp::Read,
                    success: true,
                    content: Some("<DRY-RUN>".into()),
                    bytes: None,
                    exists: None,
                    error: None,
                })),
            );

            mocks.set_value(
                &write_node,
                "response",
                Value::Response(TransportResponse::File(FileResponse {
                    path: (*path).into(),
                    operation: FileOp::Write,
                    success: true,
                    content: Some("<DRY-RUN>".into()),
                    bytes: None,
                    exists: Some(true),
                    error: None,
                })),
            );
        }

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    let animated = std::io::stdout().is_terminal();

    if mode == ExecMode::Verify {
        match execute_and_display_with_result(&dag, exec_mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                let log = result.log;
                let mut ok_count = 0;
                let mut drifted = Vec::new();

                for (key, path) in file_paths {
                    let compare_node = format!("compare_{}_content", key);
                    let fresh = log
                        .entries
                        .iter()
                        .find(|e| e.node_id == compare_node)
                        .and_then(|e| e.outputs.get("fresh"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if fresh {
                        ok_count += 1;
                    } else {
                        drifted.push(path.to_string());
                    }
                }

                if drifted.is_empty() {
                    println!(
                        "pragma --mode=verify: {} file{} up to date",
                        ok_count,
                        if ok_count == 1 { "" } else { "s" }
                    );
                    Ok(())
                } else {
                    let mut body = String::new();
                    for path in &drifted {
                        writeln!(body, "DRIFT  {path}").unwrap();
                    }
                    print_attention(
                        AttentionLevel::Error,
                        "pragma --mode=verify: drift detected",
                        body.trim_end(),
                    );
                    Err(ToolError::DriftDetected(drifted))
                }
            }
            Err(e) => Err(ToolError::Execution(e.to_string())),
        }
    } else {
        print_tool_header(
            "pragma",
            &[
                ("mode", if dry_run { "dry-run" } else { "real" }.to_string()),
                ("resource_mode", mode.to_string()),
            ],
        );
        run_tool_result(
            dag,
            exec_mode,
            RunToolOptions {
                input_mocks: Some(&input_mocks),
                ..RunToolOptions::default()
            },
        )
        .map_err(ToolError::Execution)
    }
}

// ============================================================================
// docgen
// ============================================================================

const AB_DOC_PATH: &str = "docs/ab-writing-workflows.md";

fn run_docgen(dry_run: bool) -> Result<(), ToolError> {
    let dag =
        crate::docgen::build_docgen_graph().map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    let mode = if dry_run {
        let mut mocks = build_docgen_dry_run_mocks();
        wire_fs_env_write_mock(&dag, &mut mocks);
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    print_tool_header(
        "docgen",
        &[(
            "mode",
            if dry_run { "dry-run" } else { "real" }.to_string(),
        )],
    );
    run_tool_result(
        dag,
        mode,
        RunToolOptions {
            with_freshness: true,
            ..RunToolOptions::default()
        },
    )
    .map_err(ToolError::Execution)
}

fn build_docgen_dry_run_mocks() -> BoundaryMocks {
    use crate::DOCGEN_READ_TARGETS;

    let mut mocks = BoundaryMocks::new();
    for target in DOCGEN_READ_TARGETS {
        let content = if target.name == "ab_doc_template" {
            dry_run_ab_doc_template()
        } else {
            "<DRY-RUN>"
        };
        set_docgen_read_mock(&mut mocks, target.name, target.path, content);
    }
    set_docgen_chain_mocks(
        &mut mocks,
        "ab_workflows_doc",
        AB_DOC_PATH,
        Some(dry_run_ab_doc_template()),
    );
    mocks
}

fn set_docgen_read_mock(mocks: &mut BoundaryMocks, name: &str, path: &str, content: &str) {
    let read_node = format!("execute_{name}");
    mocks.set_value(
        &read_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some(content.to_string()),
            bytes: None,
            exists: None,
            error: None,
        })),
    );
}

fn set_docgen_chain_mocks(
    mocks: &mut BoundaryMocks,
    name: &str,
    path: &str,
    read_content: Option<&str>,
) {
    let read_node = format!("execute_read_{name}");
    let write_node = format!("execute_{name}_transport");
    let read_content = read_content.unwrap_or("<DRY-RUN>").to_string();

    mocks.set_value(
        &read_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some(read_content),
            bytes: None,
            exists: None,
            error: None,
        })),
    );

    mocks.set_value(
        &write_node,
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: path.to_string(),
            operation: FileOp::Write,
            success: true,
            content: Some("<DRY-RUN>".to_string()),
            bytes: None,
            exists: None,
            error: None,
        })),
    );

    mocks.set_value(
        &write_node,
        format!("{name}_written_path"),
        Value::Str("<DRY-RUN>".to_string()),
    );
    mocks.set_value(
        &write_node,
        format!("{name}_content"),
        Value::Str("<DRY-RUN>".to_string()),
    );
    mocks.set_value(&write_node, "skip", Value::Bool(false));
    mocks.set_value(
        &write_node,
        "skip_reason",
        Value::Str("<DRY-RUN>".to_string()),
    );
}

fn dry_run_ab_doc_template() -> &'static str {
    r#"<!-- BEGIN GENERATED:clippy_mock_spec -->
<!-- END GENERATED:clippy_mock_spec -->
<!-- BEGIN GENERATED:clippy_generated_test_excerpt -->
<!-- END GENERATED:clippy_generated_test_excerpt -->
<!-- BEGIN GENERATED:appendix_a_clippy -->
<!-- END GENERATED:appendix_a_clippy -->
<!-- BEGIN GENERATED:appendix_b -->
<!-- END GENERATED:appendix_b -->
<!-- BEGIN GENERATED:appendix_c -->
<!-- END GENERATED:appendix_c -->
<!-- BEGIN GENERATED:appendix_d -->
<!-- END GENERATED:appendix_d -->
"#
}

// ============================================================================
// codegen-dag
// ============================================================================

fn run_codegen_dag(mode: ExecMode, dry_run: bool) -> Result<(), ToolError> {
    use crate::CODEGEN_STAMP_PATH;

    let dag = crate::codegen::build_codegen_graph()
        .map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    let exec_mode = if dry_run && mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
        let missing_shell = || {
            Value::Response(TransportResponse::Shell(ShellResponse::failed(
                1, "missing",
            )))
        };

        wire_fs_env_write_mock(&dag, &mut mocks);
        mocks.set_value("execute_codegen_exists", "response", missing_shell());
        mocks.set_value("execute_codegen", "response", ok_shell());
        mocks.set_value("execute_codegen", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_stamp_write",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: CODEGEN_STAMP_PATH.to_string(),
                operation: FileOp::Write,
                success: true,
                content: None,
                bytes: None,
                exists: None,
                error: None,
            })),
        );
        mocks.set_value("execute_stamp_write", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    print_tool_header(
        "codegen",
        &[
            ("mode", if dry_run { "dry-run" } else { "real" }.to_string()),
            ("resource_mode", mode.to_string()),
        ],
    );
    run_tool_result(
        dag,
        exec_mode,
        RunToolOptions {
            success_port: Some("prep_success"),
            ..RunToolOptions::default()
        },
    )
    .map_err(ToolError::Execution)
}

// ============================================================================
// review
// ============================================================================

fn run_review(dry_run: bool) -> Result<(), ToolError> {
    use gunbc_lib_review::ReviewPipelineConfig;

    let repo_path = ".".to_string();
    let depth_upper = "M".to_string();

    let (effective_provider, model) = if dry_run {
        let default_config = ReviewPipelineConfig::gunbc_default();
        (default_config.provider, default_config.model)
    } else {
        ("anthropic".to_string(), "claude-sonnet-4-20250514".to_string())
    };

    let dag = crate::build_dimension_review_graph_dsl()
        .map_err(|e| ToolError::GraphBuild(e.to_string()))?;

    // Wire entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "repo_path" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(repo_path.clone()),
                );
            }
            _ => {}
        }
    }

    let exec_mode = if dry_run {
        ExecutionMode::DryRun(BoundaryMocks::new())
    } else {
        ExecutionMode::Real
    };

    let tool_name = gunbc_ir::cargo::name("review");
    print_tool_header(
        &tool_name,
        &[
            (
                "exec",
                if dry_run { "dry-run" } else { "real" }.to_string(),
            ),
            ("provider", effective_provider),
            ("model", model),
            ("depth", depth_upper),
            ("repo", repo_path),
        ],
    );

    run_tool_result(
        dag,
        exec_mode,
        RunToolOptions {
            success_port: Some("output"),
            with_freshness: false,
            input_mocks: Some(&input_mocks),
        },
    )
    .map_err(ToolError::Execution)
}
