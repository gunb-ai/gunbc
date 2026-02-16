//! daglang CLI: dag viz, dag expand, dag manifest, dag modules.
//!
//! The development tool for the DSL compiler. Provides visualization
//! and introspection commands that make every subsequent phase
//! implementable — "see what you're building before you build it."
//!
//! # Commands
//!
//! - `daglang viz [--format ascii|mermaid] <file.dag>|--self`
//!   -- DAG visualization from compiled IR
//! - `daglang expand <file.dag>`   -- Show lowered GraphIR (nodes, edges, ports)
//! - `daglang manifest <file.dag>` -- Show derived ProgressManifest
//! - `daglang obligations <file.dag>` -- Show 4-bucket TestObligations summary
//! - `daglang show-triplets <file.dag>` -- Show transport triplet expansion
//! - `daglang modules [dir]`       -- Show the discovered module graph
//! - `daglang check <file.dag>`    -- Parse + typecheck without lowering
//! - `daglang compile <file.dag>`  -- Full compilation pipeline
//! - `daglang run <file.dag>`      -- Compile + resolve + execute makegen DAG

use std::path::PathBuf;

use daglang_cli::compile::{
    build_context, compile_from_context, compile_resolve_execute_from_context,
    makegen_dry_run_transport_mocks, makegen_entrypoint_mocks, render_expand,
    render_manifest_with_format, render_obligations_with_format, render_triplets_with_format,
    render_viz_ascii, ManifestFormat,
};
use daglang_cli::path_utils;
use daglang_cli::pipeline::{build_pipeline_dag, run_pipeline, PipelineContext, PipelineStop};
use gunbc_exec::ExecutionMode;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: daglang <command> [args...]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  viz [--format ascii|mermaid] <file.dag>|--self  DAG visualization");
        eprintln!("  expand <file.dag>    Show lowered GraphIR (nodes/edges/ports)");
        eprintln!("  manifest [--format text|json] <file.dag>  Show derived ProgressManifest");
        eprintln!("  obligations [--format text|json] <file.dag>  Show 4-bucket test obligations");
        eprintln!("  show-triplets [--format text|json] <file.dag>  Show transport triplet expansion");
        eprintln!("  modules [dir]        Show discovered module graph");
        eprintln!("  check <file.dag>     Parse + typecheck (no lowering)");
        eprintln!("  compile <file.dag>   Full compilation pipeline");
        eprintln!("  run <file.dag> [--output <path>] [--dry-run] [--check-mode]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "viz" => {
            let viz_args = match parse_viz_args(&args) {
                Ok(values) => values,
                Err(error) => {
                    eprintln!("{error}");
                    exit_usage("viz [--format ascii|mermaid] <file.dag>|--self");
                }
            };
            match viz_args.target {
                VizTarget::SelfGraph => {
                    let dag = build_pipeline_dag();
                    match viz_args.format {
                        VizFormat::Ascii => {
                            println!("{}", render_viz_ascii("daglang-compiler-pipeline", &dag))
                        }
                        VizFormat::Mermaid => {
                            println!("{}", dag.to_mermaid("daglang-compiler-pipeline"))
                        }
                    }
                }
                VizTarget::File(file) => {
                    let context = build_context(Some(&file));
                    match compile_from_context(&context) {
                        Ok(output) => match viz_args.format {
                            VizFormat::Ascii => {
                                println!("{}", render_viz_ascii("daglang-compiled", &output.lowered_dag))
                            }
                            VizFormat::Mermaid => {
                                println!("{}", output.lowered_dag.to_mermaid("daglang-compiled"))
                            }
                        },
                        Err(error) => {
                            eprintln!("{error}");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        "expand" => {
            if args.len() != 3 {
                exit_usage("expand <file.dag>");
            }
            let context = build_context(args.get(2));
            match compile_from_context(&context) {
                Ok(output) => {
                    println!("{}", render_expand(&output.lowered_dag));
                }
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        "manifest" => {
            let (manifest_path, manifest_format) = match parse_manifest_args(&args) {
                Ok(values) => values,
                Err(error) => {
                    eprintln!("{error}");
                    exit_usage("manifest [--format text|json] <file.dag>");
                }
            };
            let context = build_context(Some(&manifest_path));
            match compile_from_context(&context) {
                Ok(output) => {
                    match render_manifest_with_format(&output.derived, manifest_format) {
                        Ok(rendered) => println!("{rendered}"),
                        Err(error) => {
                            eprintln!("{error}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        "obligations" => {
            let (obligations_path, obligations_format) = match parse_obligations_args(&args) {
                Ok(values) => values,
                Err(error) => {
                    eprintln!("{error}");
                    exit_usage("obligations [--format text|json] <file.dag>");
                }
            };
            let context = build_context(Some(&obligations_path));
            match compile_from_context(&context) {
                Ok(output) => {
                    match render_obligations_with_format(&output.derived, obligations_format) {
                        Ok(rendered) => println!("{rendered}"),
                        Err(error) => {
                            eprintln!("{error}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        "show-triplets" => {
            let (triplets_path, triplets_format) = match parse_show_triplets_args(&args) {
                Ok(values) => values,
                Err(error) => {
                    eprintln!("{error}");
                    exit_usage("show-triplets [--format text|json] <file.dag>");
                }
            };
            let context = build_context(Some(&triplets_path));
            match compile_from_context(&context) {
                Ok(output) => match render_triplets_with_format(&output.lowered_dag, triplets_format)
                {
                    Ok(rendered) => println!("{rendered}"),
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(1);
                    }
                },
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        "modules" => {
            if args.len() > 3 {
                exit_usage("modules [dir]");
            }
            let roots = vec![resolve_root(args.get(2))];
            let context = PipelineContext {
                roots,
                target_file: None,
            };
            match run_pipeline(&context, PipelineStop::Report) {
                Ok(result) => {
                    if let Some(report) = result.report {
                        println!("{report}");
                    }
                }
                Err(error) => {
                    eprintln!("pipeline error: {error}");
                    std::process::exit(1);
                }
            }
        }
        "check" => {
            if args.len() > 3 {
                exit_usage("check <file.dag|dir>");
            }
            let input = args.get(2).map(|value| path_utils::normalize_cli_path(PathBuf::from(value)));
            let (roots, target_file) = match input {
                Some(path) if path_utils::has_dag_extension(&path) && !path.is_dir() =>
                {
                    let root = path
                        .parent()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()));
                    (vec![root], Some(path))
                }
                Some(path) => (vec![path], None),
                None => (vec![resolve_root(None)], None),
            };

            let context = PipelineContext { roots, target_file };
            match run_pipeline(&context, PipelineStop::Build) {
                Ok(result) => {
                    if result.diagnostics.is_empty() {
                        println!("OK: checked {} file(s)", result.parsed_count);
                    } else {
                        for diagnostic in &result.diagnostics {
                            eprintln!("{diagnostic}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(error) => {
                    eprintln!("pipeline error: {error}");
                    std::process::exit(1);
                }
            }
        }
        "compile" => {
            if args.len() > 3 {
                exit_usage("compile <file.dag|dir>");
            }
            let context = build_context(args.get(2));
            match compile_from_context(&context) {
                Ok(output) => {
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
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        "run" => {
            let run_args = match parse_run_args(&args) {
                Ok(values) => values,
                Err(error) => {
                    eprintln!("{error}");
                    exit_usage("run <file.dag> [--output <path>] [--dry-run] [--check-mode]");
                }
            };
            let context = build_context(Some(&run_args.file));
            let input_mocks = makegen_entrypoint_mocks(&run_args.output_path, run_args.check_mode);
            let mode = if run_args.dry_run {
                ExecutionMode::DryRun(makegen_dry_run_transport_mocks(&run_args.output_path))
            } else {
                ExecutionMode::Real
            };
            match compile_resolve_execute_from_context(&context, mode, Some(&input_mocks)) {
                Ok(log) => {
                    let written = log
                        .entries
                        .iter()
                        .find(|entry| entry.node_id == "tools.makegen::makegen")
                        .and_then(|entry| entry.outputs.get("written"))
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);
                    println!(
                        "Run completed: nodes={}, output={}, written={written}, mode={}",
                        log.entries.len(),
                        run_args.output_path,
                        if run_args.dry_run { "dry-run" } else { "real" }
                    );
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

fn resolve_root(arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return path_utils::normalize_cli_path(PathBuf::from(path));
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path_utils::default_root_from_cwd(&cwd)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VizFormat {
    Ascii,
    Mermaid,
}

impl VizFormat {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "ascii" => Some(Self::Ascii),
            "mermaid" => Some(Self::Mermaid),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VizTarget {
    SelfGraph,
    File(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VizArgs {
    target: VizTarget,
    format: VizFormat,
}

fn parse_viz_args(args: &[String]) -> Result<VizArgs, String> {
    let mut format = VizFormat::Ascii;
    let mut target = None::<VizTarget>;
    let mut index = 2usize;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--self" => {
                if target.is_some() {
                    return Err("viz accepts either --self or <file.dag>, not both".to_string());
                }
                target = Some(VizTarget::SelfGraph);
                index += 1;
            }
            "--format" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--format requires a value: ascii or mermaid".to_string())?;
                format = VizFormat::parse(value)
                    .ok_or_else(|| format!("unknown viz format `{value}`"))?;
                index += 2;
            }
            _ if arg.starts_with("--format=") => {
                let value = arg
                    .strip_prefix("--format=")
                    .expect("prefix should already be checked");
                format = VizFormat::parse(value)
                    .ok_or_else(|| format!("unknown viz format `{value}`"))?;
                index += 1;
            }
            _ if arg.starts_with("--") => return Err(format!("unknown viz flag `{arg}`")),
            _ => {
                if let Some(existing_target) = &target {
                    return Err(match existing_target {
                        VizTarget::SelfGraph => {
                            "viz accepts either --self or <file.dag>, not both".to_string()
                        }
                        VizTarget::File(_) => "viz takes exactly one input path".to_string(),
                    });
                }
                target = Some(VizTarget::File(args[index].clone()));
                index += 1;
            }
        }
    }

    let target = target.ok_or_else(|| "viz requires --self or <file.dag>".to_string())?;
    Ok(VizArgs { target, format })
}

fn parse_single_file_with_optional_format(
    args: &[String],
    command: &str,
) -> Result<(String, ManifestFormat), String> {
    let mut format = ManifestFormat::Text;
    let mut path = None::<String>;
    let mut index = 2usize;
    while index < args.len() {
        let arg = args[index].as_str();
        if arg == "--format" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "--format requires a value: text or json".to_string())?;
            format = ManifestFormat::parse(value)
                .ok_or_else(|| format!("unknown {command} format `{value}`"))?;
            index += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            format = ManifestFormat::parse(value)
                .ok_or_else(|| format!("unknown {command} format `{value}`"))?;
            index += 1;
            continue;
        }
        if arg.starts_with("--") {
            return Err(format!("unknown {command} flag `{arg}`"));
        }
        if path.is_some() {
            return Err(format!("{command} takes exactly one input path"));
        }
        path = Some(args[index].clone());
        index += 1;
    }

    let path = path.ok_or_else(|| format!("{command} requires <file.dag>"))?;
    Ok((path, format))
}

fn parse_manifest_args(args: &[String]) -> Result<(String, ManifestFormat), String> {
    parse_single_file_with_optional_format(args, "manifest")
}

fn parse_obligations_args(args: &[String]) -> Result<(String, ManifestFormat), String> {
    parse_single_file_with_optional_format(args, "obligations")
}

fn parse_show_triplets_args(args: &[String]) -> Result<(String, ManifestFormat), String> {
    parse_single_file_with_optional_format(args, "show-triplets")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunArgs {
    file: String,
    output_path: String,
    dry_run: bool,
    check_mode: bool,
}

fn parse_run_args(args: &[String]) -> Result<RunArgs, String> {
    let mut file = None::<String>;
    let mut output_path = "Makefile".to_string();
    let mut dry_run = false;
    let mut check_mode = false;
    let mut index = 2usize;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--check-mode" => {
                check_mode = true;
                index += 1;
            }
            "--output" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--output requires a path".to_string())?;
                output_path = value.clone();
                index += 2;
            }
            _ if arg.starts_with("--output=") => {
                output_path = arg
                    .strip_prefix("--output=")
                    .expect("prefix just checked")
                    .to_string();
                index += 1;
            }
            _ if arg.starts_with("--") => return Err(format!("unknown run flag `{arg}`")),
            _ => {
                if file.is_some() {
                    return Err("run takes exactly one <file.dag> input".to_string());
                }
                file = Some(args[index].clone());
                index += 1;
            }
        }
    }

    let file = file.ok_or_else(|| "run requires <file.dag>".to_string())?;
    Ok(RunArgs {
        file,
        output_path,
        dry_run,
        check_mode,
    })
}

fn exit_usage(command: &str) -> ! {
    eprintln!("Usage: daglang {command}");
    std::process::exit(1);
}


#[cfg(test)]
mod tests {
    use super::{
        parse_manifest_args, parse_obligations_args, parse_run_args, parse_show_triplets_args,
        parse_viz_args, RunArgs, VizArgs, VizFormat, VizTarget,
    };
    use crate::path_utils::{default_root_from_cwd, has_dag_extension, normalize_path_components};
    use daglang_cli::compile::ManifestFormat;
    use std::path::{Path, PathBuf};

    fn root_path() -> PathBuf {
        PathBuf::from(Path::new(std::path::MAIN_SEPARATOR_STR))
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
    fn default_root_from_cwd_normalizes_curdir_suffix() {
        let cwd = root_path().join("workspace").join("project").join(".");
        let normalized_root = default_root_from_cwd(&cwd);
        let expected = root_path().join("workspace").join("project").join("dsl");
        assert_eq!(normalized_root, expected);
    }

    #[test]
    fn default_root_from_cwd_collapses_parent_segments() {
        let cwd = root_path()
            .join("workspace")
            .join("project")
            .join("nested")
            .join("..");
        let normalized_root = default_root_from_cwd(&cwd);
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
    fn parse_manifest_args_supports_default_text_format() {
        let args = vec![
            "daglang".to_string(),
            "manifest".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let (path, format) = parse_manifest_args(&args).expect("parse should succeed");
        assert_eq!(path, "dsl/tools/makegen.dag");
        assert_eq!(format, ManifestFormat::Text);
    }

    #[test]
    fn parse_manifest_args_supports_json_format_flag() {
        let args = vec![
            "daglang".to_string(),
            "manifest".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let (path, format) = parse_manifest_args(&args).expect("parse should succeed");
        assert_eq!(path, "dsl/tools/makegen.dag");
        assert_eq!(format, ManifestFormat::Json);
    }

    #[test]
    fn parse_manifest_args_requires_a_path() {
        let args = vec!["daglang".to_string(), "manifest".to_string()];
        let error = parse_manifest_args(&args).expect_err("parse should fail");
        assert!(error.contains("requires <file.dag>"));
    }

    #[test]
    fn parse_obligations_args_supports_json_equals_syntax() {
        let args = vec![
            "daglang".to_string(),
            "obligations".to_string(),
            "--format=json".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let (path, format) = parse_obligations_args(&args).expect("parse should succeed");
        assert_eq!(path, "dsl/tools/makegen.dag");
        assert_eq!(format, ManifestFormat::Json);
    }

    #[test]
    fn parse_show_triplets_args_rejects_unknown_flags() {
        let args = vec![
            "daglang".to_string(),
            "show-triplets".to_string(),
            "--mystery".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = parse_show_triplets_args(&args).expect_err("parse should fail");
        assert!(error.contains("unknown show-triplets flag"));
    }

    #[test]
    fn parse_viz_args_defaults_to_ascii_for_file_target() {
        let args = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = parse_viz_args(&args).expect("parse should succeed");
        assert_eq!(
            parsed,
            VizArgs {
                target: VizTarget::File("dsl/tools/makegen.dag".to_string()),
                format: VizFormat::Ascii,
            }
        );
    }

    #[test]
    fn parse_viz_args_supports_mermaid_self_graph() {
        let args = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "--format".to_string(),
            "mermaid".to_string(),
            "--self".to_string(),
        ];
        let parsed = parse_viz_args(&args).expect("parse should succeed");
        assert_eq!(
            parsed,
            VizArgs {
                target: VizTarget::SelfGraph,
                format: VizFormat::Mermaid,
            }
        );
    }

    #[test]
    fn parse_viz_args_rejects_self_and_file_combination() {
        let args = vec![
            "daglang".to_string(),
            "viz".to_string(),
            "--self".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = parse_viz_args(&args).expect_err("parse should fail");
        assert!(error.contains("either --self or <file.dag>"));
    }

    #[test]
    fn parse_run_args_supports_defaults_and_positional_file() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = parse_run_args(&args).expect("parse should succeed");
        assert_eq!(
            parsed,
            RunArgs {
                file: "dsl/tools/makegen.dag".to_string(),
                output_path: "Makefile".to_string(),
                dry_run: false,
                check_mode: false,
            }
        );
    }

    #[test]
    fn parse_run_args_supports_output_dry_run_and_check_mode_flags() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--output".to_string(),
            "out/Generated.mk".to_string(),
            "--dry-run".to_string(),
            "--check-mode".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let parsed = parse_run_args(&args).expect("parse should succeed");
        assert_eq!(
            parsed,
            RunArgs {
                file: "dsl/tools/makegen.dag".to_string(),
                output_path: "out/Generated.mk".to_string(),
                dry_run: true,
                check_mode: true,
            }
        );
    }

    #[test]
    fn parse_run_args_rejects_unknown_flags() {
        let args = vec![
            "daglang".to_string(),
            "run".to_string(),
            "--mystery".to_string(),
            "dsl/tools/makegen.dag".to_string(),
        ];
        let error = parse_run_args(&args).expect_err("parse should fail");
        assert!(error.contains("unknown run flag"));
    }
}
