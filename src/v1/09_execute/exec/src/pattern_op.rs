//! Executable semantics for pattern-internal operations.

use crate::helpers::{
    optional_int, propagate_skipped, require_bool, require_int, require_value, OutputMap,
};
use crate::{ExecError, Executable};
use gunbc_ir::patterns::collection::CollectionKind;
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

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

                let list = list_values(&inputs, input_port);
                let count = list.len() as i64;

                let mut out = OutputMap::new()
                    .value(element_port, Value::List(Arc::new(list)))
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
                if let Some(result) =
                    propagate_skipped(&inputs, "result", &[output_port, "iterations"])
                {
                    return result;
                }

                let list = list_values(&inputs, "result");

                let count = optional_int(&inputs, "count").unwrap_or(list.len() as i64);

                OutputMap::new()
                    .value(output_port, Value::List(Arc::new(list)))
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
            PatternOp::CollectionAggregate { kind } => execute_collection_aggregate(kind, inputs),
        }
    }
}

fn execute_collection_aggregate(
    kind: &CollectionKind,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let items = match inputs.get("items") {
        Some(Value::List(values)) => (**values).clone(),
        Some(Value::Skipped) => return OutputMap::new().value("items", Value::Skipped).ok(),
        None => Vec::new(),
        Some(value) => vec![value.clone()],
    };

    let output = match kind {
        CollectionKind::Map
        | CollectionKind::Filter
        | CollectionKind::FlatMap
        | CollectionKind::FilterMap
        | CollectionKind::Append => Value::List(Arc::new(items)),
        CollectionKind::Sort | CollectionKind::SortBy => {
            let mut sorted = items;
            sorted.sort_by_key(|v| match v {
                Value::Str(s) => s.clone(),
                Value::Int(n) => n.to_string(),
                other => format!("{other:?}"),
            });
            Value::List(Arc::new(sorted))
        }
        CollectionKind::Dedup => {
            let mut out = Vec::new();
            for item in items {
                if !out.contains(&item) {
                    out.push(item);
                }
            }
            Value::List(Arc::new(out))
        }
        CollectionKind::Join => {
            let joined = items
                .iter()
                .map(|v| match v {
                    Value::Str(s) => s.clone(),
                    other => format!("{other:?}"),
                })
                .collect::<Vec<_>>()
                .join(",");
            Value::Str(joined)
        }
        CollectionKind::Fold | CollectionKind::Len | CollectionKind::Count => {
            Value::Int(items.len() as i64)
        }
        CollectionKind::Sum => {
            let total: i64 = items
                .iter()
                .map(|v| match v {
                    Value::Int(i) => *i,
                    _ => 0,
                })
                .sum();
            Value::Int(total)
        }
        CollectionKind::Any => Value::Bool(
            items
                .iter()
                .any(|v| !matches!(v, Value::Bool(false) | Value::Unit)),
        ),
        CollectionKind::All => Value::Bool(
            items
                .iter()
                .all(|v| !matches!(v, Value::Bool(false) | Value::Unit)),
        ),
        CollectionKind::Contains => {
            let needle = inputs
                .get("needle")
                .or_else(|| inputs.get("item"))
                .or_else(|| inputs.get("contains"));
            let found = needle
                .map(|needle| items.iter().any(|v| v == needle))
                .unwrap_or(false);
            Value::Bool(found)
        }
        CollectionKind::Split | CollectionKind::Zip => Value::List(Arc::new(items)),
        CollectionKind::Skip => {
            let n = inputs
                .get("n")
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            Value::List(Arc::new(items.into_iter().skip(n).collect()))
        }
        CollectionKind::Enumerate => {
            let enumerated = items
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut map = BTreeMap::new();
                    map.insert("index".to_string(), Value::Int(i as i64));
                    map.insert("value".to_string(), v);
                    Value::Map(map)
                })
                .collect();
            Value::List(Arc::new(enumerated))
        }
    };

    OutputMap::new().value("items", output).ok()
}

fn list_values(inputs: &HashMap<String, Value>, key: &str) -> Vec<Value> {
    match inputs.get(key) {
        None | Some(Value::Skipped) => Vec::new(),
        Some(Value::List(values)) | Some(Value::Set(values)) => (**values).clone(),
        Some(value) => vec![value.clone()],
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

    #[test]
    fn test_loop_pack_propagates_skipped() {
        let op = PatternOp::LoopPack {
            output_port: "items".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("result".to_string(), Value::Skipped);
        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("items"), Some(&Value::Skipped));
        assert_eq!(result.get("iterations"), Some(&Value::Skipped));
    }

    #[test]
    fn loop_pack_wraps_scalar_result_into_list() {
        let op = PatternOp::LoopPack {
            output_port: "items".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("result".to_string(), Value::Str("one".to_string()));
        let result = op.execute(inputs).expect("loop pack should accept scalar");
        assert_eq!(
            result.get("items"),
            Some(&Value::List(Arc::new(vec![Value::Str("one".to_string())])))
        );
        assert_eq!(result.get("iterations"), Some(&Value::Int(1)));
    }

    #[test]
    fn collection_skip_drops_first_n_elements() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionKind::Skip,
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(Arc::new(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
                Value::Str("c".to_string()),
            ])),
        );
        inputs.insert("n".to_string(), Value::Int(1));
        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("items"),
            Some(&Value::List(Arc::new(vec![
                Value::Str("b".to_string()),
                Value::Str("c".to_string()),
            ])))
        );
    }

    #[test]
    fn collection_skip_without_n_returns_all() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionKind::Skip,
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(Arc::new(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ])),
        );
        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("items"),
            Some(&Value::List(Arc::new(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ])))
        );
    }

    #[test]
    fn collection_enumerate_produces_index_value_pairs() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionKind::Enumerate,
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(Arc::new(vec![
                Value::Str("x".to_string()),
                Value::Str("y".to_string()),
            ])),
        );
        let result = op.execute(inputs).unwrap();
        let expected = Value::List(Arc::new(vec![
            Value::Map({
                let mut m = BTreeMap::new();
                m.insert("index".to_string(), Value::Int(0));
                m.insert("value".to_string(), Value::Str("x".to_string()));
                m
            }),
            Value::Map({
                let mut m = BTreeMap::new();
                m.insert("index".to_string(), Value::Int(1));
                m.insert("value".to_string(), Value::Str("y".to_string()));
                m
            }),
        ]));
        assert_eq!(result.get("items"), Some(&expected));
    }

    #[test]
    fn collection_enumerate_empty_list() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionKind::Enumerate,
        };
        let mut inputs = HashMap::new();
        inputs.insert("items".to_string(), Value::List(Arc::new(vec![])));
        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("items"), Some(&Value::List(Arc::new(vec![]))));
    }

    #[test]
    fn collection_skip_skipped_propagates() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionKind::Skip,
        };
        let mut inputs = HashMap::new();
        inputs.insert("items".to_string(), Value::Skipped);
        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("items"), Some(&Value::Skipped));
    }

    #[test]
    fn collection_enumerate_skipped_propagates() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionKind::Enumerate,
        };
        let mut inputs = HashMap::new();
        inputs.insert("items".to_string(), Value::Skipped);
        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("items"), Some(&Value::Skipped));
    }
}
