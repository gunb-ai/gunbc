//! T-22 substrate eval dispatch: `run_emit_host_rust` → `tools/emit_host_runner`.
//!
//! **Modeled authority:** `src/v4/compiler/emit_host.dag` + `v4.std.host_run` — host receipt
//! assembly (`emit_host_receipt_from_source`, `host_logical_run_from_exit`) is evaluated via
//! [`evaluator::eval_callable_declaration`], not re-encoded here. This module only maps runner
//! facts → substrate operand carriers and invokes the eval hook.
//!
//! **P5 receipt:** `EXPECTED_HAND_AUTHORED_NON_TEST` row in `sg0_census_test.rs`; lane
//! `T-PB-B` / `pb_rust_tests_outside_residual_zero`; dissolution: delete when substrate eval
//! owns host dispatch without this intercept (`emit_host_bridge.rs` retires with harness).

use crate::dag::{Dag, DeclarationId, LiteralBits, TypeConnective};
use crate::evaluator::{
    EvalError, EvalStateStack, EvalStrategy, NamedField, Value,
};
use emit_host_runner::{
    ExitWitness, HostExit, HostExitOutcome, HostLogicalFailure, HostSetupFailure,
};

/// Eval-time dispatch for `run_emit_host_rust` (substrate `emit_host.dag` only).
pub fn try_dispatch_emit_host_rust(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    if !is_run_emit_host_rust_decl(dag, callee_decl) {
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
    let inputs = match emit_host_fixture_inputs(&operands[2]) {
        Ok(inputs) => inputs,
        Err(err) => return Some(Err(err)),
    };
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_eval_emit_host_rust_{}",
        std::process::id()
    ));
    let receipt = match emit_host_runner::run_emit_host_rust(source, &inputs, &work_dir) {
        Ok(receipt) => receipt,
        Err(_setup) => return Some(run_emit_host_setup_rejected(dag)),
    };
    let callee = match find_fn_decl(dag, "emit_host_receipt_from_source") {
        Ok(id) => id,
        Err(err) => return Some(Err(err)),
    };
    let exit = match host_exit_value(dag, &receipt.exit) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stdout = match byte_string_value(dag, &receipt.stdout_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stderr = match byte_string_value(dag, &receipt.stderr_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let build_log = match build_log_value(dag, &receipt.build_log.lines) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    Some(
        eval_callable_declaration(
            dag,
            callee,
            vec![
                operands[0].clone(),
                Value::LiteralValue(LiteralBits::String(receipt.source_text)),
                exit,
                stdout,
                stderr,
                build_log,
            ],
            state,
            strategy,
        )
        .and_then(|value| accepted_variant(dag, value)),
    )
}

fn is_run_emit_host_rust_decl(dag: &Dag, callee_decl: DeclarationId) -> bool {
    let callee = dag.declaration(callee_decl);
    callee.name.as_deref() == Some("run_emit_host_rust")
        && matches!(callee.connective, TypeConnective::Arrow { .. })
}

fn find_fn_decl(dag: &Dag, name: &str) -> Result<DeclarationId, EvalError> {
    dag.declarations()
        .iter()
        .find(|decl| decl.name.as_deref() == Some(name))
        .map(|decl| decl.id)
        .ok_or(EvalError::BadTransformOperands {
            reason: "substrate fn declaration not found in eval dag",
        })
}

fn eval_callable_declaration(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: Vec<Value>,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<Value, EvalError> {
    crate::evaluator::eval_callable_declaration(dag, callee_decl, operands, state, strategy)
}

fn run_emit_host_setup_rejected(dag: &Dag) -> Result<Value, EvalError> {
    rejected_outcome_variant(
        dag,
        non_empty_diagnostics_singleton(dag, emit_host_setup_failure_diagnostic(dag)?)?,
    )
}

fn expect_string_operand(value: &Value) -> Result<&str, EvalError> {
    match value {
        Value::LiteralValue(LiteralBits::String(s)) => Ok(s),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected String operand",
        }),
    }
}

fn emit_host_fixture_inputs(
    value: &Value,
) -> Result<emit_host_runner::EmitHostFixtureInputs, EvalError> {
    let root = match value {
        Value::RecordValue(fields) => fields
            .iter()
            .find(|field| field.label == "root")
            .map(|field| &field.value)
            .ok_or(EvalError::BadTransformOperands {
                reason: "expected Inputs.root field",
            })?,
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "expected Inputs record",
            });
        }
    };
    let pin = match root {
        Value::LiteralValue(LiteralBits::String(s)) => s.clone(),
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "expected Inputs.root string literal",
            });
        }
    };
    Ok(emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: pin.clone(),
        expected_eval_root: pin,
    })
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

fn list_instantiation_for_element(
    dag: &Dag,
    element_ty: DeclarationId,
) -> Result<DeclarationId, EvalError> {
    let list_decl = dag
        .declaration_by_name("List")
        .ok_or(EvalError::BadTransformOperands {
            reason: "List type not found",
        })?;
    dag.declarations()
        .iter()
        .find_map(|decl| {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                return None;
            };
            (*template == list_decl.id
                && arguments.len() == 1
                && arguments[0].value == element_ty)
                .then_some(decl.id)
        })
        .ok_or(EvalError::BadTransformOperands {
            reason: "List<element> instantiation not found in dag",
        })
}

fn find_list_variant_tag(
    dag: &Dag,
    list_ty: DeclarationId,
    variant: &str,
) -> Result<DeclarationId, EvalError> {
    let list_decl = dag
        .declaration_by_name("List")
        .ok_or(EvalError::BadTransformOperands {
            reason: "List type not found",
        })?;
    let arm_ty = match &list_decl.connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|v| v.label == variant)
            .ok_or(EvalError::BadTransformOperands {
                reason: "List variant arm not found",
            })?
            .ty,
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "List is not a sum type",
            });
        }
    };
    let elem_ty = match &dag.declaration(list_ty).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if *template == list_decl.id && arguments.len() == 1 => arguments[0].value,
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "expected List<elem> instantiation",
            });
        }
    };
    dag.declarations()
        .iter()
        .find_map(|decl| {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                return None;
            };
            (*template == arm_ty && arguments.len() == 1 && arguments[0].value == elem_ty)
                .then_some(decl.id)
        })
        .ok_or(EvalError::BadTransformOperands {
            reason: "List variant constructor tag not found",
        })
}

fn empty_list_value(dag: &Dag, list_ty: DeclarationId) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: find_list_variant_tag(dag, list_ty, "Empty")?,
        payload: Box::new(Value::RecordValue(vec![])),
    })
}

fn cons_list_value(
    dag: &Dag,
    list_ty: DeclarationId,
    head: Value,
    tail: Value,
) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: find_list_variant_tag(dag, list_ty, "Cons")?,
        payload: Box::new(Value::RecordValue(vec![
            NamedField {
                label: "head".to_string(),
                value: head,
            },
            NamedField {
                label: "tail".to_string(),
                value: tail,
            },
        ])),
    })
}

fn list_from_values(
    dag: &Dag,
    list_ty: DeclarationId,
    items: Vec<Value>,
) -> Result<Value, EvalError> {
    let mut list = empty_list_value(dag, list_ty)?;
    for item in items.into_iter().rev() {
        list = cons_list_value(dag, list_ty, item, list)?;
    }
    Ok(list)
}

fn bool_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let bool_ty = dag
        .declaration_by_name("Bool")
        .ok_or(EvalError::BadTransformOperands {
            reason: "Bool type not found",
        })?
        .id;
    list_instantiation_for_element(dag, bool_ty)
}

fn byte_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let byte_ty = dag
        .declaration_by_name("Byte")
        .ok_or(EvalError::BadTransformOperands {
            reason: "Byte type not found",
        })?
        .id;
    list_instantiation_for_element(dag, byte_ty)
}

fn string_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    list_instantiation_for_element(
        dag,
        dag.declaration_by_name("String")
            .ok_or(EvalError::BadTransformOperands {
                reason: "String type not found",
            })?
            .id,
    )
}

fn byte_value(dag: &Dag, byte: u8) -> Result<Value, EvalError> {
    let bits: Vec<Value> = (0..8)
        .map(|shift| Value::LiteralValue(LiteralBits::Bool((byte >> (7 - shift)) & 1 != 0)))
        .collect();
    Ok(Value::RecordValue(vec![NamedField {
        label: "bits".to_string(),
        value: list_from_values(dag, bool_list_ty(dag)?, bits)?,
    }]))
}

fn byte_string_value(dag: &Dag, bytes: &[u8]) -> Result<Value, EvalError> {
    let elems: Result<Vec<Value>, EvalError> = bytes
        .iter()
        .copied()
        .map(|b| byte_value(dag, b))
        .collect();
    list_from_values(dag, byte_list_ty(dag)?, elems?)
}

fn string_list_value(dag: &Dag, lines: &[String]) -> Result<Value, EvalError> {
    let elems: Vec<Value> = lines
        .iter()
        .cloned()
        .map(|line| Value::LiteralValue(LiteralBits::String(line)))
        .collect();
    list_from_values(dag, string_list_ty(dag)?, elems)
}

fn build_log_value(dag: &Dag, lines: &[String]) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![NamedField {
        label: "lines".to_string(),
        value: string_list_value(dag, lines)?,
    }]))
}

fn diagnostic_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let diagnostic_ty = dag
        .declaration_by_name("Diagnostic")
        .ok_or(EvalError::BadTransformOperands {
            reason: "Diagnostic type not found",
        })?
        .id;
    list_instantiation_for_element(dag, diagnostic_ty)
}

fn diagnostics_none_variant(dag: &Dag) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Diagnostics", "None")?,
        payload: Box::new(Value::RecordValue(Vec::new())),
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

fn rejected_outcome_variant(dag: &Dag, diagnostics: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Outcome", "Rejected")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "diagnostics".to_string(),
            value: diagnostics,
        }])),
    })
}

fn non_empty_diagnostics_singleton(dag: &Dag, diagnostic: Value) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![
        NamedField {
            label: "head".to_string(),
            value: diagnostic,
        },
        NamedField {
            label: "tail".to_string(),
            value: empty_list_value(dag, diagnostic_list_ty(dag)?)?,
        },
    ]))
}

fn correction_unavailable_variant(dag: &Dag) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Correction", "Unavailable")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "reason".to_string(),
            value: Value::VariantValue {
                tag: variant_decl_id(dag, "NoCorrectionReason", "ExternalContractUnknown")?,
                payload: Box::new(Value::RecordValue(vec![])),
            },
        }])),
    })
}

fn port_locus_value(dag: &Dag, port: &str) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Locus", "PortLocus")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "anchor".to_string(),
            value: Value::RecordValue(vec![NamedField {
                label: "at".to_string(),
                value: Value::LiteralValue(LiteralBits::String(port.to_string())),
            }]),
        }])),
    })
}

fn emit_host_port_diagnostic(dag: &Dag, reason: &str) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![
        NamedField {
            label: "reason".to_string(),
            value: Value::LiteralValue(LiteralBits::String(reason.to_string())),
        },
        NamedField {
            label: "at".to_string(),
            value: port_locus_value(dag, "emit_host_transport_port")?,
        },
        NamedField {
            label: "correction".to_string(),
            value: correction_unavailable_variant(dag)?,
        },
    ]))
}

fn emit_host_setup_failure_diagnostic(dag: &Dag) -> Result<Value, EvalError> {
    emit_host_port_diagnostic(dag, "emit_host_exit_not_ok")
}

fn host_logical_failure_diagnostic(
    dag: &Dag,
    failure: &HostLogicalFailure,
) -> Result<Value, EvalError> {
    let reason = match failure {
        HostLogicalFailure::TimedOut { .. } => "emit_host_exit_timed_out",
        HostLogicalFailure::NoExitStatus { .. } => "emit_host_exit_no_status",
        HostLogicalFailure::ExitedNonzero { .. } => "emit_host_exit_nonzero",
    };
    emit_host_port_diagnostic(dag, reason)
}

fn host_setup_failure_diagnostic(dag: &Dag, _failure: &HostSetupFailure) -> Result<Value, EvalError> {
    emit_host_setup_failure_diagnostic(dag)
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

fn witness_violates_variant(dag: &Dag, diagnostic: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Witness", "Violates")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "diagnostic".to_string(),
            value: diagnostic,
        }])),
    })
}

fn host_exit_outcome_value(dag: &Dag, exit: &HostExit) -> Result<Value, EvalError> {
    match &exit.outcome {
        HostExitOutcome::Accepted(ExitWitness::Holds(ok)) => accepted_variant(
            dag,
            witness_holds_variant(
                dag,
                Value::RecordValue(vec![NamedField {
                    label: "code".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(ok.code.to_string())),
                }]),
            )?,
        ),
        HostExitOutcome::Accepted(ExitWitness::Violates(failure)) => accepted_variant(
            dag,
            witness_violates_variant(dag, host_logical_failure_diagnostic(dag, failure)?)?,
        ),
        HostExitOutcome::Rejected(setup) => rejected_outcome_variant(
            dag,
            non_empty_diagnostics_singleton(dag, host_setup_failure_diagnostic(dag, setup)?)?,
        ),
    }
}

fn host_exit_value(dag: &Dag, exit: &HostExit) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![NamedField {
        label: "outcome".to_string(),
        value: host_exit_outcome_value(dag, exit)?,
    }]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Behavior, TypeConnective};
    use crate::evaluator::{
        EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder,
    };
    use emit_host_runner::HostLogicalFailure;

    fn eager_strategy() -> EvalStrategy {
        EvalStrategy::ApplicativeOrder {
            input_order: InputEvaluationOrder::LeftFirst,
        }
    }

    #[test]
    fn is_run_emit_host_rust_decl_requires_arrow_name() {
        let dag = crate::dag::Dag::new();
        let decl_id = dag.push_declaration(
            crate::dag::Declaration {
                name: Some("run_emit_host_rust".to_string()),
                connective: TypeConnective::Arrow {
                    inputs: vec![],
                    output: dag.alloc_port(None),
                    body: crate::dag::ArrowBody::UserDefined(
                        crate::dag::BindNodeId::from_raw(0),
                    ),
                },
                span: crate::dag::span(),
                ..Default::default()
            },
            crate::dag::span(),
        );
        assert!(is_run_emit_host_rust_decl(&dag, decl_id));
    }

    #[test]
    fn emit_host_fixture_inputs_rejects_debug_fallback() {
        let err = emit_host_fixture_inputs(&Value::RecordValue(vec![NamedField {
            label: "root".to_string(),
            value: Value::LiteralValue(LiteralBits::Int("1".to_string())),
        }]))
        .expect_err("non-string root");
        assert!(matches!(
            err,
            EvalError::BadTransformOperands {
                reason: "expected Inputs.root string literal",
            }
        ));
    }

    #[test]
    fn host_exit_outcome_maps_violates_to_witness_not_empty_record() {
        let dag = crate::dag::Dag::new();
        let exit = HostExit::logical_violation(HostLogicalFailure::ExitedNonzero {
            phase: emit_host_runner::HostPhase::FixtureRun,
            code: Some(1),
        });
        let outcome = host_exit_outcome_value(&dag, &exit).expect("outcome");
        let Value::VariantValue { tag, .. } = outcome else {
            panic!("expected Outcome variant");
        };
        let tag_decl = dag.declaration(tag);
        assert_eq!(tag_decl.name.as_deref(), Some("Accepted"));
    }
}
