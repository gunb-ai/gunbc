//! Structural type emission for the syllogistic type system.
//!
//! Types are emitted by walking their `Dag<TypeOp>` structure via `TypeShape`.
//! Identity types (String, Bool, etc.) that lack structural predicates are
//! handled by `emit_opaque_fallback`, a simple name-based match.

// =========================================================================
// Structural type DAG emission (syllogistic types)
// =========================================================================

/// Target backend for structural type emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Rust,
    Go,
    C,
}

/// Platform properties derived from a type DAG's structural predicates.
///
/// This is a re-export of `gunbc_ir::StructuralProperties` for backward
/// compatibility within the emit crate.
pub type PlatformProperties = gunbc_ir::StructuralProperties;

/// Derive platform properties from a type DAG by walking its predicate nodes.
///
/// Delegates to the canonical `gunbc_ir::derive_structural_properties`, which
/// recursively walks SubDag children and inherits missing properties.
pub fn derive_platform_properties(dag: &gunbc_ir::dag::Dag<gunbc_ir::type_op::TypeOp>) -> PlatformProperties {
    gunbc_ir::derive_structural_properties(dag)
}

/// Emit a native type string from a type DAG for the given backend.
///
/// Classifies the type DAG via `type_shape` and pattern-matches on the
/// structural form: Platform, Container, Brand, Product/Coproduct, or Opaque.
/// Only Opaque types fall back to name-based lookup.
pub fn emit_type(dag: &gunbc_ir::dag::Dag<gunbc_ir::type_op::TypeOp>, backend: Backend) -> String {
    let shape = gunbc_ir::type_shape(dag);
    emit_shape(&shape, backend)
}

/// Emit a native type string from a `TypeShape` for the given backend.
///
/// Handles all structural forms recursively. Only `Opaque` names fall through
/// to the per-backend primitive lookup table.
pub fn emit_shape(shape: &gunbc_ir::TypeShape, backend: Backend) -> String {
    use gunbc_ir::{ContainerShape, TypeShape};

    match shape {
        TypeShape::Platform(props) => emit_platform_type(props, backend),
        TypeShape::Container(container) => match container {
            ContainerShape::Optional(inner) => {
                let inner_str = emit_shape(inner, backend);
                match backend {
                    Backend::Rust => format!("Option<{inner_str}>"),
                    Backend::Go => format!("*{inner_str}"),
                    Backend::C => format!("{inner_str}*"),
                }
            }
            ContainerShape::List(inner) => {
                let inner_str = emit_shape(inner, backend);
                match backend {
                    Backend::Rust => format!("Vec<{inner_str}>"),
                    Backend::Go => format!("[]{inner_str}"),
                    Backend::C => format!("{inner_str}*"),
                }
            }
            ContainerShape::Set(inner) => {
                let inner_str = emit_shape(inner, backend);
                match backend {
                    Backend::Rust => format!("HashSet<{inner_str}>"),
                    Backend::Go => format!("map[{inner_str}]struct{{}}"),
                    Backend::C => format!("{inner_str}*"),
                }
            }
            ContainerShape::Map(key, value) => {
                let key_str = emit_shape(key, backend);
                let val_str = emit_shape(value, backend);
                match backend {
                    Backend::Rust => format!("HashMap<{key_str}, {val_str}>"),
                    Backend::Go => format!("map[{key_str}]{val_str}"),
                    Backend::C => format!("{val_str}*"),
                }
            }
        },
        TypeShape::Brand(_, inner) => emit_shape(inner, backend),
        TypeShape::Product(_) | TypeShape::Coproduct(_) => {
            // Product/coproduct types emit as their name, extracted from
            // the DAG. Callers that need the name should use the TypeId.
            emit_opaque_fallback("Record", backend)
        }
        TypeShape::Opaque(name) => emit_opaque_fallback(name, backend),
    }
}

/// Emit a platform-native type from structural properties.
fn emit_platform_type(props: &gunbc_ir::StructuralProperties, backend: Backend) -> String {
    // Float types (ieee754 domain)
    if props.arithmetic {
        if let Some(domain) = &props.domain {
            if domain.starts_with("ieee754") {
                return match (backend, props.width) {
                    (Backend::Rust, Some(32)) => "f32".to_string(),
                    (Backend::Rust, _) => "f64".to_string(),
                    (Backend::Go, Some(32)) => "float32".to_string(),
                    (Backend::Go, _) => "float64".to_string(),
                    (Backend::C, Some(32)) => "float".to_string(),
                    (Backend::C, _) => "double".to_string(),
                };
            }
        }

        if let Some(width) = props.width {
            let signed = props.signed.unwrap_or(true);
            return match backend {
                Backend::Rust => {
                    let prefix = if signed { "i" } else { "u" };
                    format!("{prefix}{width}")
                }
                Backend::Go => {
                    let prefix = if signed { "int" } else { "uint" };
                    format!("{prefix}{width}")
                }
                Backend::C => {
                    let prefix = if signed { "int" } else { "uint" };
                    format!("{prefix}{width}_t")
                }
            };
        }
    }

    // Width-only without arithmetic (e.g., Byte = Width(8) + Unsigned)
    if let Some(width) = props.width {
        let signed = props.signed.unwrap_or(true);
        return match backend {
            Backend::Rust => {
                let prefix = if signed { "i" } else { "u" };
                format!("{prefix}{width}")
            }
            Backend::Go => {
                let prefix = if signed { "int" } else { "uint" };
                format!("{prefix}{width}")
            }
            Backend::C => {
                let prefix = if signed { "int" } else { "uint" };
                format!("{prefix}{width}_t")
            }
        };
    }

    // Platform with only signedness or domain — fall back to default int/string
    emit_opaque_fallback("Int", backend)
}

/// Emit a native type for an opaque/unresolved type name.
///
/// Handles identity types (String, Bool, Json, etc.) that have no structural
/// predicates. This is a simple match — the elaborate DslTypeMapping tables
/// have been replaced by the structural emit path.
fn emit_opaque_fallback(name: &str, backend: Backend) -> String {
    match backend {
        Backend::Rust => match name {
            "String" | "Path" | "NonEmptyStr" | "Url" | "GistId" | "ProjectId"
            | "ServiceAccountEmail" | "FilePath" | "Secret" => "String",
            "Bool" | "bool" => "bool",
            "Int" | "i64" | "I64" => "i64",
            "Float" | "f64" => "f64",
            "Char" => "char",
            "Bytes" => "Vec<u8>",
            "Json" | "ToolRegistry" => "serde_json::Value",
            "TransportRequest" => "TransportRequest",
            "TransportResponse" => "TransportResponse",
            "FilesystemHandle" => "PathBuf",
            "Unit" => "()",
            _ => "serde_json::Value",
        },
        Backend::Go => match name {
            "String" | "Path" | "NonEmptyStr" | "Url" | "GistId" | "ProjectId"
            | "ServiceAccountEmail" | "FilePath" | "Secret" | "FilesystemHandle" => "string",
            "Bool" | "bool" => "bool",
            "Int" | "i64" | "I64" => "int64",
            "Float" | "f64" => "float64",
            "Char" => "rune",
            "Bytes" => "[]byte",
            "Json" | "ToolRegistry" => "interface{}",
            "TransportRequest" => "transport.Request",
            "TransportResponse" => "transport.Response",
            "Unit" => "struct{}",
            _ => "interface{}",
        },
        Backend::C => match name {
            "String" | "Path" | "NonEmptyStr" | "Url" | "GistId" | "ProjectId"
            | "ServiceAccountEmail" | "FilePath" | "Secret" => "String",
            "Bool" | "bool" => "bool",
            "Int" | "i64" | "I64" => "i64",
            "Float" | "f64" => "f64",
            "Char" => "char",
            "Bytes" => "Vec<u8>",
            "Json" | "ToolRegistry" => "serde_json::Value",
            "TransportRequest" => "TransportRequest",
            "TransportResponse" => "TransportResponse",
            "FilesystemHandle" => "PathBuf",
            "Unit" => "()",
            _ => "serde_json::Value",
        },
    }
    .to_string()
}

/// Resolve a type name structurally via the registry, then emit for the backend.
///
/// This is the single entry point that all backends should use. The type name
/// is resolved through the registry to a `Dag<TypeOp>`, which is then emitted
/// via [`emit_type`] (fully structural path). If the type is not in the
/// registry, falls back to `emit_opaque_fallback` for identity types.
pub fn resolve_and_emit(
    type_name: &str,
    registry: Option<&gunbc_ir::TypeRegistry>,
    backend: Backend,
) -> String {
    if let Some(reg) = registry {
        let type_id = gunbc_ir::TypeId::new(type_name);
        if let Some(dag) = reg.resolve_type(&type_id) {
            return emit_type(&dag, backend);
        }
    }
    emit_opaque_fallback(type_name, backend)
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_fallback_rust_primitives() {
        assert_eq!(emit_opaque_fallback("String", Backend::Rust), "String");
        assert_eq!(emit_opaque_fallback("Path", Backend::Rust), "String");
        assert_eq!(emit_opaque_fallback("Bool", Backend::Rust), "bool");
        assert_eq!(emit_opaque_fallback("Int", Backend::Rust), "i64");
        assert_eq!(emit_opaque_fallback("i64", Backend::Rust), "i64");
        assert_eq!(emit_opaque_fallback("Float", Backend::Rust), "f64");
        assert_eq!(emit_opaque_fallback("ToolRegistry", Backend::Rust), "serde_json::Value");
        assert_eq!(emit_opaque_fallback("FilesystemHandle", Backend::Rust), "PathBuf");
    }

    #[test]
    fn opaque_fallback_go_primitives() {
        assert_eq!(emit_opaque_fallback("String", Backend::Go), "string");
        assert_eq!(emit_opaque_fallback("Bool", Backend::Go), "bool");
        assert_eq!(emit_opaque_fallback("Int", Backend::Go), "int64");
        assert_eq!(emit_opaque_fallback("Float", Backend::Go), "float64");
        assert_eq!(emit_opaque_fallback("ToolRegistry", Backend::Go), "interface{}");
    }

    #[test]
    fn opaque_fallback_unknown_type() {
        assert_eq!(emit_opaque_fallback("FooBar", Backend::Rust), "serde_json::Value");
        assert_eq!(emit_opaque_fallback("FooBar", Backend::Go), "interface{}");
    }

    // ── Structural emit tests ──────────────────────────────────────

    #[test]
    fn derive_platform_properties_empty_dag() {
        let dag = gunbc_ir::dag::Dag::new();
        let props = derive_platform_properties(&dag);
        assert!(props.width.is_none());
        assert!(props.signed.is_none());
        assert!(!props.arithmetic);
    }

    #[test]
    fn derive_platform_properties_from_predicates() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined("Byte", vec![
            Predicate::Width(8),
            Predicate::Unsigned,
            Predicate::Arithmetic,
        ]);
        let props = derive_platform_properties(&dag);
        assert_eq!(props.width, Some(8));
        assert_eq!(props.signed, Some(false));
        assert!(props.arithmetic);
    }

    #[test]
    fn emit_type_unsigned_8bit() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined("Byte", vec![
            Predicate::Width(8),
            Predicate::Unsigned,
            Predicate::Arithmetic,
        ]);
        assert_eq!(emit_type(&dag, Backend::Rust), "u8");
        assert_eq!(emit_type(&dag, Backend::Go), "uint8");
        assert_eq!(emit_type(&dag, Backend::C), "uint8_t");
    }

    #[test]
    fn emit_type_signed_64bit() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined("Word64", vec![
            Predicate::Width(64),
            Predicate::Signed(None),
            Predicate::Arithmetic,
        ]);
        assert_eq!(emit_type(&dag, Backend::Rust), "i64");
        assert_eq!(emit_type(&dag, Backend::Go), "int64");
        assert_eq!(emit_type(&dag, Backend::C), "int64_t");
    }

    #[test]
    fn emit_type_float64() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined("Word64", vec![
            Predicate::Width(64),
            Predicate::Domain("ieee754_binary64".to_string()),
            Predicate::Arithmetic,
        ]);
        assert_eq!(emit_type(&dag, Backend::Rust), "f64");
        assert_eq!(emit_type(&dag, Backend::Go), "float64");
        assert_eq!(emit_type(&dag, Backend::C), "double");
    }

    #[test]
    fn emit_type_fallback_to_string_mapping() {
        let dag = gunbc_ir::type_lib::string();
        assert_eq!(emit_type(&dag, Backend::Rust), "String");
        assert_eq!(emit_type(&dag, Backend::Go), "string");
    }

    // ── resolve_and_emit tests ────────────────────────────────────

    #[test]
    fn resolve_and_emit_without_registry_falls_back() {
        assert_eq!(resolve_and_emit("String", None, Backend::Rust), "String");
        assert_eq!(resolve_and_emit("Int", None, Backend::Go), "int64");
        assert_eq!(resolve_and_emit("Bool", None, Backend::C), "bool");
    }

    #[test]
    fn resolve_and_emit_with_registry_uses_structural() {
        use gunbc_ir::type_op::Predicate;
        let mut registry = gunbc_ir::TypeRegistry::with_primitives();
        registry.register(
            "Int64",
            gunbc_ir::type_lib::refined("Int", vec![
                Predicate::Width(64),
                Predicate::Signed(None),
                Predicate::Arithmetic,
            ]),
        );
        assert_eq!(resolve_and_emit("Int64", Some(&registry), Backend::Rust), "i64");
        assert_eq!(resolve_and_emit("Int64", Some(&registry), Backend::Go), "int64");
        assert_eq!(resolve_and_emit("Int64", Some(&registry), Backend::C), "int64_t");
    }

    #[test]
    fn resolve_and_emit_unknown_type_falls_back() {
        let registry = gunbc_ir::TypeRegistry::with_primitives();
        assert_eq!(
            resolve_and_emit("UnknownType", Some(&registry), Backend::Rust),
            "serde_json::Value"
        );
    }

    #[test]
    fn derive_platform_properties_inherits_from_base_dag() {
        use gunbc_ir::type_op::Predicate;
        let word32 = gunbc_ir::type_lib::refined("Word32", vec![Predicate::Width(32)]);
        let float32 = gunbc_ir::type_lib::refined_with_base(
            "Word32",
            word32,
            vec![
                Predicate::Domain("ieee754_binary32".to_string()),
                Predicate::Arithmetic,
            ],
        );
        let props = derive_platform_properties(&float32);
        assert_eq!(props.width, Some(32));
        assert_eq!(props.domain.as_deref(), Some("ieee754_binary32"));
        assert!(props.arithmetic);
        assert_eq!(emit_type(&float32, Backend::Rust), "f32");
        assert_eq!(emit_type(&float32, Backend::Go), "float32");
        assert_eq!(emit_type(&float32, Backend::C), "float");
    }

    #[test]
    fn emit_type_refined_with_base_signed_integer() {
        use gunbc_ir::type_op::Predicate;
        let byte = gunbc_ir::type_lib::refined("Byte", vec![Predicate::Width(8)]);
        let int8 = gunbc_ir::type_lib::refined_with_base(
            "Byte",
            byte,
            vec![Predicate::Signed(None), Predicate::Arithmetic],
        );
        assert_eq!(emit_type(&int8, Backend::Rust), "i8");
        assert_eq!(emit_type(&int8, Backend::Go), "int8");
        assert_eq!(emit_type(&int8, Backend::C), "int8_t");
    }

    #[test]
    fn resolve_and_emit_structural_matches_opaque_for_known_types() {
        let registry = gunbc_ir::TypeRegistry::with_core_types();
        assert_eq!(
            resolve_and_emit("String", Some(&registry), Backend::Rust),
            emit_opaque_fallback("String", Backend::Rust)
        );
        assert_eq!(
            resolve_and_emit("Bool", Some(&registry), Backend::Rust),
            emit_opaque_fallback("Bool", Backend::Rust)
        );
    }
}
