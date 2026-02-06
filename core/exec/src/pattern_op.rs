//! Executable semantics for pattern-internal operations.

use crate::helpers::{
    optional_int, optional_str_list_strict, propagate_skipped, require_bool, require_int,
    require_value, OutputMap,
};
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
                // Propagate Skipped from upstream (e.g., skip propagation tests)
                if let Some(result) =
                    propagate_skipped(&inputs, input_port, &[element_port, "count"])
                {
                    return result;
                }

                let list = optional_str_list_strict(&inputs, input_port)?.unwrap_or_default();
                let count = list.len() as i64;

                let mut out = OutputMap::new()
                    .str_list(element_port, list)
                    .int("index", 0)
                    .int("count", count);

                // Pass through extra inputs (e.g., repo_path) so
                // execute_loop_body can retrieve them from unpack outputs.
                for (key, value) in &inputs {
                    if key != input_port {
                        out = out.value(key, value.clone());
                    }
                }

                out.ok()
            }
            PatternOp::LoopPack { output_port } => {
                let list = optional_str_list_strict(&inputs, "result")?.unwrap_or_default();

                let count = optional_int(&inputs, "count").unwrap_or(list.len() as i64);

                OutputMap::new()
                    .str_list(output_port, list)
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
                let attempt = optional_int(&inputs, "attempt").unwrap_or(1);

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

                OutputMap::new().value("state_out", state.clone()).ok()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_unpack_propagates_skipped() {
        let op = PatternOp::LoopUnpack {
            input_port: "files".to_string(),
            element_port: "filename".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("files".to_string(), Value::Skipped);
        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("filename"), Some(&Value::Skipped));
        assert_eq!(result.get("count"), Some(&Value::Skipped));
    }
}
