//! Data primitives - pure data transformations.
//!
//! These operations transform data without side effects.
//! They are the building blocks for parsing, extraction, and formatting.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parse a string into structured data (JSON, TOML, YAML).
///
/// Inputs:
/// - `input`: String to parse
///
/// Outputs:
/// - `output`: Json value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseOp {
    /// Parse JSON string
    Json,
    /// Parse TOML string
    Toml,
    /// Parse YAML string (future)
    Yaml,
}

impl Executable for ParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string"))?;

        let json_value: serde_json::Value = match self {
            ParseOp::Json => serde_json::from_str(input)
                .map_err(|e| ExecError::new(format!("JSON parse error: {}", e)))?,
            ParseOp::Toml => {
                let toml_value: toml::Value = toml::from_str(input)
                    .map_err(|e| ExecError::new(format!("TOML parse error: {}", e)))?;
                serde_json::to_value(toml_value)
                    .map_err(|e| ExecError::new(format!("TOML to JSON conversion error: {}", e)))?
            }
            ParseOp::Yaml => {
                return Err(ExecError::new("YAML parsing not yet implemented"));
            }
        };

        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::Json(json_value));
        Ok(out)
    }
}

/// Extract a field from structured data using a path.
///
/// Inputs:
/// - `input`: Json value to extract from
/// - `path`: String path (e.g., "package.name" or "dependencies.0")
///
/// Outputs:
/// - `output`: Extracted value (Json, String, Int, etc.)
/// - `exists`: Bool indicating if the path exists
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractOp;

impl Executable for ExtractOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let json: &serde_json::Value = inputs
            .get("input")
            .and_then(|v| v.as_json())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' JSON"))?;

        let path = inputs
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'path' string"))?;

        // Navigate the path
        let mut current: &serde_json::Value = json;
        for segment in path.split('.') {
            current = if let Ok(index) = segment.parse::<usize>() {
                // Array index
                current.get(index).unwrap_or(&serde_json::Value::Null)
            } else {
                // Object key
                current.get(segment).unwrap_or(&serde_json::Value::Null)
            };
        }

        let exists = !current.is_null();
        let output = json_to_value(current.clone());

        let mut out = HashMap::new();
        out.insert("output".to_string(), output);
        out.insert("exists".to_string(), Value::Bool(exists));
        Ok(out)
    }
}

/// Format a template string with values.
///
/// Inputs:
/// - `template`: String with {key} placeholders
/// - `values`: MapStrStr of key-value pairs
///
/// Outputs:
/// - `output`: Formatted string
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatOp;

impl Executable for FormatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let template = inputs
            .get("template")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'template' string"))?;

        let values = inputs
            .get("values")
            .and_then(|v| v.as_map_str_str())
            .unwrap_or_default();

        let mut result = template.to_string();
        for (key, value) in &values {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::Str(result));
        Ok(out)
    }
}

/// Concatenate a list of strings with a separator.
///
/// Inputs:
/// - `input`: StrList to concatenate
/// - `separator`: String separator (default: "")
///
/// Outputs:
/// - `output`: Concatenated string
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConcatOp;

impl Executable for ConcatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

        let separator = inputs
            .get("separator")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let result = list.join(separator);

        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::Str(result));
        Ok(out)
    }
}

/// Split a string by a delimiter.
///
/// Inputs:
/// - `input`: String to split
/// - `delimiter`: String delimiter
///
/// Outputs:
/// - `output`: StrList of parts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplitOp;

impl Executable for SplitOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string"))?;

        let delimiter = inputs
            .get("delimiter")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError::new("missing or invalid 'delimiter' string"))?;

        let parts: Vec<String> = input.split(delimiter).map(|s| s.to_string()).collect();

        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::StrList(parts));
        Ok(out)
    }
}

/// Convert a serde_json::Value to a gunbc_ir::Value.
fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Str("".to_string()),
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Str(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => {
            // Try to convert to StrList if all elements are strings
            let strs: Option<Vec<String>> = arr
                .iter()
                .map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if let Some(list) = strs {
                Value::StrList(list)
            } else {
                Value::Json(serde_json::Value::Array(arr))
            }
        }
        serde_json::Value::Object(_) => Value::Json(json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let op = ParseOp::Json;
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), Value::Str(r#"{"name": "test"}"#.to_string()));

        let result = op.execute(inputs).unwrap();
        let output = result.get("output").unwrap();
        assert!(matches!(output, Value::Json(_)));
    }

    #[test]
    fn test_extract_field() {
        let op = ExtractOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::Json(serde_json::json!({"package": {"name": "test"}})),
        );
        inputs.insert("path".to_string(), Value::Str("package.name".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Str("test".to_string())));
        assert_eq!(result.get("exists"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_format() {
        let op = FormatOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "template".to_string(),
            Value::Str("Hello, {name}!".to_string()),
        );
        let mut values = std::collections::BTreeMap::new();
        values.insert("name".to_string(), "world".to_string());
        inputs.insert("values".to_string(), Value::MapStrStr(values));

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::Str("Hello, world!".to_string()))
        );
    }

    #[test]
    fn test_concat() {
        let op = ConcatOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::StrList(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        );
        inputs.insert("separator".to_string(), Value::Str(", ".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::Str("a, b, c".to_string()))
        );
    }

    #[test]
    fn test_split() {
        let op = SplitOp;
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), Value::Str("a,b,c".to_string()));
        inputs.insert("delimiter".to_string(), Value::Str(",".to_string()));

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::StrList(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ]))
        );
    }
}
