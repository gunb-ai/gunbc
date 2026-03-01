//! Shared binary entry helpers for DAG tool runners.

use gunbc_exec::{
    compose_with_freshness, execute_and_display, print_attention, AttentionLevel, BoundaryMocks,
    Executable, ExecutionMode, FreshnessStep,
};
use gunbc_ir::Dag;
use std::io::IsTerminal;

/// Controls which freshness steps are injected before tool execution.
///
/// Tools that already perform build/clippy/test should use `GenerationOnly`
/// to avoid redundant work. The overlap is enforced at the composition level:
/// `compose_with_freshness()` returns an error if freshness steps would
/// duplicate cargo operations already present in the tool DAG.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FreshnessScope {
    /// No freshness steps.
    #[default]
    None,
    /// Full freshness chain: generation + build verification.
    /// Use for tools that don't perform their own build/clippy/test.
    Full,
    /// Generation-only: codegen, codegen-dag, testgen, pragma.
    /// Use for tools that already run build/clippy/test (e.g., CI binary).
    GenerationOnly,
}

/// Run configuration for shared tool execution ceremony.
#[derive(Debug, Clone, Default)]
pub struct RunToolOptions<'a> {
    pub success_port: Option<&'a str>,
    pub input_mocks: Option<&'a BoundaryMocks>,
    pub freshness: FreshnessScope,
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
    let steps = match options.freshness {
        FreshnessScope::None => None,
        FreshnessScope::Full => gunbc_lib_transport::check_and_plan_freshness(),
        FreshnessScope::GenerationOnly => {
            gunbc_lib_transport::check_and_plan_generation_freshness()
        }
    };
    let dag_with_freshness = match compose_with_freshness(dag, steps) {
        Ok(composed) => composed,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "Freshness composition failed",
                &e.to_string(),
            );
            std::process::exit(1);
        }
    };
    let ran_freshness = dag_with_freshness
        .nodes
        .iter()
        .any(|n| n.id.0 == "freshness");
    execute_and_display(
        &dag_with_freshness,
        mode,
        animated,
        options.success_port,
        options.input_mocks,
    );
    update_freshness_manifest_if_needed(ran_freshness);
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
