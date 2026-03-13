// Binary command dispatch — eprintln is used for user-facing CLI diagnostics.
#![allow(clippy::disallowed_macros)]

use super::*;

pub(super) fn dispatch(args: &[String], cwd: &std::path::Path) {
    match args[1].as_str() {
        "viz" => {
            let (target, format) = parse_viz_args(args).unwrap_or_else(|usage| exit_usage(&usage));
            match target {
                VizTarget::SelfDag => {
                    let dag = build_compile_stage_dag();
                    let rendered = match format {
                        VizFormat::Ascii => dag.to_ascii("daglang-compiler-pipeline"),
                        VizFormat::Mermaid => dag.to_mermaid("daglang-compiler-pipeline"),
                    };
                    println!("{rendered}");
                }
                VizTarget::CompiledTarget(path) => {
                    let output = compile_target_or_exit(cwd, Some(&path));
                    let rendered = match format {
                        VizFormat::Ascii => output.lowered_dag.to_ascii("daglang-compiled"),
                        VizFormat::Mermaid => output.lowered_dag.to_mermaid("daglang-compiled"),
                    };
                    println!("{rendered}");
                }
            }
        }
        "expand" => {
            let parsed = parse_compile_command_args(
                "expand",
                args,
                "expand <file.dag> [--emit-collection-nodes]",
                true,
            )
            .unwrap_or_else(|usage| exit_usage(&usage));
            let input = parsed
                .input
                .expect("expand parser should require input target");
            let output = compile_target_or_exit_with_options(
                cwd,
                Some(&input),
                parsed.emit_collection_nodes,
            );
            println!("{}", render_expand(&output.lowered_dag));
        }
        "progress" => {
            let parsed = parse_progress_command_args(args[1].as_str(), args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit_with_options(
                cwd,
                Some(&parsed.input),
                parsed.emit_collection_nodes,
            );
            println!(
                "{}",
                render_progress_with_format(&output.derived, parsed.format)
            );
        }
        "topology" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("topology <file.dag> [--format text|json]");
            }
            let format =
                parse_output_format("topology", args).unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit(cwd, args.get(2));
            println!("{}", render_topology_with_format(&output.derived, format));
        }
        "obligations" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("obligations <file.dag> [--format text|json]");
            }
            let format =
                parse_output_format("obligations", args).unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit(cwd, args.get(2));
            println!("{}", render_obligations(&output.derived, format));
        }
        "show-triplets" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("show-triplets <file.dag> [--format text|json]");
            }
            let format = parse_output_format("show-triplets", args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit(cwd, args.get(2));
            println!("{}", render_triplets(&output.derived, format));
        }
        "report-coverage" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("report-coverage <file.dag|dir> [--format text|json]");
            }
            let format = parse_output_format("report-coverage", args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let context = match build_context(cwd, args.get(2)) {
                Ok(context) => context,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            let driver_context = daglang_driver::DriverContext {
                roots: context.roots,
                target_file: context.target_file,
            };
            let issues = daglang_driver::lint_report_coverage_from_context(&driver_context)
                .unwrap_or_else(|error| {
                    eprintln!("{error}");
                    std::process::exit(1);
                });
            match format {
                OutputFormat::Text => {
                    println!("{}", render_report_coverage_text(&issues));
                }
                OutputFormat::Json => {
                    println!("{}", render_report_coverage_json(&issues));
                }
            }
            if !issues.is_empty() {
                std::process::exit(2);
            }
        }
        "modules" => {
            let (root_arg, format) =
                parse_modules_args(args).unwrap_or_else(|usage| exit_usage(&usage));
            if let Some(root) = &root_arg {
                let normalized = path_utils::normalize_cli_path(cwd, &PathBuf::from(root));
                if let Some(error) = path_utils::check_dag_extension_casing(&normalized) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
                if let Some(error) = path_utils::check_dag_directory_conflict(&normalized) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            let roots = if let Some(root) = root_arg {
                vec![resolve_root(cwd, Some(&root))]
            } else {
                match resolve_default_roots(cwd) {
                    Ok(roots) => roots,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(1);
                    }
                }
            };
            let context = PipelineContext {
                roots,
                target_file: None,
            };
            let result = run_pipeline_or_exit(&context, PipelineStop::Report);
            match format {
                OutputFormat::Text => {
                    if let Some(report) = result.report() {
                        println!("{report}");
                    }
                }
                OutputFormat::Json => {
                    println!("{}", render_modules_result_json(&result));
                }
            }
        }
        "check" => {
            if args.len() > 3 {
                exit_usage("check <file.dag|dir>");
            }
            if let Some(input) = args.get(2) {
                let normalized = path_utils::normalize_cli_path(cwd, &PathBuf::from(input));
                if let Some(error) = path_utils::check_dag_extension_casing(&normalized) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            let configured_default_roots = if args.get(2).is_none() {
                match resolve_configured_roots(cwd) {
                    Ok(roots) => roots,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(1);
                    }
                }
            } else {
                None
            };
            let context = match build_check_pipeline_context_with_default_roots(
                cwd,
                args.get(2),
                configured_default_roots.as_deref(),
            ) {
                Ok(context) => context,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            let result = run_pipeline_or_exit(&context, PipelineStop::Build);
            if !result.diagnostics().is_empty() {
                for diagnostic in result.diagnostics() {
                    eprintln!("{diagnostic}");
                }
                std::process::exit(1);
            }
            let (parsed_files, module_graph) = match result {
                PipelineResult::Build {
                    parsed_count,
                    module_graph,
                    ..
                }
                | PipelineResult::Report {
                    parsed_count,
                    module_graph,
                    ..
                } => (parsed_count, module_graph),
                PipelineResult::Parse { .. } => {
                    eprintln!("pipeline error: expected build-stage module graph for check");
                    std::process::exit(1);
                }
            };
            match check_from_module_graph(module_graph) {
                Ok(_) => {
                    println!("OK: checked {} file(s)", parsed_files);
                }
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        "compile" => {
            let parsed = parse_compile_command_args(
                "compile",
                args,
                "compile <file.dag|dir> [--emit-collection-nodes] [--trace-stages] [--target rust|go|c|mips] [--layer 1|2] [--format summary|canonical-json] [--out <dir>|--out=<dir>] [--receipt]",
                false,
            )
            .unwrap_or_else(|usage| exit_usage(&usage));
            let normalized_out_dir = parsed
                .out_dir
                .as_ref()
                .map(|out_dir| path_utils::normalize_cli_path(cwd, &PathBuf::from(out_dir)));
            let options = CompileOptions {
                emit_collection_nodes: parsed.emit_collection_nodes,
                target: parsed.target.unwrap_or_default(),
                layer: parsed.layer.unwrap_or_default(),
                output_dir: normalized_out_dir.clone(),
                embedded_data: Default::default(),
                ..Default::default()
            };
            let mut output = compile_target_or_exit_with_compile_options(
                cwd,
                parsed.input.as_ref(),
                options.clone(),
            );
            if matches!(parsed.format, CompileOutputFormat::CanonicalJson) {
                let canonical_json =
                    render_canonical_ir_json(&output.lowered_dag).unwrap_or_else(|error| {
                        eprintln!("{error}");
                        std::process::exit(1);
                    });
                println!("{canonical_json}");
                return;
            }
            // For Layer 1 exec-runtime: embed pre-computed handler data files.
            if let Err(error) = embed_layer1_handler_data(&options, &mut output) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            let written_files = if let Some(out_dir) = normalized_out_dir.as_ref() {
                match write_emitted_files(cwd, out_dir, &output.emitted.files) {
                    Ok(files) => files,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(1);
                    }
                }
            } else {
                Vec::new()
            };
            if parsed.trace_stages {
                println!("Compilation stages:");
                for stage in [
                    "discover",
                    "parse",
                    "resolve",
                    "typecheck",
                    "lower",
                    "derive",
                    "emit",
                ] {
                    println!("  - {stage}: ok");
                }
            }
            println!(
                "Compiled {} module(s) to {} node(s), {} file(s) emitted (target={} layer={}).",
                output.emitted.summary.module_count,
                output.derived.manifest.total_nodes,
                output.emitted.files.len(),
                options.target,
                options.layer
            );
            let manifest_path = if let Some(out_dir) = normalized_out_dir.as_ref() {
                out_dir
                    .join(&output.emit_manifest_path)
                    .display()
                    .to_string()
            } else {
                output.emit_manifest_path.clone()
            };
            println!("Emit manifest: {manifest_path}");
            if let Some(progress_file) = output
                .emitted
                .files
                .iter()
                .find(|file| file.path.ends_with("progress_manifest.txt"))
            {
                let progress_path = if let Some(out_dir) = normalized_out_dir.as_ref() {
                    out_dir.join(&progress_file.path).display().to_string()
                } else {
                    progress_file.path.clone()
                };
                println!("Progress manifest: {progress_path}");
            }
            println!("Obligations: run `daglang obligations <file.dag|dir> --format text|json`");
            if parsed.out_dir.is_some() {
                for file in &written_files {
                    println!("  - {}", file.display());
                }
            } else {
                for file in &output.emitted.files {
                    println!("  - {}", file.path);
                }
            }
            // Write compile receipt JSON when --receipt is passed.
            if parsed.receipt {
                let receipt_json = match serde_json::to_string_pretty(&output.receipt) {
                    Ok(json) => json,
                    Err(error) => {
                        eprintln!("failed to serialize compile receipt: {error}");
                        std::process::exit(1);
                    }
                };
                if let Some(out_dir) = normalized_out_dir.as_ref() {
                    let receipt_path = out_dir.join("compile_receipt.json");
                    #[allow(clippy::disallowed_methods)]
                    if let Err(error) = std::fs::write(&receipt_path, &receipt_json) {
                        eprintln!("failed to write receipt: {error}");
                        std::process::exit(1);
                    }
                    println!("Receipt: {}", receipt_path.display());
                } else {
                    println!("{receipt_json}");
                }
            }
        }
        "run" => {
            let _parsed = parse_run_args(args).unwrap_or_else(|error| {
                eprintln!("{error}");
                exit_usage(
                    "run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>",
                );
            });
            eprintln!("daglang run is repo-specific and unsupported in core daglang-cli");
            std::process::exit(2);
        }
        "gen-types" => {
            let gen_args = parse_gen_types_args(args).unwrap_or_else(|usage| exit_usage(&usage));
            let pipeline_ctx = match build_context(cwd, gen_args.input.as_ref()) {
                Ok(ctx) => ctx,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            let driver_ctx = daglang_driver::DriverContext {
                roots: pipeline_ctx.roots,
                target_file: pipeline_ctx.target_file,
            };
            let filter_refs: Vec<&str> = gen_args.modules.iter().map(|s| s.as_str()).collect();
            match daglang_driver::generate_types_from_context_permissive(&driver_ctx, &filter_refs)
            {
                Ok(output) => {
                    if output.is_empty() {
                        eprintln!("No type definitions found in the specified modules.");
                        std::process::exit(1);
                    }
                    if let Some(path) = &gen_args.output {
                        #[allow(clippy::disallowed_methods)]
                        // CLI tool: direct filesystem write for codegen output
                        match std::fs::write(path, &output) {
                            Ok(()) => eprintln!("wrote {}", path),
                            Err(e) => {
                                eprintln!("failed to write {path}: {e}");
                                std::process::exit(1);
                            }
                        }
                    } else {
                        print!("{output}");
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        cmd => {
            eprintln!("Unknown command: {cmd}");
            exit_usage("<command> [args...]");
        }
    }
}

/// For Layer 1 exec-runtime compilation, embed pre-computed handler data
/// as additional files in the generated crate.
fn embed_layer1_handler_data(
    options: &CompileOptions,
    output: &mut CompileOutput,
) -> Result<(), String> {
    use daglang_driver::CodegenLayer;

    if options.layer != CodegenLayer::ExecRuntime {
        return Ok(());
    }
    let assets = daglang_emit::rust_exec_runtime::required_embedded_assets(&output.lowered_dag);
    for asset in assets {
        let path = asset.path();
        let key = asset.key();
        let data = options.embedded_data.get(key).ok_or_else(|| {
            format!("missing embedded asset `{key}` required by exec-runtime (path={path})")
        })?;
        if data.layer1_file_path != path {
            return Err(format!(
                "embedded asset `{key}` has layer1 path `{}`, expected `{path}`",
                data.layer1_file_path
            ));
        }
        let content = data.content.clone();
        output.emitted.files.push(daglang_emit::EmittedFile {
            path: path.to_string(),
            content,
        });
    }
    Ok(())
}

fn render_report_coverage_text(issues: &[daglang_driver::ReportCoverageIssue]) -> String {
    if issues.is_empty() {
        return "OK: report coverage complete".to_string();
    }
    let mut out = String::new();
    for issue in issues {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!(
            "pipeline {}.{} missing report coverage for stages: {}",
            issue.module,
            issue.pipeline,
            issue.missing_stages.join(", ")
        ));
    }
    out
}

fn render_report_coverage_json(issues: &[daglang_driver::ReportCoverageIssue]) -> String {
    let payload = json!({
        "status": if issues.is_empty() { "ok" } else { "error" },
        "issues": issues.iter().map(|issue| {
            json!({
                "module": issue.module,
                "pipeline": issue.pipeline,
                "declared_stages": issue.declared_stages,
                "covered_stages": issue.covered_stages,
                "missing_stages": issue.missing_stages,
            })
        }).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&payload).expect("report coverage json should serialize")
}
