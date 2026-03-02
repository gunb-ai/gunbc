//! Shared helpers used by both Makefile and Justfile renderers.
//!
//! The Makefile renderer evaluates DSL fn bodies compiled from `makegen.dag`.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use daglang_driver::compile_data_from_sources;
use daglang_lower::LoweredFnBody;
use gunbc_ir::cargo::{CargoCommand, Subcommand};
use gunbc_ir::Value;

use crate::makegen::registry::{BuildConfig, ToolRegistry};

const MAKEGEN_SOURCE: &str = include_str!("../../../dsl/tools/makegen.dag");
const EXTDEPS_MAKE_SOURCE: &str = include_str!("../../../dsl/extdeps/make.dag");
const BUILD_TARGETS_SOURCE: &str = include_str!("../../../dsl/config/build_targets.dag");
const STD_PATTERNS_SOURCE: &str = include_str!("../../../dsl/std/patterns.dag");
const STD_RESOURCES_SOURCE: &str = include_str!("../../../dsl/std/resources.dag");
const STD_TYPES_SOURCE: &str = include_str!("../../../dsl/std/types.dag");
const STD_FILESYSTEM_SOURCE: &str = include_str!("../../../dsl/std/filesystem.dag");
const STD_ERRORS_SOURCE: &str = include_str!("../../../dsl/std/errors.dag");
const STD_BEHAVIORAL_SOURCE: &str = include_str!("../../../dsl/std/behavioral.dag");
const SVC_SHELL_SOURCE: &str = include_str!("../../../dsl/services/shell.dag");
const SVC_GCP_SM_SOURCE: &str = include_str!("../../../dsl/services/gcp/secret_manager.dag");
const SVC_GCP_IAM_SOURCE: &str = include_str!("../../../dsl/services/gcp/iam.dag");
const SVC_GCP_STS_SOURCE: &str = include_str!("../../../dsl/services/gcp/sts.dag");
const EXT_GCP_CORE_SOURCE: &str = include_str!("../../../dsl/extdeps/cloud/gcp/core.dag");
const EXT_GCP_SM_SOURCE: &str = include_str!("../../../dsl/extdeps/cloud/gcp/secret_manager.dag");
const EXT_GCP_IAM_SOURCE: &str = include_str!("../../../dsl/extdeps/cloud/gcp/iam.dag");
const EXT_GCP_STS_SOURCE: &str = include_str!("../../../dsl/extdeps/cloud/gcp/sts.dag");
const EXT_CLOUD_CORE_SOURCE: &str = include_str!("../../../dsl/extdeps/cloud/core.dag");
const EXT_PKG_MGRS_SOURCE: &str = include_str!("../../../dsl/extdeps/tools/package_managers.dag");

// ============================================================================
// DSL-based Makefile rendering
// ============================================================================

/// Render a complete Makefile from the tool registry using DSL evaluation.
///
/// Compiles `makegen.dag`, extracts fn bodies and data declarations, then
/// evaluates `render_makefile_content()` to produce the output string.
pub fn render_makefile(registry: &ToolRegistry) -> String {
    let (fns, data_values) = compile_makegen();
    let tools = registry_tools_to_value(registry);

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
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("render_makefile_content should return a string")
        .to_string()
}

/// Compile `makegen.dag` and extract fn bodies + data declaration values.
fn compile_makegen() -> (
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
) {
    let output = compile_data_from_sources(&[
        // Leaf dependencies (no imports)
        (Path::new("<embedded>/std/types.dag"), STD_TYPES_SOURCE),
        (Path::new("<embedded>/std/errors.dag"), STD_ERRORS_SOURCE),
        (
            Path::new("<embedded>/std/behavioral.dag"),
            STD_BEHAVIORAL_SOURCE,
        ),
        (
            Path::new("<embedded>/std/resources.dag"),
            STD_RESOURCES_SOURCE,
        ),
        (
            Path::new("<embedded>/std/filesystem.dag"),
            STD_FILESYSTEM_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/cloud/core.dag"),
            EXT_CLOUD_CORE_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/cloud/gcp/core.dag"),
            EXT_GCP_CORE_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/cloud/gcp/secret_manager.dag"),
            EXT_GCP_SM_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/cloud/gcp/iam.dag"),
            EXT_GCP_IAM_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/cloud/gcp/sts.dag"),
            EXT_GCP_STS_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/tools/package_managers.dag"),
            EXT_PKG_MGRS_SOURCE,
        ),
        (
            Path::new("<embedded>/extdeps/make.dag"),
            EXTDEPS_MAKE_SOURCE,
        ),
        // Service defs
        (Path::new("<embedded>/services/shell.dag"), SVC_SHELL_SOURCE),
        (
            Path::new("<embedded>/services/gcp/secret_manager.dag"),
            SVC_GCP_SM_SOURCE,
        ),
        (
            Path::new("<embedded>/services/gcp/iam.dag"),
            SVC_GCP_IAM_SOURCE,
        ),
        (
            Path::new("<embedded>/services/gcp/sts.dag"),
            SVC_GCP_STS_SOURCE,
        ),
        // Intermediate
        (
            Path::new("<embedded>/std/patterns.dag"),
            STD_PATTERNS_SOURCE,
        ),
        (
            Path::new("<embedded>/config/build_targets.dag"),
            BUILD_TARGETS_SOURCE,
        ),
        // Target
        (Path::new("<embedded>/tools/makegen.dag"), MAKEGEN_SOURCE),
    ])
    .expect("makegen should compile");

    (output.fns, output.data_values)
}

/// Convert a `ToolRegistry`'s tools into `Value::List` matching the DSL
/// `DiscoveredTool` type shape.
fn registry_tools_to_value(registry: &ToolRegistry) -> Value {
    let config = BuildConfig::cargo();
    let tools: Vec<Value> = registry
        .tools
        .iter()
        .map(|tool| {
            let cmd = CargoCommand::new(Subcommand::Run(tool.invocation.clone()))
                .quiet()
                .release()
                .warnings(config.warnings);
            let command = format!("@{}", cmd.to_shell_with_env());
            let dry_run_command = format!("{} -- --dry-run strict", command);

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
