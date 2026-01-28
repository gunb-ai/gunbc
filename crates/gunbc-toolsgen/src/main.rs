use clap::Parser;
use gunbc_exec::TerminalObserver;
use gunbc_toolsgen::{build_toolsgen_dag, ToolsgenConfig};

#[derive(Parser, Debug)]
#[command(name = "gunbc-toolsgen", about = "Generate repo tooling sources")]
struct Cli {
    /// Path to the workspace root
    #[arg(long, default_value = ".")]
    path: String,

    /// Print what would be written without actually creating the file
    #[arg(long)]
    dry_run: bool,

    /// Force regeneration even if up-to-date
    #[arg(long)]
    force: bool,

    /// Output file path (relative to workspace)
    #[arg(long, short, default_value = "tools/cargo_wrapper.c")]
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

    let config = ToolsgenConfig {
        workspace_path: cli.path,
        output_path: cli.output,
        force: cli.force,
    };

    let dag = build_toolsgen_dag(&config, cli.dry_run);

    if cli.svg_tools {
        println!("{}", gunbc_ir::viz::tools_to_svg(&dag));
        return;
    }
    if cli.svg {
        println!("{}", gunbc_ir::viz::dag_to_svg(&dag, cli.svg_show_guards));
        return;
    }

    let mut observer = TerminalObserver::new("toolsgen");
    match gunbc_exec::execute_with_observer(&dag, Some(&mut observer)) {
        Ok(_log) => {}
        Err(e) => {
            eprintln!("Execution failed: {e}");
            std::process::exit(1);
        }
    }
}
