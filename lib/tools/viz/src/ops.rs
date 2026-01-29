//! Visualization-specific operations.
//!
//! Demonstrates decomposition into primitives where possible.
//! Domain-specific logic (DAG discovery, export) remains,
//! but file preparation delegates to primitives.

use crate::discover::discover_all_dags;
use crate::export::VizCollection;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_primitives::PrepareFileWriteOp;
use std::collections::HashMap;

/// Viz-specific operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum VizOp {
    /// Collect all DAG definitions from the workspace (discovers them generically)
    CollectDags,
    /// Export DAGs to JSON format
    ExportJson,
    /// Prepare file write with viz-specific default path (viz-data.json)
    PrepareVizOutput,
}

impl Executable for VizOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            VizOp::CollectDags => execute_collect_dags(inputs),
            VizOp::ExportJson => execute_export_json(inputs),
            VizOp::PrepareVizOutput => execute_prepare_viz_output(inputs),
        }
    }
}

/// Collect all DAG definitions by discovering them from the actual graph builders.
fn execute_collect_dags(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    // Use the generic discovery mechanism
    let graphs = discover_all_dags();

    let graph_names: Vec<String> = graphs.iter().map(|g| g.name.clone()).collect();

    let mut out = HashMap::new();
    out.insert("graph_count".to_string(), Value::Int(graphs.len() as i64));
    out.insert("graph_names".to_string(), Value::StrList(graph_names));
    out.insert(
        "graphs".to_string(),
        Value::Json(serde_json::to_value(&graphs).unwrap_or_default()),
    );
    Ok(out)
}

/// Export DAGs to JSON format.
fn execute_export_json(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let graphs: Vec<crate::export::VizGraph> = match inputs.get("graphs") {
        Some(Value::Json(j)) => serde_json::from_value(j.clone()).unwrap_or_default(),
        _ => return Err(ExecError::new("missing or invalid 'graphs' input")),
    };

    let collection = VizCollection {
        graphs,
        generated_at: chrono_lite(),
    };

    let json_content = serde_json::to_string_pretty(&collection)
        .map_err(|e| ExecError::new(format!("JSON serialization failed: {}", e)))?;

    let mut out = HashMap::new();
    out.insert("json_content".to_string(), Value::Str(json_content.clone()));
    // Also output as "content" for compatibility with FsOp::PrepareFileWrite
    out.insert("content".to_string(), Value::Str(json_content));
    Ok(out)
}

/// Prepare viz output file write using PrepareFileWriteOp primitive.
///
/// Adds viz-specific default path (viz-data.json) before delegating.
fn execute_prepare_viz_output(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let content = inputs
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new("missing or invalid 'content' input"))?;
    
    let output_path = inputs
        .get("output_path")
        .and_then(|v| v.as_str())
        .unwrap_or("viz-data.json");  // Viz-specific default
    
    // Use PrepareFileWriteOp primitive
    let mut prep_inputs = HashMap::new();
    prep_inputs.insert("path".to_string(), Value::Str(output_path.to_string()));
    prep_inputs.insert("content".to_string(), Value::Str(content.to_string()));
    
    PrepareFileWriteOp.execute(prep_inputs)
}

/// Simple timestamp.
fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_dags() {
        let result = execute_collect_dags(HashMap::new()).unwrap();

        match result.get("graph_count") {
            Some(Value::Int(n)) => assert!(*n >= 7, "Expected at least 7 DAGs"),
            _ => panic!("expected graph count"),
        }
    }

    #[test]
    fn test_export_json() {
        let collect_result = execute_collect_dags(HashMap::new()).unwrap();

        let mut inputs = HashMap::new();
        inputs.insert(
            "graphs".to_string(),
            collect_result.get("graphs").unwrap().clone(),
        );

        let result = execute_export_json(inputs).unwrap();

        match result.get("json_content") {
            Some(Value::Str(s)) => {
                assert!(s.contains("gunbc-gist"));
                assert!(s.contains("graphs"));
            }
            _ => panic!("expected JSON content"),
        }
    }
}
