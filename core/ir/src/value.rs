//! Runtime values flowing through the DAG.

use crate::transport::{TransportRequest, TransportResponse};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A string value that redacts its content in Display and Debug output.
///
/// Secrets flow through the DAG like normal values but are automatically
/// redacted when logged, printed, or formatted. The inner value is only
/// accessible via `expose()`, making accidental leakage structurally
/// harder.
///
/// # Design
///
/// Secrets are normal I/O that get resolved (upserted/ensured) at
/// execution time. Between DAG nodes they flow as values, but any
/// logging or display shows `***` instead of the actual content.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretString {
    #[serde(rename = "secret")]
    inner: String,
}

impl SecretString {
    /// Create a new secret string.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            inner: value.into(),
        }
    }

    /// Expose the secret value. Use sparingly — only at I/O boundaries
    /// where the actual value is needed (e.g., setting an HTTP header).
    pub fn expose(&self) -> &str {
        &self.inner
    }

    /// Length of the secret (safe to expose for diagnostics).
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString(***)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

/// Runtime value flowing between nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Value {
    /// Unit value (no data)
    #[default]
    Unit,
    /// Boolean
    Bool(bool),
    /// String
    Str(String),
    /// Integer
    Int(i64),
    /// List of strings
    StrList(Vec<String>),
    /// Map from string to string
    MapStrStr(BTreeMap<String, String>),
    /// JSON value (for complex data)
    Json(serde_json::Value),
    /// Transport request (for I/O operations)
    Request(TransportRequest),
    /// Transport response (from I/O operations)
    Response(TransportResponse),
    /// Secret value (redacted in logs/display, exposed only at I/O boundaries)
    Secret(SecretString),
    /// Node was skipped (guard evaluated to false)
    Skipped,
}

impl Value {
    /// Check if this value represents a skipped node.
    pub fn is_skipped(&self) -> bool {
        matches!(self, Value::Skipped)
    }

    /// Check if this is a transport request.
    pub fn is_request(&self) -> bool {
        matches!(self, Value::Request(_))
    }

    /// Check if this is a transport response.
    pub fn is_response(&self) -> bool {
        matches!(self, Value::Response(_))
    }

    /// Try to extract a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to extract an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to extract a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract a JSON value.
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Value::Json(j) => Some(j),
            _ => None,
        }
    }

    /// Try to extract a string list.
    pub fn as_str_list(&self) -> Option<Vec<String>> {
        match self {
            Value::StrList(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// Try to extract a map from string to string.
    pub fn as_map_str_str(&self) -> Option<BTreeMap<String, String>> {
        match self {
            Value::MapStrStr(m) => Some(m.clone()),
            _ => None,
        }
    }

    /// Try to extract a transport request.
    pub fn as_request(&self) -> Option<TransportRequest> {
        match self {
            Value::Request(r) => Some(r.clone()),
            _ => None,
        }
    }

    /// Try to extract a transport response.
    pub fn as_response(&self) -> Option<&TransportResponse> {
        match self {
            Value::Response(r) => Some(r),
            _ => None,
        }
    }

    /// Check if this is a secret value.
    pub fn is_secret(&self) -> bool {
        matches!(self, Value::Secret(_))
    }

    /// Try to extract the secret string (exposed value).
    ///
    /// Returns `None` if the value is not a secret. Use sparingly —
    /// the whole point of `Secret` is to avoid accidental exposure.
    pub fn as_secret(&self) -> Option<&SecretString> {
        match self {
            Value::Secret(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::StrList(v) => write!(f, "[{} items]", v.len()),
            Value::MapStrStr(m) => write!(f, "{{{} entries}}", m.len()),
            Value::Json(j) => write!(f, "{}", j),
            Value::Request(r) => write!(f, "<Request: {:?}>", std::mem::discriminant(r)),
            Value::Response(r) => write!(f, "<Response: {:?}>", std::mem::discriminant(r)),
            Value::Secret(_) => write!(f, "***"),
            Value::Skipped => write!(f, "<SKIPPED>"),
        }
    }
}

impl From<TransportRequest> for Value {
    fn from(r: TransportRequest) -> Self {
        Value::Request(r)
    }
}

impl From<TransportResponse> for Value {
    fn from(r: TransportResponse) -> Self {
        Value::Response(r)
    }
}
