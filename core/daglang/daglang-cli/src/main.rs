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
            eprintln!("TODO: dag viz -- Phase 0 deliverable");
            eprintln!("Will show ASCII DAG visualization from compiled IR.");
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
            eprintln!("TODO: dag modules -- Phase 0 deliverable");
            eprintln!("Will show the discovered module graph from filesystem scan.");
        }
        "check" => {
            eprintln!("TODO: dag check -- Phase 0 deliverable");
            eprintln!("Will parse + typecheck without lowering.");
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
