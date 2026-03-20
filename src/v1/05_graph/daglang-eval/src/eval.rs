//! Pure evaluation engine for lowered expression IR.
//!
//! Thin wrappers around the explicit-stack evaluator (eval_stack.rs) and
//! standalone collection/match operations. All recursive fn-body evaluation
//! is handled by the stack machine.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use gunbc_ir::Value;

use crate::expr::{LoweredFnBody, LoweredMatchArm};
use gunbc_ir::patterns::CollectionKind as CollectionOpKind;

// Re-export pure utilities from eval_core — single source of truth.
pub use crate::eval_core::{
    eval_binop, eval_conditional, eval_get_field, eval_list_construct, eval_literal,
    eval_null_coalesce, eval_record_construct, eval_string_interpolate, eval_unary_op,
    eval_variant_construct, field_access, match_pattern, sort_key, value_to_string, value_truthy,
    values_equal, EvalError,
};

// ── Public API ──────────────────────────────────────────────────────────────

/// Evaluate a lowered fn body with the given inputs.
pub fn evaluate_fn_body(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<HashMap<String, Value>, EvalError> {
    evaluate_fn_body_with_data(body, inputs, sibling_fns, &HashMap::new())
}

/// Like `evaluate_fn_body` but with explicit data declaration bindings.
pub fn evaluate_fn_body_with_data(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, EvalError> {
    crate::eval_stack::evaluate_stack(body, inputs, sibling_fns, data_values)
}

/// Like `evaluate_fn_body_with_data` but returns type-boundary diagnostics.
pub fn evaluate_fn_body_with_diagnostics(
    body: &LoweredFnBody,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<crate::eval_stack::EvalOutcome, EvalError> {
    crate::eval_stack::evaluate_stack_with_diagnostics(body, inputs, sibling_fns, data_values)
}

/// Evaluate a match expression. Used by DAG executor nodes (resolve.rs, interp).
///
/// Delegates to the stack evaluator's direct match implementation.
pub fn eval_match(
    scrutinee: &Value,
    arms: &[LoweredMatchArm],
    env_bindings: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, LoweredFnBody>,
) -> Result<Value, EvalError> {
    crate::eval_stack::eval_match_standalone(scrutinee, arms, env_bindings, sibling_fns)
}

/// Evaluate a collection operation.
pub fn evaluate_collection(
    kind: &CollectionOpKind,
    items: Vec<Value>,
    inputs: &HashMap<String, Value>,
) -> Result<Value, EvalError> {
    match kind {
        CollectionOpKind::Map
        | CollectionOpKind::Filter
        | CollectionOpKind::FlatMap
        | CollectionOpKind::FilterMap
        | CollectionOpKind::Append => Ok(Value::List(Arc::new(items))),
        CollectionOpKind::Sort | CollectionOpKind::SortBy => {
            let mut sorted = items;
            sorted.sort_by_key(sort_key);
            Ok(Value::List(Arc::new(sorted)))
        }
        CollectionOpKind::Dedup => {
            let mut out = Vec::new();
            for item in items {
                if !out.contains(&item) {
                    out.push(item);
                }
            }
            Ok(Value::List(Arc::new(out)))
        }
        CollectionOpKind::Join => {
            let joined = items
                .iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(",");
            Ok(Value::Str(joined))
        }
        CollectionOpKind::Fold | CollectionOpKind::Len | CollectionOpKind::Count => {
            Ok(Value::Int(items.len() as i64))
        }
        CollectionOpKind::Sum => {
            let total: i64 = items
                .iter()
                .map(|v| match v {
                    Value::Int(i) => *i,
                    _ => 0,
                })
                .sum();
            Ok(Value::Int(total))
        }
        CollectionOpKind::Any => Ok(Value::Bool(items.iter().any(value_truthy))),
        CollectionOpKind::All => Ok(Value::Bool(items.iter().all(value_truthy))),
        CollectionOpKind::Contains => {
            let needle = inputs
                .get("needle")
                .or_else(|| inputs.get("item"))
                .or_else(|| inputs.get("contains"));
            let found = needle
                .map(|n| items.iter().any(|v| v == n))
                .unwrap_or(false);
            Ok(Value::Bool(found))
        }
        CollectionOpKind::Split => Ok(Value::List(Arc::new(items))),
        CollectionOpKind::Zip => Ok(Value::List(Arc::new(items))),
        CollectionOpKind::Skip => {
            let n = inputs
                .get("n")
                .and_then(|v| {
                    if let Value::Int(i) = v {
                        Some(*i as usize)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            Ok(Value::List(Arc::new(items.into_iter().skip(n).collect())))
        }
        CollectionOpKind::Enumerate => Ok(Value::List(Arc::new(
            items
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut map = BTreeMap::new();
                    map.insert("first".to_string(), Value::Int(i as i64));
                    map.insert("second".to_string(), v);
                    Value::Map(map)
                })
                .collect(),
        ))),
    }
}

/// Check if a function name is an evaluator-handled intrinsic.
///
/// Delegates to [`gunbc_ir::patterns::is_eval_intrinsic`] — single source
/// of truth (S11).
pub fn is_intrinsic_call(name: &str) -> bool {
    gunbc_ir::patterns::is_eval_intrinsic(name)
}
