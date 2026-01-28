mod ops;
mod graph;
mod generated;

// Contract definitions — source of truth for port names, types, and topology.
// Currently consumed only by verification tests; codegen binary will read these directly.
#[cfg(test)]
mod contracts;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gunbc-gistgen", about = "Generate a GitHub Gist from repository files")]
struct Cli {
    /// Path to the repository root
    #[arg(default_value = ".")]
    path: String,

    /// Glob pattern for file selection
    #[arg(long, default_value = "**/*")]
    glob: String,

    /// Print what would be uploaded without actually creating a gist
    #[arg(long)]
    dry_run: bool,

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
    let dag = graph::build_gistgen_dag(&cli.path, &cli.glob, cli.dry_run);

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
