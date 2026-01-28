use clap::Parser;
use gunbc_exec::TerminalObserver;
use gunbc_makegen::{build_makegen_dag, MakegenConfig};

#[derive(Parser, Debug)]
#[command(name = "gunbc-makegen", about = "Generate a Makefile with gist target")]
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
        output_path: cli.output,
        force: cli.force,
    };

    let dag = build_makegen_dag(&config, cli.dry_run);

    if cli.svg_tools {
        println!("{}", gunbc_ir::viz::tools_to_svg(&dag));
        return;
    }
    if cli.svg {
        println!("{}", gunbc_ir::viz::dag_to_svg(&dag, cli.svg_show_guards));
        return;
    }

    // Execute with progress observer
    let mut observer = TerminalObserver::new("makegen");
    match gunbc_exec::execute_with_observer(&dag, Some(&mut observer)) {
        Ok(_log) => {}
        Err(e) => {
            eprintln!("Execution failed: {e}");
            std::process::exit(1);
        }
    }
}
