//! Transport operation type.

use crate::executor::execute_transport;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Transport operations.
///
/// This is the universal executor for all I/O operations.
/// It takes a `TransportRequest` and returns a `TransportResponse`.
#[derive(Debug, Clone, Copy)]
pub enum TransportOp {
    /// Execute a transport request (the only I/O boundary)
    Execute,
}

impl Executable for TransportOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            TransportOp::Execute => execute_op(inputs),
        }
    }
}

/// Execute a transport request.
fn execute_op(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let request = match inputs.get("request") {
        Some(Value::Request(r)) => r.clone(),
        Some(other) => {
            return Err(ExecError::new(format!(
                "expected Request value, got: {}",
                other
            )))
        }
        None => return Err(ExecError::new("missing 'request' input")),
    };

    let response = execute_transport(&request)
        .map_err(|e| ExecError::new(format!("transport error: {}", e)))?;

    let mut out = HashMap::new();
    out.insert("response".to_string(), Value::Response(response));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{FileRequest, TransportRequest};

    #[test]
    fn test_execute_requires_request_input() {
        let result = execute_op(HashMap::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }

    #[test]
    fn test_execute_wrong_value_type() {
        let mut inputs = HashMap::new();
        inputs.insert("request".to_string(), Value::Str("not a request".into()));

        let result = execute_op(inputs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected Request"));
    }

    #[test]
    fn test_execute_file_exists() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "request".to_string(),
            Value::Request(TransportRequest::File(FileRequest::exists("Cargo.toml"))),
        );

        let result = execute_op(inputs).unwrap();
        
        match result.get("response") {
            Some(Value::Response(_)) => {}
            _ => panic!("expected Response"),
        }
    }
}
