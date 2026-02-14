//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//! Progress display is automatic based on terminal capabilities.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --mode=verify
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

#![deny(dead_code)]
use gunbc_dag::testgen_dag::graph::build_testgen_graph;
use gunbc_dag::testgen_resource_def;
use gunbc_exec::{
    execute_and_display, execute_and_display_with_result, print_attention, AttentionLevel,
    BoundaryMocks, ExecutionMode,
};
use std::io::IsTerminal;
use gunbc_ir::resource::{
    update_resource_manifest, ExecMode, ManagedResource, ManifestEntry, ManifestUpdateError,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use gunbc_lib_transport::TransportIo;
// Force-link crates that register testgen targets.
use gunbc_deps as _;
use gunbc_gist as _;
use gunbc_lib_llm_ops as _;
use gunbc_lib_review as _;
use gunbc_testgen_registry::{iter_dag_specs, DagSpecDef};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use std::process;

/// Build all testgen targets from the auto-discovery registry.
fn build_targets() -> Vec<&'static DagSpecDef> {
    let mut targets: Vec<&'static DagSpecDef> = iter_dag_specs().collect();
    targets.sort_by(|a, b| a.name.cmp(b.name));
    targets
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from(".");
    let mut dry_run = false;
    let mut resource_mode = ExecMode::Ensure;
    let mut check_deprecated = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output-dir" => {
                i += 1;
                if i < args.len() {
                    output_dir = PathBuf::from(&args[i]);
                }
            }
            "-n" | "--dry-run" => dry_run = true,
            "-c" | "--check" => {
                resource_mode = ExecMode::Verify;
                check_deprecated = true;
            }
            "--mode" => {
                i += 1;
                if i < args.len() {
                    match ExecMode::parse_strict(&args[i]) {
                        Ok(parsed) => resource_mode = parsed,
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: --mode requires a value (verify|ensure)");
                    process::exit(1);
                }
            }
            arg if arg.starts_with("--mode=") => {
                let mode_str = arg.trim_start_matches("--mode=");
                match ExecMode::parse_strict(mode_str) {
                    Ok(parsed) => resource_mode = parsed,
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        process::exit(1);
                    }
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("error: unknown flag '{}'", other);
                process::exit(1);
            }
        }
        i += 1;
    }

    if check_deprecated {
        eprintln!("Warning: --check is deprecated; use --mode=verify");
    }

    if let Err(err) = ensure_lint_upsert() {
        eprintln!("preflight failed: {}", err);
        process::exit(1);
    }

    let targets = build_targets();
    if targets.is_empty() {
        eprintln!("No testgen targets registered.");
        process::exit(1);
    }

    // Build the graph
    let dag = match build_testgen_graph(&targets, &output_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Collect target names and output paths for wiring
    let target_info: Vec<(String, String)> = targets
        .iter()
        .map(|t| {
            let config = t.to_def();
            let path = output_dir
                .join(&config.output_path)
                .to_string_lossy()
                .to_string();
            (config.name.clone(), path)
        })
        .collect();

    let mut path_by_node: HashMap<String, String> = HashMap::new();
    for (name, path) in &target_info {
        path_by_node.insert(format!("prepare_read_{name}"), path.clone());
        path_by_node.insert(format!("prepare_write_{name}"), path.clone());
    }

    // Set up entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(resource_mode == ExecMode::Verify),
                );
            }
            "path" => {
                if let Some(path) = path_by_node.get(&node_id.0) {
                    input_mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str(path.clone()),
                    );
                }
            }
            _ => {}
        }
    }

    // Set up execution mode
    let mode = if dry_run && resource_mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();

        for (name, path) in &target_info {
            let read_node = format!("execute_read_{}", name);
            let write_node = format!("execute_{}_transport", name);

            // Read transport mock
            mocks.set_value(
                &read_node,
                "response",
                Value::Response(TransportResponse::File(FileResponse {
                    path: path.clone(),
                    operation: FileOp::Read,
                    success: true,
                    content: Some("<DRY-RUN>".into()),
                    exists: None,
                    error: None,
                })),
            );

            // Write transport mock
            let response_key = format!("{}_response", name);
            let path_key = format!("{}_written_path", name);
            let content_key = format!("{}_content", name);

            mocks.set_value(
                &write_node,
                &response_key,
                Value::Response(TransportResponse::File(FileResponse {
                    path: path.clone(),
                    operation: FileOp::Write,
                    success: true,
                    content: Some("<DRY-RUN>".into()),
                    exists: Some(true),
                    error: None,
                })),
            );
            mocks.set_value(&write_node, &path_key, Value::Str("<DRY-RUN>".to_string()));
            mocks.set_value(
                &write_node,
                &content_key,
                Value::Str("<DRY-RUN>".to_string()),
            );
            mocks.set_value(&write_node, "skip", Value::Bool(false));
            mocks.set_value(&write_node, "skip_reason", Value::Str(String::new()));
        }

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    let animated = std::io::stdout().is_terminal();

    if resource_mode == ExecMode::Verify {
        // Check mode: execute through shared display path and inspect log outputs.
        match execute_and_display_with_result(&dag, mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                let log = result.log;
                let mut ok_count = 0;
                let mut stale = Vec::new();

                for (name, path) in &target_info {
                    let compare_node = format!("compare_{}_content", name);
                    let fresh = log
                        .entries
                        .iter()
                        .find(|e| e.node_id == compare_node)
                        .and_then(|e| e.outputs.get("fresh"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if fresh {
                        println!("[{}] up to date", name);
                        ok_count += 1;
                    } else {
                        println!("[{}] STALE - needs regeneration", name);
                        stale.push(path.as_str());
                    }
                }

                println!();
                println!("check complete: {} ok, {} stale", ok_count, stale.len());

                if !stale.is_empty() {
                    let mut body = String::new();
                    body.push_str("Run `make testgen` to regenerate:\n");
                    for path in &stale {
                        writeln!(body, "  {path}").unwrap();
                    }
                    print_attention(
                        AttentionLevel::Error,
                        "testgen --mode=verify: generated tests are out of date",
                        body.trim_end(),
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(
                    AttentionLevel::Error,
                    "testgen --mode=verify failed",
                    &e.to_string(),
                );
                process::exit(1);
            }
        }
    } else {
        // Print header
        println!("testgen");
        println!("  output_dir: {}", output_dir.display());
        println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
        println!("  resource_mode: {}", resource_mode);
        println!("  targets: {}", targets.len());
        println!();

        // Execute and display (progress or classic based on terminal)
        execute_and_display(&dag, mode, animated, None, Some(&input_mocks));

        // Update manifest after successful generation (not in DAG - post-execution step)
        if !dry_run && resource_mode == ExecMode::Ensure {
            update_manifest_after_testgen();
        }
    }
}

// ============================================================================
// Resource Manifest Support
// ============================================================================

/// Update the resource manifest after successful testgen.
fn update_manifest_after_testgen() {
    println!();
    println!("Updating resource manifest...");

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
            let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
            Ok(ManifestEntry::new(key, file_count)
                .with_outputs(self.outputs.clone())
                .with_input_files(input_files))
        }
    }

    let def = testgen_resource_def();
    let resource = TestgenResource {
        def: def.clone(),
        outputs: Vec::new(),
    };

    let io = TransportIo::new();
    match update_resource_manifest(&resource, &io) {
        Ok(()) => {
            println!("Resource manifest updated.");
        }
        Err(ManifestUpdateError::Load(e)) => {
            eprintln!("Failed to load manifest: {e}");
        }
        Err(ManifestUpdateError::Save(e)) => {
            eprintln!("Failed to write manifest: {e}");
        }
        Err(ManifestUpdateError::Acquire(e)) => {
            eprintln!("Failed to update manifest: {e}");
        }
    }
}

fn print_help() {
    println!("testgen - Generate tests from DAG structures and MockSpecs");
    println!("Usage:");
    println!("    gunbc-testgen [OPTIONS]");
    println!();
    println!("Options:");
    println!("    -o, --output-dir <DIR>  Output directory (default: current)");
    println!("    -n, --dry-run           Show what would be generated without writing");
    println!("    --mode=MODE             Resource mode: verify (CI) or ensure (default)");
    println!("    -c, --check             Deprecated alias for --mode=verify");
    println!("    -h, --help              Show this help message");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
