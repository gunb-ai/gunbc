//! CLI entrypoint for gistgen.
//!
//! This binary wraps the gunbc-gistgen core library and exposes it via a CLI interface.

use clap::Parser;
use gunbc_exec::TerminalObserver;
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

    // Execute with progress observer
    let mut observer = TerminalObserver::new("gistgen");
    match gunbc_exec::execute_with_observer(&dag, Some(&mut observer)) {
        Ok(log) => {
            // Extract and display the gist URL if present
            // Look for the extract_gist_url node which has the final URL
            if let Some(entry) = log.entries.iter().find(|e| e.node_id.contains("extract_gist_url"))
            {
                if let Some(gunbc_exec::Value::Str(url)) = entry.outputs.get("gist_url") {
                    eprintln!();
                    eprintln!("  Gist URL: {url}");
                }
            }
        }
        Err(e) => {
            eprintln!("Execution failed: {e}");
            std::process::exit(1);
        }
    }
}
