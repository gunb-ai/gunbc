//! Language-independent value representation for code generation.
//!
//! `ValueExpr` sits between the runtime `Value` type and target-language
//! source text. Converting `Value → ValueExpr` is total — every variant
//! is handled, no catch-all. Rendering `ValueExpr → String` is per-backend.

use crate::transport::{TransportRequest, TransportResponse};
use crate::Value;

/// Language-independent representation of a value literal.
///
/// This is the proto-like minimal type language: bool, i64, string, json,
/// list, map, struct, unit. No unsigned integers, no pointers, no generics.
/// Complexity goes into backends, not into the core.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    /// Absence of data — null/None/undefined/Unit per language.
    Unit,
    /// Boolean — true/false/True/False per language.
    Bool(bool),
    /// String — "hello" / 'hello' per language.
    Str(String),
    /// Signed 64-bit integer — single integer type, no unsigned.
    Int(i64),
    /// Homogeneous ordered collection — vec![...] / [...] per language.
    List(Vec<ValueExpr>),
    /// Dynamic string-keyed collection — BTreeMap / dict / Record per language.
    Map(Vec<(String, ValueExpr)>),
    /// Opaque JSON — serde_json::json!() / dict literal / plain object per language.
    Json(serde_json::Value),
    /// Named product type with statically known fields.
    /// Distinct from Map: backends render as struct / dataclass / interface.
    Struct {
        name: String,
        fields: Vec<(String, ValueExpr)>,
    },
    /// Secret/redacted value — rendered as a secret constructor per language.
    Secret(String),
    /// Node was skipped (guard false) — rendered as skip sentinel per language.
    Skipped,
}

// ===========================================================================
// Value → ValueExpr (total conversion — no catch-all)
// ===========================================================================

impl From<&Value> for ValueExpr {
    fn from(value: &Value) -> Self {
        match value {
            Value::Unit => ValueExpr::Unit,
            Value::Bool(b) => ValueExpr::Bool(*b),
            Value::Str(s) => ValueExpr::Str(s.clone()),
            Value::Int(i) => ValueExpr::Int(*i),
            Value::List(v) | Value::Set(v) => {
                ValueExpr::List(v.iter().map(ValueExpr::from).collect())
            }
            Value::Map(m) => ValueExpr::Map(
                m.iter()
                    .map(|(k, v)| (k.clone(), ValueExpr::from(v)))
                    .collect(),
            ),
            Value::Json(j) => ValueExpr::Json(j.clone()),
            Value::Request(r) => request_to_value_expr(r),
            Value::Response(r) => response_to_value_expr(r),
            Value::Float(f) => ValueExpr::Json(serde_json::json!(*f)),
            Value::Bytes(b) => ValueExpr::Json(serde_json::json!({"__bytes": b.len()})),
            Value::Secret(s) => ValueExpr::Secret(s.expose_plaintext_for_transport().to_string()),
            Value::Skipped => ValueExpr::Skipped,
        }
    }
}

impl From<Value> for ValueExpr {
    fn from(value: Value) -> Self {
        ValueExpr::from(&value)
    }
}

// ===========================================================================
// Transport → Struct conversion
// ===========================================================================

/// Convert an optional value to ValueExpr (Some → value, None → Unit).
fn opt_str(s: &Option<String>) -> ValueExpr {
    match s {
        Some(v) => ValueExpr::Str(v.clone()),
        None => ValueExpr::Unit,
    }
}

/// Convert a HashMap<String, String> to a sorted ValueExpr::Map.
fn str_map_expr(m: &std::collections::HashMap<String, String>) -> ValueExpr {
    let mut entries: Vec<_> = m
        .iter()
        .map(|(k, v)| (k.clone(), ValueExpr::Str(v.clone())))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    ValueExpr::Map(entries)
}

fn request_to_value_expr(req: &TransportRequest) -> ValueExpr {
    match req {
        TransportRequest::Shell(s) => ValueExpr::Struct {
            name: "TransportRequest::Shell".to_string(),
            fields: vec![
                ("command".to_string(), ValueExpr::Str(s.command.clone())),
                (
                    "args".to_string(),
                    ValueExpr::List(s.args.iter().map(|a| ValueExpr::Str(a.clone())).collect()),
                ),
                ("env".to_string(), str_map_expr(&s.env)),
                ("cwd".to_string(), opt_str(&s.cwd)),
                ("stdin".to_string(), opt_str(&s.stdin)),
                (
                    "timeout_ms".to_string(),
                    match s.timeout_ms {
                        Some(ms) => ValueExpr::Int(ms as i64),
                        None => ValueExpr::Unit,
                    },
                ),
                ("passthrough".to_string(), ValueExpr::Bool(s.passthrough)),
            ],
        },
        TransportRequest::Rest(r) => ValueExpr::Struct {
            name: "TransportRequest::Rest".to_string(),
            fields: vec![
                (
                    "method".to_string(),
                    ValueExpr::Str(format!("{:?}", r.method)),
                ),
                ("url".to_string(), ValueExpr::Str(r.url.clone())),
                ("headers".to_string(), str_map_expr(&r.headers)),
                (
                    "body".to_string(),
                    match &r.body {
                        Some(j) => ValueExpr::Json(j.clone()),
                        None => ValueExpr::Unit,
                    },
                ),
                ("query".to_string(), str_map_expr(&r.query)),
            ],
        },
        TransportRequest::Http(h) => ValueExpr::Struct {
            name: "TransportRequest::Http".to_string(),
            fields: vec![
                (
                    "method".to_string(),
                    ValueExpr::Str(format!("{:?}", h.method)),
                ),
                ("url".to_string(), ValueExpr::Str(h.url.clone())),
                ("headers".to_string(), str_map_expr(&h.headers)),
                ("body".to_string(), opt_str(&h.body)),
            ],
        },
        TransportRequest::File(f) => ValueExpr::Struct {
            name: "TransportRequest::File".to_string(),
            fields: vec![
                ("path".to_string(), ValueExpr::Str(f.path.clone())),
                (
                    "operation".to_string(),
                    ValueExpr::Str(format!("{:?}", f.operation)),
                ),
                ("content".to_string(), opt_str(&f.content)),
                (
                    "create_parents".to_string(),
                    ValueExpr::Bool(f.create_parents),
                ),
            ],
        },
        TransportRequest::Tcp(t) => ValueExpr::Struct {
            name: "TransportRequest::Tcp".to_string(),
            fields: vec![
                ("host".to_string(), ValueExpr::Str(t.host.clone())),
                ("port".to_string(), ValueExpr::Int(t.port as i64)),
                ("data".to_string(), opt_str(&t.data)),
            ],
        },
    }
}

fn response_to_value_expr(resp: &TransportResponse) -> ValueExpr {
    match resp {
        TransportResponse::Shell(s) => ValueExpr::Struct {
            name: "TransportResponse::Shell".to_string(),
            fields: vec![
                ("exit_code".to_string(), ValueExpr::Int(s.exit_code as i64)),
                ("stdout".to_string(), ValueExpr::Str(s.stdout.clone())),
                ("stderr".to_string(), ValueExpr::Str(s.stderr.clone())),
            ],
        },
        TransportResponse::Rest(r) => ValueExpr::Struct {
            name: "TransportResponse::Rest".to_string(),
            fields: vec![
                ("status".to_string(), ValueExpr::Int(r.status as i64)),
                ("body".to_string(), ValueExpr::Json(r.body.clone())),
                ("headers".to_string(), str_map_expr(&r.headers)),
            ],
        },
        TransportResponse::Http(h) => ValueExpr::Struct {
            name: "TransportResponse::Http".to_string(),
            fields: vec![
                ("status".to_string(), ValueExpr::Int(h.status as i64)),
                ("body".to_string(), ValueExpr::Str(h.body.clone())),
                ("headers".to_string(), str_map_expr(&h.headers)),
            ],
        },
        TransportResponse::File(f) => ValueExpr::Struct {
            name: "TransportResponse::File".to_string(),
            fields: vec![
                ("path".to_string(), ValueExpr::Str(f.path.clone())),
                (
                    "operation".to_string(),
                    ValueExpr::Str(format!("{:?}", f.operation)),
                ),
                ("success".to_string(), ValueExpr::Bool(f.success)),
                ("content".to_string(), opt_str(&f.content)),
                ("bytes".to_string(), ValueExpr::Unit),
                (
                    "exists".to_string(),
                    match f.exists {
                        Some(b) => ValueExpr::Bool(b),
                        None => ValueExpr::Unit,
                    },
                ),
                ("error".to_string(), opt_str(&f.error)),
            ],
        },
        TransportResponse::Tcp(t) => ValueExpr::Struct {
            name: "TransportResponse::Tcp".to_string(),
            fields: vec![
                ("connected".to_string(), ValueExpr::Bool(t.connected)),
                ("data".to_string(), opt_str(&t.data)),
                (
                    "bytes_sent".to_string(),
                    ValueExpr::Int(t.bytes_sent as i64),
                ),
                (
                    "bytes_received".to_string(),
                    ValueExpr::Int(t.bytes_received as i64),
                ),
                ("error".to_string(), opt_str(&t.error)),
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn value_to_value_expr_is_total() {
        // Every Value variant must convert without panic.
        let values = vec![
            Value::Unit,
            Value::Bool(true),
            Value::Str("hello".into()),
            Value::Int(42),
            Value::List(vec![Value::Int(1), Value::Bool(true)]),
            Value::Map(BTreeMap::from([("k".into(), Value::Int(1))])),
            Value::Json(serde_json::json!({"a": 1})),
            Value::Skipped,
        ];
        for v in &values {
            let _expr = ValueExpr::from(v);
        }
    }

    #[test]
    fn list_preserves_heterogeneous_elements() {
        let v = Value::List(vec![
            Value::Int(1),
            Value::Str("two".into()),
            Value::Bool(true),
        ]);
        let expr = ValueExpr::from(&v);
        assert_eq!(
            expr,
            ValueExpr::List(vec![
                ValueExpr::Int(1),
                ValueExpr::Str("two".into()),
                ValueExpr::Bool(true),
            ])
        );
    }

    #[test]
    fn map_preserves_all_entries() {
        let mut m = BTreeMap::new();
        m.insert("a".into(), Value::Int(1));
        m.insert("b".into(), Value::Bool(false));
        let v = Value::Map(m);
        let expr = ValueExpr::from(&v);
        assert_eq!(
            expr,
            ValueExpr::Map(vec![
                ("a".into(), ValueExpr::Int(1)),
                ("b".into(), ValueExpr::Bool(false)),
            ])
        );
    }
}
