//! Lint guard execution for DAG-integrated lint verification.
//!
//! This module provides `execute_lint_check`, which wraps the existing
//! `ensure_lint_upsert` logic as a standard DAG node execution function.
//! Instead of running lint verification as a separate callback/render loop,
//! it runs as a regular node in the tool's DAG — the graph structure itself
//! models why downstream nodes must wait.

use crate::preflight::ensure_lint_upsert_with_observer;
use gunbc_exec::ExecError;
use gunbc_ir::Value;
use std::collections::HashMap;

/// Execute a lint freshness check as a DAG node operation.
///
/// Wraps `ensure_lint_upsert_with_observer(None)` — no observer needed because
/// the DAG progress system handles display automatically (the node shows as
/// "running" then "completed" like any other node).
///
/// Outputs `{ "done": Bool(true) }` on success. Downstream nodes wire their
/// `_lint_guard` input port to this output, making the dependency explicit
/// in the graph.
pub fn execute_lint_check(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    ensure_lint_upsert_with_observer(None)
        .map_err(|e| ExecError::new(format!("lint check failed: {e}")))?;
    Ok(HashMap::from([("done".into(), Value::Bool(true))]))
}
