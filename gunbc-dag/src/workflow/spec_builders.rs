//! DSL-backed workflow spec builders.

use super::catalog;
use super::process_registry::{default_process_unit_registry, ProcessUnitRegistry};
use super::schema::WorkflowSpec;

/// Build a workflow spec by canonical/alias name.
pub fn workflow_spec(name: &str) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry(name, &default_process_unit_registry())
}

/// Build a workflow spec against an explicit process registry.
pub fn workflow_spec_with_registry(
    name: &str,
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    catalog::build_workflow_spec(name, registry)
}

/// Return canonical tool workflow names.
pub fn all_tool_workflow_names() -> Vec<&'static str> {
    catalog::all_tool_workflow_names()
}

/// Build a tool workflow spec by name.
pub fn tool_workflow_spec(name: &str) -> Result<WorkflowSpec, String> {
    let Some(variant) = catalog::resolve_workflow_variant(name) else {
        return Err(format!("unknown tool workflow: '{name}'"));
    };
    if !variant.is_tool {
        return Err(format!("unknown tool workflow: '{name}'"));
    }
    workflow_spec(name)
}

pub fn ci_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("ci")
}

pub fn ci_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("ci", registry)
}

pub fn test_all_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("test-all")
}

pub fn test_all_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("test-all", registry)
}

pub fn gist_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("gist")
}

pub fn gist_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("gist", registry)
}

pub fn gist_diff_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("gist-diff")
}

pub fn gist_diff_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("gist-diff", registry)
}

pub fn gist_recent_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("gist-recent")
}

pub fn gist_recent_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("gist-recent", registry)
}

pub fn bootstrap_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("bootstrap")
}

pub fn bootstrap_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("bootstrap", registry)
}

pub fn makegen_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("makegen")
}

pub fn makegen_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("makegen", registry)
}

pub fn pragma_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("pragma")
}

pub fn pragma_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("pragma", registry)
}

pub fn deps_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("deps")
}

pub fn deps_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("deps", registry)
}

pub fn build_all_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("build-all")
}

pub fn build_all_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("build-all", registry)
}

pub fn sdlc_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("sdlc")
}

pub fn sdlc_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    workflow_spec_with_registry("sdlc", registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tool_workflows_build_successfully() {
        for name in all_tool_workflow_names() {
            tool_workflow_spec(name)
                .unwrap_or_else(|error| panic!("tool workflow '{name}' failed to build: {error}"));
        }
    }

    #[test]
    fn tool_workflow_spec_rejects_unknown_name() {
        assert!(tool_workflow_spec("nonexistent").is_err());
    }

    #[test]
    fn ci_and_test_all_are_not_tool_workflows() {
        assert!(tool_workflow_spec("ci").is_err());
        assert!(tool_workflow_spec("test-all").is_err());
    }

    #[test]
    fn workflow_specs_are_deterministic() {
        let a = workflow_spec("gist").expect("a");
        let b = workflow_spec("gist").expect("b");
        assert_eq!(
            a.dag.to_ascii("gist"),
            b.dag.to_ascii("gist")
        );
    }
}
