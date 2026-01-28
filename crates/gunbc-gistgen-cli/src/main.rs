//! CLI entrypoint for gistgen.
//!
//! This binary wraps the gunbc-gistgen core library and exposes it via a CLI interface.

use clap::Parser;
use gunbc_gistgen::{GistPayloadMode, UnderstandingMode};

#[derive(Parser, Debug)]
#[command(name = "gunbc-gistgen", about = "Generate a GitHub Gist from repository files")]
struct Cli {
    /// Path to the repository root
    #[arg(default_value = ".")]
    path: String,

    /// Glob pattern for file selection
    #[arg(long, default_value = "**/*")]
    glob: String,

    /// Preview what would be uploaded without creating a gist
    #[arg(long)]
    dry_run: bool,

    /// Emit multiple files in the gist (default is single markdown snapshot)
    #[arg(long)]
    multi_file: bool,

    /// Emit node-level SVG graph to stdout and exit
    #[arg(long)]
    svg: bool,

    /// Emit tool-level SVG graph to stdout and exit
    #[arg(long)]
    svg_tools: bool,

    /// Include guard expressions on input ports in SVG output
    #[arg(long)]
    svg_show_guards: bool,
}

fn main() {
    let cli = Cli::parse();

    let mode = if cli.dry_run {
        UnderstandingMode::Mock
    } else {
        UnderstandingMode::Real
    };

    let payload = if cli.multi_file {
        GistPayloadMode::FileMap
    } else {
        GistPayloadMode::SingleFile
    };

    let dag = gunbc_gistgen::build_gistgen_dag_with_payload(&cli.path, &cli.glob, mode, payload);

    if cli.svg_tools {
        println!("{}", gunbc_ir::viz::tools_to_svg(&dag));
        return;
    }
    if cli.svg {
        println!("{}", gunbc_ir::viz::dag_to_svg(&dag, cli.svg_show_guards));
        return;
    }

    eprintln!("DAG constructed ({} nodes, {} edges)", dag.nodes.len(), dag.edges.len());

    // Execute
    match gunbc_exec::execute(&dag) {
        Ok(log) => {
            eprintln!("\nExecution log:");
            eprint!("{log}");
        }
        Err(e) => {
            eprintln!("Execution failed: {e}");
            std::process::exit(1);
        }
    }
}
