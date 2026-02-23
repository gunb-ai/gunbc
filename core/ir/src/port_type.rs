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
                    return Ok(parse_known_primitive_type(primitive).expect("known primitive"));
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

/// Parse a known structural primitive type keyword.
fn parse_known_primitive_type(s: &str) -> Option<PortType> {
    match s {
        "Json" => Some(PortType::Json),
        "String" => Some(PortType::String),
        "Bytes" => Some(PortType::Bytes),
        "Bool" => Some(PortType::Bool),
        "Int" => Some(PortType::Int),
        "Float" => Some(PortType::Float),
        "Secret" => Some(PortType::Secret),
        "Any" => Some(PortType::Any),
        _ => None,
    }
}

/// Try to parse a type string into a structural PortType.
/// Returns `None` for unrecognized types (use `from_registry` for those).
pub fn try_parse_port_type(s: &str) -> Option<PortType> {
    match s {
        // Structural primitives
        s if parse_known_primitive_type(s).is_some() => parse_known_primitive_type(s),

        // Generic List<T>
        other if other.starts_with("List<") && other.ends_with('>') => {
            let inner = &other[5..other.len() - 1];
            let inner_type = try_parse_port_type(inner)?;
            Some(PortType::List(Box::new(inner_type)))
        }

        // Domain types — string-backed
        "FilePath" | "Path" | "TextFilePath"
        | "Url" | "Email" | "NonEmptyString" | "SecretName"
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
        // Credential is a compound type (secret + scheme) that serializes to
        // Value::Map via the capability-marker pattern, like ToolHandle.
        "TransportRequest" | "TransportResponse"
        | "FileResponse" | "ShellResponse" | "RestResponse" | "HttpResponse"
        | "ToolHandle" | "FilesystemHandle" | "NetworkHandle"
        | "Credential"
        | "CliResult" | "Record" => Some(PortType::Json),

        // Legacy aliases
        "StringList" | "UrlList" => Some(PortType::List(Box::new(PortType::String))),
        "OptionalString" => Some(PortType::String),

        _ => None,
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
            let back = try_parse_port_type(&type_id.0).expect("type should round-trip");
            assert_eq!(pt, &back, "round-trip failed for {pt}");
        }
    }

    #[test]
    fn parse_legacy_type_ids() {
        assert_eq!(
            try_parse_port_type("StringList").expect("StringList should parse"),
            PortType::List(Box::new(PortType::String))
        );
        assert_eq!(
            try_parse_port_type("OptionalString").expect("OptionalString should parse"),
            PortType::String
        );
    }

    #[test]
    fn domain_types_resolve_to_structural_backing() {
        // String-backed domain types
        assert_eq!(
            try_parse_port_type("FilePath").expect("FilePath should parse"),
            PortType::String
        );
        assert_eq!(
            try_parse_port_type("TextFilePath").expect("TextFilePath should parse"),
            PortType::String
        );
        assert_eq!(
            try_parse_port_type("Platform").expect("Platform should parse"),
            PortType::String
        );
        assert_eq!(
            try_parse_port_type("Url").expect("Url should parse"),
            PortType::String
        );
        assert_eq!(
            try_parse_port_type("GcpProjectId").expect("GcpProjectId should parse"),
            PortType::String
        );
        assert_eq!(
            try_parse_port_type("SecretName").expect("SecretName should parse"),
            PortType::String
        );

        // Bytes-backed domain types
        assert_eq!(
            try_parse_port_type("BinaryFilePath").expect("BinaryFilePath should parse"),
            PortType::Bytes
        );

        // Int-backed domain types
        assert_eq!(
            try_parse_port_type("Timestamp").expect("Timestamp should parse"),
            PortType::Int
        );

        // Credential is a compound type, not a scalar secret
        assert_eq!(
            try_parse_port_type("Credential").expect("Credential should parse"),
            PortType::Json
        );

        // Json/structured-backed domain types
        assert_eq!(
            try_parse_port_type("TransportRequest").expect("TransportRequest should parse"),
            PortType::Json
        );
        assert_eq!(
            try_parse_port_type("FileResponse").expect("FileResponse should parse"),
            PortType::Json
        );
        assert_eq!(
            try_parse_port_type("ToolHandle").expect("ToolHandle should parse"),
            PortType::Json
        );
        assert_eq!(
            try_parse_port_type("CliResult").expect("CliResult should parse"),
            PortType::Json
        );
    }

    #[test]
    fn unknown_type_is_not_structurally_parsed() {
        assert!(try_parse_port_type("SomeUnknownType").is_none());
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
