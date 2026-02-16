//! daglang CLI: dag viz, dag expand, dag manifest, dag modules.
//!
//! The development tool for the DSL compiler. Provides visualization
//! and introspection commands that make every subsequent phase
//! implementable — "see what you're building before you build it."
//!
//! # Commands
//!
//! - `daglang viz <file.dag>`      -- ASCII DAG visualization from compiled IR
//! - `daglang expand <file.dag>`   -- Show lowered GraphIR (nodes, edges, ports)
//! - `daglang manifest <file.dag>` -- Show derived ProgressManifest
//! - `daglang modules [dir]`       -- Show the discovered module graph
//! - `daglang check <file.dag>`    -- Parse + typecheck without lowering
//! - `daglang compile <file.dag>`  -- Full compilation pipeline

use std::path::{Path, PathBuf};

use daglang_cli::compile::{
    build_context, compile_from_context, render_expand, render_manifest,
};
use daglang_cli::path_utils;
use daglang_cli::pipeline::{build_pipeline_dag, run_pipeline, PipelineContext, PipelineStop};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: daglang <command> [args...]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  viz <file.dag>       ASCII DAG visualization");
        eprintln!("  expand <file.dag>    Show lowered GraphIR (nodes/edges/ports)");
        eprintln!("  manifest <file.dag>  Show derived ProgressManifest");
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
                    let context = build_context(args.get(2));
                    match compile_from_context(&context) {
                        Ok(output) => {
                            println!("{}", output.lowered_dag.to_mermaid("daglang-compiled"));
                        }
                        Err(error) => {
                            eprintln!("{error}");
                            std::process::exit(1);
                        }
                    }
                }
                _ => exit_usage("viz <file.dag>|viz --self"),
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
            if args.len() != 3 {
                exit_usage("manifest <file.dag>");
            }
            let context = build_context(args.get(2));
            match compile_from_context(&context) {
                Ok(output) => {
                    println!("{}", render_manifest(&output.derived));
                }
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
            let input = args.get(2).map(|value| normalize_cli_path(PathBuf::from(value)));
            let (roots, target_file) = match input {
                Some(path) if has_dag_extension(&path) && !path.is_dir() =>
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
            match run_pipeline(&context, PipelineStop::Parse) {
                Ok(result) => {
                    if result.diagnostics.is_empty() {
                        println!("OK: parsed {} file(s)", result.parsed_count);
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
        cmd => {
            eprintln!("Unknown command: {cmd}");
            exit_usage("<command> [args...]");
        }
    }
}

fn resolve_root(arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return normalize_cli_path(PathBuf::from(path));
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    default_root_from_cwd(&cwd)
}

fn exit_usage(command: &str) -> ! {
    eprintln!("Usage: daglang {command}");
    std::process::exit(1);
}

fn has_dag_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dag"))
}

fn normalize_cli_path(path: PathBuf) -> PathBuf {
    path_utils::normalize_cli_path(path)
}

fn default_root_from_cwd(cwd: &Path) -> PathBuf {
    path_utils::default_root_from_cwd(cwd)
}

#[cfg(test)]
fn normalize_path_components(path: &Path) -> PathBuf {
    path_utils::normalize_path_components(path)
}

#[cfg(test)]
mod tests {
    use super::{default_root_from_cwd, has_dag_extension, normalize_path_components};
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
}
