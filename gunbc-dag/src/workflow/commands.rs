//! DSL-backed workflow unit command catalog.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::OnceLock;

use daglang_syntax::{
    ast::{Expr, Item, Literal},
    parser,
};
use gunbc_ir::NodeId;
use gunbc_workflow::UnitCommand;

const WORKFLOW_COMMANDS_SOURCE: &str = include_str!("../../../dsl/config/workflow_commands.dag");

static WORKFLOW_COMMANDS: OnceLock<Result<HashMap<String, BTreeMap<NodeId, UnitCommand>>, String>> =
    OnceLock::new();

fn commands_by_workflow() -> Result<&'static HashMap<String, BTreeMap<NodeId, UnitCommand>>, String>
{
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

fn load_workflow_commands_from_dsl(
) -> Result<HashMap<String, BTreeMap<NodeId, UnitCommand>>, String> {
    let path = Path::new("<embedded>/config/workflow_commands.dag");
    let parsed = parser::parse_with_file_diagnostics(path, WORKFLOW_COMMANDS_SOURCE).map_err(
        |diagnostics| {
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.render())
                .collect::<Vec<_>>()
                .join("\n")
        },
    )?;

    let mut raw = None;
    for item in parsed.items {
        if let Item::DataDef(def) = item.node {
            if def.name == "workflow_commands" {
                raw = Some(def.value);
                break;
            }
        }
    }
    let raw = raw.ok_or_else(|| {
        format!(
            "workflow command catalog '{}' missing `data workflow_commands` declaration",
            path.display()
        )
    })?;
    parse_workflow_commands_expr(&raw)
}

fn parse_workflow_commands_expr(
    expr: &Expr,
) -> Result<HashMap<String, BTreeMap<NodeId, UnitCommand>>, String> {
    let Expr::List(workflows) = expr else {
        return Err("workflow_commands must be a list of records".to_string());
    };
    let mut out = HashMap::new();
    for (idx, workflow_item) in workflows.iter().enumerate() {
        let Expr::Record(_, workflow_fields) = workflow_item else {
            return Err(format!(
                "workflow_commands[{idx}] must be a record, found {:?}",
                workflow_item
            ));
        };
        let workflow = expect_string_field(workflow_fields, "workflow", idx)?;
        let commands_expr = expect_field(workflow_fields, "commands", idx)?;
        let commands = parse_command_map(commands_expr, idx)?;
        if out.insert(workflow.clone(), commands).is_some() {
            return Err(format!(
                "workflow_commands contains duplicate workflow entry '{workflow}'"
            ));
        }
    }
    Ok(out)
}

fn parse_command_map(
    expr: &Expr,
    workflow_idx: usize,
) -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    let Expr::List(command_items) = expr else {
        return Err(format!(
            "workflow_commands[{workflow_idx}].commands must be a list"
        ));
    };
    let mut commands = BTreeMap::new();
    for (cmd_idx, item) in command_items.iter().enumerate() {
        let Expr::Record(_, fields) = item else {
            return Err(format!(
                "workflow_commands[{workflow_idx}].commands[{cmd_idx}] must be a record"
            ));
        };
        let node_id = expect_string_field(fields, "node_id", cmd_idx)?;
        let label = expect_string_field(fields, "label", cmd_idx)?;
        let program = expect_string_field(fields, "program", cmd_idx)?;
        let args = expect_string_list_field(fields, "args", cmd_idx)?;
        commands.insert(
            NodeId::from(node_id),
            UnitCommand::new(label, program, args),
        );
    }
    Ok(commands)
}

fn expect_field<'a>(
    fields: &'a [(String, Expr)],
    name: &str,
    idx: usize,
) -> Result<&'a Expr, String> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("record[{idx}] missing required field '{name}'"))
}

fn expect_string_field(
    fields: &[(String, Expr)],
    name: &str,
    idx: usize,
) -> Result<String, String> {
    match expect_field(fields, name, idx)? {
        Expr::Literal(Literal::String(value)) => Ok(value.clone()),
        other => Err(format!(
            "record[{idx}].{name} must be String, found {:?}",
            other
        )),
    }
}

fn expect_string_list_field(
    fields: &[(String, Expr)],
    name: &str,
    idx: usize,
) -> Result<Vec<String>, String> {
    let Expr::List(items) = expect_field(fields, name, idx)? else {
        return Err(format!("record[{idx}].{name} must be List<String>"));
    };
    let mut values = Vec::with_capacity(items.len());
    for (item_idx, item) in items.iter().enumerate() {
        match item {
            Expr::Literal(Literal::String(value)) => values.push(value.clone()),
            other => {
                return Err(format!(
                    "record[{idx}].{name}[{item_idx}] must be String, found {:?}",
                    other
                ))
            }
        }
    }
    Ok(values)
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
