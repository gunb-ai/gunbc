//! Executable semantics for pattern-internal operations.

use crate::helpers::{require_bool, require_int, require_str_list, require_value, OutputMap};
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

                OutputMap::new()
                    .value(output_port, selected)
                    .str("branch_taken", branch_taken)
                    .ok()
            }
            PatternOp::LoopUnpack {
                input_port,
                element_port,
            } => {
                let list = require_str_list(&inputs, input_port)?;
                let count = list.len() as i64;

                OutputMap::new()
                    .value(element_port, Value::str_list(list))
                    .int("index", 0)
                    .int("count", count)
                    .ok()
            }
            PatternOp::LoopPack { output_port } => {
                let list = require_str_list(&inputs, "result")?;

                let count = inputs
                    .get("count")
                    .and_then(|v| v.as_int())
                    .unwrap_or(list.len() as i64);

                OutputMap::new()
                    .value(output_port, Value::str_list(list))
                    .int("iterations", count)
                    .ok()
            }
            PatternOp::RetryController {
                input_port,
                policy,
                classifier: _,
            } => {
                let input = require_value(&inputs, input_port)?;

                let last_error_present = inputs.contains_key("last_error");
                let should_retry = last_error_present && policy.max_attempts > 1;

                OutputMap::new()
                    .value("body_input", input.clone())
                    .int("attempt", 1)
                    .bool("should_retry", should_retry)
                    .ok()
            }
            PatternOp::RetryCollector { output_port } => {
                let attempt = inputs
                    .get("attempt")
                    .and_then(|v| v.as_int())
                    .unwrap_or(1);

                let result = inputs.get("result").cloned().unwrap_or(Value::Skipped);

                let mut out = OutputMap::new()
                    .value(output_port, result)
                    .int("attempts_made", attempt);
                if let Some(err) = inputs.get("error") {
                    out = out.value("final_error", err.clone());
                }
                out.ok()
            }
            PatternOp::WhileInit { input_port } => {
                let state = require_value(&inputs, input_port)?;

                OutputMap::new()
                    .value("state_out", state.clone())
                    .ok()
            }
            PatternOp::WhileController { max_iterations } => {
                let continue_flag = require_bool(&inputs, "continue")?;
                let next_state = require_value(&inputs, "next_state")?;

                let mut iterations = if continue_flag { 1 } else { 0 };
                if let Some(max) = max_iterations {
                    iterations = iterations.min(*max as i64);
                }

                OutputMap::new()
                    .value("final_state", next_state.clone())
                    .int("iterations", iterations)
                    .ok()
            }
            PatternOp::PollTimer {
                input_port,
                interval,
                timeout: _,
            } => {
                let input = require_value(&inputs, input_port)?;

                let elapsed_ms = interval.as_millis();
                let elapsed_ms = if elapsed_ms > i64::MAX as u128 {
                    i64::MAX
                } else {
                    elapsed_ms as i64
                };

                OutputMap::new()
                    .value("body_input", input.clone())
                    .int("poll_count", 1)
                    .int("elapsed_ms", elapsed_ms)
                    .ok()
            }
            PatternOp::PollCollector { output_port } => {
                let success = require_bool(&inputs, "success")?;
                let poll_count = require_int(&inputs, "poll_count")?;
                let elapsed_ms = require_int(&inputs, "elapsed_ms")?;

                let mut out = OutputMap::new();
                if let Some(result) = inputs.get("result") {
                    out = out.value(output_port, result.clone());
                }
                out.bool("success", success)
                    .int("polls", poll_count)
                    .int("elapsed_ms", elapsed_ms)
                    .ok()
            }
        }
    }
}
