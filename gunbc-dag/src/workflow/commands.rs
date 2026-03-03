//! DSL-backed workflow unit command catalog.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use daglang_driver::compile_data_from_module;
use gunbc_ir::{NodeId, WorkspaceLayout};
use gunbc_workflow::UnitCommand;
use serde::Deserialize;

#[derive(Deserialize)]
struct WorkflowCommandSet {
    workflow: String,
    commands: Vec<UnitCommandDef>,
}

#[derive(Deserialize)]
struct UnitCommandDef {
    node_id: String,
    label: String,
    program: String,
    args: Vec<String>,
}

type WorkflowCommandMap = HashMap<String, BTreeMap<NodeId, UnitCommand>>;

static WORKFLOW_COMMANDS: OnceLock<Result<WorkflowCommandMap, String>> = OnceLock::new();

fn commands_by_workflow() -> Result<&'static WorkflowCommandMap, String> {
    WORKFLOW_COMMANDS
        .get_or_init(load_workflow_commands_from_dsl)
        .as_ref()
        .map_err(|error| error.clone())
}

/// Build command map for CI workflow units.
pub fn ci_unit_commands() -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    workflow_unit_commands("ci")
}

/// Build command map for test-all workflow units.
pub fn test_all_unit_commands() -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    workflow_unit_commands("test-all")
}

/// Build command map for a supported workflow name.
pub fn workflow_unit_commands(
    workflow_name: &str,
) -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    let Some(variant) = super::catalog::resolve_workflow_variant(workflow_name)? else {
        return Err(format!(
            "workflow '{}' does not support execution mode; use --plan",
            workflow_name
        ));
    };
    commands_by_workflow()?
        .get(&variant.canonical_name)
        .cloned()
        .ok_or_else(|| {
            format!(
                "workflow '{}' does not support execution mode; use --plan",
                workflow_name
            )
        })
}

fn load_workflow_commands_from_dsl() -> Result<WorkflowCommandMap, String> {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|e| format!("workspace layout for workflow commands: {e}"))?;
    let dsl_root = layout.workspace_root.join("dsl");
    let output = compile_data_from_module(&dsl_root, "config/workflow_commands.dag")
        .map_err(|e| format!("config/workflow_commands.dag compilation failed: {e}"))?;
    let value = output
        .data_values
        .get("workflow_commands")
        .ok_or_else(|| {
            "config/workflow_commands.dag must declare `workflow_commands` data".to_string()
        })?;
    let sets: Vec<WorkflowCommandSet> = serde_json::from_value(value.clone())
        .map_err(|e| format!("workflow_commands deserialization failed: {e}"))?;

    let mut out = HashMap::new();
    for set in sets {
        let mut commands = BTreeMap::new();
        for cmd in set.commands {
            commands.insert(
                NodeId::from(cmd.node_id),
                UnitCommand::new(cmd.label, cmd.program, cmd.args),
            );
        }
        if out.insert(set.workflow.clone(), commands).is_some() {
            return Err(format!(
                "workflow_commands contains duplicate workflow entry '{}'",
                set.workflow
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_commands_cover_all_non_report_units() {
        let commands = ci_unit_commands().expect("ci commands");
        // 10 units have commands (report is a no-op).
        assert_eq!(commands.len(), 10);
        assert!(commands.contains_key(&NodeId::from("ci.codegen")));
        assert!(commands.contains_key(&NodeId::from("ci.build_compile")));
        assert!(commands.contains_key(&NodeId::from("ci.test_run")));
        assert!(commands.contains_key(&NodeId::from("ci.clippy_run")));
        assert!(!commands.contains_key(&NodeId::from("ci.report")));
    }

    #[test]
    fn test_all_commands_cover_all_non_report_units() {
        let commands = test_all_unit_commands().expect("test-all commands");
        // 6 units have commands (report is a no-op).
        assert_eq!(commands.len(), 6);
        assert!(commands.contains_key(&NodeId::from("test_all.codegen")));
        assert!(commands.contains_key(&NodeId::from("test_all.build_compile")));
        assert!(commands.contains_key(&NodeId::from("test_all.cargo_test_xl")));
        assert!(!commands.contains_key(&NodeId::from("test_all.report")));
    }

    #[test]
    fn workflow_unit_commands_supports_bootstrap() {
        let commands = workflow_unit_commands("bootstrap").expect("bootstrap commands");
        assert!(commands.contains_key(&NodeId::from("bootstrap.compilation_ensure")));
        assert!(commands.contains_key(&NodeId::from("bootstrap.codegen_ensure")));
        assert!(commands.contains_key(&NodeId::from("bootstrap.upsert_makefile")));
        assert!(commands.contains_key(&NodeId::from("bootstrap.upsert_gitignore")));
    }

    #[test]
    fn workflow_unit_commands_supports_gist_aliases() {
        let gist = workflow_unit_commands("gist").expect("gist");
        let snapshot = workflow_unit_commands("gist-snapshot").expect("snapshot");
        assert_eq!(gist.len(), snapshot.len());
        assert!(gist.contains_key(&NodeId::from("gist.gist_create")));
    }
}
