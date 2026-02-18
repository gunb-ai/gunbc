//! gunbc-makegen main entry point.
//!
//! Generates Makefile from tool registry.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_codegen::file_writer::format_diff;
use gunbc_dag::resources::MAKEFILE_OUTPUT_PATH;
use gunbc_dag::{
    build_makegen_graph_dsl, freshness_steps_planned, makefile_resource_def, print_tool_header,
    run_tool, update_freshness_manifest_if_needed, wire_fs_env_write_mock, RunToolOptions,
};
use gunbc_exec::{
    compose_with_freshness, execute_and_display_with_result, print_attention, AttentionLevel,
    BoundaryMocks, ExecutionMode,
};
use gunbc_ir::resource::{
    update_resource_manifest, ExecMode, ManagedResource, ManifestEntry, ManifestUpdateError,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::TransportIo;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process;

fn main() {
    let parsed = BinaryArgs::new()
        .with_mode()
        .with_string_param("path", Some('o'), Some("Makefile"))
        .parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;
    let resource_mode = parsed.resource_mode.unwrap_or(ExecMode::Ensure);
    let path = parsed.get_string("path").unwrap_or("Makefile").to_string();

    // Build the graph
    let dag = match build_makegen_graph_dsl() {
        Ok(d) => d,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Graph build failed", &e.to_string());
            process::exit(1);
        }
    };

    // Set up entrypoint inputs
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
                    Value::Bool(resource_mode == ExecMode::Verify),
                );
            }
            _ => {}
        }
    }

    // Set up execution mode
    // In verify mode, we run Real (read transport must execute), but check_mode=true
    // forces compare_content to set skip=true on the write transport.
    // In --dry-run mode (without verify), mock all transports.
    let mode = if dry_run && resource_mode != ExecMode::Verify {
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
                exists: None,
                error: None,
            })),
        );
        mocks.set_value(
            "execute_makegen_transport",
            "makegen_response",
            Value::Response(TransportResponse::File(FileResponse {
                path: path.clone(),
                operation: FileOp::Write,
                success: true,
                content: Some("<DRY-RUN>".to_string()),
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
    if resource_mode == ExecMode::Verify {
        // Check mode: execute through shared display path and inspect log outputs.
        match execute_and_display_with_result(&dag, mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                update_freshness_manifest_if_needed(ran_freshness_steps);
                let log = result.log;
                // Scan log for compare_*_content.fresh
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
                } else {
                    print_attention(
                        AttentionLevel::Error,
                        "makegen --mode=verify: drift detected",
                        &format!("DRIFT  {path}"),
                    );
                    // Try to show a diff between on-disk content and newly rendered output.
                    let expected = log
                        .entries
                        .iter()
                        .find(|e| e.node_id == "render_makefile")
                        .and_then(|e| e.outputs.get("makefile_content"))
                        .and_then(|v| v.as_str());

                    let actual = log
                        .entries
                        .iter()
                        .find(|e| e.node_id == "execute_read_makegen")
                        .and_then(|e| e.outputs.get("response"))
                        .and_then(|v| match v {
                            Value::Response(TransportResponse::File(f)) => f.content.as_deref(),
                            _ => None,
                        });

                    if let (Some(old), Some(new)) = (actual, expected) {
                        eprintln!();
                        eprintln!("--- Drift diff (expected vs disk) ---");
                        eprintln!("{}", format_diff(old, new));
                    } else {
                        eprintln!();
                        eprintln!("(no diff available: missing expected or actual content)");
                    }

                    let fix_cmd = if path == "Makefile" {
                        "make makegen".to_string()
                    } else {
                        format!(
                            "cargo run -p gunbc-dag --bin gunbc-makegen -- --path {}",
                            path
                        )
                    };
                    eprintln!();
                    eprintln!("To fix:");
                    eprintln!("  {}", fix_cmd);
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(
                    AttentionLevel::Error,
                    "makegen --mode=verify failed",
                    &e.to_string(),
                );
                process::exit(1);
            }
        }
    } else {
        print_tool_header(
            "makegen",
            &[
                ("path", path.to_string()),
                ("mode", if dry_run { "dry-run" } else { "real" }.to_string()),
                ("resource_mode", resource_mode.to_string()),
            ],
        );
        run_tool(
            dag,
            mode,
            RunToolOptions {
                input_mocks: Some(&input_mocks),
                ..RunToolOptions::default()
            },
        );
        update_freshness_manifest_if_needed(ran_freshness_steps);

        if !dry_run && resource_mode == ExecMode::Ensure {
            update_manifest_after_makegen(&path);
        }
    }
}

fn update_manifest_after_makegen(path: &str) {
    if path != MAKEFILE_OUTPUT_PATH {
        println!(
            "Skipping resource manifest update for non-canonical Makefile path: {}",
            path
        );
        return;
    }

    println!();
    println!("Updating resource manifest...");

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
            let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
            Ok(ManifestEntry::new(key, file_count)
                .with_outputs(self.outputs.clone())
                .with_input_files(input_files))
        }
    }

    let resource = MakefileResource {
        def: makefile_resource_def(),
        outputs: vec![PathBuf::from(MAKEFILE_OUTPUT_PATH)],
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
    println!("makegen - Generate Makefile from tool registry");
    println!();
    println!("USAGE:");
    println!("    makegen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --path <VAL>     Output Makefile path");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    --mode=MODE          Resource mode: verify (CI) or ensure (default)");
    println!("    -h, --help           Print this help");
}
