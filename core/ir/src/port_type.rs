//! Structural port type enum aligned with the-gunbai's `PortType`.
//!
//! gunbc uses opaque `TypeId` strings for port types. the-gunbai uses a
//! structural `PortType` enum. This module provides the structural enum
//! and bidirectional conversion with `TypeId`, enabling both repos to
//! reason about port types in a compatible way.
//!
//! # Conversion rules
//!
//! | `PortType` | `TypeId` string |
//! |---|---|
//! | `Json` | `"Json"` |
//! | `String` | `"String"` |
//! | `Bytes` | `"Bytes"` |
//! | `Bool` | `"Bool"` |
//! | `Int` | `"Int"` |
//! | `Float` | `"Float"` |
//! | `List(inner)` | `"List<{inner}>"` |
//! | `Secret` | `"Secret"` |
//! | `Any` | `"Any"` |

use serde::{Deserialize, Serialize};

use crate::types::TypeId;

/// Structural port type classification.
///
/// Mirrors `gunbai-types::PortType` from the-gunbai for cross-repo compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PortType {
    /// JSON-serializable data.
    Json,
    /// Plain string.
    String,
    /// Raw bytes.
    Bytes,
    /// Boolean.
    Bool,
    /// Integer.
    Int,
    /// Floating point.
    Float,
    /// List of items (for Scatter).
    List(Box<PortType>),
    /// Secret value — always sensitive, never logged.
    Secret,
    /// Any type (wildcard).
    #[default]
    Any,
}

impl PortType {
    /// Whether two port types are compatible for edge wiring.
    ///
    /// `Any` is compatible with everything. `Secret` is strict —
    /// only compatible with `Secret` or `Any`.
    pub fn is_compatible_with(&self, other: &PortType) -> bool {
        match (self, other) {
            _ if self == other => true,
            (PortType::Any, _) | (_, PortType::Any) => true,
            (PortType::List(a), PortType::List(b)) => a.is_compatible_with(b),
            (PortType::Secret, _) | (_, PortType::Secret) => false,
            _ => false,
        }
    }

    /// Whether this type represents sensitive data.
    pub fn is_sensitive(&self) -> bool {
        matches!(self, PortType::Secret)
    }

    /// Convert this structural type to a `TypeId` string.
    pub fn to_type_id(&self) -> TypeId {
        TypeId::new(self.to_type_id_string())
    }

    fn to_type_id_string(&self) -> std::string::String {
        match self {
            PortType::Json => "Json".to_string(),
            PortType::String => "String".to_string(),
            PortType::Bytes => "Bytes".to_string(),
            PortType::Bool => "Bool".to_string(),
            PortType::Int => "Int".to_string(),
            PortType::Float => "Float".to_string(),
            PortType::List(inner) => format!("List<{}>", inner.to_type_id_string()),
            PortType::Secret => "Secret".to_string(),
            PortType::Any => "Any".to_string(),
        }
    }
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_type_id_string())
    }
}

/// Parse a `TypeId` string into a structural `PortType`.
///
/// Returns `PortType::Any` for unrecognized type strings (fail-open).
///
/// This is an intentional forward-compatibility choice at the type-parsing
/// boundary: domain-specific types like `"TransportRequest"`, `"ToolRegistry"`,
/// etc. are opaque to structural port typing and map to `Any` so that port
/// wiring doesn't fail on types that only the runtime understands. The
/// tradeoff is that typos in type strings won't be caught here — they
/// silently become `Any` and pass compatibility checks.
///
/// See also: `value_backing_for_type_id()` in `types.rs` which adds a
/// second layer of domain-specific recognition for `PortType::Any` types.
impl From<&TypeId> for PortType {
    fn from(type_id: &TypeId) -> Self {
        parse_port_type(&type_id.0)
    }
}

impl From<&str> for PortType {
    fn from(s: &str) -> Self {
        parse_port_type(s)
    }
}

fn parse_port_type(s: &str) -> PortType {
    match s {
        "Json" => PortType::Json,
        "String" => PortType::String,
        "Bytes" => PortType::Bytes,
        "Bool" => PortType::Bool,
        "Int" => PortType::Int,
        "Float" => PortType::Float,
        "Secret" => PortType::Secret,
        "Any" => PortType::Any,
        other if other.starts_with("List<") && other.ends_with('>') => {
            let inner = &other[5..other.len() - 1];
            PortType::List(Box::new(parse_port_type(inner)))
        }
        // Legacy gunbc TypeId strings
        "StringList" => PortType::List(Box::new(PortType::String)),
        "UrlList" => PortType::List(Box::new(PortType::String)),
        "Unit" | "Void" => PortType::Any,
        "OptionalString" => PortType::String,
        _ => PortType::Any,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_same_type() {
        assert!(PortType::String.is_compatible_with(&PortType::String));
        assert!(PortType::Int.is_compatible_with(&PortType::Int));
    }

    #[test]
    fn any_matches_everything() {
        assert!(PortType::Any.is_compatible_with(&PortType::String));
        assert!(PortType::Int.is_compatible_with(&PortType::Any));
    }

    #[test]
    fn secret_is_strict() {
        assert!(!PortType::Secret.is_compatible_with(&PortType::String));
        assert!(!PortType::String.is_compatible_with(&PortType::Secret));
        assert!(PortType::Secret.is_compatible_with(&PortType::Any));
        assert!(PortType::Secret.is_compatible_with(&PortType::Secret));
    }

    #[test]
    fn list_compatibility() {
        let list_str = PortType::List(Box::new(PortType::String));
        let list_int = PortType::List(Box::new(PortType::Int));
        assert!(list_str.is_compatible_with(&list_str));
        assert!(!list_str.is_compatible_with(&list_int));
    }

    #[test]
    fn round_trip_type_id() {
        let cases = [
            PortType::Json,
            PortType::String,
            PortType::Bytes,
            PortType::Bool,
            PortType::Int,
            PortType::Float,
            PortType::Secret,
            PortType::Any,
            PortType::List(Box::new(PortType::String)),
            PortType::List(Box::new(PortType::List(Box::new(PortType::Int)))),
        ];
        for pt in &cases {
            let type_id = pt.to_type_id();
            let back: PortType = PortType::from(&type_id);
            assert_eq!(pt, &back, "round-trip failed for {pt}");
        }
    }

    #[test]
    fn parse_legacy_type_ids() {
        assert_eq!(
            PortType::from("StringList"),
            PortType::List(Box::new(PortType::String))
        );
        assert_eq!(PortType::from("Unit"), PortType::Any);
        assert_eq!(PortType::from("Void"), PortType::Any);
        assert_eq!(PortType::from("OptionalString"), PortType::String);
    }

    #[test]
    fn unknown_type_id_maps_to_any() {
        assert_eq!(PortType::from("TransportRequest"), PortType::Any);
        assert_eq!(PortType::from("ToolRegistry"), PortType::Any);
    }

    #[test]
    fn display() {
        assert_eq!(PortType::String.to_string(), "String");
        assert_eq!(
            PortType::List(Box::new(PortType::Int)).to_string(),
            "List<Int>"
        );
    }

    #[test]
    fn sensitive() {
        assert!(PortType::Secret.is_sensitive());
        assert!(!PortType::String.is_sensitive());
    }

    #[test]
    fn serde_round_trip() {
        let pt = PortType::List(Box::new(PortType::String));
        let json = serde_json::to_string(&pt).unwrap();
        let back: PortType = serde_json::from_str(&json).unwrap();
        assert_eq!(pt, back);
    }
}
