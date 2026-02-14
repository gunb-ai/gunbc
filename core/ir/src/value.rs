//! Runtime values flowing through the DAG.
//!
//! Values are built from compositional layers:
//!
//! | Layer      | Variant         | Description                          |
//! |------------|-----------------|--------------------------------------|
//! | Unit       | `Unit`          | No data (signal only)                |
//! | Scalar     | `Bool`, `Int`   | Primitive scalars                    |
//! | String     | `Str`           | Character sequences                  |
//! | Collection | `List`, `Map`   | Homogeneous list / string-keyed map  |
//! | Dynamic    | `Json`          | Arbitrary structured data            |
//! | Transport  | `Request/Response` | I/O boundary values               |
//! | Security   | `Secret`        | Redacted strings                     |
//! | Control    | `Skipped`       | Guard-skipped nodes                  |
//!
//! Compound types like "list of strings" or "map from string to int" are
//! expressed by nesting: `Value::List(vec![Value::Str(...)])`. There are
//! no special-case variants for specific element types.

use crate::transport::{TransportRequest, TransportResponse};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

/// Build a serialization-based lookup index for a slice of Values.
/// Used for O(1) containment checks in set operations.
fn value_index(values: &[Value]) -> HashSet<String> {
    values
        .iter()
        .filter_map(|v| serde_json::to_string(v).ok())
        .collect()
}

/// Check if a Value is present in a serialization-based index.
fn in_index(v: &Value, idx: &HashSet<String>) -> bool {
    serde_json::to_string(v)
        .map(|k| idx.contains(&k))
        .unwrap_or(false)
}

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
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
///
/// Values compose from primitive layers through generic containers.
/// `List` and `Map` hold `Value` elements, so any nesting is expressible:
/// `List<Str>`, `Map<Str, Int>`, `List<List<Bool>>`, etc.
///
/// `Set` provides unordered, unique-element collection semantics distinct
/// from `List`. Set algebra (union, intersection, difference) is defined
/// as methods on `Value` and as `SetOp` in the collection primitives.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    /// Homogeneous list of values (ordered, allows duplicates)
    List(Vec<Value>),
    /// Unordered collection of unique values (set semantics)
    ///
    /// Internally stored as a `Vec<Value>` with uniqueness enforced at
    /// construction time via `PartialEq`. Order is not guaranteed.
    /// Use `Value::set()` or `Value::str_set()` to construct.
    Set(Vec<Value>),
    /// String-keyed map of values
    Map(BTreeMap<String, Value>),
    /// JSON value (for complex/dynamic data)
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
    // =========================================================================
    // Construction helpers (convenience for common compound types)
    // =========================================================================

    /// Create a list of strings.
    ///
    /// This is the compositional replacement for the old `StrList` variant.
    /// Equivalent to `Value::List(strings.into_iter().map(Value::Str).collect())`.
    pub fn str_list(strings: Vec<String>) -> Self {
        Value::List(strings.into_iter().map(Value::Str).collect())
    }

    /// Create a string-keyed map with string values.
    ///
    /// This is the compositional replacement for the old `MapStrStr` variant.
    /// Equivalent to `Value::Map(map.into_iter().map(|(k, v)| (k, Value::Str(v))).collect())`.
    pub fn str_map(map: BTreeMap<String, String>) -> Self {
        Value::Map(map.into_iter().map(|(k, v)| (k, Value::Str(v))).collect())
    }

    /// Create a set from a vector of values, deduplicating.
    ///
    /// Elements are deduplicated: if the same value appears multiple times,
    /// only the first occurrence is kept.
    pub fn set(values: Vec<Value>) -> Self {
        let mut seen = HashSet::new();
        let mut unique = Vec::with_capacity(values.len());
        for v in values {
            if let Ok(key) = serde_json::to_string(&v) {
                if seen.insert(key) {
                    unique.push(v);
                }
            }
        }
        Value::Set(unique)
    }

    /// Create a set of strings, deduplicating.
    pub fn str_set(strings: Vec<String>) -> Self {
        Self::set(strings.into_iter().map(Value::Str).collect())
    }

    // =========================================================================
    // Set algebra (language-agnostic operations)
    // =========================================================================

    /// Set union: elements in either self or other.
    ///
    /// Returns `None` if either value is not a `Set`.
    pub fn set_union(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Set(a), Value::Set(b)) => {
                let existing = value_index(a);
                let mut result = a.clone();
                for v in b {
                    if !in_index(v, &existing) {
                        result.push(v.clone());
                    }
                }
                Some(Value::Set(result))
            }
            _ => None,
        }
    }

    /// Set intersection: elements in both self and other.
    ///
    /// Returns `None` if either value is not a `Set`.
    pub fn set_intersection(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Set(a), Value::Set(b)) => {
                let b_idx = value_index(b);
                let result: Vec<Value> =
                    a.iter().filter(|v| in_index(v, &b_idx)).cloned().collect();
                Some(Value::Set(result))
            }
            _ => None,
        }
    }

    /// Set difference: elements in self but not in other.
    ///
    /// Returns `None` if either value is not a `Set`.
    pub fn set_difference(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Set(a), Value::Set(b)) => {
                let b_idx = value_index(b);
                let result: Vec<Value> =
                    a.iter().filter(|v| !in_index(v, &b_idx)).cloned().collect();
                Some(Value::Set(result))
            }
            _ => None,
        }
    }

    /// Symmetric difference: elements in either but not both.
    ///
    /// Returns `None` if either value is not a `Set`.
    pub fn set_symmetric_difference(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::Set(a), Value::Set(b)) => {
                let b_idx = value_index(b);
                let a_idx = value_index(a);
                let mut result: Vec<Value> =
                    a.iter().filter(|v| !in_index(v, &b_idx)).cloned().collect();
                for v in b {
                    if !in_index(v, &a_idx) {
                        result.push(v.clone());
                    }
                }
                Some(Value::Set(result))
            }
            _ => None,
        }
    }

    /// Check if self is a subset of other.
    ///
    /// Returns `None` if either value is not a `Set`.
    pub fn is_subset(&self, other: &Value) -> Option<bool> {
        match (self, other) {
            (Value::Set(a), Value::Set(b)) => {
                let b_idx = value_index(b);
                Some(a.iter().all(|v| in_index(v, &b_idx)))
            }
            _ => None,
        }
    }

    /// Check if a set contains a specific value.
    ///
    /// Returns `None` if self is not a `Set`.
    pub fn set_contains(&self, value: &Value) -> Option<bool> {
        match self {
            Value::Set(elements) => {
                let idx = value_index(elements);
                Some(in_index(value, &idx))
            }
            _ => None,
        }
    }

    // =========================================================================
    // Emptiness
    // =========================================================================

    /// Whether this value is semantically empty.
    ///
    /// Emptiness is defined per variant:
    /// - `Unit`, `Skipped` — always empty (no data / didn't run)
    /// - `Str` — empty string
    /// - `List` — zero elements
    /// - `Map` — zero entries
    /// - `Secret` — empty inner string
    /// - `Bool`, `Int`, `Json`, `Request`, `Response` — never empty
    ///   (they carry data by existence)
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Unit | Value::Skipped => true,
            Value::Str(s) => s.is_empty(),
            Value::List(v) | Value::Set(v) => v.is_empty(),
            Value::Map(m) => m.is_empty(),
            Value::Secret(s) => s.is_empty(),
            Value::Bool(_)
            | Value::Int(_)
            | Value::Json(_)
            | Value::Request(_)
            | Value::Response(_) => false,
        }
    }

    // =========================================================================
    // Type predicates
    // =========================================================================

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

    /// Check if this is a secret value.
    pub fn is_secret(&self) -> bool {
        matches!(self, Value::Secret(_))
    }

    /// Check if this is a set.
    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(_))
    }

    // =========================================================================
    // Scalar extraction
    // =========================================================================

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

    // =========================================================================
    // Collection extraction (generic)
    // =========================================================================

    /// Try to extract a list of values.
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    /// Try to extract a set of values.
    pub fn as_set(&self) -> Option<&[Value]> {
        match self {
            Value::Set(v) => Some(v),
            _ => None,
        }
    }

    /// Try to extract a map of values.
    pub fn as_map(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    // =========================================================================
    // Typed collection extraction (validates element types)
    // =========================================================================

    /// Try to extract a list of strings.
    ///
    /// Returns `Some` only if the value is a `List` where every element
    /// is a `Str`. Returns `None` for non-list values or lists containing
    /// non-string elements.
    pub fn as_str_list(&self) -> Option<Vec<String>> {
        match self {
            Value::List(items) => {
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        Value::Str(s) => result.push(s.clone()),
                        _ => return None,
                    }
                }
                Some(result)
            }
            _ => None,
        }
    }

    /// Try to extract a set of strings.
    ///
    /// Returns `Some` only if the value is a `Set` where every element
    /// is a `Str`. Returns `None` for non-set values or sets containing
    /// non-string elements.
    pub fn as_str_set(&self) -> Option<Vec<String>> {
        match self {
            Value::Set(items) => {
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        Value::Str(s) => result.push(s.clone()),
                        _ => return None,
                    }
                }
                Some(result)
            }
            _ => None,
        }
    }

    /// Try to extract a map from string to string.
    ///
    /// Returns `Some` only if the value is a `Map` where every value
    /// is a `Str`. Returns `None` for non-map values or maps containing
    /// non-string values.
    pub fn as_map_str_str(&self) -> Option<BTreeMap<String, String>> {
        match self {
            Value::Map(map) => {
                let mut result = BTreeMap::new();
                for (k, v) in map {
                    match v {
                        Value::Str(s) => {
                            result.insert(k.clone(), s.clone());
                        }
                        _ => return None,
                    }
                }
                Some(result)
            }
            _ => None,
        }
    }

    // =========================================================================
    // Transport extraction
    // =========================================================================

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

/// Manual PartialEq: all variants use derived equality except Set,
/// which uses order-independent comparison (set semantics).
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Unit, Value::Unit) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                let b_idx = value_index(b);
                a.iter().all(|v| in_index(v, &b_idx))
            }
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Json(a), Value::Json(b)) => a == b,
            (Value::Request(a), Value::Request(b)) => a == b,
            (Value::Response(a), Value::Response(b)) => a == b,
            (Value::Secret(a), Value::Secret(b)) => a == b,
            (Value::Skipped, Value::Skipped) => true,
            _ => false,
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
            Value::List(v) => write!(f, "[{} items]", v.len()),
            Value::Set(v) => write!(f, "{{{} items}}", v.len()),
            Value::Map(m) => write!(f, "{{{} entries}}", m.len()),
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
