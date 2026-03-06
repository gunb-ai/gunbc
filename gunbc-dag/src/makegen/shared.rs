//! Shared helpers used by both Makefile and Justfile renderers.
//!
//! The Makefile renderer evaluates DSL fn bodies compiled from `makegen.dag`.

use std::collections::HashMap;

use daglang_driver::compile_data_from_module;
use daglang_lower::LoweredFnBody;
use gunbc_ir::cargo::Warnings;
use gunbc_ir::Value;

use super::model::{
    load_build_targets_data, reserved_target_names, validate_target_namespace_with_data,
    MakegenModelError,
};
use super::tools::{
    discover_makegen_tools, filter_reserved_tools, tools_to_value, DiscoveredToolData,
};

// ============================================================================
// DSL-based Makefile rendering
// ============================================================================

/// Render a complete Makefile from discovered tool data using DSL evaluation.
///
/// Compiles `makegen.dag`, extracts fn bodies and data declarations, then
/// evaluates `render_makefile_content()` to produce the output string.
pub fn render_makefile(tools: &[DiscoveredToolData]) -> Result<String, MakegenModelError> {
    let build_targets = load_build_targets_data()?;
    let reserved = reserved_target_names(&build_targets);
    let filtered = filter_reserved_tools(tools, &reserved);
    validate_target_namespace_with_data(&filtered, &build_targets)?;
    let (fns, data_values) = compile_makegen();
    let tools = tools_to_value(&filtered, Warnings::Deny);

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

/// Discover tools from DSL metadata and render the Makefile.
pub fn render_makefile_from_dsl_discovery() -> Result<String, MakegenModelError> {
    let tools = discover_makegen_tools().map_err(|details| MakegenModelError::ToolDiscovery {
        details,
    })?;
    render_makefile(&tools)
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
    let output =
        compile_data_from_module(&dsl_root, "tools/makegen.dag").expect("makegen should compile");

    (output.fns, output.data_values)
}
