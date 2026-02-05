//! Data primitives - pure data transformations.
//!
//! These operations transform data without side effects.
//! They are the building blocks for parsing, extraction, and formatting.

use gunbc_exec::{
    optional_map_str_str, optional_str, require_json, require_map_str_str, require_str,
    require_str_list, ExecError, Executable, IntoExecResult, OutputMap,
};
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
        let input = require_str(&inputs, "input")?;

        let json_value: serde_json::Value = match self {
            ParseOp::Json => serde_json::from_str(input).exec_context("JSON parse error")?,
            ParseOp::Toml => {
                let toml_value: toml::Value =
                    toml::from_str(input).exec_context("TOML parse error")?;
                serde_json::to_value(toml_value).exec_context("TOML to JSON conversion error")?
            }
            ParseOp::Yaml => {
                return Err(ExecError::new("YAML parsing not yet implemented"));
            }
        };

        OutputMap::new().json("output", json_value).ok()
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
        let json: &serde_json::Value = require_json(&inputs, "input")?;
        let path = require_str(&inputs, "path")?;

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

        OutputMap::new()
            .value("output", output)
            .bool("exists", exists)
            .ok()
    }
}

/// Format a template string with values.
///
/// Inputs:
/// - `template`: String with {key} placeholders
/// - `values`: Map of key-value pairs
///
/// Outputs:
/// - `output`: Formatted string
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatOp;

impl Executable for FormatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let template = require_str(&inputs, "template")?;

        let values = optional_map_str_str(&inputs, "values").unwrap_or_default();

        let mut result = template.to_string();
        for (key, value) in &values {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        OutputMap::new().str("output", result).ok()
    }
}

/// Concatenate a list of strings with a separator.
///
/// Inputs:
/// - `input`: List to concatenate
/// - `separator`: String separator (default: "")
///
/// Outputs:
/// - `output`: Concatenated string
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConcatOp;

impl Executable for ConcatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        let separator = optional_str(&inputs, "separator").unwrap_or("");

        let result = list.join(separator);

        OutputMap::new().str("output", result).ok()
    }
}

/// Split a string by a delimiter.
///
/// Inputs:
/// - `input`: String to split
/// - `delimiter`: String delimiter
///
/// Outputs:
/// - `output`: List of parts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplitOp;

impl Executable for SplitOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let input = require_str(&inputs, "input")?;
        let delimiter = require_str(&inputs, "delimiter")?;

        let parts: Vec<String> = input.split(delimiter).map(|s| s.to_string()).collect();

        OutputMap::new().str_list("output", parts).ok()
    }
}

/// Generate a stable hash from string inputs.
///
/// This operation computes a SHA256 hash of the concatenation of input strings,
/// useful for generating deterministic IDs from multiple keys. The hash is
/// truncated to 32 hex characters (16 bytes) for readability while maintaining
/// sufficient collision resistance.
///
/// Inputs:
/// - `parts`: List of strings to hash (concatenated with ":" separator)
///
/// Outputs:
/// - `hash`: 32-character hex string (first 16 bytes of SHA256)
///
/// # Example
///
/// ```ignore
/// // Input: ["check_id", "issue_key"]
/// // Output: "a1b2c3d4e5f6..." (32 hex chars)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StableHashOp;

impl StableHashOp {
    /// Compute a stable hash from parts.
    ///
    /// Delegates to [`gunbc_infra::hash::hash_parts`] — the canonical
    /// implementation of length-prefix multi-part hashing.
    pub fn hash_parts(parts: &[&str]) -> String {
        gunbc_infra::hash::hash_parts(parts)
    }
}

impl Executable for StableHashOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let parts = require_str_list(&inputs, "parts")?;

        let refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
        let hash = Self::hash_parts(&refs);

        OutputMap::new().str("hash", hash).ok()
    }
}

/// Format a map of key-value pairs into a single output string.
///
/// This operation takes a map of string→string entries and formats each
/// entry using a configurable format, then joins them with a separator.
///
/// Inputs:
/// - `entries`: Map of string keys to string values
///
/// Outputs:
/// - `output`: Formatted string
///
/// # Variants
///
/// - `DiffArtifact`: Formats as "--- {key}\n{value}" with "\n\n" separator,
///   suitable for displaying diff files. Returns "(no changes)" if empty.
///
/// # Example
///
/// ```ignore
/// // Input: {"src/main.rs": "@@ ...", "src/lib.rs": "@@ ..."}
/// // Output with DiffArtifact format:
/// // "--- src/main.rs\n@@ ...\n\n--- src/lib.rs\n@@ ..."
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatMapOp {
    /// Format as diff artifact: "--- {key}\n{value}" joined with "\n\n"
    DiffArtifact,
}

impl FormatMapOp {
    /// Format entries using the configured format.
    pub fn format_entries(
        &self,
        entries: &std::collections::BTreeMap<String, String>,
    ) -> String {
        match self {
            FormatMapOp::DiffArtifact => {
                if entries.is_empty() {
                    return "(no changes)".to_string();
                }
                let parts: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| format!("--- {}\n{}", key, value))
                    .collect();
                parts.join("\n\n")
            }
        }
    }
}

impl Executable for FormatMapOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let entries = require_map_str_str(&inputs, "entries")?;
        let output = self.format_entries(&entries);
        OutputMap::new().str("output", output).ok()
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
            // Try to convert to List if all elements are strings
            let strs: Option<Vec<String>> = arr
                .iter()
                .map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if let Some(list) = strs {
                Value::str_list(list)
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
        inputs.insert(
            "input".to_string(),
            Value::Str(r#"{"name": "test"}"#.to_string()),
        );

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
        inputs.insert("values".to_string(), Value::str_map(values));

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
            Value::str_list(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
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
            Some(&Value::str_list(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ]))
        );
    }

    #[test]
    fn test_stable_hash_deterministic() {
        // Same inputs produce same hash
        let hash1 = StableHashOp::hash_parts(&["check_id", "issue_key"]);
        let hash2 = StableHashOp::hash_parts(&["check_id", "issue_key"]);
        assert_eq!(hash1, hash2);

        // Different inputs produce different hash
        let hash3 = StableHashOp::hash_parts(&["other_check", "issue_key"]);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_stable_hash_op() {
        let op = StableHashOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "parts".to_string(),
            Value::str_list(vec!["check_id".to_string(), "issue_key".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        let hash = result.get("hash").unwrap();

        // Should be a 32-char hex string
        if let Value::Str(s) = hash {
            assert_eq!(s.len(), 32);
            assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
        } else {
            panic!("expected string output");
        }
    }

    #[test]
    fn test_stable_hash_order_matters() {
        // Order of parts matters
        let hash1 = StableHashOp::hash_parts(&["a", "b"]);
        let hash2 = StableHashOp::hash_parts(&["b", "a"]);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_stable_hash_no_delimiter_collision() {
        // Parts containing delimiters must not collide with different part boundaries.
        // Without length-prefix encoding, these would both produce "a:b:c" bytes.
        let hash1 = StableHashOp::hash_parts(&["a", "b:c"]);
        let hash2 = StableHashOp::hash_parts(&["a:b", "c"]);
        assert_ne!(hash1, hash2);

        // Also test the three-part case
        let hash3 = StableHashOp::hash_parts(&["a", "b", "c"]);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_format_map_diff_artifact() {
        use std::collections::BTreeMap;

        let mut entries = BTreeMap::new();
        entries.insert(
            "src/main.rs".to_string(),
            "@@ -1,3 +1,4 @@\n fn main() {}".to_string(),
        );
        entries.insert(
            "src/lib.rs".to_string(),
            "@@ -1 +1,2 @@\n pub fn foo() {}".to_string(),
        );

        let result = FormatMapOp::DiffArtifact.format_entries(&entries);

        assert!(result.contains("--- src/main.rs"));
        assert!(result.contains("--- src/lib.rs"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_format_map_diff_artifact_empty() {
        use std::collections::BTreeMap;

        let entries = BTreeMap::new();
        let result = FormatMapOp::DiffArtifact.format_entries(&entries);

        assert_eq!(result, "(no changes)");
    }

    #[test]
    fn test_format_map_op_execute() {
        use std::collections::BTreeMap;

        let op = FormatMapOp::DiffArtifact;
        let mut inputs = HashMap::new();

        let mut entries = BTreeMap::new();
        entries.insert("file.rs".to_string(), "content".to_string());
        inputs.insert("entries".to_string(), Value::str_map(entries));

        let result = op.execute(inputs).unwrap();
        let output = result.get("output").unwrap().as_str().unwrap();

        assert!(output.contains("--- file.rs"));
        assert!(output.contains("content"));
    }
}
