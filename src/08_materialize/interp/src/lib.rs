//! **Stage 10 — Interpret**: Transforms a `LoweredOp` + inputs into
//! `HashMap<String, Value>` node outputs via direct dispatch.
//!
//! # Pipeline position
//!
//! - **Before**: a `VerifiedDag<LoweredOp>` is available from compilation
//! - **After**: caller consumes per-node output values
//!
//! # Sequential steps
//!
//! 1. Match on the `LoweredOp` variant
//! 2. Dispatch `Primitive` ops to `daglang_eval` pure evaluators
//! 3. Dispatch `Callable` ops with fn bodies to `evaluate_fn_body`
//! 4. Dispatch `Collection` ops to `evaluate_collection`
//! 5. Delegate transport ops to the `gunbc_lib_transport` layer
//!
//! # Purity
//!
//! Pure dispatch — no I/O. Transport ops are forwarded to the transport
//! layer without performing I/O in this crate.
//!
//! # Failure
//!
//! Returns `ExecError` when evaluation or dispatch fails.

use std::collections::HashMap;

use daglang_eval::eval;
use daglang_lower::{LoweredOp, PrimitiveOpKind};
use gunbc_exec::ExecError;
use gunbc_ir::Value;

/// Execute a single `LoweredOp` node given its inputs.
///
/// This is the core dispatch: each `LoweredOp` variant maps to an evaluator
/// function or transport call. Returns the node's output values.
pub fn execute_lowered_op(
    op: &LoweredOp,
    inputs: HashMap<String, Value>,
    sibling_fns: &HashMap<String, daglang_eval::LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    match op {
        LoweredOp::Primitive { kind, .. } => execute_primitive(kind, inputs),
        LoweredOp::Callable { fn_body, .. } => {
            execute_callable(fn_body.as_deref(), &inputs, sibling_fns, data_values)
        }
        LoweredOp::Transport { .. } => {
            execute_callable(None, &inputs, sibling_fns, data_values)
        }
        LoweredOp::Collection { kind, .. } => {
            execute_collection(kind, inputs)
        }
        LoweredOp::Pattern(pattern_op) => {
            // Patterns are self-executing via the Executable trait.
            use gunbc_exec::Executable;
            pattern_op.execute(inputs)
        }
        LoweredOp::Pipeline { .. } => {
            // Pipeline nodes are compile-time metadata; no runtime execution.
            Ok(HashMap::new())
        }
        LoweredOp::UnsupportedPattern { name } => {
            Err(ExecError::new(format!("unsupported pattern: {name}")))
        }
    }
}

fn execute_primitive(
    kind: &PrimitiveOpKind,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    match kind {
        PrimitiveOpKind::GetField { field } => {
            let value = inputs.values().next().cloned().unwrap_or(Value::Skipped);
            let result = eval::eval_get_field(&value, field)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::StringInterpolate { parts, input_ports } => {
            let values: Vec<Value> = input_ports
                .iter()
                .map(|port| inputs.get(port).cloned().unwrap_or(Value::Skipped))
                .collect();
            let result = eval::eval_string_interpolate(parts, &values)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::BinaryOp { op } => {
            let left = inputs.get("left").cloned().unwrap_or(Value::Skipped);
            let right = inputs.get("right").cloned().unwrap_or(Value::Skipped);
            let result = eval::eval_binop(&left, *op, &right)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::UnaryOp { op } => {
            let val = inputs.get("operand").cloned().unwrap_or(Value::Skipped);
            let result = eval::eval_unary_op(*op, &val)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::Conditional => {
            let condition = inputs.get("condition").cloned().unwrap_or(Value::Skipped);
            let then_val = inputs.get("then").cloned().unwrap_or(Value::Skipped);
            let else_val = inputs.get("else");
            let result = eval::eval_conditional(&condition, &then_val, else_val);
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::RecordConstruct { fields } => {
            let field_values: Vec<(String, Value)> = fields
                .iter()
                .map(|f| (f.clone(), inputs.get(f).cloned().unwrap_or(Value::Skipped)))
                .collect();
            let result = eval::eval_record_construct(&field_values)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::NullCoalesce => {
            let value = inputs.get("value").cloned().unwrap_or(Value::Skipped);
            let default = inputs.get("default").cloned().unwrap_or(Value::Unit);
            let result = eval::eval_null_coalesce(&value, &default);
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::VariantConstruct { tag, fields } => {
            let field_values: Vec<(String, Value)> = fields
                .iter()
                .map(|f| (f.clone(), inputs.get(f).cloned().unwrap_or(Value::Skipped)))
                .collect();
            let result = eval::eval_variant_construct(tag, &field_values)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::ListConstruct { count } => {
            let elements: Vec<Value> = (0..*count)
                .map(|i| inputs.get(&format!("elem_{i}")).cloned().unwrap_or(Value::Skipped))
                .collect();
            let result = eval::eval_list_construct(elements)
                .map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::MatchDispatch { arms, sibling_fns } => {
            let scrutinee = inputs.get("scrutinee").cloned().unwrap_or(Value::Skipped);
            if matches!(scrutinee, Value::Skipped) {
                return Ok([("value".to_string(), Value::Skipped)].into_iter().collect());
            }
            let env: HashMap<String, Value> = inputs
                .iter()
                .filter(|(k, _)| k.as_str() != "scrutinee")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let sibling_fns_map: HashMap<String, daglang_eval::LoweredFnBody> =
                sibling_fns.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let result = eval::eval_match(&scrutinee, arms, &env, &sibling_fns_map)
                .map_err(|e| ExecError::new(format!("MatchDispatch: {e}")))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        // I/O and transport primitives are handled by the transport layer,
        // not by the pure evaluator. These pass through for now.
        _ => Ok(inputs),
    }
}

fn execute_callable(
    fn_body: Option<&daglang_eval::LoweredFnBody>,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, daglang_eval::LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let Some(body) = fn_body else {
        // No fn body — passthrough. Map __out: prefixed inputs to outputs,
        // filtering internal ports (__deps, _freshness).
        let mut outputs = HashMap::new();
        for (key, value) in inputs {
            if let Some(output_name) = key.strip_prefix("__out:") {
                outputs.insert(output_name.to_string(), value.clone());
            } else if key != "__deps" && key != "_freshness" && !key.starts_with("res:") {
                outputs.insert(key.clone(), value.clone());
            }
        }
        return Ok(outputs);
    };

    let eval_inputs: HashMap<String, Value> = inputs
        .iter()
        .filter(|(k, _)| !k.starts_with("__out:") && *k != "__deps" && *k != "_freshness")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    match eval::evaluate_fn_body_with_data(body, &eval_inputs, sibling_fns, data_values) {
        Ok(results) => Ok(results),
        Err(eval_err) => {
            let has_real_inputs = eval_inputs.values().any(|v| !matches!(v, Value::Skipped));
            if has_real_inputs {
                Err(ExecError::new(format!(
                    "FnBody evaluation failed with real inputs: {eval_err}"
                )))
            } else {
                Ok(HashMap::new())
            }
        }
    }
}

fn execute_collection(
    kind: &gunbc_ir::patterns::CollectionKind,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let items = inputs
        .get("items")
        .and_then(|v| match v {
            Value::List(items) => Some(items.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let result = eval::evaluate_collection(kind, items, &inputs)
        .map_err(|e| ExecError::new(e.to_string()))?;
    Ok([("value".to_string(), result)].into_iter().collect())
}
