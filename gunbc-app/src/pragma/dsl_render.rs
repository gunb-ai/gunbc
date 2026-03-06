//! DSL-evaluated pragma rendering.
//!
//! Compiles `config/clippy_policy.dag`, extracts fn bodies and data
//! declarations, then evaluates `derive_*` functions to produce pragma
//! output strings.
//!
//! The compilation result is cached via `OnceLock` (CP-67) since all three
//! render functions compile the same module.

use std::collections::HashMap;
use std::sync::OnceLock;

use daglang_driver::compile_data_from_module;
use daglang_lower::LoweredFnBody;
use gunbc_ir::{Value, WorkspaceLayout};

/// Cached compilation of `config/clippy_policy.dag`.
///
/// All three render functions need the same fn bodies and data values.
/// Compiling once and caching avoids 3× redundant DSL compilation (CP-67).
static CLIPPY_POLICY_CACHE: OnceLock<(
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
)> = OnceLock::new();

fn cached_clippy_policy() -> &'static (
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
) {
    CLIPPY_POLICY_CACHE.get_or_init(|| {
        let layout = WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout for clippy policy DSL");
        let dsl_root = layout.workspace_root.join("dsl");
        let output = compile_data_from_module(&dsl_root, "config/clippy_policy.dag")
            .expect("clippy_policy.dag should compile");
        (output.fns, output.data_values)
    })
}

/// Evaluate `derive_disallowed_methods_allowlist()` from DSL.
pub fn render_allowlist_via_dsl() -> String {
    let (fns, data_values) = cached_clippy_policy();

    let mut inputs = HashMap::new();
    inputs.insert(
        "patterns".to_string(),
        Value::from(data_values["allowlist_patterns"].clone()),
    );

    let body = fns
        .get("derive_disallowed_methods_allowlist")
        .expect("derive_disallowed_methods_allowlist fn body should exist");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, fns)
        .expect("derive_disallowed_methods_allowlist should evaluate");
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("should return a string")
        .to_string()
}

/// Evaluate `derive_pragma_lint_policy()` from DSL.
pub fn render_lint_policy_via_dsl() -> String {
    let (fns, data_values) = cached_clippy_policy();

    let mut inputs = HashMap::new();
    inputs.insert(
        "dead_code".to_string(),
        Value::from(data_values["dead_code_allowances"].clone()),
    );
    inputs.insert(
        "allow_lints".to_string(),
        Value::from(data_values["pragma_allow_lints"].clone()),
    );

    let body = fns
        .get("derive_pragma_lint_policy")
        .expect("derive_pragma_lint_policy fn body should exist");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, fns)
        .expect("derive_pragma_lint_policy should evaluate");
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("should return a string")
        .to_string()
}

/// Evaluate `derive_clippy_toml()` from DSL.
pub fn render_clippy_toml_via_dsl() -> String {
    let (fns, data_values) = cached_clippy_policy();

    let mut inputs = HashMap::new();
    inputs.insert(
        "threshold".to_string(),
        Value::from(data_values["large_error_threshold"].clone()),
    );
    inputs.insert(
        "crate_exemptions".to_string(),
        Value::from(data_values["exemptions"].clone()),
    );
    inputs.insert(
        "method_groups".to_string(),
        Value::from(data_values["disallowed_method_groups"].clone()),
    );
    inputs.insert(
        "type_groups".to_string(),
        Value::from(data_values["disallowed_type_groups"].clone()),
    );

    let body = fns
        .get("derive_clippy_toml")
        .expect("derive_clippy_toml fn body should exist");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, fns)
        .expect("derive_clippy_toml should evaluate");
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("should return a string")
        .to_string()
}
