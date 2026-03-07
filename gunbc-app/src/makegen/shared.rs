//! Shared helpers used by both Makefile and Justfile renderers.
//!
//! These helpers evaluate DSL fn bodies compiled from `makegen.dag` /
//! `justgen.dag`. Imported leaf serializer modules are compiled separately and
//! merged into the sibling-fn map because `evaluate_fn_body()` only sees the fn
//! bodies we explicitly load.

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
    render_dsl_file(
        tools,
        "tools/makegen.dag",
        &["extdeps/make_render.dag"],
        "render_makefile_content",
    )
}

/// Render a complete Justfile from discovered tool data using DSL evaluation.
pub fn render_justfile(tools: &[DiscoveredToolData]) -> Result<String, MakegenModelError> {
    render_dsl_file(
        tools,
        "tools/justgen.dag",
        &["extdeps/justfile_render.dag"],
        "render_justfile_content",
    )
}

fn render_dsl_file(
    tools: &[DiscoveredToolData],
    module_path: &str,
    imported_render_modules: &[&str],
    render_fn: &str,
) -> Result<String, MakegenModelError> {
    let build_targets = load_build_targets_data()?;
    let reserved = reserved_target_names(&build_targets);
    let filtered = filter_reserved_tools(tools, &reserved);
    validate_target_namespace_with_data(&filtered, &build_targets)?;
    let (fns, data_values) = compile_renderer_modules(module_path, imported_render_modules);
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
        .get(render_fn)
        .unwrap_or_else(|| panic!("{render_fn} fn body should exist in {module_path}"));
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .unwrap_or_else(|err| panic!("{render_fn} should evaluate: {err}"));
    Ok(result
        .get("return")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{render_fn} should return a string"))
        .to_string())
}

/// Discover tools from DSL metadata and render the Makefile.
pub fn render_makefile_from_dsl_discovery() -> Result<String, MakegenModelError> {
    let tools =
        discover_makegen_tools().map_err(|details| MakegenModelError::ToolDiscovery { details })?;
    render_makefile(&tools)
}

/// Compile `tools/makegen.dag` and extract fn bodies + data declaration values.
///
/// Uses filesystem-based import resolution to automatically discover transitive
/// dependencies. Imported leaf serializer modules are compiled separately and
/// merged into the fn-body map so direct DSL evaluation can follow those calls.
fn compile_renderer_modules(
    module_path: &str,
    imported_render_modules: &[&str],
) -> (
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
) {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout for renderer DSL");
    let dsl_root = layout.workspace_root.join("dsl");
    let mut fns = HashMap::new();
    let mut data_values = HashMap::new();

    for (idx, module) in std::iter::once(module_path)
        .chain(imported_render_modules.iter().copied())
        .enumerate()
    {
        let output = compile_data_from_module(&dsl_root, module)
            .unwrap_or_else(|_| panic!("{module} should compile"));
        if idx == 0 {
            data_values = output.data_values;
        } else {
            data_values.extend(output.data_values);
        }
        fns.extend(output.fns);
    }

    (fns, data_values)
}
