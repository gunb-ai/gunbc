//! DSL-evaluated pragma rendering.
//!
//! Compiles `config/clippy_policy.dag`, extracts fn bodies and data
//! declarations, then evaluates `derive_*` functions to produce pragma
//! output strings.

use std::collections::HashMap;
use std::path::Path;

use daglang_driver::compile_data_from_sources;
use daglang_lower::LoweredFnBody;
use gunbc_ir::Value;

const CLIPPY_POLICY_SOURCE: &str = include_str!("../../../dsl/config/clippy_policy.dag");
const EXTDEPS_CLIPPY_SOURCE: &str = include_str!("../../../dsl/extdeps/clippy.dag");
const ARCH_RULES_SOURCE: &str = include_str!("../../../dsl/config/arch_rules.dag");
const CLIPPY_DISALLOWED_SOURCE: &str = include_str!("../../../dsl/config/clippy_disallowed.dag");
const STD_LINT_SOURCE: &str = include_str!("../../../dsl/std/lint.dag");

/// Compile `config/clippy_policy.dag` and extract fn bodies + data values.
fn compile_clippy_policy() -> (
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
) {
    let output = compile_data_from_sources(&[
        (Path::new("<embedded>/std/lint.dag"), STD_LINT_SOURCE),
        (
            Path::new("<embedded>/extdeps/clippy.dag"),
            EXTDEPS_CLIPPY_SOURCE,
        ),
        (
            Path::new("<embedded>/config/arch_rules.dag"),
            ARCH_RULES_SOURCE,
        ),
        (
            Path::new("<embedded>/config/clippy_disallowed.dag"),
            CLIPPY_DISALLOWED_SOURCE,
        ),
        (
            Path::new("<embedded>/config/clippy_policy.dag"),
            CLIPPY_POLICY_SOURCE,
        ),
    ])
    .expect("clippy_policy.dag should compile");

    (output.fns, output.data_values)
}

/// Evaluate `derive_disallowed_methods_allowlist()` from DSL.
pub fn render_allowlist_via_dsl() -> String {
    let (fns, data_values) = compile_clippy_policy();

    let mut inputs = HashMap::new();
    inputs.insert(
        "patterns".to_string(),
        Value::from(data_values["allowlist_patterns"].clone()),
    );

    let body = fns
        .get("derive_disallowed_methods_allowlist")
        .expect("derive_disallowed_methods_allowlist fn body should exist");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .expect("derive_disallowed_methods_allowlist should evaluate");
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("should return a string")
        .to_string()
}

/// Evaluate `derive_pragma_lint_policy()` from DSL.
pub fn render_lint_policy_via_dsl() -> String {
    let (fns, data_values) = compile_clippy_policy();

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
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .expect("derive_pragma_lint_policy should evaluate");
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("should return a string")
        .to_string()
}

/// Evaluate `derive_clippy_toml()` from DSL.
pub fn render_clippy_toml_via_dsl() -> String {
    let (fns, data_values) = compile_clippy_policy();

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
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .expect("derive_clippy_toml should evaluate");
    result
        .get("return")
        .and_then(Value::as_str)
        .expect("should return a string")
        .to_string()
}
