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
//!
//! Domain types are resolved to their structural backing types:
//! `FilePath` → `String`, `BinaryFilePath` → `Bytes`, `Credential` → `Json`, etc.

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
    /// Explicit `Any` type — only matches the literal `"Any"` TypeId string.
    /// No longer acts as a universal wildcard in compatibility checks.
    #[default]
    Any,
}

impl PortType {
    /// Whether two port types are structurally compatible for edge wiring.
    ///
    /// Strict equality — `Secret` is only compatible with `Secret`,
    /// `Any` is only compatible with `Any`. Use `TypeRegistry::is_compatible()`
    /// for full semantic type checking with coercion paths.
    pub fn is_compatible_with(&self, other: &PortType) -> bool {
        match (self, other) {
            _ if self == other => true,
            (PortType::List(a), PortType::List(b)) => a.is_compatible_with(b),
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

    /// Resolve a type string to a structural `PortType`, using the
    /// `TypeRegistry` for domain types not structurally recognized.
    ///
    /// Returns `Err` if the type string is not recognized by either
    /// structural parsing or the registry.
    pub fn from_registry(type_str: &str, registry: &crate::type_registry::TypeRegistry) -> Result<PortType, std::string::String> {
        // First try structural parse
        if let Some(pt) = try_parse_port_type(type_str) {
            return Ok(pt);
        }
        // Then check if the registry knows the type and can resolve its backing
        let type_id = crate::types::TypeId::from(type_str);
        if registry.get(&type_id).is_some() {
            // Type is registered — resolve to its structural backing via
            // coercion chain: find the nearest primitive ancestor.
            for primitive in &["String", "Int", "Float", "Bool", "Bytes", "Secret", "Json"] {
                let prim_id = crate::types::TypeId::from(*primitive);
                if registry.coercion_path(&type_id, &prim_id).is_some() {
                    return Ok(parse_known_type(primitive));
                }
            }
            // Registered but no coercion to a primitive — treat as Json (structured)
            return Ok(PortType::Json);
        }
        Err(format!("unrecognized type: `{type_str}` — not a structural type and not registered in TypeRegistry"))
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
/// Known domain types are resolved to their structural backing types.
/// Truly unrecognized type strings fall through to `PortType::Any`.
///
/// For strict resolution that rejects unknown types, use
/// `PortType::from_registry()` instead.
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

/// Parse a known structural type keyword. Always succeeds for the 8 primitives.
fn parse_known_type(s: &str) -> PortType {
    match s {
        "Json" => PortType::Json,
        "String" => PortType::String,
        "Bytes" => PortType::Bytes,
        "Bool" => PortType::Bool,
        "Int" => PortType::Int,
        "Float" => PortType::Float,
        "Secret" => PortType::Secret,
        "Any" => PortType::Any,
        _ => PortType::Any,
    }
}

/// Try to parse a type string into a structural PortType.
/// Returns `None` for unrecognized types (use `from_registry` for those).
fn try_parse_port_type(s: &str) -> Option<PortType> {
    match s {
        // Structural primitives
        "Json" => Some(PortType::Json),
        "String" => Some(PortType::String),
        "Bytes" => Some(PortType::Bytes),
        "Bool" => Some(PortType::Bool),
        "Int" => Some(PortType::Int),
        "Float" => Some(PortType::Float),
        "Secret" => Some(PortType::Secret),
        "Any" => Some(PortType::Any),

        // Generic List<T>
        other if other.starts_with("List<") && other.ends_with('>') => {
            let inner = &other[5..other.len() - 1];
            Some(PortType::List(Box::new(parse_port_type(inner))))
        }

        // Domain types — string-backed
        "FilePath" | "Path" | "TextFilePath"
        | "Url" | "Email" | "NonEmptyString"
        | "Platform" | "ContentEncoding"
        | "OidcAudience" | "WifAudience"
        | "GcpProjectId" | "GcpSecretId" | "GcpSecretVersion"
        | "GcpServiceAccountEmail" | "GcpSubjectToken" | "OidcSubjectToken" => {
            Some(PortType::String)
        }

        // Domain types — bytes-backed
        "BinaryFilePath" => Some(PortType::Bytes),

        // Domain types — int-backed
        "Timestamp" => Some(PortType::Int),

        // Domain types — json/structured-backed
        "Credential"
        | "TransportRequest" | "TransportResponse"
        | "FileResponse" | "ShellResponse" | "RestResponse" | "HttpResponse"
        | "ToolHandle" | "FilesystemHandle" | "NetworkHandle"
        | "CliResult" | "Record" => Some(PortType::Json),

        // Legacy aliases
        "StringList" | "UrlList" => Some(PortType::List(Box::new(PortType::String))),
        "OptionalString" => Some(PortType::String),

        _ => None,
    }
}

/// Full parse with fallback to `PortType::Any` for unrecognized types.
/// Prefer `try_parse_port_type` or `PortType::from_registry` for strict use.
fn parse_port_type(s: &str) -> PortType {
    try_parse_port_type(s).unwrap_or(PortType::Any)
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
    fn any_is_not_wildcard() {
        // Any only matches itself — no longer a universal wildcard
        assert!(PortType::Any.is_compatible_with(&PortType::Any));
        assert!(!PortType::Any.is_compatible_with(&PortType::String));
        assert!(!PortType::Int.is_compatible_with(&PortType::Any));
    }

    #[test]
    fn secret_is_strict() {
        assert!(!PortType::Secret.is_compatible_with(&PortType::String));
        assert!(!PortType::String.is_compatible_with(&PortType::Secret));
        assert!(!PortType::Secret.is_compatible_with(&PortType::Any));
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
        assert_eq!(PortType::from("OptionalString"), PortType::String);
    }

    #[test]
    fn domain_types_resolve_to_structural_backing() {
        // String-backed domain types
        assert_eq!(PortType::from("FilePath"), PortType::String);
        assert_eq!(PortType::from("TextFilePath"), PortType::String);
        assert_eq!(PortType::from("Platform"), PortType::String);
        assert_eq!(PortType::from("Url"), PortType::String);
        assert_eq!(PortType::from("GcpProjectId"), PortType::String);

        // Bytes-backed domain types
        assert_eq!(PortType::from("BinaryFilePath"), PortType::Bytes);

        // Int-backed domain types
        assert_eq!(PortType::from("Timestamp"), PortType::Int);

        // Json/structured-backed domain types (including Credential)
        assert_eq!(PortType::from("Credential"), PortType::Json);
        assert_eq!(PortType::from("TransportRequest"), PortType::Json);
        assert_eq!(PortType::from("FileResponse"), PortType::Json);
        assert_eq!(PortType::from("ToolHandle"), PortType::Json);
        assert_eq!(PortType::from("CliResult"), PortType::Json);
    }

    #[test]
    fn unknown_type_still_falls_back_to_any() {
        // Unrecognized types still get Any via parse_port_type
        assert_eq!(PortType::from("SomeUnknownType"), PortType::Any);
    }

    #[test]
    fn from_registry_resolves_registered_types() {
        let registry = crate::type_registry::TypeRegistry::with_core_types();
        // TextFilePath is registered and has coercion to String
        let pt = PortType::from_registry("TextFilePath", &registry);
        assert!(pt.is_ok());
        assert_eq!(pt.unwrap(), PortType::String);
    }

    #[test]
    fn from_registry_rejects_unknown_types() {
        let registry = crate::type_registry::TypeRegistry::with_core_types();
        let pt = PortType::from_registry("CompletelyFakeType", &registry);
        assert!(pt.is_err());
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
