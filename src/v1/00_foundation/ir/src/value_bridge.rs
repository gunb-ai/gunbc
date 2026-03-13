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
        Value::Enum { .. } => ValueCategory::Shared,
        Value::Map(_) | Value::Set(_) | Value::Request(_) | Value::Response(_) | Value::Skipped => {
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
        Value::Enum { ty, variant } => Some(serde_json::json!({
            "__enum": {
                "ty": ty,
                "variant": variant
            }
        })),
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
        serde_json::Value::Object(obj) => {
            if let Some(enum_obj) = obj.get("__enum").and_then(|v| v.as_object()) {
                let ty = enum_obj
                    .get("ty")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let variant = enum_obj
                    .get("variant")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                Value::Enum { ty, variant }
            } else {
                Value::Json(json.clone())
            }
        }
    }
}

/// Known primitive type IDs that should NOT be treated as enum types.
const PRIMITIVE_TYPE_IDS: &[&str] = &[
    "String", "Int", "Bool", "Unit", "Float", "Secret", "Json", "Bytes", "Any", "List", "Map",
    "Option",
];

/// Type-aware JSON deserialization: uses the expected `type_id` to reconstruct
/// typed values (enums, bytes) without relying on magic keys like `__enum`.
///
/// When `type_id` indicates a non-primitive type and JSON is a string,
/// reconstructs `Value::Enum { ty, variant }` directly.
/// When `type_id` is "Bytes" and JSON is an array of numbers, reconstructs
/// `Value::Bytes`.
///
/// Falls through to `from_bridge_json` for primitive types or when the
/// JSON shape doesn't match the expected type.
pub fn from_bridge_json_typed(json: &serde_json::Value, type_id: &str) -> Value {
    // Strip optional wrapper upfront: "Foo?" → ("Foo", true), "Option<Foo>" → ("Foo", true).
    let (inner_type, is_optional) = strip_optional_wrapper(type_id);

    // Handle null for optional types.
    if is_optional && json.is_null() {
        return Value::Unit;
    }

    // Strip generic wrappers (e.g., "List<Foo>" → check inner type separately).
    let base_type = inner_type.split('<').next().unwrap_or(inner_type);

    if base_type == "Bytes" {
        return match json {
            serde_json::Value::Array(arr) => {
                let bytes: Vec<u8> = arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                Value::Bytes(bytes)
            }
            serde_json::Value::String(s) => {
                // Base64 or raw string → bytes.
                Value::Bytes(s.as_bytes().to_vec())
            }
            // Legacy { "__bytes": N } format — reconstruct empty vec with length info.
            serde_json::Value::Object(obj) if obj.contains_key("__bytes") => {
                let len = obj.get("__bytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                Value::Bytes(vec![0; len])
            }
            _ => from_bridge_json(json),
        };
    }

    // For non-primitive types, a JSON string is likely an enum variant name.
    // Use the stripped base_type (not original type_id) for the enum's ty field.
    if !PRIMITIVE_TYPE_IDS.contains(&base_type) {
        match json {
            serde_json::Value::String(s) => {
                return Value::Enum {
                    ty: base_type.to_string(),
                    variant: s.clone(),
                };
            }
            // Legacy __enum format — still supported.
            serde_json::Value::Object(obj) if obj.contains_key("__enum") => {
                return from_bridge_json(json);
            }
            _ => {}
        }
    }

    // For generic containers, recurse with inner type.
    if let Some(list_inner) = inner_type
        .strip_prefix("List<")
        .and_then(|s| s.strip_suffix('>'))
    {
        if let serde_json::Value::Array(arr) = json {
            return Value::List(
                arr.iter()
                    .map(|item| from_bridge_json_typed(item, list_inner))
                    .collect(),
            );
        }
    }

    // Map<K,V> generic container support.
    if let Some(map_inner) = inner_type
        .strip_prefix("Map<")
        .and_then(|s| s.strip_suffix('>'))
    {
        if let serde_json::Value::Object(obj) = json {
            // Parse "K, V" — find the comma that's not inside angle brackets.
            if let Some(value_type) = split_map_type_params(map_inner) {
                let mut map = std::collections::BTreeMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), from_bridge_json_typed(v, value_type));
                }
                return Value::Map(map);
            }
        }
    }

    from_bridge_json(json)
}

/// Strip optional wrapper from a type string.
/// Returns (inner_type, is_optional).
fn strip_optional_wrapper(type_id: &str) -> (&str, bool) {
    if let Some(inner) = type_id.strip_suffix('?') {
        (inner, true)
    } else if let Some(inner) = type_id
        .strip_prefix("Optional<")
        .and_then(|s| s.strip_suffix('>'))
    {
        (inner, true)
    } else if let Some(inner) = type_id
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        // Legacy Rust-style "Option<T>" for backward compatibility.
        (inner, true)
    } else {
        (type_id, false)
    }
}

/// Split Map type params "K, V" at the top-level comma, returning the value type.
fn split_map_type_params(params: &str) -> Option<&str> {
    let mut depth = 0;
    for (i, c) in params.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let value_part = params[i + 1..].trim();
                return Some(value_part);
            }
            _ => {}
        }
    }
    None
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

    // C30: Type-aware bridging tests.

    #[test]
    fn typed_bridge_enum_from_string() {
        let json = serde_json::json!("GET");
        let val = from_bridge_json_typed(&json, "HttpMethod");
        assert_eq!(
            val,
            Value::Enum {
                ty: "HttpMethod".to_string(),
                variant: "GET".to_string()
            }
        );
    }

    #[test]
    fn typed_bridge_enum_legacy_format() {
        let json = serde_json::json!({"__enum": {"ty": "HttpMethod", "variant": "POST"}});
        let val = from_bridge_json_typed(&json, "HttpMethod");
        assert_eq!(
            val,
            Value::Enum {
                ty: "HttpMethod".to_string(),
                variant: "POST".to_string()
            }
        );
    }

    #[test]
    fn typed_bridge_bytes_from_array() {
        let json = serde_json::json!([72, 101, 108, 108, 111]);
        let val = from_bridge_json_typed(&json, "Bytes");
        assert_eq!(val, Value::Bytes(vec![72, 101, 108, 108, 111]));
    }

    #[test]
    fn typed_bridge_bytes_from_string() {
        let json = serde_json::json!("Hello");
        let val = from_bridge_json_typed(&json, "Bytes");
        assert_eq!(val, Value::Bytes(b"Hello".to_vec()));
    }

    #[test]
    fn typed_bridge_bytes_legacy_format() {
        let json = serde_json::json!({"__bytes": 5});
        let val = from_bridge_json_typed(&json, "Bytes");
        assert_eq!(val, Value::Bytes(vec![0; 5]));
    }

    #[test]
    fn typed_bridge_primitives_unchanged() {
        assert_eq!(
            from_bridge_json_typed(&serde_json::json!(42), "Int"),
            Value::Int(42)
        );
        assert_eq!(
            from_bridge_json_typed(&serde_json::json!(true), "Bool"),
            Value::Bool(true)
        );
        assert_eq!(
            from_bridge_json_typed(&serde_json::json!("hello"), "String"),
            Value::Str("hello".to_string())
        );
        assert_eq!(
            from_bridge_json_typed(&serde_json::Value::Null, "Unit"),
            Value::Unit
        );
    }

    #[test]
    fn typed_bridge_list_of_enums() {
        let json = serde_json::json!(["GET", "POST", "DELETE"]);
        let val = from_bridge_json_typed(&json, "List<HttpMethod>");
        assert_eq!(
            val,
            Value::List(vec![
                Value::Enum {
                    ty: "HttpMethod".to_string(),
                    variant: "GET".to_string()
                },
                Value::Enum {
                    ty: "HttpMethod".to_string(),
                    variant: "POST".to_string()
                },
                Value::Enum {
                    ty: "HttpMethod".to_string(),
                    variant: "DELETE".to_string()
                },
            ])
        );
    }
}
