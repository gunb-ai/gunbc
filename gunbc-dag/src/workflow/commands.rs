//! DSL-backed process-unit-to-command mappings for executable workflows.
//!
//! Report and aggregate nodes intentionally have no command and are treated
//! as no-ops by the executor.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::OnceLock;

use daglang_driver::{compile_from_context, DriverContext};
use gunbc_ir::NodeId;
use serde::Deserialize;

use super::executor::UnitCommand;

#[derive(Debug, Clone, Deserialize)]
struct UnitCommandDef {
    node_id: String,
    label: String,
    program: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkflowCommandSetDef {
    workflow: String,
    aliases: Vec<String>,
    commands: Vec<UnitCommandDef>,
}

#[derive(Debug, Clone)]
struct WorkflowCommandCatalog {
    by_workflow: HashMap<String, BTreeMap<NodeId, UnitCommand>>,
}

static COMMAND_CATALOG: OnceLock<Result<WorkflowCommandCatalog, String>> = OnceLock::new();

fn command_catalog() -> Result<&'static WorkflowCommandCatalog, String> {
    COMMAND_CATALOG
        .get_or_init(load_command_catalog)
        .as_ref()
        .map_err(|error| error.clone())
}

#[allow(clippy::disallowed_methods)]
fn load_command_catalog() -> Result<WorkflowCommandCatalog, String> {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let dag_file = dsl_root.join("config/workflow_commands.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file.clone()),
    };
    let output = compile_from_context(&context)
        .map_err(|error| format!("failed to compile {}: {error}", dag_file.display()))?;

    let sets_value = output
        .data_values
        .get("workflow_command_sets")
        .cloned()
        .ok_or_else(|| "missing data value 'workflow_command_sets'".to_string())?;
    let sets: Vec<WorkflowCommandSetDef> = serde_json::from_value(sets_value)
        .map_err(|error| format!("invalid workflow_command_sets data: {error}"))?;

    let mut by_workflow: HashMap<String, BTreeMap<NodeId, UnitCommand>> = HashMap::new();
    for set in sets {
        let mut commands: BTreeMap<NodeId, UnitCommand> = BTreeMap::new();
        for command in set.commands {
            let node_id = NodeId::from(command.node_id.clone());
            let unit_command = UnitCommand::new(command.label, command.program, command.args);
            if commands.insert(node_id.clone(), unit_command).is_some() {
                return Err(format!(
                    "duplicate command declaration for workflow '{}' node '{}'",
                    set.workflow, node_id.0
                ));
            }
        }

        let mut names = vec![set.workflow.clone(), set.workflow.replace('_', "-")];
        names.extend(set.aliases);

        for alias in names {
            let normalized = alias.replace('_', "-");
            if by_workflow.contains_key(&normalized) {
                return Err(format!(
                    "duplicate workflow command set mapping for alias '{}'",
                    normalized
                ));
            }
            by_workflow.insert(normalized, commands.clone());
        }
    }

    Ok(WorkflowCommandCatalog { by_workflow })
}

/// Build command map for CI workflow units.
pub fn ci_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    workflow_unit_commands("ci").expect("workflow commands should define 'ci'")
}

/// Build command map for test-all workflow units.
pub fn test_all_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    workflow_unit_commands("test-all").expect("workflow commands should define 'test-all'")
}

/// Build command map for a supported workflow name.
pub fn workflow_unit_commands(
    workflow_name: &str,
) -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    let normalized = workflow_name.replace('_', "-");
    let commands = command_catalog()?
        .by_workflow
        .get(&normalized)
        .cloned()
        .ok_or_else(|| {
            format!(
                "workflow '{}' does not support execution mode; use --plan",
                normalized
            )
        })?;
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_commands_cover_all_non_report_units() {
        let commands = ci_unit_commands();
        assert_eq!(commands.len(), 10);
        assert!(commands.contains_key(&NodeId::from("ci.codegen")));
        assert!(commands.contains_key(&NodeId::from("ci.build_compile")));
        assert!(commands.contains_key(&NodeId::from("ci.test_run")));
        assert!(commands.contains_key(&NodeId::from("ci.clippy_run")));
        assert!(!commands.contains_key(&NodeId::from("ci.report")));
    }

    #[test]
    fn test_all_commands_cover_all_non_report_units() {
        let commands = test_all_unit_commands();
        assert_eq!(commands.len(), 6);
        assert!(commands.contains_key(&NodeId::from("test_all.codegen")));
        assert!(commands.contains_key(&NodeId::from("test_all.build_compile")));
        assert!(commands.contains_key(&NodeId::from("test_all.cargo_test_xl")));
        assert!(!commands.contains_key(&NodeId::from("test_all.report")));
    }

    #[test]
    fn ci_verify_uses_codegen_dag_binary() {
        let ci_commands = ci_unit_commands();
        let verify = ci_commands
            .get(&NodeId::from("ci.verify"))
            .expect("ci.verify command");
        assert_eq!(verify.program, "cargo");
        assert!(
            verify.args.contains(&"gunbc-codegen-dag".to_string()),
            "ci.verify must use codegen DAG wrapper binary for --mode support"
        );
    }

    #[test]
    fn workflow_unit_commands_supports_gist_aliases() {
        let gist = workflow_unit_commands("gist").expect("gist");
        let snapshot = workflow_unit_commands("gist-snapshot").expect("snapshot");
        assert_eq!(gist.len(), snapshot.len());
        assert!(gist.contains_key(&NodeId::from("gist.gist_create")));
    }
}
