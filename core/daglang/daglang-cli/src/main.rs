//! daglang CLI: dag viz, dag expand, dag manifest, dag modules.
//!
//! The development tool for the DSL compiler. Provides visualization
//! and introspection commands that make every subsequent phase
//! implementable — "see what you're building before you build it."
//!
//! # Commands
//!
//! - `daglang viz <file.dag> [--format ascii|mermaid]`
//!                                  -- DAG visualization from compiled IR
//! - `daglang expand <file.dag>`   -- Show lowered GraphIR (nodes, edges, ports)
//! - `daglang manifest <file.dag> [--format text|json]`
//!                                  -- Show derived ProgressManifest
//! - `daglang obligations <file.dag> [--format text|json]`
//!                                  -- Show derived test obligations summary
//! - `daglang show-triplets <file.dag> [--format text|json]`
//!                                  -- Show transport triplet expansions
//! - `daglang modules [dir] [--format text|json]`
//!                                  -- Show the discovered module graph
//! - `daglang check <file.dag|dir>` -- Parse modules; single-file targets also typecheck
//! - `daglang compile <file.dag>`  -- Full compilation pipeline
//! - `daglang run [--output <path>|--output=<path>] [--dry-run] <file.dag>`
//!                                  -- Compile + resolve + execute makegen DAG

use std::path::PathBuf;

use daglang_cli::compile::{
    build_context, check_from_context, compile_from_context, render_expand,
    render_manifest_with_format, render_obligations, render_triplets, CompileOutput, OutputFormat,
};
use daglang_cli::path_utils;
use daglang_cli::pipeline::{
    build_pipeline_dag, run_pipeline, PipelineContext, PipelineResult, PipelineStop,
};
use daglang_exec_bridge::{
    execute_resolved_dag, makegen_check_mode_transport_mocks, makegen_dry_run_transport_mocks,
    makegen_entrypoint_mocks, resolve_lowered_dag,
};
use daglang_syntax::diagnostic::DiagnosticKind;
use gunbc_exec::ExecutionMode;
use gunbc_ir::Value;
use serde::Deserialize;
use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if args.len() < 2 {
        eprintln!("Usage: daglang <command> [args...]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  viz <file.dag>|--self [--format ascii|mermaid]");
        eprintln!("                      DAG visualization (default: ascii)");
        eprintln!("  expand <file.dag>    Show lowered GraphIR (nodes/edges/ports)");
        eprintln!("  manifest <file.dag> [--format text|json]");
        eprintln!("                      Show derived ProgressManifest");
        eprintln!("  obligations <file.dag> [--format text|json]");
        eprintln!("                      Show derived test obligations summary");
        eprintln!("  show-triplets <file.dag> [--format text|json]");
        eprintln!("                      Show transport triplet expansions");
        eprintln!("  modules [dir] [--format text|json]");
        eprintln!("                      Show discovered module graph");
        eprintln!("  check <file.dag|dir> Parse (single-file targets also typecheck)");
        eprintln!("  compile <file.dag>   Full compilation pipeline");
        eprintln!("  run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>");
        eprintln!("                      Compile + resolve + execute makegen DAG");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "viz" => {
            let (target, format) = parse_viz_args(&args)
                .unwrap_or_else(|usage| exit_usage(&usage));
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
                    let output = compile_target_or_exit(&cwd, Some(&path));
                    let rendered = match format {
                        VizFormat::Ascii => output.lowered_dag.to_ascii("daglang-compiled"),
                        VizFormat::Mermaid => output.lowered_dag.to_mermaid("daglang-compiled"),
                    };
                    println!("{rendered}");
                }
            }
        }
        "expand" => {
            if args.len() != 3 {
                exit_usage("expand <file.dag>");
            }
            let output = compile_target_or_exit(&cwd, args.get(2));
            println!("{}", render_expand(&output.lowered_dag));
        }
        "manifest" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("manifest <file.dag> [--format text|json]");
            }
            let format = parse_output_format("manifest", &args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit(&cwd, args.get(2));
            println!("{}", render_manifest_with_format(&output.derived, format));
        }
        "obligations" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("obligations <file.dag> [--format text|json]");
            }
            let format = parse_output_format("obligations", &args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit(&cwd, args.get(2));
            println!("{}", render_obligations(&output.derived, format));
        }
        "show-triplets" => {
            if args.len() != 3 && args.len() != 5 {
                exit_usage("show-triplets <file.dag> [--format text|json]");
            }
            let format = parse_output_format("show-triplets", &args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let output = compile_target_or_exit(&cwd, args.get(2));
            println!("{}", render_triplets(&output.lowered_dag, format));
        }
        "modules" => {
            let (root_arg, format) = parse_modules_args(&args)
                .unwrap_or_else(|usage| exit_usage(&usage));
            let roots = if let Some(root) = root_arg {
                vec![resolve_root(&cwd, Some(&root))]
            } else {
                match resolve_configured_roots(&cwd) {
                    Ok(Some(config_roots)) => config_roots,
                    Ok(None) => vec![resolve_root(&cwd, None)],
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
            let context = build_check_pipeline_context(&cwd, args.get(2));
            if context.target_file.is_some() {
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
            } else {
                let result = run_pipeline_or_exit(&context, PipelineStop::Build);
                if result.diagnostics().is_empty() {
                    println!("OK: checked {} file(s)", result.parsed_count());
                } else {
                    for diagnostic in result.diagnostics() {
                        eprintln!("{diagnostic}");
                    }
                    std::process::exit(1);
                }
            }
        }
        "compile" => {
            if args.len() > 3 {
                exit_usage("compile <file.dag|dir>");
            }
            let output = compile_target_or_exit(&cwd, args.get(2));
            println!(
                "Compiled {} module(s) to {} node(s), {} file(s) emitted.",
                output.emitted.summary.module_count,
                output.derived.manifest.total_nodes,
                output.emitted.files.len()
            );
            for file in &output.emitted.files {
                println!("  - {}", file.path);
            }
        }
        "run" => {
            let parsed = parse_run_args(&args).unwrap_or_else(|error| {
                eprintln!("{error}");
                exit_usage(
                    "run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>",
                );
            });
            let normalized_output_path =
                path_utils::normalize_cli_path(&cwd, &PathBuf::from(&parsed.output_path));
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
            let context = build_context(&cwd, Some(&parsed.input_path));
            let output = match compile_from_context(&context) {
                Ok(output) => output,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            let resolved = match resolve_lowered_dag(&output.lowered_dag) {
                Ok(resolved) => resolved,
                Err(error) => {
                    eprintln!("resolve error: {error}");
                    std::process::exit(1);
                }
            };
            let log = match execute_resolved_dag(&resolved, mode, Some(&input_mocks)) {
                Ok(log) => log,
                Err(error) => {
                    eprintln!("execution error: {error}");
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Real,
    DryRun,
    CheckMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunArgs {
    input_path: String,
    output_path: String,
    mode: RunMode,
}

#[derive(Debug, Deserialize)]
struct DaglangConfig {
    discovery: Option<DiscoveryConfig>,
}

#[derive(Debug, Deserialize)]
struct DiscoveryConfig {
    roots: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VizFormat {
    Ascii,
    Mermaid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VizTarget {
    SelfDag,
    CompiledTarget(String),
}

fn resolve_root(cwd: &std::path::Path, arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return path_utils::normalize_cli_path(cwd, &PathBuf::from(path));
    }
    path_utils::resolve_default_root(cwd)
}

#[allow(clippy::disallowed_methods)]
fn resolve_configured_roots(cwd: &std::path::Path) -> Result<Option<Vec<PathBuf>>, String> {
    let config_path = cwd.join("daglang.toml");
    if !config_path.exists() {
        return Ok(None);
    }
    let config_text = std::fs::read_to_string(&config_path)
        .map_err(|error| format!("failed to read {}: {error}", config_path.display()))?;
    let parsed: DaglangConfig = toml::from_str(&config_text)
        .map_err(|error| format!("failed to parse {}: {error}", config_path.display()))?;
    let Some(discovery) = parsed.discovery else {
        return Ok(None);
    };
    let Some(config_roots) = discovery.roots else {
        return Ok(None);
    };
    if config_roots.is_empty() {
        return Err("discovery.roots in daglang.toml must not be empty".to_string());
    }
    let mut normalized = config_roots
        .iter()
        .map(|root| path_utils::normalize_cli_path(cwd, &PathBuf::from(root)))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    Ok(Some(normalized))
}

/// Builds check pipeline context from CLI input.
///
/// Unlike compile context construction, `.dag`-suffixed paths that resolve to
/// directories stay in directory mode for `daglang check`.
fn build_check_pipeline_context(
    cwd: &std::path::Path,
    input: Option<&String>,
) -> PipelineContext {
    let normalized_input = input
        .map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    let (roots, target_file) = match normalized_input {
        Some(path) if path_utils::is_single_file_target(&path, false) => {
            let root = path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| cwd.to_path_buf());
            (vec![root], Some(path))
        }
        Some(path) => (vec![path], None),
        None => (vec![resolve_root(cwd, None)], None),
    };
    PipelineContext { roots, target_file }
}

fn compile_target_or_exit(cwd: &std::path::Path, input: Option<&String>) -> CompileOutput {
    let context = build_context(cwd, input);
    match compile_from_context(&context) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn run_pipeline_or_exit(context: &PipelineContext, stop: PipelineStop) -> PipelineResult {
    match run_pipeline(context, stop) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("pipeline error: {error}");
            std::process::exit(1);
        }
    }
}

fn parse_viz_args(args: &[String]) -> Result<(VizTarget, VizFormat), String> {
    if args.is_empty() || args.get(1).map(String::as_str) != Some("viz") {
        return Err(
            "internal error: parse_viz_args expects full `daglang viz ...` argv".to_string(),
        );
    }
    if args.len() == 2 {
        return Err("viz <file.dag>|--self [--format ascii|mermaid]".to_string());
    }

    let mut target: Option<VizTarget> = None;
    let mut format = VizFormat::Ascii;
    let mut i = 2usize;
    while i < args.len() {
        let token = &args[i];
        if token == "--format" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| "viz <file.dag>|--self [--format ascii|mermaid]".to_string())?;
            format = match value.as_str() {
                "ascii" => VizFormat::Ascii,
                "mermaid" => VizFormat::Mermaid,
                _ => return Err("viz <file.dag>|--self [--format ascii|mermaid]".to_string()),
            };
            i += 2;
            continue;
        }
        if token.starts_with("--") && token != "--self" {
            return Err("viz <file.dag>|--self [--format ascii|mermaid]".to_string());
        }
        if target.is_some() {
            return Err("viz <file.dag>|--self [--format ascii|mermaid]".to_string());
        }
        target = Some(if token == "--self" {
            VizTarget::SelfDag
        } else {
            VizTarget::CompiledTarget(token.clone())
        });
        i += 1;
    }

    let Some(target) = target else {
        return Err("viz <file.dag>|--self [--format ascii|mermaid]".to_string());
    };

    Ok((target, format))
}

fn parse_modules_args(args: &[String]) -> Result<(Option<String>, OutputFormat), String> {
    if args.is_empty() || args.get(1).map(String::as_str) != Some("modules") {
        return Err(
            "internal error: parse_modules_args expects full `daglang modules ...` argv"
                .to_string(),
        );
    }
    let mut root_arg: Option<String> = None;
    let mut format = OutputFormat::Text;
    let mut saw_format = false;
    let mut i = 2usize;
    while i < args.len() {
        let token = &args[i];
        if token == "--format" {
            if saw_format {
                return Err("modules [dir] [--format text|json]".to_string());
            }
            let value = args
                .get(i + 1)
                .ok_or_else(|| "modules [dir] [--format text|json]".to_string())?;
            format = match value.as_str() {
                "text" => OutputFormat::Text,
                "json" => OutputFormat::Json,
                _ => return Err("modules [dir] [--format text|json]".to_string()),
            };
            saw_format = true;
            i += 2;
            continue;
        }
        if token.starts_with("--") {
            return Err("modules [dir] [--format text|json]".to_string());
        }
        if root_arg.is_some() {
            return Err("modules [dir] [--format text|json]".to_string());
        }
        root_arg = Some(token.clone());
        i += 1;
    }
    Ok((root_arg, format))
}

fn render_modules_result_json(result: &PipelineResult) -> String {
    let diagnostics = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.render())
        .collect::<Vec<_>>();
    let diagnostics_detail = result
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            json!({
                "kind": diagnostic_kind_label(&diagnostic.kind),
                "message": diagnostic.message.clone(),
                "file": diagnostic.file.as_ref().map(|path| path.display().to_string()),
                "line": diagnostic.line,
                "column": diagnostic.column,
                "rendered": diagnostic.render(),
            })
        })
        .collect::<Vec<_>>();
    let diagnostic_kind_counts = json!({
        "lex": result.diagnostics().iter().filter(|d| matches!(d.kind, DiagnosticKind::Lex)).count(),
        "parse": result.diagnostics().iter().filter(|d| matches!(d.kind, DiagnosticKind::Parse)).count(),
        "resolve": result.diagnostics().iter().filter(|d| matches!(d.kind, DiagnosticKind::Resolve)).count(),
        "pipeline": result.diagnostics().iter().filter(|d| matches!(d.kind, DiagnosticKind::Pipeline)).count(),
    });
    let Some(graph) = result.module_graph() else {
        return json!({
            "summary": {
                "parsed_files": result.parsed_count(),
                "module_count": 0,
                "diagnostic_count": diagnostics.len(),
                "diagnostic_kinds": diagnostic_kind_counts,
            },
            "parsed_files": result.parsed_count(),
            "module_order": [],
            "modules": [],
            "diagnostics": diagnostics,
            "diagnostics_detail": diagnostics_detail,
        })
        .to_string();
    };
    let module_order = graph
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect::<Vec<_>>();
    let modules = graph
        .modules
        .iter()
        .map(|module| {
            let dependencies = module
                .dependencies
                .iter()
                .filter_map(|dep| graph.modules.get(*dep))
                .map(|dependency| dependency.module_path.join("."))
                .collect::<Vec<_>>();
            json!({
                "module": module.module_path.join("."),
                "path": module.path.display().to_string(),
                "items": module.ast.items.len(),
                "dependencies": dependencies,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "summary": {
            "parsed_files": result.parsed_count(),
            "module_count": modules.len(),
            "diagnostic_count": diagnostics.len(),
            "diagnostic_kinds": diagnostic_kind_counts,
        },
        "parsed_files": result.parsed_count(),
        "module_order": module_order,
        "modules": modules,
        "diagnostics": diagnostics,
        "diagnostics_detail": diagnostics_detail,
    })
    .to_string()
}

fn diagnostic_kind_label(kind: &DiagnosticKind) -> &'static str {
    match kind {
        DiagnosticKind::Lex => "lex",
        DiagnosticKind::Parse => "parse",
        DiagnosticKind::Resolve => "resolve",
        DiagnosticKind::Pipeline => "pipeline",
    }
}

fn parse_output_format(command: &str, args: &[String]) -> Result<OutputFormat, String> {
    if args.len() == 3 {
        return Ok(OutputFormat::Text);
    }
    if args.len() != 5 || args[3] != "--format" {
        return Err(format!("{command} <file.dag> [--format text|json]"));
    }
    match args[4].as_str() {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err(format!("{command} <file.dag> [--format text|json]")),
    }
}

fn parse_run_args(args: &[String]) -> Result<RunArgs, String> {
    if args.is_empty() || args.get(1).map(String::as_str) != Some("run") {
        return Err(
            "internal error: parse_run_args expects full `daglang run ...` argv".to_string(),
        );
    }

    let mut mode = RunMode::Real;
    let mut output_path = "Makefile".to_string();
    let mut has_output = false;
    let mut input_path: Option<String> = None;
    let mut i = 2usize;

    while i < args.len() {
        let token = &args[i];
        if token == "--dry-run" {
            if mode == RunMode::DryRun {
                return Err("duplicate --dry-run flag".to_string());
            }
            if mode == RunMode::CheckMode {
                return Err("--dry-run and --check-mode cannot be combined".to_string());
            }
            mode = RunMode::DryRun;
            i += 1;
            continue;
        }
        if token == "--check-mode" {
            if mode == RunMode::CheckMode {
                return Err("duplicate --check-mode flag".to_string());
            }
            if mode == RunMode::DryRun {
                return Err("--dry-run and --check-mode cannot be combined".to_string());
            }
            mode = RunMode::CheckMode;
            i += 1;
            continue;
        }
        if token == "--output" {
            if has_output {
                return Err("duplicate --output flag".to_string());
            }
            let value = args
                .get(i + 1)
                .ok_or_else(|| "--output requires a path".to_string())?;
            if value.starts_with("--") {
                return Err("--output requires a non-flag path value".to_string());
            }
            output_path = value.clone();
            has_output = true;
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--output=") {
            if has_output {
                return Err("duplicate --output flag".to_string());
            }
            if value.is_empty() {
                return Err("--output requires a non-empty path value".to_string());
            }
            output_path = value.to_string();
            has_output = true;
            i += 1;
            continue;
        }
        if token.starts_with("--") {
            return Err(format!("unknown flag `{token}`"));
        }
        if input_path.is_some() {
            return Err("run accepts exactly one <file.dag> input".to_string());
        }
        input_path = Some(token.clone());
        i += 1;
    }

    let input_path = input_path.ok_or_else(|| "run requires <file.dag> input".to_string())?;
    if !path_utils::has_dag_extension(&PathBuf::from(&input_path)) {
        return Err(format!(
            "run input must be a .dag file path, got `{input_path}`"
        ));
    }

    Ok(RunArgs {
        input_path,
        output_path,
        mode,
    })
}

fn exit_usage(command: &str) -> ! {
    eprintln!("Usage: daglang {command}");
    std::process::exit(1);
}


#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use crate::path_utils::{has_dag_extension, normalize_path_components, resolve_default_root};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root_path() -> PathBuf {
        PathBuf::from(Path::new(std::path::MAIN_SEPARATOR_STR))
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "daglang_cli_main_{label}_{}_{}",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn normalize_path_components_collapses_curdir_and_parent_segments() {
        let path = root_path().join("workspace").join(".").join("core").join("..").join("dsl");
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_keeps_root_when_parent_traversal_exceeds_depth() {
        let path = root_path().join("..").join("..").join("workspace").join("dsl");
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_preserves_clean_absolute_paths() {
        let path = root_path().join("workspace").join("dsl").join("tools").join("makegen.dag");
        let normalized = normalize_path_components(&path);
        assert_eq!(normalized, path);
    }

    #[test]
    fn normalize_path_components_collapses_redundant_separators_and_trailing_separator() {
        let path = PathBuf::from(format!(
            "{}workspace//dsl///tools/",
            std::path::MAIN_SEPARATOR
        ));
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl").join("tools");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_handles_mixed_parent_curdir_and_redundant_separators() {
        let path = PathBuf::from(format!(
            "{}workspace//core/./../dsl//tools/../makegen.dag",
            std::path::MAIN_SEPARATOR
        ));
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl").join("makegen.dag");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_preserves_single_absolute_root() {
        let path = root_path();
        let normalized = normalize_path_components(&path);
        assert_eq!(normalized, path);
    }

    #[test]
    fn normalize_path_components_drops_curdir_suffix_segment() {
        let path = root_path().join("workspace").join("dsl").join(".");
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_drops_curdir_suffix_with_redundant_separators() {
        let path = PathBuf::from(format!(
            "{}workspace//dsl//./.",
            std::path::MAIN_SEPARATOR
        ));
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_drops_curdir_suffix_on_file_paths() {
        let path = root_path()
            .join("workspace")
            .join("dsl")
            .join("makegen.dag")
            .join(".");
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl").join("makegen.dag");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn resolve_default_root_normalizes_curdir_suffix() {
        let cwd = root_path().join("workspace").join("project").join(".");
        let normalized_root = resolve_default_root(&cwd);
        let expected = root_path().join("workspace").join("project").join("dsl");
        assert_eq!(normalized_root, expected);
    }

    #[test]
    fn resolve_default_root_collapses_parent_segments() {
        let cwd = root_path()
            .join("workspace")
            .join("project")
            .join("nested")
            .join("..");
        let normalized_root = resolve_default_root(&cwd);
        let expected = root_path().join("workspace").join("project").join("dsl");
        assert_eq!(normalized_root, expected);
    }

    #[test]
    fn normalize_path_components_preserves_leading_relative_parent_segments() {
        let path = PathBuf::from("../workspace/./dsl/tools/..");
        let normalized = normalize_path_components(&path);
        let expected = PathBuf::from("../workspace/dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_preserves_multiple_relative_parent_segments() {
        let path = PathBuf::from("../../workspace/dsl");
        let normalized = normalize_path_components(&path);
        let expected = PathBuf::from("../../workspace/dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_collapses_empty_relative_path_to_curdir() {
        let normalized = normalize_path_components(Path::new(""));
        assert_eq!(normalized, PathBuf::from("."));
    }

    #[test]
    fn normalize_path_components_collapses_relative_path_that_resolves_to_curdir() {
        let path = PathBuf::from("tools/../.");
        let normalized = normalize_path_components(&path);
        assert_eq!(normalized, PathBuf::from("."));
    }

    #[test]
    fn has_dag_extension_is_case_insensitive() {
        assert!(has_dag_extension(Path::new("main.dag")));
        assert!(has_dag_extension(Path::new("main.DAG")));
        assert!(has_dag_extension(Path::new("main.DaG")));
    }

    #[test]
    fn has_dag_extension_rejects_non_dag_extensions() {
        assert!(!has_dag_extension(Path::new("main.dag.bak")));
        assert!(!has_dag_extension(Path::new("main.txt")));
        assert!(!has_dag_extension(Path::new("main")));
    }

    #[test]
    fn resolve_configured_roots_returns_none_when_config_missing() {
        let cwd = unique_temp_dir("config_missing");
        std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");

        let roots = super::resolve_configured_roots(&cwd).expect("missing config should parse");
        assert!(roots.is_none());

        std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
    }

    #[test]
    fn resolve_configured_roots_parses_and_normalizes_entries() {
        let cwd = unique_temp_dir("config_roots");
        std::fs::create_dir_all(cwd.join("dsl")).expect("failed to create dsl dir");
        std::fs::create_dir_all(cwd.join("nested")).expect("failed to create nested dir");
        std::fs::write(
            cwd.join("daglang.toml"),
            "[discovery]\nroots = [\"./dsl\", \"nested/..//dsl\", \"nested\"]\n",
        )
        .expect("failed to write daglang.toml");

        let roots =
            super::resolve_configured_roots(&cwd).expect("configured roots should parse");
        let roots = roots.expect("roots should be present");
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0], cwd.join("dsl"));
        assert_eq!(roots[1], cwd.join("nested"));

        std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
    }

    #[test]
    fn resolve_configured_roots_rejects_empty_roots_list() {
        let cwd = unique_temp_dir("config_empty_roots");
        std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
        std::fs::write(cwd.join("daglang.toml"), "[discovery]\nroots = []\n")
            .expect("failed to write daglang.toml");

        let error =
            super::resolve_configured_roots(&cwd).expect_err("empty roots should fail");
        assert!(error.contains("must not be empty"));

        std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
    }

    #[test]
    fn parse_output_format_defaults_to_text_for_three_args() {
        let args = vec![
            "daglang".to_string(),
            "obligations".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let format = super::parse_output_format("obligations", &args)
            .expect("three-argument form should parse");
        assert!(matches!(format, super::OutputFormat::Text));
    }

    #[test]
    fn parse_output_format_accepts_json_and_text_flags() {
        let json_args = vec![
            "daglang".to_string(),
            "obligations".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let text_args = vec![
            "daglang".to_string(),
            "show-triplets".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "text".to_string(),
        ];

        let json_format = super::parse_output_format("obligations", &json_args)
            .expect("json format should parse");
        let text_format = super::parse_output_format("show-triplets", &text_args)
            .expect("text format should parse");

        assert!(matches!(json_format, super::OutputFormat::Json));
        assert!(matches!(text_format, super::OutputFormat::Text));
    }

    #[test]
    fn parse_output_format_rejects_invalid_shapes() {
        let missing_value = vec![
            "daglang".to_string(),
            "obligations".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
        ];
        let bad_flag = vec![
            "daglang".to_string(),
            "obligations".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--fmt".to_string(),
            "json".to_string(),
        ];
        let bad_value = vec![
            "daglang".to_string(),
            "show-triplets".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "yaml".to_string(),
        ];
        let bad_case_variant_value = vec![
            "daglang".to_string(),
            "obligations".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "JSON".to_string(),
        ];
        let bad_title_case_value = vec![
            "daglang".to_string(),
            "show-triplets".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "Json".to_string(),
        ];

        let expected_usage = "obligations <file.dag> [--format text|json]";
        assert_eq!(
            super::parse_output_format("obligations", &missing_value),
            Err(expected_usage.to_string())
        );
        assert_eq!(
            super::parse_output_format("obligations", &bad_flag),
            Err(expected_usage.to_string())
        );
        assert_eq!(
            super::parse_output_format("show-triplets", &bad_value),
            Err("show-triplets <file.dag> [--format text|json]".to_string())
        );
        assert_eq!(
            super::parse_output_format("obligations", &bad_case_variant_value),
            Err(expected_usage.to_string())
        );
        assert_eq!(
            super::parse_output_format("show-triplets", &bad_title_case_value),
            Err("show-triplets <file.dag> [--format text|json]".to_string())
        );
    }

    #[test]
    fn parse_viz_args_defaults_to_ascii_for_self_target() {
        let args = vec!["daglang".to_string(), "viz".to_string(), "--self".to_string()];
        let (target, format) = super::parse_viz_args(&args).expect("viz args should parse");
        assert!(matches!(target, super::VizTarget::SelfDag));
        assert!(matches!(format, super::VizFormat::Ascii));
    }

    #[test]
    fn parse_viz_args_accepts_compiled_target_with_mermaid_format() {
        let args = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "mermaid".to_string(),
        ];
        let (target, format) = super::parse_viz_args(&args).expect("viz args should parse");
        assert!(matches!(
            target,
            super::VizTarget::CompiledTarget(ref path) if path == "dsl/tools/makegen.dag"
        ));
        assert!(matches!(format, super::VizFormat::Mermaid));
    }

    #[test]
    fn parse_viz_args_rejects_invalid_shapes() {
        let missing_target = vec!["daglang".to_string(), "viz".to_string()];
        let bad_format = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "--self".to_string(),
            "--format".to_string(),
            "graphviz".to_string(),
        ];
        let multiple_targets = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "--self".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let usage = "viz <file.dag>|--self [--format ascii|mermaid]".to_string();
        assert_eq!(super::parse_viz_args(&missing_target), Err(usage.clone()));
        assert_eq!(super::parse_viz_args(&bad_format), Err(usage.clone()));
        assert_eq!(super::parse_viz_args(&multiple_targets), Err(usage));
    }

    #[test]
    fn parse_modules_args_supports_optional_root_and_json_format() {
        let args = vec![
            "daglang".to_string(),
            "modules".to_string(),
            "dsl".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let (root, format) = super::parse_modules_args(&args).expect("modules args should parse");
        assert_eq!(root.as_deref(), Some("dsl"));
        assert!(matches!(format, super::OutputFormat::Json));
    }

    #[test]
    fn parse_modules_args_rejects_invalid_shapes() {
        let duplicate_roots = vec![
            "daglang".to_string(),
            "modules".to_string(),
            "dsl".to_string(),
            "other".to_string(),
        ];
        let bad_format = vec![
            "daglang".to_string(),
            "modules".to_string(),
            "--format".to_string(),
            "yaml".to_string(),
        ];
        let usage = "modules [dir] [--format text|json]".to_string();
        assert_eq!(super::parse_modules_args(&duplicate_roots), Err(usage.clone()));
        assert_eq!(super::parse_modules_args(&bad_format), Err(usage));
    }

    #[test]
    fn parse_run_args_supports_defaults() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = super::parse_run_args(&args).expect("run args should parse");
        assert_eq!(parsed.input_path, "dsl/tools/makegen.dag");
        assert_eq!(parsed.output_path, "Makefile");
        assert_eq!(parsed.mode, super::RunMode::Real);
    }

    #[test]
    fn parse_run_args_supports_output_and_dry_run_flags() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--dry-run".to_string(),
            "--output=tmp/generated.mk".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = super::parse_run_args(&args).expect("run args should parse");
        assert_eq!(parsed.input_path, "dsl/tools/makegen.dag");
        assert_eq!(parsed.output_path, "tmp/generated.mk");
        assert_eq!(parsed.mode, super::RunMode::DryRun);
    }

    #[test]
    fn parse_run_args_supports_check_mode_flag() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--check-mode".to_string(),
            "--output".to_string(),
            "tmp/generated.mk".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = super::parse_run_args(&args).expect("run args should parse");
        assert_eq!(parsed.input_path, "dsl/tools/makegen.dag");
        assert_eq!(parsed.output_path, "tmp/generated.mk");
        assert_eq!(parsed.mode, super::RunMode::CheckMode);
    }

    #[test]
    fn parse_run_args_rejects_duplicate_output_flags() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--output".to_string(),
            "a.mk".to_string(),
            "--output=b.mk".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("duplicate output should fail");
        assert!(error.contains("duplicate --output"));
    }

    #[test]
    fn parse_run_args_rejects_unknown_flag() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--mystery".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn parse_run_args_rejects_non_dag_input() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "dsl/tools".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("non-dag input should fail");
        assert!(error.contains("must be a .dag file"));
    }

    #[test]
    fn parse_run_args_rejects_duplicate_dry_run_flag() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--dry-run".to_string(),
            "--dry-run".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("duplicate dry-run should fail");
        assert!(error.contains("duplicate --dry-run"));
    }

    #[test]
    fn parse_run_args_rejects_duplicate_check_mode_flag() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--check-mode".to_string(),
            "--check-mode".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("duplicate check-mode should fail");
        assert!(error.contains("duplicate --check-mode"));
    }

    #[test]
    fn parse_run_args_rejects_conflicting_dry_run_and_check_mode_flags() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--dry-run".to_string(),
            "--check-mode".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("conflicting modes should fail");
        assert!(error.contains("cannot be combined"));
    }

    #[test]
    fn parse_run_args_rejects_output_without_value() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--output".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("missing output value should fail");
        assert!(error.contains("--output requires a path"));
    }

    #[test]
    fn parse_run_args_rejects_multiple_inputs() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "dsl/tools/other.dag".to_string(),
        ];
        let error = super::parse_run_args(&args).expect_err("multiple inputs should fail");
        assert!(error.contains("exactly one <file.dag>"));
    }

    #[test]
    fn build_check_pipeline_context_defaults_to_cwd_dsl_root_without_input() {
        let cwd = root_path().join("workspace").join("project").join(".");
        let context = super::build_check_pipeline_context(&cwd, None);
        assert_eq!(context.target_file, None);
        assert_eq!(
            context.roots,
            vec![root_path().join("workspace").join("project").join("dsl")]
        );
    }

    #[test]
    fn build_check_pipeline_context_treats_dag_file_as_target_input() {
        let cwd = root_path().join("workspace").join("project");
        let input = "nested/tool.dag".to_string();
        let context = super::build_check_pipeline_context(&cwd, Some(&input));
        assert_eq!(
            context.target_file,
            Some(cwd.join("nested").join("tool.dag"))
        );
        assert_eq!(context.roots, vec![cwd.join("nested")]);
    }

    #[test]
    fn build_check_pipeline_context_treats_dag_named_directory_as_root() {
        let root = unique_temp_dir("check_context_dag_dir");
        let cwd = root.join("workspace");
        let dag_dir = cwd.join("bundle.dag");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .dag directory fixture");
        let input = "bundle.dag".to_string();

        let context = super::build_check_pipeline_context(&cwd, Some(&input));

        assert_eq!(context.target_file, None);
        assert_eq!(context.roots, vec![dag_dir.clone()]);

        std::fs::remove_dir_all(&root).expect("failed to cleanup temp fixture");
    }

    #[test]
    fn build_check_pipeline_context_treats_uppercase_dag_named_directory_as_root() {
        let root = unique_temp_dir("check_context_uppercase_dag_dir");
        let cwd = root.join("workspace");
        let dag_dir = cwd.join("bundle.DAG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory fixture");
        let input = "bundle.DAG".to_string();

        let context = super::build_check_pipeline_context(&cwd, Some(&input));

        assert_eq!(context.target_file, None);
        assert_eq!(context.roots, vec![dag_dir.clone()]);

        std::fs::remove_dir_all(&root).expect("failed to cleanup temp fixture");
    }

    #[test]
    fn build_check_pipeline_context_treats_mixed_case_dag_named_directory_with_trailing_slash_as_root(
    ) {
        let root = unique_temp_dir("check_context_mixed_case_dag_dir");
        let cwd = root.join("workspace");
        let dag_dir = cwd.join("bundle.DaG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory fixture");
        let input = "bundle.DaG/".to_string();

        let context = super::build_check_pipeline_context(&cwd, Some(&input));

        assert_eq!(context.target_file, None);
        assert_eq!(context.roots, vec![dag_dir.clone()]);

        std::fs::remove_dir_all(&root).expect("failed to cleanup temp fixture");
    }

    #[test]
    fn resolve_root_defaults_to_cwd_dsl_when_arg_missing() {
        let cwd = root_path().join("workspace").join("project").join(".");
        let resolved = super::resolve_root(&cwd, None);
        let expected = root_path().join("workspace").join("project").join("dsl");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolve_root_normalizes_relative_arg_against_cwd() {
        let cwd = root_path().join("workspace").join("project");
        let arg = "dsl/./tools/../services".to_string();
        let resolved = super::resolve_root(&cwd, Some(&arg));
        let expected = root_path()
            .join("workspace")
            .join("project")
            .join("dsl")
            .join("services");
        assert_eq!(resolved, expected);
    }
}
