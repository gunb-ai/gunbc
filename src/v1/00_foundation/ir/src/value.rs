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
use std::fmt::{self, Write as _};
use std::sync::Arc;

/// Shared maximum line count for human-readable text truncation.
pub const HUMAN_TEXT_MAX_LINES: usize = 40;
/// Shared maximum line width for human-readable text truncation.
pub const HUMAN_TEXT_MAX_LINE_WIDTH: usize = 500;

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

    /// Expose plaintext secret value for transport boundaries only.
    ///
    /// This API exists specifically for boundary adapters that must materialize
    /// plaintext for outbound requests.
    pub fn expose_plaintext_for_transport(&self) -> &str {
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

    /// Return an opt-in diagnostic view that reveals a suffix.
    pub fn hint(&self) -> SecretHint<'_> {
        SecretHint(self)
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

/// Opt-in diagnostic display that distinguishes empty from non-empty secrets.
///
/// Empty secrets show `""` — immediately obvious something is wrong.
/// Non-empty secrets show `"***"` — no partial character reveal.
pub struct SecretHint<'a>(&'a SecretString);

impl fmt::Display for SecretHint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.inner.is_empty() {
            write!(f, "\"\"")
        } else {
            write!(f, "\"***\"")
        }
    }
}

impl fmt::Debug for SecretHint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretHint(***)")
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
    /// Floating point
    Float(f64),
    /// Raw bytes (binary data)
    Bytes(Vec<u8>),
    /// Homogeneous list of values (ordered, allows duplicates).
    ///
    /// Wrapped in `Arc` for copy-on-write: `list_push` avoids cloning the
    /// entire Vec when refcount is 1 (the common single-owner case).
    List(Arc<Vec<Value>>),
    /// Unordered collection of unique values (set semantics).
    ///
    /// Wrapped in `Arc` for consistency with `List` (both are collections).
    /// Internally stored as a `Vec<Value>` with uniqueness enforced at
    /// construction time via `PartialEq`. Order is not guaranteed.
    /// Use `Value::set()` or `Value::str_set()` to construct.
    Set(Arc<Vec<Value>>),
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
    /// Typed enum variant (sum-type unit variant).
    ///
    /// `ty` is the enum type name (e.g., "TestClass"), and `variant` is the
    /// selected variant (e.g., "Hermetic").
    Enum { ty: String, variant: String },
    /// Node was skipped (guard evaluated to false)
    Skipped,
}

/// Explicit control flow for DAG execution.
///
/// Replaces the implicit dual-use of `Value::Skipped` (which conflates
/// "guard condition failed" with "missing/null"). Nodes that may skip
/// execution return `ControlFlow::Skipped` with an optional reason;
/// nodes that produce a value return `ControlFlow::Continue(value)`.
///
/// Migration: new code should use `ControlFlow` at node boundaries.
/// `Value::Skipped` remains for backward compat during incremental migration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlFlow {
    /// Node executed normally and produced a value.
    Continue(Box<Value>),
    /// Node was skipped (guard failed, branch not taken, etc.).
    Skipped { reason: Option<String> },
}

impl ControlFlow {
    /// Unwrap the value, panicking if skipped.
    pub fn unwrap(self) -> Value {
        match self {
            ControlFlow::Continue(v) => *v,
            ControlFlow::Skipped { reason } => panic!(
                "called unwrap() on Skipped: {}",
                reason.as_deref().unwrap_or("no reason")
            ),
        }
    }

    /// Get the value if Continue, None if Skipped.
    pub fn into_value(self) -> Option<Value> {
        match self {
            ControlFlow::Continue(v) => Some(*v),
            ControlFlow::Skipped { .. } => None,
        }
    }

    /// Whether this is a Continue variant.
    pub fn is_continue(&self) -> bool {
        matches!(self, ControlFlow::Continue(_))
    }

    /// Whether this is a Skipped variant.
    pub fn is_skipped(&self) -> bool {
        matches!(self, ControlFlow::Skipped { .. })
    }

    /// Convert from legacy `Value` (maps `Value::Skipped` to `ControlFlow::Skipped`).
    pub fn from_value(v: Value) -> Self {
        match v {
            Value::Skipped => ControlFlow::Skipped { reason: None },
            other => ControlFlow::Continue(Box::new(other)),
        }
    }

    /// Convert back to legacy `Value` (maps `Skipped` to `Value::Skipped`).
    pub fn into_legacy_value(self) -> Value {
        match self {
            ControlFlow::Continue(v) => *v,
            ControlFlow::Skipped { .. } => Value::Skipped,
        }
    }
}

/// Discriminant tag for [`Value`] variants.
///
/// Allows type-level dispatch without manufacturing strings.
/// Used by [`ValueBacking::accepts_value_kind`] for mock compatibility checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Unit,
    Bool,
    String,
    Int,
    Float,
    Bytes,
    List,
    Set,
    Map,
    Json,
    TransportRequest,
    TransportResponse,
    Secret,
    Enum,
    Skipped,
}

impl ValueKind {
    /// Canonical display name for diagnostics/error messages.
    pub fn type_name(self) -> &'static str {
        match self {
            ValueKind::Unit => "Unit",
            ValueKind::Bool => "Bool",
            ValueKind::String => "String",
            ValueKind::Int => "Int",
            ValueKind::Float => "Float",
            ValueKind::Bytes => "Bytes",
            ValueKind::List => "List",
            ValueKind::Set => "Set",
            ValueKind::Map => "Map",
            ValueKind::Json => "Json",
            ValueKind::TransportRequest => "TransportRequest",
            ValueKind::TransportResponse => "TransportResponse",
            ValueKind::Secret => "Secret",
            ValueKind::Enum => "Enum",
            ValueKind::Skipped => "Skipped",
        }
    }
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_name())
    }
}

impl Value {
    /// Returns the discriminant tag for this value.
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Unit => ValueKind::Unit,
            Value::Bool(_) => ValueKind::Bool,
            Value::Str(_) => ValueKind::String,
            Value::Int(_) => ValueKind::Int,
            Value::Float(_) => ValueKind::Float,
            Value::Bytes(_) => ValueKind::Bytes,
            Value::List(_) => ValueKind::List,
            Value::Set(_) => ValueKind::Set,
            Value::Map(_) => ValueKind::Map,
            Value::Json(_) => ValueKind::Json,
            Value::Request(_) => ValueKind::TransportRequest,
            Value::Response(_) => ValueKind::TransportResponse,
            Value::Secret(_) => ValueKind::Secret,
            Value::Enum { .. } => ValueKind::Enum,
            Value::Skipped => ValueKind::Skipped,
        }
    }

    // =========================================================================
    // Construction helpers (convenience for common compound types)
    // =========================================================================

    /// Create a list of strings.
    ///
    /// This is the compositional replacement for the old `StrList` variant.
    /// Equivalent to `Value::List(strings.into_iter().map(Value::Str).collect())`.
    pub fn str_list(strings: Vec<String>) -> Self {
        Value::List(Arc::new(strings.into_iter().map(Value::Str).collect()))
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
        Value::Set(Arc::new(unique))
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
                let mut result: Vec<Value> = (**a).clone();
                for v in b.iter() {
                    if !in_index(v, &existing) {
                        result.push(v.clone());
                    }
                }
                Some(Value::Set(Arc::new(result)))
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
                Some(Value::Set(Arc::new(result)))
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
                Some(Value::Set(Arc::new(result)))
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
                for v in b.iter() {
                    if !in_index(v, &a_idx) {
                        result.push(v.clone());
                    }
                }
                Some(Value::Set(Arc::new(result)))
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
            Value::Bytes(b) => b.is_empty(),
            Value::List(v) | Value::Set(v) => v.is_empty(),
            Value::Map(m) => m.is_empty(),
            Value::Secret(s) => s.is_empty(),
            Value::Enum { .. } => false,
            Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
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
                for item in items.iter() {
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
                for item in items.iter() {
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

    // =========================================================================
    // Display chokepoint (the single sanctioned path for human-visible text)
    // =========================================================================

    /// Render this value for human display, redacting secrets.
    ///
    /// This is the **single chokepoint** for all human-visible Value text.
    /// Delegates to the `Display` impl (which already redacts secrets).
    /// Prefer this over `format!("{}", value)` to make redaction intent explicit.
    pub fn display_redacted(&self) -> String {
        format!("{}", self)
    }

    /// Render this value for human display with truncation, redacting secrets.
    ///
    /// For `Str` values: truncates to `max_lines` lines, with individual lines
    /// capped at `max_line_width` characters. For all other variants: delegates
    /// to `display_redacted()` (which is already concise).
    pub fn display_redacted_truncated(&self, max_lines: usize, max_line_width: usize) -> String {
        match self {
            Value::Str(s) => {
                let lines: Vec<&str> = s.lines().collect();
                if lines.len() <= max_lines {
                    // Still truncate individual long lines
                    let truncated: Vec<String> = lines
                        .iter()
                        .map(|line| {
                            if line.len() > max_line_width {
                                format!("{}...", &line[..max_line_width])
                            } else {
                                (*line).to_string()
                            }
                        })
                        .collect();
                    return truncated.join("\n");
                }

                let head = 5.min(max_lines);
                let tail = max_lines.saturating_sub(head);
                let omitted = lines.len() - head - tail;

                let mut out = String::new();
                for line in &lines[..head] {
                    if line.len() > max_line_width {
                        out.push_str(&line[..max_line_width]);
                        out.push_str("...");
                    } else {
                        out.push_str(line);
                    }
                    out.push('\n');
                }
                let _ = write!(&mut out, "    ... ({omitted} lines omitted) ...");
                out.push('\n');
                for (i, line) in lines[lines.len() - tail..].iter().enumerate() {
                    if line.len() > max_line_width {
                        out.push_str(&line[..max_line_width]);
                        out.push_str("...");
                    } else {
                        out.push_str(line);
                    }
                    if i < tail - 1 {
                        out.push('\n');
                    }
                }
                out
            }
            _ => self.display_redacted(),
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
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
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
            (
                Value::Enum {
                    ty: a_ty,
                    variant: a_variant,
                },
                Value::Enum {
                    ty: b_ty,
                    variant: b_variant,
                },
            ) => a_ty == b_ty && a_variant == b_variant,
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
            Value::Float(v) => write!(f, "{v}"),
            Value::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Value::List(v) => write!(f, "[{} items]", v.len()),
            Value::Set(v) => write!(f, "{{{} items}}", v.len()),
            Value::Map(m) => write!(f, "{{{} entries}}", m.len()),
            Value::Json(j) => write!(f, "{}", j),
            Value::Request(r) => write!(f, "<Request: {:?}>", std::mem::discriminant(r)),
            Value::Response(r) => write!(f, "<Response: {:?}>", std::mem::discriminant(r)),
            Value::Secret(_) => write!(f, "***"),
            Value::Enum { ty, variant } => write!(f, "{ty}.{variant}"),
            Value::Skipped => write!(f, "<SKIPPED>"),
        }
    }
}

impl From<serde_json::Value> for Value {
    /// Convert a `serde_json::Value` to a native `Value` with proper types.
    ///
    /// Unlike `Value::Json(v)` which wraps JSON opaquely, this produces
    /// `Value::Map`, `Value::List`, `Value::Str`, etc. so field access,
    /// iteration, and pattern matching work natively.
    fn from(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Unit,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Str(n.to_string())
                }
            }
            serde_json::Value::String(s) => Value::Str(s),
            serde_json::Value::Array(arr) => {
                Value::List(Arc::new(arr.into_iter().map(Value::from).collect()))
            }
            serde_json::Value::Object(obj) => {
                let map = obj.into_iter().map(|(k, v)| (k, Value::from(v))).collect();
                Value::Map(map)
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_redacted_secret() {
        let v = Value::Secret(SecretString::new("top-secret"));
        assert_eq!(v.display_redacted(), "***");
    }

    #[test]
    fn display_redacted_str() {
        let v = Value::Str("hello".to_string());
        assert_eq!(v.display_redacted(), "hello");
    }

    #[test]
    fn display_redacted_bool() {
        assert_eq!(Value::Bool(true).display_redacted(), "true");
    }

    #[test]
    fn display_redacted_truncated_short_string() {
        let v = Value::Str("line1\nline2\nline3".to_string());
        let result = v.display_redacted_truncated(10, 500);
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn display_redacted_truncated_long_string() {
        let lines: Vec<String> = (0..20).map(|i| format!("line {i}")).collect();
        let v = Value::Str(lines.join("\n"));
        let result = v.display_redacted_truncated(5, 500);
        // Should have head (5) + omission marker + tail (0) = truncated
        assert!(result.contains("lines omitted"));
        assert!(result.contains("line 0"));
    }

    #[test]
    fn display_redacted_truncated_long_lines() {
        let long_line = "x".repeat(600);
        let v = Value::Str(long_line);
        let result = v.display_redacted_truncated(10, 100);
        assert!(result.len() < 200);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn display_redacted_truncated_secret() {
        let v = Value::Secret(SecretString::new("top-secret"));
        assert_eq!(v.display_redacted_truncated(10, 500), "***");
    }

    #[test]
    fn display_redacted_truncated_non_str() {
        let v = Value::Int(42);
        assert_eq!(v.display_redacted_truncated(10, 500), "42");
    }

    #[test]
    fn secret_string_format_paths_are_always_redacted() {
        let secret = SecretString::new("top-secret");
        assert_eq!(format!("{secret}"), "***");
        assert_eq!(format!("{secret:?}"), "SecretString(***)");
        assert_eq!(secret.to_string(), "***");
    }

    /// M7 regression: Value::Secret must never leak plaintext through any
    /// format path (Display, Debug, to_string).
    #[test]
    fn secret_value_display_and_debug_never_leak_plaintext() {
        let secret = SecretString::new("super-secret-token-12345");
        let value = Value::Secret(secret);

        // Display must redact
        let display = format!("{value}");
        assert!(
            !display.contains("super-secret"),
            "Display leaked plaintext: {display}"
        );
        assert_eq!(display, "***");

        // Debug must redact (derived Debug delegates to SecretString::Debug)
        let debug = format!("{value:?}");
        assert!(
            !debug.contains("super-secret"),
            "Debug leaked plaintext: {debug}"
        );

        // to_string() must redact (delegates to Display)
        let to_str = value.to_string();
        assert!(
            !to_str.contains("super-secret"),
            "to_string() leaked plaintext: {to_str}"
        );

        // display_redacted() must redact
        let redacted = value.display_redacted();
        assert!(
            !redacted.contains("super-secret"),
            "display_redacted() leaked plaintext: {redacted}"
        );
        assert_eq!(redacted, "***");

        // display_redacted_truncated() must redact
        let truncated = value.display_redacted_truncated(10, 500);
        assert!(
            !truncated.contains("super-secret"),
            "display_redacted_truncated() leaked plaintext: {truncated}"
        );
        assert_eq!(truncated, "***");
    }

    #[test]
    fn secret_hint_non_empty() {
        let s = SecretString::new("abc");
        assert_eq!(format!("{}", s.hint()), "\"***\"");
    }

    #[test]
    fn secret_hint_empty() {
        let s = SecretString::new("");
        assert_eq!(format!("{}", s.hint()), "\"\"");
    }

    #[test]
    fn secret_hint_long_secret() {
        let s = SecretString::new("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(format!("{}", s.hint()), "\"***\"");
    }

    #[test]
    fn secret_hint_preserves_display() {
        let s = SecretString::new("my-secret-token");
        assert_eq!(format!("{}", s), "***");
    }

    #[test]
    fn secret_hint_debug_redacted() {
        let s = SecretString::new("token123");
        assert_eq!(format!("{:?}", s.hint()), "SecretHint(***)");
    }

    /// M7 intentional: serde_json::to_string serializes plaintext for storage/wire.
    /// This is by design -- secrets must be materializable at transport boundaries.
    /// This test documents the deliberate choice rather than testing for redaction.
    #[test]
    fn secret_value_serde_serializes_plaintext_intentionally() {
        let secret = SecretString::new("storage-token-xyz");
        let value = Value::Secret(secret);
        let json = serde_json::to_string(&value).expect("serialization should succeed");
        // Serde DOES contain the plaintext -- this is intentional for persistence/transport.
        assert!(
            json.contains("storage-token-xyz"),
            "serde should serialize plaintext for storage: {json}"
        );
    }

    #[test]
    fn control_flow_from_value_maps_skipped() {
        let cf = ControlFlow::from_value(Value::Skipped);
        assert!(cf.is_skipped());
        assert_eq!(cf.into_legacy_value(), Value::Skipped);
    }

    #[test]
    fn control_flow_from_value_maps_continue() {
        let cf = ControlFlow::from_value(Value::Int(42));
        assert!(cf.is_continue());
        assert_eq!(cf.unwrap(), Value::Int(42));
    }

    #[test]
    fn control_flow_into_value_returns_none_for_skipped() {
        let cf = ControlFlow::Skipped {
            reason: Some("guard failed".into()),
        };
        assert!(cf.into_value().is_none());
    }

    #[test]
    fn control_flow_round_trip() {
        let original = Value::Str("hello".into());
        let cf = ControlFlow::from_value(original.clone());
        assert_eq!(cf.into_legacy_value(), original);
    }
}
