//! Runtime values flowing through the DAG.

use crate::transport::{TransportRequest, TransportResponse};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Runtime value flowing between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// Unit value (no data)
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

    /// Try to extract a transport request.
    pub fn as_request(&self) -> Option<&TransportRequest> {
        match self {
            Value::Request(r) => Some(r),
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
            Value::Skipped => write!(f, "<SKIPPED>"),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Unit
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
