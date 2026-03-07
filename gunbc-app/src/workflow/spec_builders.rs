//! DSL-backed workflow spec builders.

use super::catalog;
use super::catalog::default_process_unit_registry;
use gunbc_workflow::WorkflowSpec;

/// Build a workflow spec by canonical/alias name.
pub fn workflow_spec(name: &str) -> Result<WorkflowSpec, String> {
    let registry = default_process_unit_registry()?;
    catalog::build_workflow_spec(name, &registry)
}

/// Return canonical tool workflow names.
pub fn all_tool_workflow_names() -> Result<Vec<&'static str>, String> {
    catalog::all_tool_workflow_names()
}

/// Build a tool workflow spec by name.
pub fn tool_workflow_spec(name: &str) -> Result<WorkflowSpec, String> {
    let Some(variant) = catalog::resolve_workflow_variant(name)? else {
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

pub fn test_all_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("test-all")
}

pub fn sdlc_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("sdlc")
}

pub fn gist_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("gist")
}

pub fn gist_diff_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("gist-diff")
}

pub fn gist_recent_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("gist-recent")
}

pub fn bootstrap_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("bootstrap")
}

pub fn makegen_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("makegen")
}

pub fn pragma_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("pragma")
}

pub fn deps_workflow_spec() -> Result<WorkflowSpec, String> {
    workflow_spec("deps")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tool_workflows_build_successfully() {
        let names = all_tool_workflow_names().expect("derive tool workflow names");
        for name in names {
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
    fn sdlc_is_not_a_tool_workflow() {
        assert!(tool_workflow_spec("sdlc").is_err());
    }

    #[test]
    fn sdlc_workflow_builds_successfully() {
        let spec = sdlc_workflow_spec().expect("sdlc workflow spec");
        assert!(spec.dag.nodes.iter().any(|node| node.id.0 == "sdlc.worker"));
        assert!(spec.dag.nodes.iter().any(|node| node.id.0 == "sdlc.report"));
    }

    #[test]
    fn workflow_specs_are_deterministic() {
        let a = workflow_spec("gist").expect("a");
        let b = workflow_spec("gist").expect("b");
        assert_eq!(a.dag.to_ascii("gist"), b.dag.to_ascii("gist"));
    }
}
