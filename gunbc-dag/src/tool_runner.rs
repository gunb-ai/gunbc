//! Shared binary entry helpers for DAG tool runners.

use gunbc_exec::{
    compose_with_freshness, execute_and_display, BoundaryMocks, Executable, ExecutionMode,
};
use gunbc_ir::Dag;
use std::io::IsTerminal;

/// Run configuration for shared tool execution ceremony.
#[derive(Debug, Clone, Default)]
pub struct RunToolOptions<'a> {
    pub success_port: Option<&'a str>,
    pub input_mocks: Option<&'a BoundaryMocks>,
    pub with_freshness: bool,
}

/// Print a standard tool banner and key-value metadata lines.
pub fn print_tool_header(tool: &str, metadata: &[(&str, String)]) {
    println!("{tool}");
    for (key, value) in metadata {
        println!("  {key}: {value}");
    }
    println!();
}

/// Execute a DAG using shared display/freshness ceremony.
pub fn run_tool<T: Executable + Clone + Send + 'static>(
    dag: Dag<T>,
    mode: ExecutionMode,
    options: RunToolOptions<'_>,
) {
    let animated = std::io::stdout().is_terminal();
    if options.with_freshness {
        let steps = gunbc_lib_transport::check_and_plan_freshness();
        let dag_with_freshness = compose_with_freshness(dag, steps);
        execute_and_display(
            &dag_with_freshness,
            mode,
            animated,
            options.success_port,
            options.input_mocks,
        );
    } else {
        execute_and_display(
            &dag,
            mode,
            animated,
            options.success_port,
            options.input_mocks,
        );
    }
}
