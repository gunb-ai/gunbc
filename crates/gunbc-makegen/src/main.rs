mod ops;
mod graph;
mod types;

use clap::Parser;
use types::MakegenConfig;

#[derive(Parser, Debug)]
#[command(name = "gunbc-makegen", about = "Generate a Makefile for a Rust workspace")]
struct Cli {
    /// Path to the workspace root
    #[arg(long, default_value = ".")]
    path: String,

    /// Print what would be written without actually creating the Makefile
    #[arg(long)]
    dry_run: bool,

    /// Force regeneration even if up-to-date
    #[arg(long)]
    force: bool,

    /// Disable per-crate targets (build-foo, test-foo)
    #[arg(long)]
    no_per_crate: bool,

    /// Disable lint targets (lint, fmt)
    #[arg(long)]
    no_lint: bool,

    /// Output file path (relative to workspace)
    #[arg(long, short, default_value = "Makefile")]
    output: String,

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

    let config = MakegenConfig {
        workspace_path: cli.path,
        per_crate_targets: !cli.no_per_crate,
        lint_targets: !cli.no_lint,
        output_path: cli.output,
        force: cli.force,
    };

    let dag = graph::build_makegen_dag(&config, cli.dry_run);

    if cli.svg_tools {
        println!("{}", gunbc_ir::viz::tools_to_svg(&dag));
        return;
    }
    if cli.svg {
        println!("{}", gunbc_ir::viz::dag_to_svg(&dag, cli.svg_show_guards));
        return;
    }

    // Validate
    if let Err(errors) = gunbc_validate::validate(&dag) {
        eprintln!("Validation failed:");
        for e in &errors {
            eprintln!("  - {e}");
        }
        std::process::exit(1);
    }
    eprintln!("DAG validated successfully ({} nodes, {} edges)", dag.nodes.len(), dag.edges.len());

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
