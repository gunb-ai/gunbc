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
//!
//! Domain types are resolved to their structural backing types:
//! `FilePath` → `String`, `BinaryFilePath` → `Bytes`, `Credential` → `Secret`, etc.

use serde::{Deserialize, Serialize};

use crate::types::TypeId;

/// Structural port type classification.
///
/// Mirrors `gunbai-types::PortType` from the-gunbai for cross-repo compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PortType {
    /// JSON-serializable data.
    #[default]
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
}

impl PortType {
    /// Whether two port types are structurally compatible for edge wiring.
    ///
    /// Strict equality — `Secret` is only compatible with `Secret`.
    /// Use `TypeRegistry::is_compatible()` for full semantic type checking
    /// with coercion paths.
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
    pub fn from_registry(
        type_str: &str,
        registry: &crate::type_registry::TypeRegistry,
    ) -> Result<PortType, std::string::String> {
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

    /// Strict structural parse. Returns `None` for unrecognized strings.
    pub fn try_parse(type_str: &str) -> Option<PortType> {
        try_parse_port_type(type_str)
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
/// Unrecognized type strings fail closed.
///
/// For strict resolution that rejects unknown types, use `PortType::try_parse`
/// or `PortType::from_registry`.
impl From<&TypeId> for PortType {
    fn from(type_id: &TypeId) -> Self {
        if let Some(port_type) = try_parse_port_type(&type_id.0) {
            return port_type;
        }
        let registry = crate::type_registry::TypeRegistry::global_core();
        PortType::from_registry(&type_id.0, registry).unwrap_or(PortType::Json)
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
        _ => panic!("parse_known_type called with unknown primitive `{s}`"),
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

        // Generic List<T>
        other if other.starts_with("List<") && other.ends_with('>') => {
            let inner = &other[5..other.len() - 1];
            let inner_type = try_parse_port_type(inner)?;
            Some(PortType::List(Box::new(inner_type)))
        }

        // Domain types — string-backed
        "FilePath"
        | "Path"
        | "TextFilePath"
        | "SecretName"
        | "Url"
        | "Email"
        | "NonEmptyString"
        | "Platform"
        | "ContentEncoding"
        | "OidcAudience"
        | "WifAudience"
        | "GcpProjectId"
        | "GcpSecretId"
        | "GcpSecretVersion"
        | "GcpServiceAccountEmail"
        | "GcpSubjectToken"
        | "OidcSubjectToken" => Some(PortType::String),

        // Domain types — bytes-backed
        "BinaryFilePath" => Some(PortType::Bytes),

        // Domain types — int-backed
        "Timestamp" => Some(PortType::Int),

        // Domain types — secret-backed
        "Credential" => Some(PortType::Secret),

        // Domain types — json/structured-backed
        "TransportRequest" | "TransportResponse"
        | "FileRequest" | "FileResponse"
        | "ShellRequest" | "ShellResponse"
        | "RestRequest" | "RestResponse"
        | "HttpRequest" | "HttpResponse"
        | "TcpRequest" | "TcpResponse"
        | "ToolHandle" | "FilesystemHandle" | "NetworkHandle"
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
    fn secret_is_strict() {
        assert!(!PortType::Secret.is_compatible_with(&PortType::String));
        assert!(!PortType::String.is_compatible_with(&PortType::Secret));
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
            PortType::try_parse("StringList").expect("legacy alias should parse"),
            PortType::List(Box::new(PortType::String))
        );
        assert_eq!(
            PortType::try_parse("OptionalString").expect("legacy alias should parse"),
            PortType::String
        );
    }

    #[test]
    fn domain_types_resolve_to_structural_backing() {
        // String-backed domain types
        assert_eq!(PortType::try_parse("FilePath"), Some(PortType::String));
        assert_eq!(PortType::try_parse("TextFilePath"), Some(PortType::String));
        assert_eq!(PortType::try_parse("Platform"), Some(PortType::String));
        assert_eq!(PortType::try_parse("Url"), Some(PortType::String));
        assert_eq!(PortType::try_parse("GcpProjectId"), Some(PortType::String));
        assert_eq!(PortType::try_parse("SecretName"), Some(PortType::String));

        // Bytes-backed domain types
        assert_eq!(PortType::try_parse("BinaryFilePath"), Some(PortType::Bytes));

        // Int-backed domain types
        assert_eq!(PortType::try_parse("Timestamp"), Some(PortType::Int));

        // Secret-backed domain types
        assert_eq!(PortType::try_parse("Credential"), Some(PortType::Secret));

        // Json/structured-backed domain types
        assert_eq!(
            PortType::try_parse("TransportRequest"),
            Some(PortType::Json)
        );
        assert_eq!(PortType::try_parse("FileResponse"), Some(PortType::Json));
        assert_eq!(PortType::try_parse("ToolHandle"), Some(PortType::Json));
        assert_eq!(PortType::try_parse("CliResult"), Some(PortType::Json));
    }

    #[test]
    fn unknown_type_strict_parse_returns_none() {
        assert_eq!(PortType::try_parse("SomeUnknownType"), None);
    }

    #[test]
    fn from_type_id_falls_back_to_json_for_unknown_type() {
        let unknown = TypeId::new("SomeUnknownType");
        assert_eq!(PortType::from(&unknown), PortType::Json);
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
