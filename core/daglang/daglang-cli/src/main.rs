//! daglang CLI: dag viz, dag expand, dag manifest, dag modules.
//!
//! The development tool for the DSL compiler. Provides visualization
//! and introspection commands that make every subsequent phase
//! implementable — "see what you're building before you build it."
//!
//! # Commands
//!
//! - `daglang viz <file.dag>`      -- Mermaid DAG visualization from compiled IR
//! - `daglang expand <file.dag>`   -- Show lowered GraphIR (nodes, edges, ports)
//! - `daglang manifest <file.dag>` -- Show derived ProgressManifest
//! - `daglang obligations <file.dag> [--format text|json]`
//!                                  -- Show derived test obligations summary
//! - `daglang show-triplets <file.dag> [--format text|json]`
//!                                  -- Show transport triplet expansions
//! - `daglang modules [dir]`       -- Show the discovered module graph
//! - `daglang check <file.dag>`    -- Parse + typecheck without lowering
//! - `daglang compile <file.dag>`  -- Full compilation pipeline

use std::path::PathBuf;

use daglang_cli::compile::{
    build_context, compile_from_context, render_expand, render_manifest, render_obligations,
    render_triplets, CompileOutput, OutputFormat,
};
use daglang_cli::path_utils;
use daglang_cli::pipeline::{
    build_pipeline_dag, run_pipeline, PipelineContext, PipelineResult, PipelineStop,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if args.len() < 2 {
        eprintln!("Usage: daglang <command> [args...]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  viz <file.dag>       Mermaid DAG visualization");
        eprintln!("  expand <file.dag>    Show lowered GraphIR (nodes/edges/ports)");
        eprintln!("  manifest <file.dag>  Show derived ProgressManifest");
        eprintln!("  obligations <file.dag> [--format text|json]");
        eprintln!("                      Show derived test obligations summary");
        eprintln!("  show-triplets <file.dag> [--format text|json]");
        eprintln!("                      Show transport triplet expansions");
        eprintln!("  modules [dir]        Show discovered module graph");
        eprintln!("  check <file.dag>     Parse + typecheck (no lowering)");
        eprintln!("  compile <file.dag>   Full compilation pipeline");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "viz" => {
            match args.len() {
                2 => exit_usage("viz <file.dag>|viz --self"),
                3 if args[2] == "--self" => {
                    let dag = build_pipeline_dag();
                    println!("{}", dag.to_mermaid("daglang-compiler-pipeline"));
                }
                3 => {
                    let output = compile_target_or_exit(&cwd, args.get(2));
                    println!("{}", output.lowered_dag.to_mermaid("daglang-compiled"));
                }
                _ => exit_usage("viz <file.dag>|viz --self"),
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
            if args.len() != 3 {
                exit_usage("manifest <file.dag>");
            }
            let output = compile_target_or_exit(&cwd, args.get(2));
            println!("{}", render_manifest(&output.derived));
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
            if args.len() > 3 {
                exit_usage("modules [dir]");
            }
            let roots = vec![resolve_root(&cwd, args.get(2))];
            let context = PipelineContext {
                roots,
                target_file: None,
            };
            let result = run_pipeline_or_exit(&context, PipelineStop::Report);
            if let Some(report) = result.report() {
                println!("{report}");
            }
        }
        "check" => {
            if args.len() > 3 {
                exit_usage("check <file.dag|dir>");
            }
            let context = build_check_pipeline_context(&cwd, args.get(2));
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
        cmd => {
            eprintln!("Unknown command: {cmd}");
            exit_usage("<command> [args...]");
        }
    }
}

fn resolve_root(cwd: &std::path::Path, arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return path_utils::normalize_cli_path(cwd, &PathBuf::from(path));
    }
    path_utils::resolve_default_root(cwd)
}

fn build_check_pipeline_context(
    cwd: &std::path::Path,
    input: Option<&String>,
) -> PipelineContext {
    let normalized_input = input
        .map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    let (roots, target_file) = match normalized_input {
        Some(path) if path_utils::has_dag_extension(&path) && !path.is_dir() => {
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

fn exit_usage(command: &str) -> ! {
    eprintln!("Usage: daglang {command}");
    std::process::exit(1);
}


#[cfg(test)]
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
}
