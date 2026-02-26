//! DSL-evaluated pragma rendering.
//!
//! Compiles `config/clippy_policy.dag`, extracts fn bodies and data
//! declarations, then evaluates `derive_*` functions to produce pragma
//! output strings. Follows the proven pattern from `makegen/shared.rs`.

use std::collections::HashMap;
use std::path::PathBuf;

use daglang_driver::{compile_from_context, DriverContext};
use daglang_lower::{CallableKind, LoweredFnBody, LoweredOp};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Value;

/// Compile `config/clippy_policy.dag` and extract fn bodies + data values.
fn compile_clippy_policy() -> (HashMap<String, LoweredFnBody>, HashMap<String, serde_json::Value>)
{
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let dag_file = dsl_root.join("config/clippy_policy.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file),
    };
    let output = compile_from_context(&context).expect("clippy_policy.dag should compile");

    let mut fns = HashMap::new();
    for node in &output.lowered_dag.nodes {
        if let NodeBody::Opaque(LoweredOp::Callable {
            kind: CallableKind::Fn,
            name,
            fn_body: Some(body),
            ..
        }) = &node.body
        {
            fns.insert(name.clone(), *body.clone());
        }
    }

    (fns, output.data_values)
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
