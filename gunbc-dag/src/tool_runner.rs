//! Shared binary entry helpers for DAG tool runners.

use gunbc_exec::{
    compose_with_freshness, execute_and_display, print_attention, AttentionLevel, BoundaryMocks,
    run_lowering_preflight, run_small_preflight, Executable, ExecutionMode, FreshnessStep,
};
use gunbc_ir::Dag;
use std::io::IsTerminal;
use std::process;

/// Run configuration for shared tool execution ceremony.
#[derive(Debug, Clone)]
pub struct RunToolOptions<'a> {
    pub success_port: Option<&'a str>,
    pub input_mocks: Option<&'a BoundaryMocks>,
    pub with_freshness: bool,
    pub small_preflight: bool,
}

impl Default for RunToolOptions<'_> {
    fn default() -> Self {
        Self {
            success_port: None,
            input_mocks: None,
            with_freshness: false,
            small_preflight: true,
        }
    }
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
    let run_small_tests = options.small_preflight && matches!(mode, ExecutionMode::Real);
    if options.with_freshness {
        if run_small_tests {
            if let Err(error) = run_small_preflight(&dag, options.input_mocks) {
                print_attention(
                    AttentionLevel::Error,
                    "Small-test preflight failed",
                    &error.to_string(),
                );
                process::exit(1);
            }
        }
        let steps = gunbc_lib_transport::check_and_plan_freshness();
        let should_update_manifest = steps.as_ref().is_some_and(|s| !s.is_empty());
        let dag_with_freshness = compose_with_freshness(dag, steps);
        if run_small_tests {
            if let Err(error) = run_lowering_preflight(&dag_with_freshness) {
                print_attention(
                    AttentionLevel::Error,
                    "Small-test preflight failed",
                    &error.to_string(),
                );
                process::exit(1);
            }
        }
        execute_and_display(
            &dag_with_freshness,
            mode,
            animated,
            options.success_port,
            options.input_mocks,
        );
        update_freshness_manifest_if_needed(should_update_manifest);
    } else {
        if run_small_tests {
            if let Err(error) = run_small_preflight(&dag, options.input_mocks) {
                print_attention(
                    AttentionLevel::Error,
                    "Small-test preflight failed",
                    &error.to_string(),
                );
                process::exit(1);
            }
        }
        execute_and_display(
            &dag,
            mode,
            animated,
            options.success_port,
            options.input_mocks,
        );
    }
}

/// Persist freshness state after successful execution when freshness steps ran.
pub fn update_freshness_manifest_if_needed(ran_freshness_steps: bool) {
    if !ran_freshness_steps {
        return;
    }
    if let Err(error) = gunbc_lib_transport::update_freshness_manifest() {
        print_attention(
            AttentionLevel::Warning,
            "Freshness state not persisted",
            &error,
        );
    }
}

/// Helper for callers that already hold optional planned freshness steps.
pub fn freshness_steps_planned(steps: Option<&[FreshnessStep]>) -> bool {
    steps.is_some_and(|planned| !planned.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freshness_steps_planned_handles_none_and_empty() {
        assert!(!freshness_steps_planned(None));
        assert!(!freshness_steps_planned(Some(&[])));
    }

    #[test]
    fn freshness_steps_planned_detects_non_empty() {
        let steps = vec![FreshnessStep {
            id: "codegen-dag".to_string(),
            command: vec!["echo".to_string(), "ok".to_string()],
        }];
        assert!(freshness_steps_planned(Some(&steps)));
    }
}
