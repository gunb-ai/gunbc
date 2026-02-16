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

use std::path::PathBuf;

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
            if args.get(2).map(String::as_str) == Some("--self") {
                let dag = build_pipeline_dag();
                println!("{}", dag.to_mermaid("daglang-compiler-pipeline"));
            } else {
                eprintln!("TODO: dag viz <file.dag> -- file visualization is not implemented yet.");
            }
        }
        "expand" => {
            eprintln!("TODO: dag expand -- Phase 0 deliverable");
            eprintln!("Will show lowered GraphIR: every Node, Edge, Port.");
        }
        "manifest" => {
            eprintln!("TODO: dag manifest -- Phase 0 deliverable");
            eprintln!("Will show derived ProgressManifest: topology, waves, boundaries.");
        }
        "modules" => {
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
            let input = args.get(2).map(PathBuf::from);
            let (roots, target_file) = match input {
                Some(path) if path.extension().and_then(|ext| ext.to_str()) == Some("dag") => {
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
            eprintln!("TODO: dag compile -- Phase 1 deliverable");
            eprintln!("Will run full pipeline: parse → resolve → typecheck → lower → validate → derive → emit.");
        }
        cmd => {
            eprintln!("Unknown command: {cmd}");
            std::process::exit(1);
        }
    }
}

fn resolve_root(arg: Option<&String>) -> PathBuf {
    if let Some(path) = arg {
        return PathBuf::from(path);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dsl = cwd.join("dsl");
    if dsl.exists() {
        dsl
    } else {
        PathBuf::from("dsl")
    }
}
