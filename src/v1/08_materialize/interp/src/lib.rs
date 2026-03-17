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
//! 5. Reject runtime metadata/transport nodes that should have been
//!    intercepted earlier in the pipeline
//!
//! # Purity
//!
//! Pure dispatch — no I/O. Transport and pipeline nodes are not executed
//! here; reaching the interpreter is a structural runtime error.
//!
//! # Failure
//!
//! Returns `ExecError` when evaluation fails or an invalid lowered node
//! reaches the interpreter at runtime.

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
        LoweredOp::Transport { .. } => Err(ExecError::new(
            "transport node reached interpreter; transport execution must happen before runtime interpretation",
        )),
        LoweredOp::Collection { kind, .. } => {
            execute_collection(kind, inputs)
        }
        LoweredOp::Pattern(pattern_op) => {
            // Patterns are self-executing via the Executable trait.
            use gunbc_exec::Executable;
            pattern_op.execute(inputs)
        }
        LoweredOp::Pipeline { .. } => Err(ExecError::new(
            "pipeline node reached interpreter; pipeline metadata must not execute at runtime",
        )),
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
        PrimitiveOpKind::GetField { field, input_port } => {
            let value = inputs.get(input_port).cloned().ok_or_else(|| {
                ExecError::new(format!(
                    "GetField `{field}`: missing input port `{input_port}`"
                ))
            })?;
            let result =
                eval::eval_get_field(&value, field).map_err(|e| ExecError::new(e.to_string()))?;
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
            let result =
                eval::eval_binop(&left, *op, &right).map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::UnaryOp { op } => {
            let val = inputs.get("operand").cloned().unwrap_or(Value::Skipped);
            let result =
                eval::eval_unary_op(*op, &val).map_err(|e| ExecError::new(e.to_string()))?;
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
                .map(|i| {
                    inputs
                        .get(&format!("elem_{i}"))
                        .cloned()
                        .unwrap_or(Value::Skipped)
                })
                .collect();
            let result =
                eval::eval_list_construct(elements).map_err(|e| ExecError::new(e.to_string()))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        PrimitiveOpKind::MatchDispatch { arms, sibling_fns } => {
            let scrutinee = inputs.get("scrutinee").cloned().unwrap_or(Value::Skipped);
            if matches!(scrutinee, Value::Skipped) {
                return Ok([("value".to_string(), Value::Skipped)]
                    .into_iter()
                    .collect());
            }
            let env: HashMap<String, Value> = inputs
                .iter()
                .filter(|(k, _)| k.as_str() != "scrutinee")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let sibling_fns_map: HashMap<String, daglang_eval::LoweredFnBody> = sibling_fns
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let result = eval::eval_match(&scrutinee, arms, &env, &sibling_fns_map)
                .map_err(|e| ExecError::new(format!("MatchDispatch: {e}")))?;
            Ok([("value".to_string(), result)].into_iter().collect())
        }
        _ => Err(ExecError::new(format!(
            "primitive op {kind:?} is not supported by the interpreter"
        ))),
    }
}

fn execute_callable(
    fn_body: Option<&daglang_eval::LoweredFnBody>,
    inputs: &HashMap<String, Value>,
    sibling_fns: &HashMap<String, daglang_eval::LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let Some(body) = fn_body else {
        return Err(ExecError::new(
            "callable node reached interpreter without a fn_body",
        ));
    };

    let eval_inputs: HashMap<String, Value> = inputs
        .iter()
        .filter(|(k, _)| !k.starts_with("__out:") && *k != "__deps" && *k != "_freshness")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    match eval::evaluate_fn_body_with_data(body, &eval_inputs, sibling_fns, data_values) {
        Ok(results) => Ok(results),
        Err(eval_err) => Err(ExecError::new(format!(
            "FnBody evaluation failed: {eval_err}"
        ))),
    }
}

fn execute_collection(
    kind: &gunbc_ir::patterns::CollectionKind,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let items_value = inputs
        .get("items")
        .ok_or_else(|| ExecError::new("collection operation missing `items` input"))?;
    let items = match items_value {
        Value::List(items) => items.clone(),
        other => {
            return Err(ExecError::new(format!(
                "collection `items` input must be List, got {}",
                other.kind()
            )))
        }
    };
    let result = eval::evaluate_collection(kind, items, &inputs)
        .map_err(|e| ExecError::new(e.to_string()))?;
    Ok([("value".to_string(), result)].into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_eval::{LoweredExpr, LoweredFnBody, LoweredStmt};
    use daglang_lower::{
        CallableKind, CallableObligation, ServiceCallMetadata, ServiceTransportClass,
        TransportObligation,
    };
    use gunbc_ir::patterns::CollectionKind;

    fn empty_sibling_fns() -> HashMap<String, daglang_eval::LoweredFnBody> {
        HashMap::new()
    }

    fn empty_data_values() -> HashMap<String, Value> {
        HashMap::new()
    }

    #[test]
    fn getfield_missing_input_port_errors() {
        let op = LoweredOp::Primitive {
            module: "test".to_string(),
            name: "get_field".to_string(),
            kind: PrimitiveOpKind::GetField {
                field: "name".to_string(),
                input_port: "record".to_string(),
            },
        };
        // Provide an input under a different key than the expected input_port
        let inputs = [("other".to_string(), Value::Str("val".to_string()))]
            .into_iter()
            .collect();

        let err = execute_lowered_op(&op, inputs, &empty_sibling_fns(), &empty_data_values())
            .expect_err("GetField with missing input port should fail closed");

        assert_eq!(err.message, "GetField `name`: missing input port `record`");
    }

    #[test]
    fn unsupported_primitive_errors_instead_of_passthrough() {
        let op = LoweredOp::Primitive {
            module: "test".to_string(),
            name: "fs_env".to_string(),
            kind: PrimitiveOpKind::FsEnv,
        };
        let inputs = [("path".to_string(), Value::Str("HOME".to_string()))]
            .into_iter()
            .collect();

        let err = execute_lowered_op(&op, inputs, &empty_sibling_fns(), &empty_data_values())
            .expect_err("unsupported primitive should fail closed");

        assert_eq!(
            err.message,
            "primitive op FsEnv is not supported by the interpreter"
        );
    }

    #[test]
    fn transport_nodes_error_instead_of_passthrough() {
        let op = LoweredOp::Transport {
            module: "test".to_string(),
            kind: CallableKind::Func,
            name: "transport_call".to_string(),
            obligation: TransportObligation::Execute,
            service_metadata: Box::new(ServiceCallMetadata {
                service: "svc".to_string(),
                operation: "op".to_string(),
                transport: ServiceTransportClass::LocalDirect,
                idempotent: true,
                readonly: false,
                spec: None,
                response_provider: None,
            }),
            is_interactive: false,
            resource_target: None,
        };
        let inputs = [
            ("value".to_string(), Value::Int(7)),
            ("__out:result".to_string(), Value::Bool(true)),
        ]
        .into_iter()
        .collect();

        let err = execute_lowered_op(&op, inputs, &empty_sibling_fns(), &empty_data_values())
            .expect_err("transport nodes should fail closed");

        assert_eq!(
            err.message,
            "transport node reached interpreter; transport execution must happen before runtime interpretation"
        );
    }

    #[test]
    fn pipeline_nodes_error_instead_of_returning_empty_outputs() {
        let op = LoweredOp::Pipeline {
            module: "test".to_string(),
            name: "pipeline".to_string(),
            stages: 1,
            stage_names: vec!["stage".to_string()],
        };

        let err = execute_lowered_op(
            &op,
            HashMap::new(),
            &empty_sibling_fns(),
            &empty_data_values(),
        )
        .expect_err("pipeline nodes should fail closed");

        assert_eq!(
            err.message,
            "pipeline node reached interpreter; pipeline metadata must not execute at runtime"
        );
    }

    #[test]
    fn missing_fn_body_errors_instead_of_passthrough() {
        let op = LoweredOp::Callable {
            module: "test".to_string(),
            kind: CallableKind::Func,
            name: "callable".to_string(),
            obligation: CallableObligation::None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        };
        let inputs = [("input".to_string(), Value::Int(1))].into_iter().collect();

        let err = execute_lowered_op(&op, inputs, &empty_sibling_fns(), &empty_data_values())
            .expect_err("callables without fn_body should fail closed");

        assert_eq!(
            err.message,
            "callable node reached interpreter without a fn_body"
        );
    }

    #[test]
    fn skipped_input_fn_body_failures_error_instead_of_returning_empty_outputs() {
        let op = LoweredOp::Callable {
            module: "test".to_string(),
            kind: CallableKind::Fn,
            name: "callable".to_string(),
            obligation: CallableObligation::None,
            is_interactive: false,
            resource_target: None,
            fn_body: Some(Box::new(LoweredFnBody::from_stmts(vec![
                LoweredStmt::Let(
                    "tmp".to_string(),
                    LoweredExpr::Call {
                        name: "missing_builtin".to_string(),
                        args: vec![(
                            Some("value".to_string()),
                            LoweredExpr::Ident("record".to_string()),
                        )],
                    },
                ),
                LoweredStmt::Return(vec![(
                    "value".to_string(),
                    LoweredExpr::Ident("tmp".to_string()),
                )]),
            ]))),
        };
        let inputs = [("record".to_string(), Value::Skipped)]
            .into_iter()
            .collect();

        let err = execute_lowered_op(&op, inputs, &empty_sibling_fns(), &empty_data_values())
            .expect_err("skipped-input fn-body failures should surface");

        assert!(
            err.message.contains("FnBody evaluation failed:")
                && err.message.contains("unknown function: missing_builtin"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn collection_missing_items_errors_instead_of_defaulting_to_empty() {
        let op = LoweredOp::Collection {
            module: "test".to_string(),
            callable: "count".to_string(),
            kind: CollectionKind::Count,
        };

        let err = execute_lowered_op(
            &op,
            HashMap::new(),
            &empty_sibling_fns(),
            &empty_data_values(),
        )
        .expect_err("missing items input should fail closed");

        assert_eq!(err.message, "collection operation missing `items` input");
    }

    #[test]
    fn collection_non_list_items_error_instead_of_defaulting_to_empty() {
        let op = LoweredOp::Collection {
            module: "test".to_string(),
            callable: "count".to_string(),
            kind: CollectionKind::Count,
        };
        let inputs = [("items".to_string(), Value::Str("wrong".to_string()))]
            .into_iter()
            .collect();

        let err = execute_lowered_op(&op, inputs, &empty_sibling_fns(), &empty_data_values())
            .expect_err("non-list items should fail closed");

        assert_eq!(
            err.message,
            "collection `items` input must be List, got String"
        );
    }
}
