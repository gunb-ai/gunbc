//! Shared helpers used by both Makefile and Justfile renderers.
//!
//! The Makefile renderer evaluates DSL fn bodies compiled from `makegen.dag`.

use std::collections::{BTreeMap, HashMap};

use daglang_driver::compile_data_from_module;
use daglang_lower::LoweredFnBody;
use gunbc_ir::cargo::{CargoCommand, Subcommand};
use gunbc_ir::Value;

use super::model::{
    load_build_targets_data, reserved_target_names, validate_target_namespace_with_data,
    MakegenModelError,
};
use super::registry::{BuildConfig, ToolRegistry};

// ============================================================================
// DSL-based Makefile rendering
// ============================================================================

/// Render a complete Makefile from the tool registry using DSL evaluation.
///
/// Compiles `makegen.dag`, extracts fn bodies and data declarations, then
/// evaluates `render_makefile_content()` to produce the output string.
pub fn render_makefile(registry: &ToolRegistry) -> Result<String, MakegenModelError> {
    // Filter out tools whose short_name collides with a core/meta target.
    // Core targets already have their own recipe in the Makefile; discovered
    // tools that map to the same name should not generate a duplicate target.
    let build_targets = load_build_targets_data()?;
    let reserved = reserved_target_names(&build_targets);
    let filtered = registry.without_reserved(&reserved);
    validate_target_namespace_with_data(&filtered, &build_targets)?;
    let (fns, data_values) = compile_makegen();
    let tools = registry_tools_to_value(&filtered);

    let mut inputs = HashMap::new();
    inputs.insert(
        "workflows".to_string(),
        Value::from(data_values["core_workflows"].clone()),
    );
    inputs.insert(
        "metas".to_string(),
        Value::from(data_values["meta_targets"].clone()),
    );
    inputs.insert("tools".to_string(), tools);
    inputs.insert(
        "res_targets".to_string(),
        Value::from(data_values["resource_targets"].clone()),
    );

    let body = fns
        .get("render_makefile_content")
        .expect("render_makefile_content fn body should exist in makegen.dag");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .expect("render_makefile_content should evaluate");
    Ok(result
        .get("return")
        .and_then(Value::as_str)
        .expect("render_makefile_content should return a string")
        .to_string())
}

/// Compile `tools/makegen.dag` and extract fn bodies + data declaration values.
///
/// Uses filesystem-based import resolution to automatically discover all 19
/// transitive dependencies (std/*, extdeps/*, services/*, config/*).
fn compile_makegen() -> (
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
) {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout for makegen DSL");
    let dsl_root = layout.workspace_root.join("dsl");
    let output = compile_data_from_module(&dsl_root, "tools/makegen.dag")
        .expect("makegen should compile");

    (output.fns, output.data_values)
}

/// Convert a `ToolRegistry`'s tools into `Value::List` matching the DSL
/// `DiscoveredTool` type shape.
pub fn registry_tools_to_value(registry: &ToolRegistry) -> Value {
    let config = BuildConfig::cargo();
    let tools: Vec<Value> = registry
        .tools
        .iter()
        .map(|tool| {
            let cmd = CargoCommand::new(Subcommand::Run(tool.invocation.clone()))
                .quiet()
                .release()
                .warnings(config.warnings);
            let base_command = format!("@{}", cmd.to_shell_with_env());
            // Tools with profiles default to the `*.local` profile for make targets.
            // Profile names are fully-qualified (e.g., "profiles.gist.local"), so we
            // find the one ending in ".local" and pass it as-is to --profile.
            let local_profile = tool.available_profiles.iter().find(|p| p.ends_with(".local"));
            let (command, dry_run_command) = if let Some(profile) = local_profile {
                let cmd = format!("{} -- --profile {}", base_command, profile);
                let dry_cmd = format!("{} -- --profile {} --dry-run strict", base_command, profile);
                (cmd, dry_cmd)
            } else {
                let dry_cmd = format!("{} -- --dry-run strict", base_command);
                (base_command, dry_cmd)
            };

            let deps: Vec<Value> = if tool.needs_generated_cli {
                vec![Value::Str("ensure-codegen".to_string())]
            } else {
                vec![]
            };

            let entrypoints: Vec<Value> = tool
                .entrypoints
                .iter()
                .map(|ep| {
                    let mut map = BTreeMap::new();
                    map.insert("port_name".to_string(), Value::Str(ep.port_name.clone()));
                    map.insert("make_var".to_string(), Value::Str(ep.make_var.clone()));
                    map.insert("cli_flag".to_string(), Value::Str(ep.cli_flag.clone()));
                    map.insert("type_hint".to_string(), Value::Str(ep.type_hint.clone()));
                    map.insert(
                        "default".to_string(),
                        match &ep.default {
                            Some(d) => Value::Str(d.clone()),
                            None => Value::Unit,
                        },
                    );
                    map.insert("repeatable".to_string(), Value::Bool(ep.repeatable));
                    Value::Map(map)
                })
                .collect();

            let extra_targets: Vec<Value> = tool
                .extra_targets
                .iter()
                .map(|extra| {
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
                                .map(|c| Value::Str(c.clone()))
                                .collect(),
                        ),
                    );
                    Value::Map(map)
                })
                .collect();

            let live_secrets: Vec<Value> = tool
                .live_secrets
                .iter()
                .map(|s| Value::Str(s.clone()))
                .collect();

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
            map.insert("command".to_string(), Value::Str(command));
            map.insert("dry_run_command".to_string(), Value::Str(dry_run_command));
            map.insert("deps".to_string(), Value::List(deps));
            map.insert("entrypoints".to_string(), Value::List(entrypoints));
            map.insert("extra_targets".to_string(), Value::List(extra_targets));
            map.insert("live_secrets".to_string(), Value::List(live_secrets));
            Value::Map(map)
        })
        .collect();

    Value::List(tools)
}
