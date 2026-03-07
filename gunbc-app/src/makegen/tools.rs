//! Tool discovery data used by makegen renderers.
//!
//! This layer adapts generic `ToolDef` discovery directly into the structural
//! data that `makegen.dag` and `justgen.dag` consume. It deliberately avoids a
//! second repo-local registry type.

use std::collections::{BTreeMap, BTreeSet};

use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand, Warnings};
use gunbc_ir::Value;

use gunbc_codegen::cli_gen::CliEntrypoint;
use gunbc_codegen::registry::ToolDef;
use gunbc_codegen::tool_discovery::discover_tool_defs_from_dsl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrypointParamData {
    pub port_name: String,
    pub make_var: String,
    pub cli_flag: String,
    pub type_hint: String,
    pub default: Option<String>,
    pub repeatable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraTargetData {
    pub suffix: String,
    pub description: String,
    pub post_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredToolData {
    pub invocation: CargoInvocation,
    pub short_name: String,
    pub description: String,
    pub entrypoints: Vec<EntrypointParamData>,
    pub extra_targets: Vec<ExtraTargetData>,
}

impl DiscoveredToolData {
    pub fn from_tool_def(def: &ToolDef) -> Option<Self> {
        let invocation = def.invocation.as_ref()?;
        let entrypoints = def
            .entrypoints
            .iter()
            .filter_map(entrypoint_param_from_cli)
            .collect();

        Some(Self {
            invocation: invocation.clone(),
            short_name: def.meta.tool_name.to_string(),
            description: def.meta.description.to_string(),
            entrypoints,
            extra_targets: Vec::new(),
        })
    }

    pub fn binary_name(&self) -> &str {
        &self.invocation.binary
    }

    pub fn dependency_targets(&self) -> Vec<String> {
        vec!["ensure-codegen".to_string()]
    }

    pub fn command(&self, warnings: Warnings, dry_run: bool) -> String {
        let command = CargoCommand::new(Subcommand::Run(self.invocation.clone()))
            .quiet()
            .release()
            .warnings(warnings);
        let base = format!("@{}", command.to_shell_with_env());
        match dry_run {
            false => base,
            true => format!("{base} -- --dry-run strict"),
        }
    }
}

pub fn discover_makegen_tools() -> Result<Vec<DiscoveredToolData>, String> {
    let mut tools: Vec<DiscoveredToolData> = discover_tool_defs_from_dsl()?
        .iter()
        .filter_map(DiscoveredToolData::from_tool_def)
        .collect();
    tools.sort_by(|a, b| a.short_name.cmp(&b.short_name));
    Ok(tools)
}

pub fn filter_reserved_tools(
    tools: &[DiscoveredToolData],
    reserved: &BTreeSet<String>,
) -> Vec<DiscoveredToolData> {
    tools
        .iter()
        .filter(|tool| tool_is_renderable(tool, reserved))
        .cloned()
        .collect()
}

pub fn tools_to_value(tools: &[DiscoveredToolData], warnings: Warnings) -> Value {
    Value::List(
        tools
            .iter()
            .map(|tool| {
                let mut map = BTreeMap::new();
                map.insert(
                    "short_name".to_string(),
                    Value::Str(tool.short_name.clone()),
                );
                map.insert(
                    "description".to_string(),
                    Value::Str(tool.description.clone()),
                );
                map.insert(
                    "binary_name".to_string(),
                    Value::Str(tool.binary_name().to_string()),
                );
                map.insert(
                    "command".to_string(),
                    Value::Str(tool.command(warnings, false)),
                );
                map.insert(
                    "dry_run_command".to_string(),
                    Value::Str(tool.command(warnings, true)),
                );
                map.insert(
                    "deps".to_string(),
                    Value::List(
                        tool.dependency_targets()
                            .into_iter()
                            .map(Value::Str)
                            .collect(),
                    ),
                );
                map.insert(
                    "entrypoints".to_string(),
                    Value::List(
                        tool.entrypoints
                            .iter()
                            .map(entrypoint_param_to_value)
                            .collect(),
                    ),
                );
                map.insert(
                    "extra_targets".to_string(),
                    Value::List(
                        tool.extra_targets
                            .iter()
                            .map(extra_target_to_value)
                            .collect(),
                    ),
                );
                Value::Map(map)
            })
            .collect(),
    )
}

fn entrypoint_param_from_cli(entrypoint: &CliEntrypoint) -> Option<EntrypointParamData> {
    let make_var = entrypoint.make_var.as_ref()?;
    Some(EntrypointParamData {
        port_name: entrypoint.port_name.clone(),
        make_var: make_var.clone(),
        cli_flag: format!("--{}", entrypoint.flag_name()),
        type_hint: entrypoint.type_id.to_string(),
        default: entrypoint.default_value.clone(),
        repeatable: entrypoint.cardinality.allows_many(),
    })
}

fn entrypoint_param_to_value(param: &EntrypointParamData) -> Value {
    let mut map = BTreeMap::new();
    map.insert("port_name".to_string(), Value::Str(param.port_name.clone()));
    map.insert("make_var".to_string(), Value::Str(param.make_var.clone()));
    map.insert("cli_flag".to_string(), Value::Str(param.cli_flag.clone()));
    map.insert("type_hint".to_string(), Value::Str(param.type_hint.clone()));
    map.insert(
        "default".to_string(),
        match &param.default {
            Some(value) => Value::Str(value.clone()),
            None => Value::Unit,
        },
    );
    map.insert("repeatable".to_string(), Value::Bool(param.repeatable));
    Value::Map(map)
}

fn extra_target_to_value(extra: &ExtraTargetData) -> Value {
    let mut map = BTreeMap::new();
    map.insert("suffix".to_string(), Value::Str(extra.suffix.clone()));
    map.insert(
        "description".to_string(),
        Value::Str(extra.description.clone()),
    );
    map.insert(
        "post_commands".to_string(),
        Value::List(
            extra
                .post_commands
                .iter()
                .cloned()
                .map(Value::Str)
                .collect(),
        ),
    );
    Value::Map(map)
}

fn tool_is_renderable(tool: &DiscoveredToolData, reserved: &BTreeSet<String>) -> bool {
    if reserved.contains(&tool.short_name) || reserved.contains(&format!("{}-dry", tool.short_name))
    {
        return false;
    }
    !tool
        .extra_targets
        .iter()
        .any(|extra| reserved.contains(&format!("{}-{}", tool.short_name, extra.suffix)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_makegen_tools_derives_tools_from_dsl() {
        let tools = discover_makegen_tools().expect("tool discovery should succeed");
        assert!(tools.iter().any(|tool| tool.short_name == "deps"));
        assert!(tools.iter().any(|tool| tool.short_name == "makegen"));
        assert!(tools.iter().any(|tool| tool.short_name == "pragma"));
    }

    #[test]
    fn discover_makegen_tools_has_unique_short_names() {
        let tools = discover_makegen_tools().expect("tool discovery should succeed");
        let mut seen = BTreeSet::new();
        for tool in &tools {
            assert!(
                seen.insert(tool.short_name.clone()),
                "duplicate tool short_name in discovered tools: {}",
                tool.short_name
            );
        }
    }
}
