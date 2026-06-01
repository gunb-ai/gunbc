//! T-22 substrate eval dispatch: `v4.compiler.emit_host` host transport rows → `emit_host_runner`.
//!
//! Modeled authority remains `src/v4/compiler/emit_host.dag`; this module is the eval hook that
//! executes real host-process transport when the evaluator calls `run_emit_host_*` (dissolves
//! `emit_host_transport_not_wired` for wired rows without fabricating receipts in `.dag`).

use crate::dag::{Dag, DeclarationId, LiteralBits, TypeConnective};
use crate::evaluator::{EvalError, NamedField, Value};

/// Eval-time dispatch for `run_emit_host_rust` in `emit_host.dag`.
pub fn try_dispatch_emit_host_rust(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
) -> Option<Result<Value, EvalError>> {
    let callee = dag.declaration(callee_decl);
    if callee.name.as_deref() != Some("run_emit_host_rust")
        || !callee.span.file.ends_with("v4/compiler/emit_host.dag")
    {
        return None;
    }
    if operands.len() != 3 {
        return Some(Err(EvalError::TransformArityMismatch {
            expected: 3,
            got: operands.len(),
        }));
    }

    let source = match expect_string_operand(&operands[1]) {
        Ok(source) => source,
        Err(err) => return Some(Err(err)),
    };
    let input_pin = emit_host_input_pin(&operands[2]);
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: input_pin.clone(),
        expected_eval_root: input_pin,
    };
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_eval_emit_host_rust_{}",
        std::process::id()
    ));
    let receipt = match emit_host_runner::run_emit_host_rust(source, &inputs, &work_dir) {
        Ok(receipt) => receipt,
        Err(_) => {
            return Some(Err(EvalError::BadTransformOperands {
                reason: "run_emit_host_rust host setup failed",
            }));
        }
    };
    Some(accepted_variant(
        dag,
        emit_host_receipt_value(dag, &operands[0], receipt),
    ))
}

fn expect_string_operand(value: &Value) -> Result<&str, EvalError> {
    match value {
        Value::LiteralValue(LiteralBits::String(s)) => Ok(s),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected String operand",
        }),
    }
}

fn emit_host_input_pin(value: &Value) -> String {
    match value {
        Value::RecordValue(fields) => fields
            .iter()
            .find(|field| field.label == "root")
            .map(|field| format!("{:?}", field.value))
            .unwrap_or_else(|| format!("{value:?}")),
        _ => format!("{value:?}"),
    }
}

fn variant_decl_id(
    dag: &Dag,
    type_name: &str,
    variant_name: &str,
) -> Result<DeclarationId, EvalError> {
    let decl = dag
        .declaration_by_name(type_name)
        .ok_or(EvalError::BadTransformOperands {
            reason: "variant carrier type not found",
        })?;
    let TypeConnective::Disj { variants } = &decl.connective else {
        return Err(EvalError::BadTransformOperands {
            reason: "variant carrier is not a sum type",
        });
    };
    variants
        .iter()
        .find(|variant| variant.label == variant_name)
        .map(|variant| variant.ty)
        .ok_or(EvalError::BadTransformOperands {
            reason: "variant arm not found",
        })
}

fn accepted_variant(dag: &Dag, value: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Outcome", "Accepted")?,
        payload: Box::new(Value::RecordValue(vec![
            NamedField {
                label: "value".to_string(),
                value,
            },
            NamedField {
                label: "diagnostics".to_string(),
                value: diagnostics_none_variant(dag)?,
            },
        ])),
    })
}

fn diagnostics_none_variant(dag: &Dag) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Diagnostics", "None")?,
        payload: Box::new(Value::RecordValue(Vec::new())),
    })
}

fn witness_holds_variant(dag: &Dag, value: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Witness", "Holds")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "value".to_string(),
            value,
        }])),
    })
}

fn emit_host_receipt_value(
    dag: &Dag,
    target: &Value,
    receipt: emit_host_runner::EmitHostRunReceipt,
) -> Value {
    let exit_outcome = match receipt.exit.outcome {
        emit_host_runner::HostExitOutcome::Accepted(emit_host_runner::ExitWitness::Holds(ok)) => {
            accepted_variant(
                dag,
                witness_holds_variant(
                    dag,
                    Value::RecordValue(vec![NamedField {
                        label: "code".to_string(),
                        value: Value::LiteralValue(LiteralBits::Int(ok.code.to_string())),
                    }]),
                )
                .unwrap_or_else(|_| Value::RecordValue(Vec::new())),
            )
            .unwrap_or_else(|_| Value::RecordValue(Vec::new()))
        }
        _ => Value::RecordValue(Vec::new()),
    };
    Value::RecordValue(vec![
        NamedField {
            label: "target".to_string(),
            value: target.clone(),
        },
        NamedField {
            label: "source_text".to_string(),
            value: Value::LiteralValue(LiteralBits::String(receipt.source_text)),
        },
        NamedField {
            label: "exit".to_string(),
            value: Value::RecordValue(vec![NamedField {
                label: "outcome".to_string(),
                value: exit_outcome,
            }]),
        },
        NamedField {
            label: "logical_run".to_string(),
            value: accepted_variant(
                dag,
                Value::RecordValue(vec![NamedField {
                    label: "stdout".to_string(),
                    value: Value::RecordValue(vec![NamedField {
                        label: "bytes".to_string(),
                        value: Value::LiteralValue(LiteralBits::String(
                            String::from_utf8_lossy(&receipt.stdout_bytes).to_string(),
                        )),
                    }]),
                }]),
            )
            .unwrap_or_else(|_| Value::RecordValue(Vec::new())),
        },
        NamedField {
            label: "stderr_bytes".to_string(),
            value: Value::LiteralValue(LiteralBits::String(
                String::from_utf8_lossy(&receipt.stderr_bytes).to_string(),
            )),
        },
        NamedField {
            label: "build_log".to_string(),
            value: Value::RecordValue(vec![NamedField {
                label: "lines".to_string(),
                value: Value::LiteralValue(LiteralBits::String(receipt.build_log.lines.join("\n"))),
            }]),
        },
    ])
}
