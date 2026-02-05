//! Helpers for `Executable::execute` implementations.
//!
//! These reduce boilerplate in the three most common patterns:
//!
//! 1. **Input extraction** — `inputs.require_str("key")?` replaces
//!    `inputs.get("key").and_then(|v| v.as_str()).ok_or_else(|| ...)?`
//!
//! 2. **Output construction** — `OutputMap::new().str("k", v).ok()` replaces
//!    `let mut out = HashMap::new(); out.insert(...); Ok(out)`
//!
//! 3. **Response type matching** — `response.require_shell()?` replaces
//!    `match response { TransportResponse::Shell(s) => s, _ => return Err(...) }`
//!
//! # InputsExt Trait
//!
//! The [`InputsExt`] trait provides method syntax for input extraction:
//!
//! ```ignore
//! use gunbc_exec::InputsExt;
//!
//! fn execute(&self, inputs: HashMap<String, Value>) -> Result<..., ExecError> {
//!     let name = inputs.require_str("name")?;
//!     let count = inputs.optional_int("count").unwrap_or(10);
//!     // ...
//! }
//! ```

use std::collections::{BTreeMap, HashMap};

use gunbc_ir::transport::{
    FileResponse, HttpResponse, RestResponse, ShellResponse, TcpResponse, TransportRequest,
    TransportResponse,
};
use gunbc_ir::value::SecretString;
use gunbc_ir::Value;

use crate::ExecError;

// ---------------------------------------------------------------------------
// Input extraction
// ---------------------------------------------------------------------------

/// Extract a required string input.
pub fn require_str<'a>(
    inputs: &'a HashMap<String, Value>,
    key: &str,
) -> Result<&'a str, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required JSON input.
pub fn require_json<'a>(
    inputs: &'a HashMap<String, Value>,
    key: &str,
) -> Result<&'a serde_json::Value, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_json())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required boolean input.
pub fn require_bool(inputs: &HashMap<String, Value>, key: &str) -> Result<bool, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required integer input.
pub fn require_int(inputs: &HashMap<String, Value>, key: &str) -> Result<i64, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_int())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required string list input.
pub fn require_str_list(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<Vec<String>, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_str_list())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required map input.
pub fn require_map_str_str(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<BTreeMap<String, String>, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_map_str_str())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required transport response input.
pub fn require_response<'a>(
    inputs: &'a HashMap<String, Value>,
    key: &str,
) -> Result<&'a TransportResponse, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required transport request input.
pub fn require_request(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<TransportRequest, ExecError> {
    inputs
        .get(key)
        .and_then(|v| v.as_request())
        .ok_or_else(|| ExecError::new(format!("missing or invalid '{}' input", key)))
}

/// Extract a required Value input (any type).
pub fn require_value<'a>(
    inputs: &'a HashMap<String, Value>,
    key: &str,
) -> Result<&'a Value, ExecError> {
    inputs
        .get(key)
        .ok_or_else(|| ExecError::new(format!("missing '{}' input", key)))
}

/// Extract an optional string input.
pub fn optional_str<'a>(inputs: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
    inputs.get(key).and_then(|v| v.as_str())
}

/// Extract an optional JSON input.
pub fn optional_json<'a>(
    inputs: &'a HashMap<String, Value>,
    key: &str,
) -> Option<&'a serde_json::Value> {
    inputs.get(key).and_then(|v| v.as_json())
}

/// Extract an optional boolean input.
pub fn optional_bool(inputs: &HashMap<String, Value>, key: &str) -> Option<bool> {
    inputs.get(key).and_then(|v| v.as_bool())
}

/// Extract an optional string list input.
pub fn optional_str_list(inputs: &HashMap<String, Value>, key: &str) -> Option<Vec<String>> {
    inputs.get(key).and_then(|v| v.as_str_list())
}

/// Extract an optional map input.
pub fn optional_map_str_str(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Option<BTreeMap<String, String>> {
    inputs.get(key).and_then(|v| v.as_map_str_str())
}

// ---------------------------------------------------------------------------
// InputsExt trait (method syntax for input extraction)
// ---------------------------------------------------------------------------

/// Extension trait for input maps, providing method syntax for extraction.
///
/// This trait provides the same functionality as the free functions above,
/// but with method syntax that reduces import lists and reads more naturally:
///
/// ```ignore
/// // Before (free functions):
/// use gunbc_exec::{require_str, require_json, optional_bool};
/// let name = require_str(&inputs, "name")?;
///
/// // After (trait methods):
/// use gunbc_exec::InputsExt;
/// let name = inputs.require_str("name")?;
/// ```
pub trait InputsExt {
    /// Extract a required string input.
    fn require_str(&self, key: &str) -> Result<&str, ExecError>;

    /// Extract a required JSON input.
    fn require_json(&self, key: &str) -> Result<&serde_json::Value, ExecError>;

    /// Extract a required boolean input.
    fn require_bool(&self, key: &str) -> Result<bool, ExecError>;

    /// Extract a required integer input.
    fn require_int(&self, key: &str) -> Result<i64, ExecError>;

    /// Extract a required string list input.
    fn require_str_list(&self, key: &str) -> Result<Vec<String>, ExecError>;

    /// Extract a required map input.
    fn require_map_str_str(&self, key: &str) -> Result<BTreeMap<String, String>, ExecError>;

    /// Extract a required transport response input.
    fn require_response(&self, key: &str) -> Result<&TransportResponse, ExecError>;

    /// Extract a required transport request input.
    fn require_request(&self, key: &str) -> Result<TransportRequest, ExecError>;

    /// Extract a required Value input (any type).
    fn require_value(&self, key: &str) -> Result<&Value, ExecError>;

    /// Extract an optional string input.
    fn optional_str(&self, key: &str) -> Option<&str>;

    /// Extract an optional JSON input.
    fn optional_json(&self, key: &str) -> Option<&serde_json::Value>;

    /// Extract an optional boolean input.
    fn optional_bool(&self, key: &str) -> Option<bool>;

    /// Extract an optional integer input.
    fn optional_int(&self, key: &str) -> Option<i64>;

    /// Extract an optional string list input.
    fn optional_str_list(&self, key: &str) -> Option<Vec<String>>;

    /// Extract an optional map input.
    fn optional_map_str_str(&self, key: &str) -> Option<BTreeMap<String, String>>;
}

impl InputsExt for HashMap<String, Value> {
    fn require_str(&self, key: &str) -> Result<&str, ExecError> {
        require_str(self, key)
    }

    fn require_json(&self, key: &str) -> Result<&serde_json::Value, ExecError> {
        require_json(self, key)
    }

    fn require_bool(&self, key: &str) -> Result<bool, ExecError> {
        require_bool(self, key)
    }

    fn require_int(&self, key: &str) -> Result<i64, ExecError> {
        require_int(self, key)
    }

    fn require_str_list(&self, key: &str) -> Result<Vec<String>, ExecError> {
        require_str_list(self, key)
    }

    fn require_map_str_str(&self, key: &str) -> Result<BTreeMap<String, String>, ExecError> {
        require_map_str_str(self, key)
    }

    fn require_response(&self, key: &str) -> Result<&TransportResponse, ExecError> {
        require_response(self, key)
    }

    fn require_request(&self, key: &str) -> Result<TransportRequest, ExecError> {
        require_request(self, key)
    }

    fn require_value(&self, key: &str) -> Result<&Value, ExecError> {
        require_value(self, key)
    }

    fn optional_str(&self, key: &str) -> Option<&str> {
        optional_str(self, key)
    }

    fn optional_json(&self, key: &str) -> Option<&serde_json::Value> {
        optional_json(self, key)
    }

    fn optional_bool(&self, key: &str) -> Option<bool> {
        optional_bool(self, key)
    }

    fn optional_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_int())
    }

    fn optional_str_list(&self, key: &str) -> Option<Vec<String>> {
        optional_str_list(self, key)
    }

    fn optional_map_str_str(&self, key: &str) -> Option<BTreeMap<String, String>> {
        optional_map_str_str(self, key)
    }
}

// ---------------------------------------------------------------------------
// Output construction
// ---------------------------------------------------------------------------

/// Builder for `execute()` output maps.
///
/// ```ignore
/// OutputMap::new()
///     .str("key", content)
///     .bool("flag", true)
///     .ok()
/// ```
pub struct OutputMap(HashMap<String, Value>);

impl OutputMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert a string value.
    pub fn str(mut self, key: &str, val: impl Into<String>) -> Self {
        self.0.insert(key.to_string(), Value::Str(val.into()));
        self
    }

    /// Insert a boolean value.
    pub fn bool(mut self, key: &str, val: bool) -> Self {
        self.0.insert(key.to_string(), Value::Bool(val));
        self
    }

    /// Insert an integer value.
    pub fn int(mut self, key: &str, val: i64) -> Self {
        self.0.insert(key.to_string(), Value::Int(val));
        self
    }

    /// Insert a JSON value.
    pub fn json(mut self, key: &str, val: serde_json::Value) -> Self {
        self.0.insert(key.to_string(), Value::Json(val));
        self
    }

    /// Insert a string list value.
    pub fn str_list(mut self, key: &str, val: Vec<String>) -> Self {
        self.0.insert(key.to_string(), Value::str_list(val));
        self
    }

    /// Insert a map value.
    pub fn map_str_str(mut self, key: &str, val: BTreeMap<String, String>) -> Self {
        self.0.insert(key.to_string(), Value::str_map(val));
        self
    }

    /// Insert a transport request value.
    pub fn request(mut self, key: &str, val: TransportRequest) -> Self {
        self.0.insert(key.to_string(), Value::Request(val));
        self
    }

    /// Insert a transport response value.
    pub fn response(mut self, key: &str, val: TransportResponse) -> Self {
        self.0.insert(key.to_string(), Value::Response(val));
        self
    }

    /// Insert a secret value.
    pub fn secret(mut self, key: &str, val: SecretString) -> Self {
        self.0.insert(key.to_string(), Value::Secret(val));
        self
    }

    /// Insert a unit value.
    pub fn unit(mut self, key: &str) -> Self {
        self.0.insert(key.to_string(), Value::Unit);
        self
    }

    /// Insert any Value directly.
    pub fn value(mut self, key: &str, val: Value) -> Self {
        self.0.insert(key.to_string(), val);
        self
    }

    /// Consume the builder, returning the map.
    pub fn build(self) -> HashMap<String, Value> {
        self.0
    }

    /// Consume the builder, returning `Ok(map)`.
    pub fn ok(self) -> Result<HashMap<String, Value>, ExecError> {
        Ok(self.0)
    }
}

impl Default for OutputMap {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Skip propagation
// ---------------------------------------------------------------------------

/// Check if an input is `Value::Skipped` and, if so, propagate it to all
/// listed output keys.
///
/// Returns `Some(Ok(outputs))` when the input was skipped (caller should
/// return early), or `None` when the input was present and execution should
/// continue normally.
///
/// This eliminates the repeated pattern:
/// ```ignore
/// if matches!(inputs.get("response"), Some(Value::Skipped)) {
///     return OutputMap::new()
///         .value("out1", Value::Skipped)
///         .value("out2", Value::Skipped)
///         .ok();
/// }
/// ```
///
/// becomes:
/// ```ignore
/// if let Some(result) = propagate_skipped(&inputs, "response", &["out1", "out2"]) {
///     return result;
/// }
/// ```
pub fn propagate_skipped(
    inputs: &HashMap<String, Value>,
    input_key: &str,
    output_keys: &[&str],
) -> Option<Result<HashMap<String, Value>, ExecError>> {
    if matches!(inputs.get(input_key), Some(Value::Skipped)) {
        let mut map = OutputMap::new();
        for key in output_keys {
            map = map.value(key, Value::Skipped);
        }
        Some(map.ok())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// TransportResponse extraction
// ---------------------------------------------------------------------------

/// Extension trait for typed extraction from [`TransportResponse`].
pub trait TransportResponseExt {
    /// Require a Shell response, returning an error if the variant doesn't match.
    fn require_shell(&self) -> Result<&ShellResponse, ExecError>;
    /// Require a REST response, returning an error if the variant doesn't match.
    fn require_rest(&self) -> Result<&RestResponse, ExecError>;
    /// Require a File response, returning an error if the variant doesn't match.
    fn require_file(&self) -> Result<&FileResponse, ExecError>;
    /// Require an HTTP response, returning an error if the variant doesn't match.
    fn require_http(&self) -> Result<&HttpResponse, ExecError>;
    /// Require a TCP response, returning an error if the variant doesn't match.
    fn require_tcp(&self) -> Result<&TcpResponse, ExecError>;
}

impl TransportResponseExt for TransportResponse {
    fn require_shell(&self) -> Result<&ShellResponse, ExecError> {
        match self {
            TransportResponse::Shell(s) => Ok(s),
            other => Err(ExecError::new(format!(
                "expected Shell response, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    fn require_rest(&self) -> Result<&RestResponse, ExecError> {
        match self {
            TransportResponse::Rest(r) => Ok(r),
            other => Err(ExecError::new(format!(
                "expected Rest response, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    fn require_file(&self) -> Result<&FileResponse, ExecError> {
        match self {
            TransportResponse::File(f) => Ok(f),
            other => Err(ExecError::new(format!(
                "expected File response, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    fn require_http(&self) -> Result<&HttpResponse, ExecError> {
        match self {
            TransportResponse::Http(h) => Ok(h),
            other => Err(ExecError::new(format!(
                "expected Http response, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }

    fn require_tcp(&self) -> Result<&TcpResponse, ExecError> {
        match self {
            TransportResponse::Tcp(t) => Ok(t),
            other => Err(ExecError::new(format!(
                "expected Tcp response, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_str_returns_value() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Str("hello".to_string()));
        assert_eq!(require_str(&inputs, "name").unwrap(), "hello");
    }

    #[test]
    fn require_str_missing_key_errors() {
        let inputs = HashMap::new();
        let err = require_str(&inputs, "name").unwrap_err();
        assert!(err.0.contains("missing or invalid 'name' input"));
    }

    #[test]
    fn require_str_wrong_type_errors() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Bool(true));
        let err = require_str(&inputs, "name").unwrap_err();
        assert!(err.0.contains("missing or invalid 'name' input"));
    }

    #[test]
    fn optional_str_returns_none_when_missing() {
        let inputs = HashMap::new();
        assert_eq!(optional_str(&inputs, "name"), None);
    }

    #[test]
    fn output_map_builds_correctly() {
        let map = OutputMap::new()
            .str("name", "hello")
            .bool("flag", true)
            .int("count", 42)
            .build();
        assert_eq!(map.get("name"), Some(&Value::Str("hello".to_string())));
        assert_eq!(map.get("flag"), Some(&Value::Bool(true)));
        assert_eq!(map.get("count"), Some(&Value::Int(42)));
    }

    #[test]
    fn output_map_ok_wraps_in_result() {
        let result = OutputMap::new().str("x", "y").ok();
        assert!(result.is_ok());
    }

    #[test]
    fn require_shell_on_shell_response() {
        let resp: TransportResponse = ShellResponse::ok("ok").into();
        let shell = resp.require_shell().unwrap();
        assert_eq!(shell.stdout, "ok");
    }

    #[test]
    fn require_shell_on_rest_response_errors() {
        let resp = TransportResponse::Rest(RestResponse {
            status: 200,
            body: serde_json::Value::Null,
            headers: HashMap::new(),
        });
        assert!(resp.require_shell().is_err());
    }

    #[test]
    fn propagate_skipped_returns_some_when_skipped() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Skipped);

        let result = propagate_skipped(&inputs, "response", &["out1", "out2"]);
        assert!(result.is_some());

        let outputs = result.unwrap().unwrap();
        assert_eq!(outputs.get("out1"), Some(&Value::Skipped));
        assert_eq!(outputs.get("out2"), Some(&Value::Skipped));
    }

    #[test]
    fn propagate_skipped_returns_none_when_present() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Str("data".to_string()));

        let result = propagate_skipped(&inputs, "response", &["out1"]);
        assert!(result.is_none());
    }

    #[test]
    fn propagate_skipped_returns_none_when_missing() {
        let inputs = HashMap::new();
        let result = propagate_skipped(&inputs, "response", &["out1"]);
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // InputsExt trait tests
    // -----------------------------------------------------------------------

    #[test]
    fn inputs_ext_require_str() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Str("hello".to_string()));

        // Method syntax via InputsExt
        assert_eq!(inputs.require_str("name").unwrap(), "hello");
    }

    #[test]
    fn inputs_ext_require_str_missing() {
        let inputs: HashMap<String, Value> = HashMap::new();
        let err = inputs.require_str("name").unwrap_err();
        assert!(err.0.contains("missing or invalid 'name' input"));
    }

    #[test]
    fn inputs_ext_optional_str() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Str("hello".to_string()));

        assert_eq!(inputs.optional_str("name"), Some("hello"));
        assert_eq!(inputs.optional_str("missing"), None);
    }

    #[test]
    fn inputs_ext_require_bool() {
        let mut inputs = HashMap::new();
        inputs.insert("flag".to_string(), Value::Bool(true));

        assert!(inputs.require_bool("flag").unwrap());
    }

    #[test]
    fn inputs_ext_require_int() {
        let mut inputs = HashMap::new();
        inputs.insert("count".to_string(), Value::Int(42));

        assert_eq!(inputs.require_int("count").unwrap(), 42);
    }

    #[test]
    fn inputs_ext_optional_int() {
        let mut inputs = HashMap::new();
        inputs.insert("count".to_string(), Value::Int(42));

        assert_eq!(inputs.optional_int("count"), Some(42));
        assert_eq!(inputs.optional_int("missing"), None);
    }
}
