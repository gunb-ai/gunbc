//! Structural type emission for the syllogistic type system.
//!
//! Types are emitted by walking their `Dag<TypeOp>` structure via `TypeShape`,
//! then resolving against the backend language model. All type shapes —
//! Platform scalars, containers, brands, Products, Coproducts — resolve
//! through the language model. Named types try model lookup first; unknown
//! Products/Coproducts emit structural field/variant syntax. Opaque types
//! resolve through named entries or return verbatim with a warning.

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
pub fn derive_platform_properties(
    dag: &gunbc_ir::dag::Dag<gunbc_ir::type_op::TypeOp>,
) -> PlatformProperties {
    gunbc_ir::derive_structural_properties(dag)
}

/// Emit a native type string from a type DAG for the given backend.
///
/// Classifies the type DAG via `type_shape` and pattern-matches on the
/// structural form: Platform, Container, Brand, Product/Coproduct, or Opaque.
/// Only Opaque types use identity-type name-based mapping.
pub fn emit_type(dag: &gunbc_ir::dag::Dag<gunbc_ir::type_op::TypeOp>, backend: Backend) -> String {
    let shape = match gunbc_ir::type_shape(dag) {
        Ok(s) => s,
        Err(_) => gunbc_ir::TypeShape::Opaque("Unknown".to_string()),
    };
    emit_shape(&shape, backend)
}

/// Emit a native type string from a `TypeShape` for the given backend.
///
/// Handles all structural forms recursively. Only `Opaque` names use the
/// identity-type name-based mapping.
pub fn emit_shape(shape: &gunbc_ir::TypeShape, backend: Backend) -> String {
    use gunbc_ir::{ContainerShape, TypeShape};

    match shape {
        TypeShape::Platform(props) => emit_platform_type(props, backend),
        TypeShape::Container(container) => {
            use crate::language_model::{self, ContainerKind};
            let model = language_model::model_for_backend(backend);
            match container {
                ContainerShape::Optional(inner) => {
                    let inner_str = emit_shape(inner, backend);
                    language_model::resolve_container(
                        ContainerKind::Optional,
                        &inner_str,
                        None,
                        model,
                    )
                    .unwrap_or_else(|| format!("Optional<{inner_str}>"))
                }
                ContainerShape::List(inner) => {
                    let inner_str = emit_shape(inner, backend);
                    language_model::resolve_container(ContainerKind::List, &inner_str, None, model)
                        .unwrap_or_else(|| format!("List<{inner_str}>"))
                }
                ContainerShape::Set(inner) => {
                    let inner_str = emit_shape(inner, backend);
                    language_model::resolve_container(ContainerKind::Set, &inner_str, None, model)
                        .unwrap_or_else(|| format!("Set<{inner_str}>"))
                }
                ContainerShape::Map(key, value) => {
                    let key_str = emit_shape(key, backend);
                    let val_str = emit_shape(value, backend);
                    language_model::resolve_container(
                        ContainerKind::Map,
                        &val_str,
                        Some(&key_str),
                        model,
                    )
                    .unwrap_or_else(|| format!("Map<{key_str}, {val_str}>"))
                }
            }
        }
        TypeShape::Brand(name, inner) => {
            // Try the brand name in the language model first — brands like
            // FilesystemHandle, ToolHandle, Credential may have backend-
            // specific representations distinct from their inner type.
            let model = crate::language_model::model_for_backend(backend);
            if let Some(syntax) = crate::language_model::resolve_named(name, model) {
                syntax.to_string()
            } else {
                emit_shape(inner, backend)
            }
        }
        TypeShape::Product(name, fields) => {
            let model = crate::language_model::model_for_backend(backend);
            if let Some(n) = name {
                if let Some(syntax) = crate::language_model::resolve_named(n, model) {
                    return syntax.to_string();
                }
            }
            let typed_fields: Vec<(String, String)> = fields
                .iter()
                .map(|(n, s)| (n.clone(), emit_shape(s, backend)))
                .collect();
            (model.composite.format_product)(name.as_deref(), &typed_fields)
        }
        TypeShape::Coproduct(name, variants) => {
            let model = crate::language_model::model_for_backend(backend);
            if let Some(n) = name {
                if let Some(syntax) = crate::language_model::resolve_named(n, model) {
                    return syntax.to_string();
                }
            }
            let typed_variants: Vec<(String, String)> = variants
                .iter()
                .map(|(n, s)| (n.clone(), emit_shape(s, backend)))
                .collect();
            (model.composite.format_coproduct)(name.as_deref(), &typed_variants)
        }
        TypeShape::Opaque(name) => {
            let model = crate::language_model::model_for_backend(backend);
            if name == "Unit" {
                return model.unit_syntax.to_string();
            }
            if let Some(syntax) = crate::language_model::resolve_named(name, model) {
                syntax.to_string()
            } else {
                // Unknown opaque type — return the name verbatim as a
                // best-effort fallback. The type was already classified as
                // Opaque by type_shape, so this is the expected path for
                // user-defined types without a language model entry.
                name.to_string()
            }
        }
    }
}

/// Emit a platform-native type from structural properties.
///
/// Delegates to the language model's scalar resolver. Returns the
/// resolved syntax or warns and returns a fallback if no entry matches.
fn emit_platform_type(props: &gunbc_ir::StructuralProperties, backend: Backend) -> String {
    let model = crate::language_model::model_for_backend(backend);
    if let Some(syntax) = crate::language_model::resolve_scalar(props, model) {
        return syntax.to_string();
    }
    // No scalar entry matched — return the model's opaque fallback. This
    // can happen for platform types with unusual predicate combinations.
    model.opaque_fallback.to_string()
}

/// Resolve a type name structurally via the registry, then emit for the backend.
///
/// The type name is resolved through the registry to a `Dag<TypeOp>`, which
/// is emitted via [`emit_type`] (structural path). If the type is not in the
/// registry, resolves through the language model's named entries.
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
    let model = crate::language_model::model_for_backend(backend);
    if type_name == "Unit" {
        return model.unit_syntax.to_string();
    }
    if let Some(syntax) = crate::language_model::resolve_named(type_name, model) {
        return syntax.to_string();
    }
    // Unresolved type — return verbatim as a best-effort fallback.
    type_name.to_string()
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_type_rust_primitives() {
        assert_eq!(resolve_and_emit("String", None, Backend::Rust), "String");
        assert_eq!(resolve_and_emit("Path", None, Backend::Rust), "String");
        assert_eq!(resolve_and_emit("Bool", None, Backend::Rust), "bool");
        assert_eq!(resolve_and_emit("Int", None, Backend::Rust), "i64");
        assert_eq!(resolve_and_emit("i64", None, Backend::Rust), "i64");
        assert_eq!(resolve_and_emit("Float", None, Backend::Rust), "f64");
        assert_eq!(
            resolve_and_emit("ToolRegistry", None, Backend::Rust),
            "serde_json::Value"
        );
        assert_eq!(
            resolve_and_emit("FilesystemHandle", None, Backend::Rust),
            "PathBuf"
        );
    }

    #[test]
    fn named_type_go_primitives() {
        assert_eq!(resolve_and_emit("String", None, Backend::Go), "string");
        assert_eq!(resolve_and_emit("Bool", None, Backend::Go), "bool");
        assert_eq!(resolve_and_emit("Int", None, Backend::Go), "int64");
        assert_eq!(resolve_and_emit("Float", None, Backend::Go), "float64");
        assert_eq!(
            resolve_and_emit("ToolRegistry", None, Backend::Go),
            "interface{}"
        );
    }

    #[test]
    fn unknown_type_emits_name_verbatim() {
        assert_eq!(resolve_and_emit("FooBar", None, Backend::Rust), "FooBar");
        assert_eq!(resolve_and_emit("FooBar", None, Backend::Go), "FooBar");
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
        let dag = gunbc_ir::type_lib::refined(
            "Byte",
            vec![
                Predicate::Width(8),
                Predicate::Unsigned,
                Predicate::Arithmetic,
            ],
        );
        let props = derive_platform_properties(&dag);
        assert_eq!(props.width, Some(8));
        assert_eq!(props.signed, Some(false));
        assert!(props.arithmetic);
    }

    #[test]
    fn emit_type_unsigned_8bit() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined(
            "Byte",
            vec![
                Predicate::Width(8),
                Predicate::Unsigned,
                Predicate::Arithmetic,
            ],
        );
        assert_eq!(emit_type(&dag, Backend::Rust), "u8");
        assert_eq!(emit_type(&dag, Backend::Go), "uint8");
        assert_eq!(emit_type(&dag, Backend::C), "uint8_t");
    }

    #[test]
    fn emit_type_signed_64bit() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined(
            "Word64",
            vec![
                Predicate::Width(64),
                Predicate::Signed(None),
                Predicate::Arithmetic,
            ],
        );
        assert_eq!(emit_type(&dag, Backend::Rust), "i64");
        assert_eq!(emit_type(&dag, Backend::Go), "int64");
        assert_eq!(emit_type(&dag, Backend::C), "int64_t");
    }

    #[test]
    fn emit_type_float64() {
        use gunbc_ir::type_op::Predicate;
        let dag = gunbc_ir::type_lib::refined(
            "Word64",
            vec![
                Predicate::Width(64),
                Predicate::Domain("ieee754_binary64".to_string()),
                Predicate::Arithmetic,
            ],
        );
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
            gunbc_ir::type_lib::refined(
                "Int",
                vec![
                    Predicate::Width(64),
                    Predicate::Signed(None),
                    Predicate::Arithmetic,
                ],
            ),
        );
        assert_eq!(
            resolve_and_emit("Int64", Some(&registry), Backend::Rust),
            "i64"
        );
        assert_eq!(
            resolve_and_emit("Int64", Some(&registry), Backend::Go),
            "int64"
        );
        assert_eq!(
            resolve_and_emit("Int64", Some(&registry), Backend::C),
            "int64_t"
        );
    }

    #[test]
    fn resolve_and_emit_unknown_type_emits_name_verbatim() {
        let registry = gunbc_ir::TypeRegistry::with_primitives();
        assert_eq!(
            resolve_and_emit("UnknownType", Some(&registry), Backend::Rust),
            "UnknownType"
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
    fn resolve_and_emit_structural_matches_named_for_known_types() {
        let registry = gunbc_ir::TypeRegistry::with_core_types();
        assert_eq!(
            resolve_and_emit("String", Some(&registry), Backend::Rust),
            resolve_and_emit("String", None, Backend::Rust)
        );
        assert_eq!(
            resolve_and_emit("Bool", Some(&registry), Backend::Rust),
            resolve_and_emit("Bool", None, Backend::Rust)
        );
    }

    // ── C backend emits C types, not Rust types ─────────────────────

    #[test]
    fn c_backend_emits_correct_native_types() {
        assert_eq!(resolve_and_emit("String", None, Backend::C), "const char*");
        assert_eq!(resolve_and_emit("Int", None, Backend::C), "int64_t");
        assert_eq!(resolve_and_emit("Float", None, Backend::C), "double");
        assert_eq!(resolve_and_emit("Bytes", None, Backend::C), "uint8_t*");
        assert_eq!(resolve_and_emit("Json", None, Backend::C), "void*");
        assert_eq!(
            resolve_and_emit("FilesystemHandle", None, Backend::C),
            "const char*"
        );
        assert_eq!(resolve_and_emit("Unit", None, Backend::C), "void");
        assert_eq!(resolve_and_emit("Bool", None, Backend::C), "bool");
    }

    // ── Product/Coproduct structural emission ─────────────────────

    #[test]
    fn emit_product_structurally() {
        let dag = gunbc_ir::type_lib::product_resolved(
            "CliResult",
            vec![
                ("stdout", gunbc_ir::type_lib::string()),
                ("stderr", gunbc_ir::type_lib::string()),
            ],
        );
        // CliResult is not in the named entries, so it emits structurally.
        assert!(emit_type(&dag, Backend::Rust).contains("stdout"));
        assert!(emit_type(&dag, Backend::Go).contains("stdout"));
        assert!(emit_type(&dag, Backend::C).contains("stdout"));
    }

    #[test]
    fn emit_named_product_uses_language_model() {
        // String IS in the named entries — should resolve to the backend name.
        let dag = gunbc_ir::type_lib::product_resolved(
            "String",
            vec![
                ("bytes", gunbc_ir::type_lib::identity("Bytes")),
                ("encoding", gunbc_ir::type_lib::identity("Encoding")),
            ],
        );
        assert_eq!(emit_type(&dag, Backend::Rust), "String");
        assert_eq!(emit_type(&dag, Backend::Go), "string");
        assert_eq!(emit_type(&dag, Backend::C), "const char*");
    }

    #[test]
    fn emit_coproduct_structurally() {
        let dag = gunbc_ir::type_lib::coproduct(
            "ContentEncoding",
            vec![("UTF8", "String"), ("Binary", "Bytes")],
        );
        // ContentEncoding is not in the named entries, emits as enum structure.
        assert!(emit_type(&dag, Backend::Rust).contains("UTF8"));
        assert!(emit_type(&dag, Backend::Go).contains("ContentEncoding"));
    }
}
