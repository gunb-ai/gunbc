//! Value bridge for cross-repo compatibility with the-gunbai.
//!
//! the-gunbai's `gunbai-types::Value` and gunbc's `Value` overlap in
//! primitives but diverge in domain-specific variants:
//!
//! | gunbc | gunbai | Notes |
//! |---|---|---|
//! | `Unit` | `Null` | Same semantics |
//! | `Bool(bool)` | `Bool(bool)` | Identical |
//! | `Str(String)` | `String(String)` | Same, different variant name |
//! | `Int(i64)` | `Int(i64)` | Identical |
//! | `List(Vec<Value>)` | `List(Vec<Value>)` | Same structure |
//! | `Map(BTreeMap<..>)` | — | gunbc-only (no gunbai equivalent) |
//! | `Set(Vec<Value>)` | — | gunbc-only |
//! | `Json(serde_json::Value)` | `Json(serde_json::Value)` | Identical |
//! | — | `Float(f64)` | gunbai-only |
//! | — | `Bytes(Vec<u8>)` | gunbai-only |
//! | `Secret(SecretString)` | `Secret(SecretRef)` | Same intent, different representation |
//! | `Request(..)` | — | gunbc I/O boundary |
//! | `Response(..)` | — | gunbc I/O boundary |
//! | `Skipped` | — | gunbc control flow |
//! | — | `Artifact(ArtifactRef)` | gunbai content-addressed storage |
//! | — | `Capability(..)` | gunbai typed capability handles |
//!
//! This module provides classification of values into cross-repo categories
//! and lossy conversion helpers. Full round-trip fidelity is not possible
//! (e.g., gunbai `Artifact` has no gunbc equivalent), but the common
//! primitive subset converts losslessly.

use crate::value::Value;

/// Classification of a Value variant for cross-repo bridging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCategory {
    /// Shared between gunbc and gunbai — converts losslessly.
    Shared,
    /// gunbc-specific — no gunbai equivalent.
    GunbcOnly,
}

/// Classify whether a value has a gunbai equivalent.
pub fn classify_value(value: &Value) -> ValueCategory {
    match value {
        Value::Unit | Value::Bool(_) | Value::Str(_) | Value::Int(_) | Value::List(_) => {
            ValueCategory::Shared
        }
        Value::Json(_) | Value::Secret(_) | Value::Float(_) | Value::Bytes(_) => {
            ValueCategory::Shared
        }
        Value::Map(_) | Value::Set(_) | Value::Request(_) | Value::Response(_)
        | Value::Enum { .. } | Value::Skipped => {
            ValueCategory::GunbcOnly
        }
    }
}

/// Convert a gunbc Value to a JSON representation compatible with gunbai's
/// `serde_json::Value` wire format.
///
/// This is the recommended serialization path for cross-repo data exchange:
/// gunbc Value → JSON → gunbai Value (via serde).
///
/// Returns `None` for gunbc-only variants that have no JSON representation
/// (Request, Response, Skipped).
pub fn to_bridge_json(value: &Value) -> Option<serde_json::Value> {
    match value {
        Value::Unit => Some(serde_json::Value::Null),
        Value::Bool(b) => Some(serde_json::Value::Bool(*b)),
        Value::Str(s) => Some(serde_json::Value::String(s.clone())),
        Value::Int(i) => Some(serde_json::json!(*i)),
        Value::List(items) => {
            let converted: Option<Vec<serde_json::Value>> =
                items.iter().map(to_bridge_json).collect();
            converted.map(serde_json::Value::Array)
        }
        Value::Set(items) => {
            let converted: Option<Vec<serde_json::Value>> =
                items.iter().map(to_bridge_json).collect();
            converted.map(serde_json::Value::Array)
        }
        Value::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), to_bridge_json(v)?);
            }
            Some(serde_json::Value::Object(obj))
        }
        Value::Json(j) => Some(j.clone()),
        Value::Float(f) => Some(serde_json::json!(*f)),
        Value::Bytes(b) => Some(serde_json::json!({
            "__bytes": b.len(),
        })),
        Value::Secret(_) => Some(serde_json::Value::String("***".to_string())),
        Value::Enum { variant } => Some(serde_json::Value::String(variant.clone())),
        Value::Request(_) | Value::Response(_) | Value::Skipped => None,
    }
}

/// Convert a JSON value (from gunbai wire format) back to a gunbc Value.
///
/// This is a best-effort conversion — some gunbai-specific types
/// (Artifact, Capability, Float, Bytes) are represented as their JSON
/// forms and converted to the closest gunbc equivalent.
pub fn from_bridge_json(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Unit,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Json(json.clone())
            }
        }
        serde_json::Value::Array(arr) => Value::List(arr.iter().map(from_bridge_json).collect()),
        serde_json::Value::Object(_) => Value::Json(json.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn classify_shared_values() {
        assert_eq!(classify_value(&Value::Unit), ValueCategory::Shared);
        assert_eq!(classify_value(&Value::Bool(true)), ValueCategory::Shared);
        assert_eq!(
            classify_value(&Value::Str("hello".into())),
            ValueCategory::Shared
        );
        assert_eq!(classify_value(&Value::Int(42)), ValueCategory::Shared);
        assert_eq!(classify_value(&Value::List(vec![])), ValueCategory::Shared);
        assert_eq!(
            classify_value(&Value::Json(serde_json::json!({}))),
            ValueCategory::Shared
        );
    }

    #[test]
    fn classify_gunbc_only_values() {
        assert_eq!(
            classify_value(&Value::Map(BTreeMap::new())),
            ValueCategory::GunbcOnly
        );
        assert_eq!(
            classify_value(&Value::Set(vec![])),
            ValueCategory::GunbcOnly
        );
        assert_eq!(classify_value(&Value::Skipped), ValueCategory::GunbcOnly);
    }

    #[test]
    fn bridge_json_round_trip_primitives() {
        let cases = vec![
            Value::Unit,
            Value::Bool(true),
            Value::Str("hello".into()),
            Value::Int(42),
        ];
        for val in cases {
            let json = to_bridge_json(&val).expect("should convert");
            let back = from_bridge_json(&json);
            assert_eq!(format!("{val:?}"), format!("{back:?}"), "round-trip failed");
        }
    }

    #[test]
    fn bridge_json_list() {
        let val = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let json = to_bridge_json(&val).unwrap();
        assert_eq!(json, serde_json::json!([1, 2, 3]));
        let back = from_bridge_json(&json);
        assert_eq!(
            back,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn bridge_json_map() {
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), Value::Str("value".into()));
        let val = Value::Map(map);
        let json = to_bridge_json(&val).unwrap();
        assert_eq!(json, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn bridge_json_skipped_returns_none() {
        assert!(to_bridge_json(&Value::Skipped).is_none());
    }

    #[test]
    fn bridge_json_secret_redacted() {
        let val = Value::Secret(crate::value::SecretString::new("s3cret"));
        let json = to_bridge_json(&val).unwrap();
        assert_eq!(json, serde_json::Value::String("***".to_string()));
    }
}
