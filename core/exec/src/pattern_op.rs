//! Executable semantics for pattern-internal operations.

use crate::{ExecError, Executable};
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::Value;
use std::collections::HashMap;

impl Executable for PatternOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            PatternOp::BranchMerge { output_port } => {
                let true_val = inputs.get("true_result");
                let false_val = inputs.get("false_result");

                let (selected, branch_taken) = match true_val {
                    Some(v) if !matches!(v, Value::Skipped) => (v.clone(), "true"),
                    _ => match false_val {
                        Some(v) if !matches!(v, Value::Skipped) => (v.clone(), "false"),
                        _ => (Value::Skipped, "none"),
                    },
                };

                let mut out = HashMap::new();
                out.insert(output_port.clone(), selected);
                out.insert("branch_taken".to_string(), Value::Str(branch_taken.to_string()));
                Ok(out)
            }
            PatternOp::LoopUnpack {
                input_port,
                element_port,
            } => {
                let list = inputs
                    .get(input_port)
                    .and_then(|v| v.as_str_list())
                    .ok_or_else(|| {
                        ExecError::new(format!(
                            "missing or invalid '{}' string list",
                            input_port
                        ))
                    })?;

                let mut out = HashMap::new();
                out.insert(element_port.clone(), Value::str_list(list.clone()));
                out.insert("index".to_string(), Value::Int(0));
                out.insert("count".to_string(), Value::Int(list.len() as i64));
                Ok(out)
            }
            PatternOp::LoopPack { output_port } => {
                let list = inputs
                    .get("result")
                    .and_then(|v| v.as_str_list())
                    .ok_or_else(|| ExecError::new("missing or invalid 'result' string list"))?;

                let count = inputs
                    .get("count")
                    .and_then(|v| v.as_int())
                    .unwrap_or(list.len() as i64);

                let mut out = HashMap::new();
                out.insert(output_port.clone(), Value::str_list(list));
                out.insert("iterations".to_string(), Value::Int(count));
                Ok(out)
            }
            PatternOp::RetryController {
                input_port,
                policy,
                classifier: _,
            } => {
                let input = inputs.get(input_port).ok_or_else(|| {
                    ExecError::new(format!("missing '{}' input", input_port))
                })?;

                let last_error_present = inputs.contains_key("last_error");
                let should_retry = last_error_present && policy.max_attempts > 1;

                let mut out = HashMap::new();
                out.insert("body_input".to_string(), input.clone());
                out.insert("attempt".to_string(), Value::Int(1));
                out.insert("should_retry".to_string(), Value::Bool(should_retry));
                Ok(out)
            }
            PatternOp::RetryCollector { output_port } => {
                let attempt = inputs
                    .get("attempt")
                    .and_then(|v| v.as_int())
                    .unwrap_or(1);

                let result = inputs.get("result").cloned().unwrap_or(Value::Skipped);

                let mut out = HashMap::new();
                out.insert(output_port.clone(), result);
                out.insert("attempts_made".to_string(), Value::Int(attempt));
                if let Some(err) = inputs.get("error") {
                    out.insert("final_error".to_string(), err.clone());
                }
                Ok(out)
            }
            PatternOp::WhileInit { input_port } => {
                let state = inputs.get(input_port).ok_or_else(|| {
                    ExecError::new(format!("missing '{}' input", input_port))
                })?;

                let mut out = HashMap::new();
                out.insert("state_out".to_string(), state.clone());
                Ok(out)
            }
            PatternOp::WhileController { max_iterations } => {
                let continue_flag = inputs
                    .get("continue")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| ExecError::new("missing or invalid 'continue' bool"))?;
                let next_state = inputs
                    .get("next_state")
                    .ok_or_else(|| ExecError::new("missing 'next_state' input"))?;

                let mut iterations = if continue_flag { 1 } else { 0 };
                if let Some(max) = max_iterations {
                    iterations = iterations.min(*max as i64);
                }

                let mut out = HashMap::new();
                out.insert("final_state".to_string(), next_state.clone());
                out.insert("iterations".to_string(), Value::Int(iterations));
                Ok(out)
            }
            PatternOp::PollTimer {
                input_port,
                interval,
                timeout: _,
            } => {
                let input = inputs.get(input_port).ok_or_else(|| {
                    ExecError::new(format!("missing '{}' input", input_port))
                })?;

                let elapsed_ms = interval.as_millis();
                let elapsed_ms = if elapsed_ms > i64::MAX as u128 {
                    i64::MAX
                } else {
                    elapsed_ms as i64
                };

                let mut out = HashMap::new();
                out.insert("body_input".to_string(), input.clone());
                out.insert("poll_count".to_string(), Value::Int(1));
                out.insert("elapsed_ms".to_string(), Value::Int(elapsed_ms));
                Ok(out)
            }
            PatternOp::PollCollector { output_port } => {
                let success = inputs
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| ExecError::new("missing or invalid 'success' bool"))?;
                let poll_count = inputs
                    .get("poll_count")
                    .and_then(|v| v.as_int())
                    .ok_or_else(|| ExecError::new("missing or invalid 'poll_count' int"))?;
                let elapsed_ms = inputs
                    .get("elapsed_ms")
                    .and_then(|v| v.as_int())
                    .ok_or_else(|| ExecError::new("missing or invalid 'elapsed_ms' int"))?;

                let mut out = HashMap::new();
                if let Some(result) = inputs.get("result") {
                    out.insert(output_port.clone(), result.clone());
                }
                out.insert("success".to_string(), Value::Bool(success));
                out.insert("polls".to_string(), Value::Int(poll_count));
                out.insert("elapsed_ms".to_string(), Value::Int(elapsed_ms));
                Ok(out)
            }
        }
    }
}
