use super::*;

pub(super) fn dispatch(args: &[String], cwd: &std::path::Path) {
    match args[1].as_str() {
        "viz" => {
            let (target, format) = parse_viz_args(args).unwrap_or_else(|usage| exit_usage(&usage));
            match target {
                VizTarget::SelfDag => {
                    let dag = build_pipeline_dag();
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
        "manifest" => {
            let parsed =
                parse_manifest_command_args(args).unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit_with_options(
                cwd,
                Some(&parsed.input),
                parsed.emit_collection_nodes,
            );
            println!(
                "{}",
                render_manifest_with_format(&output.derived, parsed.format)
            );
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
        "modules" => {
            let (root_arg, format) =
                parse_modules_args(args).unwrap_or_else(|usage| exit_usage(&usage));
            if let Some(root) = &root_arg {
                let normalized = path_utils::normalize_cli_path(cwd, &PathBuf::from(root));
                if let Some(error) = path_utils::check_dag_extension_casing(&normalized) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            let roots = if let Some(root) = root_arg {
                vec![resolve_root(cwd, Some(&root))]
            } else {
                match resolve_configured_roots(cwd) {
                    Ok(Some(config_roots)) => config_roots,
                    Ok(None) => vec![resolve_root(cwd, None)],
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
            let context = build_check_pipeline_context_with_default_roots(
                cwd,
                args.get(2),
                configured_default_roots.as_deref(),
            );
            let result = run_pipeline_or_exit(&context, PipelineStop::Build);
            if !result.diagnostics().is_empty() {
                for diagnostic in result.diagnostics() {
                    eprintln!("{diagnostic}");
                }
                std::process::exit(1);
            }
            match check_from_context(&context) {
                Ok(output) => {
                    println!("OK: checked {} file(s)", output.parsed_files);
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
                "compile <file.dag|dir> [--emit-collection-nodes] [--target rust|go|c|mips] [--layer 1|2] [--out <dir>|--out=<dir>]",
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
            };
            let output = compile_target_or_exit_with_compile_options(
                cwd,
                parsed.input.as_ref(),
                options.clone(),
            );
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
            println!(
                "Compiled {} module(s) to {} node(s), {} file(s) emitted (target={} layer={}).",
                output.emitted.summary.module_count,
                output.derived.manifest.total_nodes,
                output.emitted.files.len(),
                options.target,
                options.layer
            );
            if parsed.out_dir.is_some() {
                for file in &written_files {
                    println!("  - {}", file.display());
                }
            } else {
                for file in &output.emitted.files {
                    println!("  - {}", file.path);
                }
            }
        }
        "run" => {
            let parsed = parse_run_args(args).unwrap_or_else(|error| {
                eprintln!("{error}");
                exit_usage(
                    "run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>",
                );
            });
            let normalized_output_path =
                path_utils::normalize_cli_path(cwd, &PathBuf::from(&parsed.output_path));
            let output_path_str = normalized_output_path.to_string_lossy().to_string();
            let input_mocks = makegen_entrypoint_mocks(&output_path_str);
            let mode = match parsed.mode {
                RunMode::Real => ExecutionMode::Real,
                RunMode::DryRun => {
                    ExecutionMode::DryRun(makegen_dry_run_transport_mocks(&output_path_str))
                }
                RunMode::CheckMode => {
                    ExecutionMode::DryRun(makegen_check_mode_transport_mocks(&output_path_str))
                }
            };
            let context = build_context(cwd, Some(&parsed.input_path));
            let log = match compile_resolve_execute_from_context(&context, mode, Some(&input_mocks))
            {
                Ok(log) => log,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            let written = log
                .get("tools.makegen::makegen")
                .and_then(|entry| entry.outputs.get("written"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let fresh = log
                .get("compare_makegen_content")
                .and_then(|entry| entry.outputs.get("fresh"))
                .and_then(Value::as_bool);
            let mode_label = match parsed.mode {
                RunMode::Real => "real",
                RunMode::DryRun => "dry-run",
                RunMode::CheckMode => "check-mode",
            };
            if parsed.mode == RunMode::CheckMode && fresh == Some(false) {
                eprintln!(
                    "check-mode failed: output is stale at {}",
                    normalized_output_path.display()
                );
                std::process::exit(2);
            }
            println!(
                "OK: run mode={mode_label} output={} written={written}",
                normalized_output_path.display()
            );
        }
        cmd => {
            eprintln!("Unknown command: {cmd}");
            exit_usage("<command> [args...]");
        }
    }
}
