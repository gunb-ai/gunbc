//! daglang CLI: dag viz, dag expand, dag progress, dag topology, dag modules.
//!
//! The development tool for the DSL compiler. Provides visualization
//! and introspection commands that make every subsequent phase
//! implementable — "see what you're building before you build it."
//!
//! # Commands
//!
//! - `daglang viz <file.dag> [--format ascii|mermaid]`: DAG visualization from compiled IR
//! - `daglang expand <file.dag> [--emit-collection-nodes]`: Show lowered GraphIR (nodes, edges, ports)
//! - `daglang progress <file.dag> [--format text|json] [--emit-collection-nodes]`: Show progress metrics
//! - `daglang topology <file.dag> [--format text|json]`: Show graph topology
//! - `daglang obligations <file.dag> [--format text|json]`: Show derived test obligations summary
//! - `daglang show-triplets <file.dag> [--format text|json]`: Show transport triplet expansions
//! - `daglang report-coverage <file.dag|dir> [--format text|json]`: Lint report-stage coverage across pipeline stages
//! - `daglang modules [dir] [--format text|json]`: Show the discovered module graph
//! - `daglang check <file.dag|dir>` -- Parse + typecheck modules (no lowering)
//! - `daglang compile <file.dag|dir> [--emit-collection-nodes] [--trace-stages] [--profile <name>] [--target rust|go|c|mips] [--layer 1|2] [--format summary|canonical-json] [--out <dir>|--out=<dir>]`: Full compilation pipeline
//! - `daglang run [--output <path>|--output=<path>] [--dry-run] <file.dag>`: Compile + resolve + execute makegen DAG

use std::path::PathBuf;

use daglang_cli::compile::{
    build_context, build_context_with_default_roots, check_from_module_graph,
    compile_from_context_with_options, compile_resolve_execute_from_context,
    makegen_check_mode_transport_mocks, makegen_dry_run_transport_mocks, makegen_entrypoint_mocks,
    render_canonical_ir_json, render_expand, render_obligations, render_progress_with_format,
    render_topology_with_format, render_triplets,
    resolve_configured_roots as resolve_configured_roots_from_context,
    resolve_default_roots as resolve_default_roots_from_context, CompileOptions, CompileOutput,
    OutputFormat,
};
use daglang_cli::path_utils;
use daglang_cli::pipeline::{
    build_compile_stage_dag, run_pipeline, PipelineContext, PipelineResult, PipelineStop,
};
use daglang_driver::{CodegenLayer, CodegenTarget};
use daglang_syntax::diagnostic::DiagnosticKind;
use gunbc_exec::ExecutionMode;
use gunbc_ir::Value;
use serde_json::json;

mod commands;

const VIZ_USAGE: &str = "viz <file.dag>|--self [--format ascii|mermaid]";
/// Decision lock (DL8): ASCII is the default viz format. Tests lock this.
const DEFAULT_VIZ_FORMAT: VizFormat = VizFormat::Ascii;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if args.len() < 2 {
        eprintln!("Usage: daglang <command> [args...]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  {VIZ_USAGE}");
        eprintln!(
            "                      DAG visualization (default: {})",
            default_viz_format_label()
        );
        eprintln!("  expand <file.dag> [--emit-collection-nodes]");
        eprintln!("                      Show lowered GraphIR (nodes/edges/ports)");
        eprintln!("  progress <file.dag> [--format text|json] [--emit-collection-nodes]");
        eprintln!("                      Show progress metrics");
        eprintln!("  topology <file.dag> [--format text|json]");
        eprintln!("                      Show graph topology (nodes, depths, labels, boundaries)");
        eprintln!("  obligations <file.dag> [--format text|json]");
        eprintln!("                      Show derived test obligations summary");
        eprintln!("  show-triplets <file.dag> [--format text|json]");
        eprintln!("                      Show transport triplet expansions");
        eprintln!("  report-coverage <file.dag|dir> [--format text|json]");
        eprintln!("                      Lint report-stage coverage across pipeline stages");
        eprintln!("  modules [dir] [--format text|json]");
        eprintln!("                      Show discovered module graph");
        eprintln!("  check <file.dag|dir> Parse + typecheck modules (no lowering)");
        eprintln!(
            "  compile <file.dag|dir> [--emit-collection-nodes] [--trace-stages] [--profile <name>] [--target rust|go|c|mips] [--layer 1|2] [--format summary|canonical-json] [--out <dir>|--out=<dir>]"
        );
        eprintln!("                      Full compilation pipeline (canonical-json: IR only, no file writes)");
        eprintln!("  run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>");
        eprintln!("                      Compile + resolve + execute makegen DAG");
        eprintln!("  gen-types [<dir>] [--module <module.path>]...");
        eprintln!("                      Generate Rust types from DSL TypeDef definitions");
        std::process::exit(1);
    }

    commands::dispatch(&args, &cwd);
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
    params: std::collections::BTreeMap<String, String>,
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

const fn default_viz_format_label() -> &'static str {
    match DEFAULT_VIZ_FORMAT {
        VizFormat::Ascii => "ascii",
        VizFormat::Mermaid => "mermaid",
    }
}

fn resolve_root(cwd: &std::path::Path, arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return path_utils::normalize_cli_path(cwd, &PathBuf::from(path));
    }
    path_utils::resolve_default_root(cwd)
}

fn resolve_configured_roots(cwd: &std::path::Path) -> Result<Option<Vec<PathBuf>>, String> {
    resolve_configured_roots_from_context(cwd)
}

/// Builds check pipeline context from CLI input.
///
/// Paths ending in `.dag` that are regular files are treated as single-file
/// targets. Directories named with a `.dag` suffix are rejected with an
/// explicit error — matching compile-mode behavior.
#[cfg(test)]
fn build_check_pipeline_context(
    cwd: &std::path::Path,
    input: Option<&String>,
) -> Result<PipelineContext, String> {
    build_check_pipeline_context_with_default_roots(cwd, input, None)
}

fn build_check_pipeline_context_with_default_roots(
    cwd: &std::path::Path,
    input: Option<&String>,
    default_roots: Option<&[PathBuf]>,
) -> Result<PipelineContext, String> {
    build_context_with_default_roots(cwd, input, default_roots)
}

fn compile_target_or_exit(cwd: &std::path::Path, input: Option<&String>) -> CompileOutput {
    compile_target_or_exit_with_options(cwd, input, false)
}

fn compile_target_or_exit_with_options(
    cwd: &std::path::Path,
    input: Option<&String>,
    emit_collection_nodes: bool,
) -> CompileOutput {
    compile_target_or_exit_with_compile_options(
        cwd,
        input,
        CompileOptions {
            emit_collection_nodes,
            ..CompileOptions::default()
        },
    )
}

fn compile_target_or_exit_with_compile_options(
    cwd: &std::path::Path,
    input: Option<&String>,
    options: CompileOptions,
) -> CompileOutput {
    if let Some(value) = input {
        let normalized = path_utils::normalize_cli_path(cwd, &PathBuf::from(value));
        if let Some(error) = path_utils::check_dag_extension_casing(&normalized) {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
    let context = if input.is_none() {
        match resolve_default_roots(cwd) {
            Ok(roots) => PipelineContext {
                roots,
                target_file: None,
            },
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    } else {
        match build_context(cwd, input) {
            Ok(context) => context,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    };
    match compile_from_context_with_options(&context, options) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::disallowed_methods)]
fn write_emitted_files(
    cwd: &std::path::Path,
    out_dir: &std::path::Path,
    files: &[daglang_emit::EmittedFile],
) -> Result<Vec<PathBuf>, String> {
    let out_root = path_utils::normalize_cli_path(cwd, out_dir);
    let mut written = Vec::with_capacity(files.len());
    for file in files {
        let raw_path = std::path::Path::new(&file.path);
        if raw_path.is_absolute() {
            return Err(format!(
                "emitted file path `{}` is absolute; paths must be relative to output directory",
                file.path
            ));
        }
        let destination = path_utils::normalize_path_components(&out_root.join(raw_path));
        if !destination.starts_with(&out_root) {
            return Err(format!(
                "emitted file path `{}` escapes output directory `{}`",
                file.path,
                out_root.display()
            ));
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create output directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        std::fs::write(&destination, &file.content)
            .map_err(|error| format!("failed to write {}: {error}", destination.display()))?;
        written.push(destination);
    }
    Ok(written)
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

fn resolve_default_roots(cwd: &std::path::Path) -> Result<Vec<PathBuf>, String> {
    resolve_default_roots_from_context(cwd)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompileCommandArgs {
    input: Option<String>,
    emit_collection_nodes: bool,
    trace_stages: bool,
    profile: Option<String>,
    target: Option<CodegenTarget>,
    layer: Option<CodegenLayer>,
    format: CompileOutputFormat,
    out_dir: Option<String>,
    receipt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompileOutputFormat {
    Summary,
    CanonicalJson,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProgressCommandArgs {
    input: String,
    format: OutputFormat,
    emit_collection_nodes: bool,
}

pub(crate) fn parse_compile_command_args(
    command: &str,
    args: &[String],
    usage: &str,
    require_input: bool,
) -> Result<CompileCommandArgs, String> {
    if args.is_empty() || args.get(1).map(String::as_str) != Some(command) {
        return Err(format!(
            "internal error: parse_compile_command_args expects full `daglang {command} ...` argv"
        ));
    }
    let mut input: Option<String> = None;
    let mut emit_collection_nodes = false;
    let mut trace_stages = false;
    let mut profile: Option<String> = None;
    let mut target: Option<CodegenTarget> = None;
    let mut layer: Option<CodegenLayer> = None;
    let mut format = CompileOutputFormat::Summary;
    let mut saw_format = false;
    let mut out_dir: Option<String> = None;
    let mut receipt = false;
    let mut i = 2usize;
    while i < args.len() {
        let token = &args[i];
        if token == "--receipt" {
            if command != "compile" || receipt {
                return Err(usage.to_string());
            }
            receipt = true;
            i += 1;
            continue;
        }
        if token == "--emit-collection-nodes" {
            if emit_collection_nodes {
                return Err(usage.to_string());
            }
            emit_collection_nodes = true;
            i += 1;
            continue;
        }
        if token == "--trace-stages" {
            if command != "compile" || trace_stages {
                return Err(usage.to_string());
            }
            trace_stages = true;
            i += 1;
            continue;
        }
        if token == "--target" {
            if command != "compile" {
                return Err(usage.to_string());
            }
            if target.is_some() {
                return Err(usage.to_string());
            }
            let value = args.get(i + 1).ok_or_else(|| usage.to_string())?;
            target = Some(parse_codegen_target(value).ok_or_else(|| usage.to_string())?);
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--target=") {
            if command != "compile" {
                return Err(usage.to_string());
            }
            if target.is_some() {
                return Err(usage.to_string());
            }
            target = Some(parse_codegen_target(value).ok_or_else(|| usage.to_string())?);
            i += 1;
            continue;
        }
        if token == "--profile" {
            if command != "compile" || profile.is_some() {
                return Err(usage.to_string());
            }
            let value = args.get(i + 1).ok_or_else(|| usage.to_string())?;
            if value.starts_with("--") || value.is_empty() {
                return Err(usage.to_string());
            }
            profile = Some(value.clone());
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--profile=") {
            if command != "compile" || profile.is_some() || value.is_empty() {
                return Err(usage.to_string());
            }
            profile = Some(value.to_string());
            i += 1;
            continue;
        }
        if token == "--layer" {
            if command != "compile" {
                return Err(usage.to_string());
            }
            if layer.is_some() {
                return Err(usage.to_string());
            }
            let value = args.get(i + 1).ok_or_else(|| usage.to_string())?;
            layer = Some(parse_codegen_layer(value).ok_or_else(|| usage.to_string())?);
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--layer=") {
            if command != "compile" {
                return Err(usage.to_string());
            }
            if layer.is_some() {
                return Err(usage.to_string());
            }
            layer = Some(parse_codegen_layer(value).ok_or_else(|| usage.to_string())?);
            i += 1;
            continue;
        }
        if token == "--out" {
            if command != "compile" || out_dir.is_some() {
                return Err(usage.to_string());
            }
            let value = args.get(i + 1).ok_or_else(|| usage.to_string())?;
            if value.starts_with("--") {
                return Err(usage.to_string());
            }
            out_dir = Some(value.clone());
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--out=") {
            if command != "compile" || out_dir.is_some() || value.is_empty() {
                return Err(usage.to_string());
            }
            out_dir = Some(value.to_string());
            i += 1;
            continue;
        }
        if token == "--format" {
            if command != "compile" || saw_format {
                return Err(usage.to_string());
            }
            let value = args.get(i + 1).ok_or_else(|| usage.to_string())?;
            format = parse_compile_output_format(value).ok_or_else(|| usage.to_string())?;
            saw_format = true;
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--format=") {
            if command != "compile" || saw_format {
                return Err(usage.to_string());
            }
            format = parse_compile_output_format(value).ok_or_else(|| usage.to_string())?;
            saw_format = true;
            i += 1;
            continue;
        }
        if token.starts_with("--") {
            return Err(usage.to_string());
        }
        if input.is_some() {
            return Err(usage.to_string());
        }
        input = Some(token.clone());
        i += 1;
    }
    if require_input && input.is_none() {
        return Err(usage.to_string());
    }
    if command == "compile"
        && matches!(format, CompileOutputFormat::CanonicalJson)
        && (out_dir.is_some() || trace_stages)
    {
        return Err(usage.to_string());
    }
    Ok(CompileCommandArgs {
        input,
        emit_collection_nodes,
        trace_stages,
        profile,
        target,
        layer,
        format,
        out_dir,
        receipt,
    })
}

fn parse_compile_output_format(value: &str) -> Option<CompileOutputFormat> {
    match value {
        "summary" => Some(CompileOutputFormat::Summary),
        "canonical-json" => Some(CompileOutputFormat::CanonicalJson),
        _ => None,
    }
}

fn parse_codegen_target(value: &str) -> Option<CodegenTarget> {
    match value {
        "rust" => Some(CodegenTarget::Rust),
        "go" => Some(CodegenTarget::Go),
        "c" => Some(CodegenTarget::C),
        "mips" => Some(CodegenTarget::Mips),
        _ => None,
    }
}

fn parse_codegen_layer(value: &str) -> Option<CodegenLayer> {
    match value {
        "1" | "exec-runtime" => Some(CodegenLayer::ExecRuntime),
        "2" | "native" => Some(CodegenLayer::Native),
        _ => None,
    }
}

pub(crate) fn parse_progress_command_args(
    command: &str,
    args: &[String],
) -> Result<ProgressCommandArgs, String> {
    let usage = format!("{command} <file.dag> [--format text|json] [--emit-collection-nodes]");
    if args.is_empty() || args.get(1).map(String::as_str) != Some(command) {
        return Err(format!(
            "internal error: parse_progress_command_args expects full `daglang {command} ...` argv"
        ));
    }
    let Some(input) = args.get(2).cloned() else {
        return Err(usage.to_string());
    };
    if input.starts_with("--") {
        return Err(usage.to_string());
    }
    let mut format = OutputFormat::Text;
    let mut saw_format = false;
    let mut emit_collection_nodes = false;
    let mut i = 3usize;
    while i < args.len() {
        let token = &args[i];
        if token == "--emit-collection-nodes" {
            if emit_collection_nodes {
                return Err(usage.to_string());
            }
            emit_collection_nodes = true;
            i += 1;
            continue;
        }
        if token == "--format" {
            if saw_format {
                return Err(usage.clone());
            }
            let Some(value) = args.get(i + 1) else {
                return Err(usage.clone());
            };
            format = match value.as_str() {
                "text" => OutputFormat::Text,
                "json" => OutputFormat::Json,
                _ => return Err(usage.clone()),
            };
            saw_format = true;
            i += 2;
            continue;
        }
        return Err(usage.clone());
    }
    Ok(ProgressCommandArgs {
        input,
        format,
        emit_collection_nodes,
    })
}

pub(crate) fn parse_viz_args(args: &[String]) -> Result<(VizTarget, VizFormat), String> {
    if args.is_empty() || args.get(1).map(String::as_str) != Some("viz") {
        return Err(
            "internal error: parse_viz_args expects full `daglang viz ...` argv".to_string(),
        );
    }
    if args.len() == 2 {
        return Err(VIZ_USAGE.to_string());
    }

    let mut target: Option<VizTarget> = None;
    let mut format = DEFAULT_VIZ_FORMAT;
    let mut i = 2usize;
    while i < args.len() {
        let token = &args[i];
        if token == "--format" {
            let value = args.get(i + 1).ok_or_else(|| VIZ_USAGE.to_string())?;
            format = match value.as_str() {
                "ascii" => VizFormat::Ascii,
                "mermaid" => VizFormat::Mermaid,
                _ => return Err(VIZ_USAGE.to_string()),
            };
            i += 2;
            continue;
        }
        if token.starts_with("--") && token != "--self" {
            return Err(VIZ_USAGE.to_string());
        }
        if target.is_some() {
            return Err(VIZ_USAGE.to_string());
        }
        target = Some(if token == "--self" {
            VizTarget::SelfDag
        } else {
            VizTarget::CompiledTarget(token.clone())
        });
        i += 1;
    }

    let Some(target) = target else {
        return Err(VIZ_USAGE.to_string());
    };

    Ok((target, format))
}

pub(crate) fn parse_modules_args(
    args: &[String],
) -> Result<(Option<String>, OutputFormat), String> {
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

pub(crate) fn render_modules_result_json(result: &PipelineResult) -> String {
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

pub(crate) fn parse_output_format(command: &str, args: &[String]) -> Result<OutputFormat, String> {
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

pub(crate) fn parse_run_args(args: &[String]) -> Result<RunArgs, String> {
    if args.is_empty() || args.get(1).map(String::as_str) != Some("run") {
        return Err(
            "internal error: parse_run_args expects full `daglang run ...` argv".to_string(),
        );
    }

    let mut mode = RunMode::Real;
    let mut output_path = "Makefile".to_string();
    let mut has_output = false;
    let mut input_path: Option<String> = None;
    let mut params = std::collections::BTreeMap::new();
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
        if token == "--param" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| "--param requires KEY=VALUE".to_string())?;
            let (key, val) = value
                .split_once('=')
                .ok_or_else(|| format!("--param value must be KEY=VALUE, got `{value}`"))?;
            params.insert(key.to_string(), val.to_string());
            i += 2;
            continue;
        }
        if let Some(kv) = token.strip_prefix("--param=") {
            let (key, val) = kv
                .split_once('=')
                .ok_or_else(|| format!("--param value must be KEY=VALUE, got `{kv}`"))?;
            params.insert(key.to_string(), val.to_string());
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
    let input_pb = PathBuf::from(&input_path);
    if let Some(error) = path_utils::check_dag_extension_casing(&input_pb) {
        return Err(error);
    }
    if !path_utils::has_dag_extension(&input_pb) {
        return Err(format!(
            "run input must be a .dag file path, got `{input_path}`"
        ));
    }

    Ok(RunArgs {
        input_path,
        output_path,
        mode,
        params,
    })
}

pub(crate) struct GenTypesArgs {
    pub input: Option<String>,
    pub modules: Vec<String>,
    pub output: Option<String>,
}

pub(crate) fn parse_gen_types_args(
    args: &[String],
) -> Result<GenTypesArgs, String> {
    let usage = "gen-types [<dir>] [--module <module.path>]... [--output <path>]";
    if args.is_empty() || args.get(1).map(String::as_str) != Some("gen-types") {
        return Err(usage.to_string());
    }
    let mut input: Option<String> = None;
    let mut modules: Vec<String> = Vec::new();
    let mut output: Option<String> = None;
    let mut i = 2usize;
    while i < args.len() {
        let token = &args[i];
        if token == "--module" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| usage.to_string())?;
            if value.starts_with("--") || value.is_empty() {
                return Err(usage.to_string());
            }
            modules.push(value.clone());
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--module=") {
            if value.is_empty() {
                return Err(usage.to_string());
            }
            modules.push(value.to_string());
            i += 1;
            continue;
        }
        if token == "--output" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| usage.to_string())?;
            output = Some(value.clone());
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--output=") {
            if value.is_empty() {
                return Err(usage.to_string());
            }
            output = Some(value.to_string());
            i += 1;
            continue;
        }
        if token.starts_with("--") {
            return Err(usage.to_string());
        }
        if input.is_some() {
            return Err(usage.to_string());
        }
        input = Some(token.clone());
        i += 1;
    }
    Ok(GenTypesArgs { input, modules, output })
}

pub(crate) fn exit_usage(command: &str) -> ! {
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
        let path = root_path()
            .join("workspace")
            .join(".")
            .join("core")
            .join("..")
            .join("dsl");
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_keeps_root_when_parent_traversal_exceeds_depth() {
        let path = root_path()
            .join("..")
            .join("..")
            .join("workspace")
            .join("dsl");
        let normalized = normalize_path_components(&path);
        let expected = root_path().join("workspace").join("dsl");
        assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_path_components_preserves_clean_absolute_paths() {
        let path = root_path()
            .join("workspace")
            .join("dsl")
            .join("tools")
            .join("makegen.dag");
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
        let expected = root_path()
            .join("workspace")
            .join("dsl")
            .join("makegen.dag");
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
        let path = PathBuf::from(format!("{}workspace//dsl//./.", std::path::MAIN_SEPARATOR));
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
        let expected = root_path()
            .join("workspace")
            .join("dsl")
            .join("makegen.dag");
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
    fn has_dag_extension_accepts_lowercase_only() {
        assert!(has_dag_extension(Path::new("main.dag")));
        assert!(!has_dag_extension(Path::new("main.DAG")));
        assert!(!has_dag_extension(Path::new("main.DaG")));
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

        let roots = super::resolve_configured_roots(&cwd).expect("configured roots should parse");
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

        let error = super::resolve_configured_roots(&cwd).expect_err("empty roots should fail");
        assert!(error.contains("must not be empty"));

        std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
    }

    #[test]
    fn resolve_default_roots_prefers_configured_roots() {
        let cwd = unique_temp_dir("config_default_roots");
        std::fs::create_dir_all(cwd.join("custom")).expect("failed to create custom dir");
        std::fs::write(
            cwd.join("daglang.toml"),
            "[discovery]\nroots = [\"custom\"]\n",
        )
        .expect("failed to write daglang.toml");

        let roots =
            super::resolve_default_roots(&cwd).expect("configured default roots should resolve");
        assert_eq!(roots, vec![cwd.join("custom")]);

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
        let args = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "--self".to_string(),
        ];
        let (target, format) = super::parse_viz_args(&args).expect("viz args should parse");
        assert!(matches!(target, super::VizTarget::SelfDag));
        assert!(matches!(format, super::VizFormat::Ascii));
        assert_eq!(super::default_viz_format_label(), "ascii");
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
        assert_eq!(
            super::parse_modules_args(&duplicate_roots),
            Err(usage.clone())
        );
        assert_eq!(super::parse_modules_args(&bad_format), Err(usage));
    }

    #[test]
    fn parse_compile_command_args_accepts_collection_flag_and_target() {
        let args = vec![
            "daglang".to_string(),
            "expand".to_string(),
            "--emit-collection-nodes".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = super::parse_compile_command_args(
            "expand",
            &args,
            "expand <file.dag> [--emit-collection-nodes]",
            true,
        )
        .expect("expand compile args should parse");
        assert_eq!(parsed.input.as_deref(), Some("dsl/tools/makegen.dag"));
        assert!(parsed.emit_collection_nodes);
        assert!(parsed.target.is_none());
        assert!(parsed.layer.is_none());
        assert!(matches!(parsed.format, super::CompileOutputFormat::Summary));
        assert!(parsed.profile.is_none());
        assert!(!parsed.trace_stages);
        assert!(parsed.out_dir.is_none());
    }

    #[test]
    fn parse_compile_command_args_handles_codegen_and_output_flags() {
        let usage = "compile <file.dag|dir> [--emit-collection-nodes] [--trace-stages] [--profile <name>] [--target rust|go|c|mips] [--layer 1|2] [--format summary|canonical-json] [--out <dir>|--out=<dir>]";
        let valid = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--trace-stages".to_string(),
            "--profile".to_string(),
            "unit_test".to_string(),
            "--target".to_string(),
            "rust".to_string(),
            "--layer".to_string(),
            "1".to_string(),
            "--out=target/generated/test".to_string(),
        ];
        let canonical_json = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "canonical-json".to_string(),
        ];
        let duplicate_flag = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "--emit-collection-nodes".to_string(),
            "--emit-collection-nodes".to_string(),
            "dsl".to_string(),
        ];
        let duplicate_out = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl".to_string(),
            "--out".to_string(),
            "target/generated/one".to_string(),
            "--out=target/generated/two".to_string(),
        ];
        let unknown_flag = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "--collection".to_string(),
            "dsl".to_string(),
        ];
        let bad_target = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl".to_string(),
            "--target".to_string(),
            "zig".to_string(),
        ];
        let bad_layer = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl".to_string(),
            "--layer".to_string(),
            "3".to_string(),
        ];
        let bad_format = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl".to_string(),
            "--format".to_string(),
            "yaml".to_string(),
        ];
        let canonical_with_out = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl".to_string(),
            "--format".to_string(),
            "canonical-json".to_string(),
            "--out".to_string(),
            "target/generated/one".to_string(),
        ];
        let canonical_with_trace = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "dsl".to_string(),
            "--format".to_string(),
            "canonical-json".to_string(),
            "--trace-stages".to_string(),
        ];
        let missing_target = vec!["daglang".to_string(), "compile".to_string()];

        let parsed_valid = super::parse_compile_command_args("compile", &valid, usage, false)
            .expect("compile parser should accept full codegen args");
        assert_eq!(parsed_valid.input.as_deref(), Some("dsl/tools/makegen.dag"));
        assert_eq!(
            parsed_valid.target,
            Some(daglang_driver::CodegenTarget::Rust)
        );
        assert_eq!(
            parsed_valid.layer,
            Some(daglang_driver::CodegenLayer::ExecRuntime)
        );
        assert_eq!(
            parsed_valid.out_dir.as_deref(),
            Some("target/generated/test")
        );
        assert!(parsed_valid.trace_stages);
        assert_eq!(parsed_valid.profile.as_deref(), Some("unit_test"));
        assert!(matches!(
            parsed_valid.format,
            super::CompileOutputFormat::Summary
        ));

        let parsed_canonical =
            super::parse_compile_command_args("compile", &canonical_json, usage, false)
                .expect("compile parser should accept canonical-json format");
        assert!(matches!(
            parsed_canonical.format,
            super::CompileOutputFormat::CanonicalJson
        ));
        assert!(!parsed_canonical.trace_stages);
        assert!(parsed_canonical.profile.is_none());
        assert!(parsed_canonical.out_dir.is_none());

        assert_eq!(
            super::parse_compile_command_args("compile", &duplicate_flag, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &duplicate_out, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &unknown_flag, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &bad_target, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &bad_layer, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &bad_format, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &canonical_with_out, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &canonical_with_trace, usage, false),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &missing_target, usage, false)
                .expect("compile allows default root")
                .input,
            None
        );
        assert_eq!(
            super::parse_compile_command_args("compile", &missing_target, usage, true),
            Err(usage.to_string())
        );
    }

    #[test]
    fn parse_compile_command_args_rejects_codegen_flags_for_expand() {
        let usage = "expand <file.dag> [--emit-collection-nodes]";
        let with_target = vec![
            "daglang".to_string(),
            "expand".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--target".to_string(),
            "rust".to_string(),
        ];
        let with_out = vec![
            "daglang".to_string(),
            "expand".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--out".to_string(),
            "target/generated".to_string(),
        ];
        let with_format = vec![
            "daglang".to_string(),
            "expand".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--format".to_string(),
            "canonical-json".to_string(),
        ];
        let with_profile = vec![
            "daglang".to_string(),
            "expand".to_string(),
            "dsl/tools/makegen.dag".to_string(),
            "--profile".to_string(),
            "unit_test".to_string(),
        ];

        assert_eq!(
            super::parse_compile_command_args("expand", &with_target, usage, true),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("expand", &with_out, usage, true),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("expand", &with_format, usage, true),
            Err(usage.to_string())
        );
        assert_eq!(
            super::parse_compile_command_args("expand", &with_profile, usage, true),
            Err(usage.to_string())
        );
    }

    #[test]
    fn parse_compile_command_args_rejects_invalid_shapes() {
        let usage = "compile <file.dag|dir> [--emit-collection-nodes] [--trace-stages] [--profile <name>] [--target rust|go|c|mips] [--layer 1|2] [--format summary|canonical-json] [--out <dir>|--out=<dir>]";
        let unknown_flag = vec![
            "daglang".to_string(),
            "compile".to_string(),
            "--collection".to_string(),
            "dsl".to_string(),
        ];
        assert_eq!(
            super::parse_compile_command_args("compile", &unknown_flag, usage, false),
            Err(usage.to_string())
        );
    }

    #[test]
    fn parse_progress_command_args_accepts_format_and_collection_flag() {
        let args = vec![
            "daglang".to_string(),
            "progress".to_string(),
            "dsl/tools/gist.dag".to_string(),
            "--emit-collection-nodes".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let parsed = super::parse_progress_command_args("progress", &args)
            .expect("progress args should parse");
        assert_eq!(parsed.input, "dsl/tools/gist.dag");
        assert!(matches!(parsed.format, super::OutputFormat::Json));
        assert!(parsed.emit_collection_nodes);
    }

    #[test]
    fn parse_progress_command_args_parses_correctly() {
        let args = vec![
            "daglang".to_string(),
            "progress".to_string(),
            "dsl/tools/gist.dag".to_string(),
        ];
        let parsed = super::parse_progress_command_args("progress", &args)
            .expect("progress args should parse");
        assert_eq!(parsed.input, "dsl/tools/gist.dag");
        assert!(matches!(parsed.format, super::OutputFormat::Text));
    }

    #[test]
    fn parse_progress_command_args_rejects_invalid_shapes() {
        let usage =
            "progress <file.dag> [--format text|json] [--emit-collection-nodes]".to_string();
        let missing_input = vec!["daglang".to_string(), "progress".to_string()];
        let duplicate_format = vec![
            "daglang".to_string(),
            "progress".to_string(),
            "dsl/tools/gist.dag".to_string(),
            "--format".to_string(),
            "text".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let duplicate_collection_flag = vec![
            "daglang".to_string(),
            "progress".to_string(),
            "dsl/tools/gist.dag".to_string(),
            "--emit-collection-nodes".to_string(),
            "--emit-collection-nodes".to_string(),
        ];
        let unknown_flag = vec![
            "daglang".to_string(),
            "progress".to_string(),
            "dsl/tools/gist.dag".to_string(),
            "--collection".to_string(),
        ];
        assert_eq!(
            super::parse_progress_command_args("progress", &missing_input),
            Err(usage.clone())
        );
        assert_eq!(
            super::parse_progress_command_args("progress", &duplicate_format),
            Err(usage.clone())
        );
        assert_eq!(
            super::parse_progress_command_args("progress", &duplicate_collection_flag),
            Err(usage.clone())
        );
        assert_eq!(
            super::parse_progress_command_args("progress", &unknown_flag),
            Err(usage)
        );
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
        let context = super::build_check_pipeline_context(&cwd, None)
            .expect("build_check_pipeline_context should succeed");
        assert_eq!(context.target_file, None);
        assert_eq!(
            context.roots,
            vec![root_path().join("workspace").join("project").join("dsl")]
        );
    }

    #[test]
    fn build_check_pipeline_context_uses_provided_default_roots_without_input() {
        let cwd = root_path().join("workspace").join("project");
        let defaults = vec![cwd.join("custom"), cwd.join("shared")];
        let context =
            super::build_check_pipeline_context_with_default_roots(&cwd, None, Some(&defaults))
                .expect("build_check_pipeline_context should succeed");
        assert_eq!(context.target_file, None);
        assert_eq!(context.roots, defaults);
    }

    #[test]
    fn build_check_pipeline_context_treats_dag_file_as_target_input() {
        let cwd = root_path().join("workspace").join("project");
        let input = "nested/tool.dag".to_string();
        let context = super::build_check_pipeline_context(&cwd, Some(&input))
            .expect("build_check_pipeline_context should succeed");
        assert_eq!(
            context.target_file,
            Some(cwd.join("nested").join("tool.dag"))
        );
        assert_eq!(context.roots, vec![cwd.join("nested")]);
    }

    #[test]
    fn dag_named_directory_is_rejected_by_conflict_check() {
        let root = unique_temp_dir("check_context_dag_dir");
        let cwd = root.join("workspace");
        let dag_dir = cwd.join("bundle.dag");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .dag directory fixture");

        let input = "bundle.dag".to_string();
        let error = super::build_check_pipeline_context(&cwd, Some(&input))
            .expect_err(".dag directory should be rejected");
        assert!(
            error.contains("is a directory"),
            "error should mention directory: {error}"
        );

        std::fs::remove_dir_all(&root).expect("failed to cleanup temp fixture");
    }

    #[test]
    fn build_check_pipeline_context_treats_uppercase_dag_named_directory_as_root() {
        let root = unique_temp_dir("check_context_uppercase_dag_dir");
        let cwd = root.join("workspace");
        let dag_dir = cwd.join("bundle.DAG");
        std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory fixture");
        let input = "bundle.DAG".to_string();

        let context = super::build_check_pipeline_context(&cwd, Some(&input))
            .expect("non-lowercase .dag extension should not be rejected");

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

        let context = super::build_check_pipeline_context(&cwd, Some(&input))
            .expect("non-lowercase .dag extension should not be rejected");

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
