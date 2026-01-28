mod ops;
mod graph;

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
}

fn main() {
    let cli = Cli::parse();
    let dag = graph::build_gistgen_dag(&cli.path, &cli.glob, cli.dry_run);

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
