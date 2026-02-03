//! Control primitives - control flow patterns.
//!
//! These operations implement control flow patterns like loops and branches.
//! They are typically implemented as higher-order patterns that expand into
//! SubDag nodes, but these primitives provide the leaf execution.

use gunbc_exec::{require_bool, require_str_list, ExecError, Executable, OutputMap};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Loop operation - iterate over a collection.
///
/// Note: Full loop functionality requires the LoopBuilder pattern which
/// creates SubDag nodes. This primitive handles simple iteration cases
/// where the body is a built-in transformation.
///
/// Inputs:
/// - `input`: List to iterate over
/// - `index`: Optional starting index (default: 0)
///
/// Outputs:
/// - `items`: List (same as input, for chaining)
/// - `count`: Int number of items
/// - `indices`: List of index strings ["0", "1", "2", ...]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoopOp;

impl Executable for LoopOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        let count = list.len() as i64;
        let indices: Vec<String> = (0..list.len()).map(|i| i.to_string()).collect();

        OutputMap::new()
            .value("items", Value::str_list(list))
            .int("count", count)
            .value("indices", Value::str_list(indices))
            .ok()
    }
}

/// Branch operation - conditional execution.
///
/// Note: Full branch functionality uses guarded ports. This primitive
/// handles simple if/else cases by selecting between two values.
///
/// Inputs:
/// - `condition`: Bool to evaluate
/// - `if_true`: Value to return if condition is true
/// - `if_false`: Value to return if condition is false
///
/// Outputs:
/// - `output`: Selected value based on condition
/// - `branch`: String "true" or "false" indicating which branch was taken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchOp;

impl Executable for BranchOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let condition = require_bool(&inputs, "condition")?;

        // Get both possible values
        let if_true = inputs.get("if_true").cloned().unwrap_or(Value::Bool(true));
        let if_false = inputs.get("if_false").cloned().unwrap_or(Value::Bool(false));

        let output = if condition { if_true } else { if_false };
        let branch = if condition { "true" } else { "false" };

        OutputMap::new()
            .value("output", output)
            .str("branch", branch)
            .ok()
    }
}

/// Guard operation - conditional pass-through.
///
/// Passes the input through only if the guard condition is true.
/// Otherwise outputs Value::Skipped.
///
/// Inputs:
/// - `input`: Value to pass through
/// - `guard`: Bool condition
///
/// Outputs:
/// - `output`: Input value if guard is true, Skipped otherwise
/// - `passed`: Bool indicating if the guard passed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardOp;

impl Executable for GuardOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let guard = require_bool(&inputs, "guard")?;

        let input = inputs.get("input").cloned().unwrap_or(Value::Bool(true));

        if guard {
            OutputMap::new()
                .value("output", input)
                .bool("passed", true)
                .ok()
        } else {
            OutputMap::new()
                .value("output", Value::Skipped)
                .bool("passed", false)
                .ok()
        }
    }
}

/// Sequence operation - ensure ordering.
///
/// Takes multiple inputs and outputs them in order, ensuring dependencies
/// are respected. This is useful for operations that must happen in sequence.
///
/// Inputs:
/// - `first`: First value (must complete before second)
/// - `second`: Second value
/// - `third`: Optional third value
///
/// Outputs:
/// - `result`: The last non-skipped value
/// - `completed`: Int count of completed steps
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SequenceOp;

impl Executable for SequenceOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let first = inputs.get("first");
        let second = inputs.get("second");
        let third = inputs.get("third");

        let mut completed = 0;
        let mut result = Value::Bool(true);

        if let Some(v) = first {
            if !matches!(v, Value::Skipped) {
                completed += 1;
                result = v.clone();
            }
        }

        if let Some(v) = second {
            if !matches!(v, Value::Skipped) {
                completed += 1;
                result = v.clone();
            }
        }

        if let Some(v) = third {
            if !matches!(v, Value::Skipped) {
                completed += 1;
                result = v.clone();
            }
        }

        OutputMap::new()
            .value("result", result)
            .int("completed", completed)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_indices() {
        let op = LoopOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::str_list(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("count"), Some(&Value::Int(3)));
        assert_eq!(
            result.get("indices"),
            Some(&Value::str_list(vec![
                "0".to_string(),
                "1".to_string(),
                "2".to_string()
            ]))
        );
    }

    #[test]
    fn test_branch_true() {
        let op = BranchOp;
        let mut inputs = HashMap::new();
        inputs.insert("condition".to_string(), Value::Bool(true));
        inputs.insert("if_true".to_string(), Value::Str("yes".to_string()));
        inputs.insert("if_false".to_string(), Value::Str("no".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Str("yes".to_string())));
        assert_eq!(result.get("branch"), Some(&Value::Str("true".to_string())));
    }

    #[test]
    fn test_branch_false() {
        let op = BranchOp;
        let mut inputs = HashMap::new();
        inputs.insert("condition".to_string(), Value::Bool(false));
        inputs.insert("if_true".to_string(), Value::Str("yes".to_string()));
        inputs.insert("if_false".to_string(), Value::Str("no".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Str("no".to_string())));
        assert_eq!(result.get("branch"), Some(&Value::Str("false".to_string())));
    }

    #[test]
    fn test_guard_pass() {
        let op = GuardOp;
        let mut inputs = HashMap::new();
        inputs.insert("guard".to_string(), Value::Bool(true));
        inputs.insert("input".to_string(), Value::Str("data".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Str("data".to_string())));
        assert_eq!(result.get("passed"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_guard_fail() {
        let op = GuardOp;
        let mut inputs = HashMap::new();
        inputs.insert("guard".to_string(), Value::Bool(false));
        inputs.insert("input".to_string(), Value::Str("data".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Skipped));
        assert_eq!(result.get("passed"), Some(&Value::Bool(false)));
    }
}
