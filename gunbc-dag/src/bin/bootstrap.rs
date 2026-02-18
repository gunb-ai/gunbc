//! gunbc-bootstrap main entry point.
//!
//! Bootstrap tool for initializing gunbc projects.
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::resources::{GITIGNORE_OUTPUT_PATH, MAKEFILE_OUTPUT_PATH};
use gunbc_dag::{
    build_bootstrap_graph, gitignore_resource_def, makefile_resource_def, print_tool_header,
    run_tool, wire_fs_env_write_mock, RunToolOptions,
};
use gunbc_exec::{
    compose_with_freshness, execute_and_display_with_result, print_attention, AttentionLevel,
    BoundaryMocks, ExecutionMode,
};
use gunbc_ir::resource::{
    update_resource_manifest, ExecMode, ManagedResource, ManifestEntry, ManifestUpdateError,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_lib_transport::TransportIo;
use std::fmt::Write;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process;

fn main() {
    let parsed = BinaryArgs::new().with_mode().parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;
    let resource_mode = parsed.resource_mode.unwrap_or(ExecMode::Ensure);

    let animated = std::io::stdout().is_terminal();

    // Build the graph
    let dag = match build_bootstrap_graph() {
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
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(resource_mode == ExecMode::Verify),
                );
            }
            "path" => {
                // Set read paths for the file upsert check
                let path = if node_id.0.contains("makefile") {
                    "Makefile"
                } else if node_id.0.contains("gitignore") {
                    ".gitignore"
                } else {
                    continue;
                };
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(path.to_string()),
                );
            }
            _ => {}
        }
    }

    // Set up execution mode
    // In verify mode, we run Real (read transports must execute), but check_mode=true
    // forces compare_content to set skip=true on the write transports.
    // In --dry-run mode (without verify), mock all transports.
    let mode = if dry_run && resource_mode != ExecMode::Verify {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));
        wire_fs_env_write_mock(&dag, &mut mocks);

        // Scan workspace
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
                exists: None,
                error: None,
            })),
        );

        // Makefile write transport
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_response",
            ok_shell(),
        );
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
    let dag = compose_with_freshness(dag, steps);
    if resource_mode == ExecMode::Verify {
        // Check mode: execute through shared display path and inspect log outputs.
        match execute_and_display_with_result(&dag, mode, animated, None, Some(&input_mocks)) {
            Ok(result) => {
                let log = result.log;
                // Scan log for compare_*_content.fresh
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

                let mut ok_count = 0;
                let mut drifted = Vec::new();

                if makefile_fresh {
                    ok_count += 1;
                } else {
                    drifted.push("Makefile");
                }
                if gitignore_fresh {
                    ok_count += 1;
                } else {
                    drifted.push(".gitignore");
                }

                if drifted.is_empty() {
                    println!(
                        "bootstrap --mode=verify: {} file{} up to date",
                        ok_count,
                        if ok_count == 1 { "" } else { "s" }
                    );
                } else {
                    let mut body = String::new();
                    for path in &drifted {
                        writeln!(body, "DRIFT  {path}").unwrap();
                    }
                    if ok_count > 0 {
                        write!(
                            body,
                            "({} file{} ok)",
                            ok_count,
                            if ok_count == 1 { "" } else { "s" }
                        )
                        .unwrap();
                    }
                    print_attention(
                        AttentionLevel::Error,
                        "bootstrap --mode=verify: drift detected",
                        body.trim_end(),
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(
                    AttentionLevel::Error,
                    "bootstrap --mode=verify failed",
                    &e.to_string(),
                );
                process::exit(1);
            }
        }
    } else {
        print_tool_header(
            "bootstrap",
            &[
                (
                    "mode",
                    if dry_run && resource_mode != ExecMode::Verify {
                        "dry-run"
                    } else {
                        "real"
                    }
                    .to_string(),
                ),
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

        if !dry_run && resource_mode == ExecMode::Ensure {
            update_manifest_after_bootstrap();
        }
    }
}

fn update_manifest_after_bootstrap() {
    println!();
    println!("Updating resource manifest...");

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
            let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
            Ok(ManifestEntry::new(key, file_count)
                .with_outputs(self.outputs.clone())
                .with_input_files(input_files))
        }
    }

    let io = TransportIo::new();
    let resources = [
        BootstrapResource {
            def: makefile_resource_def(),
            outputs: vec![PathBuf::from(MAKEFILE_OUTPUT_PATH)],
        },
        BootstrapResource {
            def: gitignore_resource_def(),
            outputs: vec![PathBuf::from(GITIGNORE_OUTPUT_PATH)],
        },
    ];

    let mut had_error = false;
    for resource in &resources {
        match update_resource_manifest(resource, &io) {
            Ok(()) => {}
            Err(ManifestUpdateError::Load(e)) => {
                had_error = true;
                eprintln!("Failed to load manifest: {e}");
            }
            Err(ManifestUpdateError::Save(e)) => {
                had_error = true;
                eprintln!("Failed to write manifest: {e}");
            }
            Err(ManifestUpdateError::Acquire(e)) => {
                had_error = true;
                eprintln!("Failed to update manifest: {e}");
            }
        }
    }

    if !had_error {
        println!("Resource manifest updated.");
    }
}

fn print_help() {
    println!("bootstrap - Generate Makefile and .gitignore");
    println!();
    println!("USAGE:");
    println!("    bootstrap [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    --mode=MODE          Resource mode: verify (CI) or ensure (default)");
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
