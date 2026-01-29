//! Visualization operations.

use crate::discover::discover_all_dags;
use crate::export::VizCollection;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, FileResponse, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use gunbc_transport::execute_transport;
use std::collections::HashMap;

/// Operations for the visualization tool.
#[derive(Debug, Clone)]
pub enum VizOp {
    /// Collect all DAG definitions from the workspace (discovers them generically)
    CollectDags,
    /// Export DAGs to JSON format
    ExportJson,
    /// Prepare file write request (PURE - no I/O)
    PrepareFileWrite,
    /// Execute transport (BOUNDARY - world write)
    ExecuteTransport,
}

impl Executable for VizOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            VizOp::CollectDags => execute_collect_dags(inputs),
            VizOp::ExportJson => execute_export_json(inputs),
            VizOp::PrepareFileWrite => execute_prepare_file_write(inputs),
            VizOp::ExecuteTransport => execute_transport_op(inputs),
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
    out.insert("json_content".to_string(), Value::Str(json_content));
    Ok(out)
}

/// Prepare file write request.
fn execute_prepare_file_write(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let json_content = match inputs.get("json_content") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing or invalid 'json_content' input")),
    };

    let output_path = match inputs.get("output_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => "viz-data.json".to_string(),
    };

    let request = TransportRequest::File(FileRequest::write(&output_path, json_content));

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    Ok(out)
}

/// Execute transport request.
fn execute_transport_op(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let request = match inputs.get("request") {
        Some(Value::Request(r)) => r.clone(),
        _ => return Err(ExecError::new("missing or invalid 'request' input")),
    };

    let response = execute_transport(&request)
        .map_err(|e| ExecError::new(format!("transport error: {}", e)))?;

    let written_path = match &response {
        TransportResponse::File(FileResponse { path, .. }) => path.clone(),
        _ => "unknown".to_string(),
    };

    let mut out = HashMap::new();
    out.insert("response".to_string(), Value::Response(response));
    out.insert("written_path".to_string(), Value::Str(written_path));
    Ok(out)
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
