//! Shared DSL-to-language type mapping tables.
//!
//! Each emit backend (Rust, Go, C) maps the same set of abstract DSL type
//! names to language-specific representations.  This module centralises the
//! primitive lookup table so that adding a new DSL type requires exactly one
//! change per backend, in one place.

/// A single primitive mapping entry: DSL names → target language type string.
///
/// `dsl_names` lists *all* recognised spellings of the same concept (e.g.
/// `["Int", "i64", "I64"]`).  `target` is the target-language representation.
pub(crate) struct PrimitiveMapping {
    pub(crate) dsl_names: &'static [&'static str],
    pub(crate) target: &'static str,
}

/// Language-specific type mapping configuration.
///
/// Internal to the emit crate. External callers should use [`resolve_and_emit`]
/// or [`emit_type`] instead of accessing these tables directly.
pub(crate) struct DslTypeMapping {
    /// Primitive type mappings (checked in order; first match wins).
    pub(crate) primitives: &'static [PrimitiveMapping],
    /// Format wrapper for `List<T>` — receives the mapped inner type.
    /// Example: `"Vec<{}>"` (Rust), `"[]{}"` (Go).
    pub(crate) list_fmt: &'static str,
    /// Format wrapper for `Optional<T>`.
    /// Example: `"Option<{}>"` (Rust), `"*{}"` (Go).  `None` means no
    /// explicit wrapper (C backend).
    pub(crate) optional_fmt: Option<&'static str>,
    /// Format wrapper for `Map<K,V>`.
    /// Example: `"HashMap<{}, {}>"` (Rust), `"map[{}]{}"` (Go).  `None`
    /// means fall through to `fallback`.
    pub(crate) map_fmt: Option<&'static str>,
    /// The default type when no mapping matches (e.g. `"serde_json::Value"`,
    /// `"interface{}"`, or a sentinel the caller interprets).
    pub(crate) fallback: &'static str,
}

/// Look up a primitive type name in the mapping table.
///
/// Returns the mapped target name, or `None` if not found. Unlike
/// [`map_abstract_type`], this does NOT handle generics or fallback — callers
/// that handle generic structure themselves (e.g., `type_expr_to_rust`) should
/// use this to avoid double-wrapping.
pub(crate) fn lookup_primitive(mapping: &DslTypeMapping, name: &str) -> Option<&'static str> {
    for entry in mapping.primitives {
        if entry.dsl_names.contains(&name) {
            return Some(entry.target);
        }
    }
    None
}

/// Resolve an abstract type string using the given mapping table.
///
/// Handles primitives, `List<T>`, `Optional<T>`, and `Map<K,V>` generics
/// recursively.  Falls back to [`DslTypeMapping::fallback`] for unknown types.
pub(crate) fn map_abstract_type(mapping: &DslTypeMapping, abstract_type: &str) -> String {
    // 1. Check primitives.
    for entry in mapping.primitives {
        if entry.dsl_names.contains(&abstract_type) {
            return entry.target.to_string();
        }
    }

    // 2. List<T>.
    if let Some(inner) = abstract_type
        .strip_prefix("List<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        let mapped_inner = map_abstract_type(mapping, inner);
        return mapping.list_fmt.replace("{}", &mapped_inner);
    }

    // 3. Optional<T>.
    if let Some(fmt) = mapping.optional_fmt {
        if let Some(inner) = abstract_type
            .strip_prefix("Optional<")
            .and_then(|rest| rest.strip_suffix('>'))
        {
            let mapped_inner = map_abstract_type(mapping, inner);
            return fmt.replace("{}", &mapped_inner);
        }
    }

    // 4. Map<K,V>.
    if let Some(fmt) = mapping.map_fmt {
        if let Some(inner) = abstract_type
            .strip_prefix("Map<")
            .and_then(|rest| rest.strip_suffix('>'))
        {
            if let Some(comma_pos) = inner.find(',') {
                let key = inner[..comma_pos].trim();
                let val = inner[comma_pos + 1..].trim();
                let mapped_key = map_abstract_type(mapping, key);
                let mapped_val = map_abstract_type(mapping, val);
                // fmt must contain exactly two `{}` placeholders.
                return fmt
                    .replacen("{}", &mapped_key, 1)
                    .replacen("{}", &mapped_val, 1);
            }
        }
    }

    // 5. Fallback.
    mapping.fallback.to_string()
}

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
/// This is the only remaining name-based path. It handles identity types
/// (String, Bool, Json, etc.) that have no structural predicates.
fn emit_opaque_fallback(name: &str, backend: Backend) -> String {
    let mapping = match backend {
        Backend::Rust => &RUST_TYPE_MAPPING,
        Backend::Go => &GO_TYPE_MAPPING,
        Backend::C => &RUST_TYPE_MAPPING,
    };
    if let Some(target) = lookup_primitive(mapping, name) {
        return target.to_string();
    }
    mapping.fallback.to_string()
}

/// Resolve a type name structurally via the registry, then emit for the backend.
///
/// This is the single entry point that all backends should use. When a registry
/// is provided and contains a structural definition for `type_name`, the type
/// DAG is resolved and emitted via [`emit_type`] (fully structural path).
/// Without a registry, falls back to `map_abstract_type` for backward
/// compatibility with string-based generic syntax.
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
    // No registry or type not found — use string-based mapping for
    // backward compatibility with generic syntax (List<T>, Optional<T>, etc.)
    let mapping = match backend {
        Backend::Rust => &RUST_TYPE_MAPPING,
        Backend::Go => &GO_TYPE_MAPPING,
        Backend::C => &RUST_TYPE_MAPPING,
    };
    map_abstract_type(mapping, type_name)
}

// =========================================================================
// Per-backend static tables
// =========================================================================

/// Rust backend type mapping table.
pub(crate) static RUST_TYPE_MAPPING: DslTypeMapping = DslTypeMapping {
    primitives: &[
        PrimitiveMapping {
            dsl_names: &[
                "String",
                "Path",
                "NonEmptyStr",
                "Url",
                "GistId",
                "ProjectId",
                "ServiceAccountEmail",
            ],
            target: "String",
        },
        PrimitiveMapping {
            dsl_names: &["Bool", "bool"],
            target: "bool",
        },
        PrimitiveMapping {
            dsl_names: &["Int", "i64", "I64"],
            target: "i64",
        },
        PrimitiveMapping {
            dsl_names: &["Float", "f64"],
            target: "f64",
        },
        PrimitiveMapping {
            dsl_names: &["Char"],
            target: "char",
        },
        PrimitiveMapping {
            dsl_names: &["Secret"],
            target: "String",
        },
        PrimitiveMapping {
            dsl_names: &["Bytes"],
            target: "Vec<u8>",
        },
        PrimitiveMapping {
            dsl_names: &["Json"],
            target: "serde_json::Value",
        },
        PrimitiveMapping {
            dsl_names: &["ToolRegistry"],
            target: "serde_json::Value",
        },
        PrimitiveMapping {
            dsl_names: &["TransportRequest"],
            target: "TransportRequest",
        },
        PrimitiveMapping {
            dsl_names: &["TransportResponse"],
            target: "TransportResponse",
        },
        PrimitiveMapping {
            dsl_names: &["FilesystemHandle"],
            target: "PathBuf",
        },
    ],
    list_fmt: "Vec<{}>",
    optional_fmt: Some("Option<{}>"),
    map_fmt: Some("HashMap<{}, {}>"),
    fallback: "serde_json::Value",
};

/// Go backend type mapping table.
pub(crate) static GO_TYPE_MAPPING: DslTypeMapping = DslTypeMapping {
    primitives: &[
        PrimitiveMapping {
            dsl_names: &[
                "String",
                "Path",
                "NonEmptyStr",
                "Url",
                "GistId",
                "ProjectId",
                "ServiceAccountEmail",
            ],
            target: "string",
        },
        PrimitiveMapping {
            dsl_names: &["Bool", "bool"],
            target: "bool",
        },
        PrimitiveMapping {
            dsl_names: &["Int", "i64", "I64"],
            target: "int64",
        },
        PrimitiveMapping {
            dsl_names: &["Float", "f64"],
            target: "float64",
        },
        PrimitiveMapping {
            dsl_names: &["Char"],
            target: "rune",
        },
        PrimitiveMapping {
            dsl_names: &["Secret"],
            target: "string",
        },
        PrimitiveMapping {
            dsl_names: &["Bytes"],
            target: "[]byte",
        },
        PrimitiveMapping {
            dsl_names: &["Json"],
            target: "interface{}",
        },
        PrimitiveMapping {
            dsl_names: &["ToolRegistry"],
            target: "interface{}",
        },
        PrimitiveMapping {
            dsl_names: &["TransportRequest"],
            target: "transport.Request",
        },
        PrimitiveMapping {
            dsl_names: &["TransportResponse"],
            target: "transport.Response",
        },
        PrimitiveMapping {
            dsl_names: &["FilesystemHandle"],
            target: "string",
        },
    ],
    list_fmt: "[]{}",
    optional_fmt: Some("*{}"),
    map_fmt: Some("map[{}]{}"),
    fallback: "interface{}",
};

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_primitives() {
        assert_eq!(map_abstract_type(&RUST_TYPE_MAPPING, "String"), "String");
        assert_eq!(map_abstract_type(&RUST_TYPE_MAPPING, "Path"), "String");
        assert_eq!(map_abstract_type(&RUST_TYPE_MAPPING, "Bool"), "bool");
        assert_eq!(map_abstract_type(&RUST_TYPE_MAPPING, "Int"), "i64");
        assert_eq!(map_abstract_type(&RUST_TYPE_MAPPING, "i64"), "i64");
        assert_eq!(map_abstract_type(&RUST_TYPE_MAPPING, "Float"), "f64");
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "ToolRegistry"),
            "serde_json::Value"
        );
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "FilesystemHandle"),
            "PathBuf"
        );
    }

    #[test]
    fn go_primitives() {
        assert_eq!(map_abstract_type(&GO_TYPE_MAPPING, "String"), "string");
        assert_eq!(map_abstract_type(&GO_TYPE_MAPPING, "Bool"), "bool");
        assert_eq!(map_abstract_type(&GO_TYPE_MAPPING, "Int"), "int64");
        assert_eq!(map_abstract_type(&GO_TYPE_MAPPING, "Float"), "float64");
        assert_eq!(
            map_abstract_type(&GO_TYPE_MAPPING, "ToolRegistry"),
            "interface{}"
        );
    }

    #[test]
    fn rust_list() {
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "List<String>"),
            "Vec<String>"
        );
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "List<Int>"),
            "Vec<i64>"
        );
    }

    #[test]
    fn go_list() {
        assert_eq!(
            map_abstract_type(&GO_TYPE_MAPPING, "List<String>"),
            "[]string"
        );
        assert_eq!(map_abstract_type(&GO_TYPE_MAPPING, "List<Int>"), "[]int64");
    }

    #[test]
    fn go_optional() {
        assert_eq!(
            map_abstract_type(&GO_TYPE_MAPPING, "Optional<String>"),
            "*string"
        );
    }

    #[test]
    fn rust_optional() {
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "Optional<Bool>"),
            "Option<bool>"
        );
    }

    #[test]
    fn go_map() {
        assert_eq!(
            map_abstract_type(&GO_TYPE_MAPPING, "Map<String, Int>"),
            "map[string]int64"
        );
    }

    #[test]
    fn rust_map() {
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "Map<String, Int>"),
            "HashMap<String, i64>"
        );
    }

    #[test]
    fn unknown_falls_back() {
        assert_eq!(
            map_abstract_type(&RUST_TYPE_MAPPING, "FooBar"),
            "serde_json::Value"
        );
        assert_eq!(map_abstract_type(&GO_TYPE_MAPPING, "FooBar"), "interface{}");
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
        // Register a structural Int64 type
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
        // Word32 has Width(32)
        let word32 = gunbc_ir::type_lib::refined("Word32", vec![Predicate::Width(32)]);
        // Float32 = Word32 where domain("ieee754_binary32"), arithmetic
        // Uses refined_with_base to embed Word32's DAG
        let float32 = gunbc_ir::type_lib::refined_with_base(
            "Word32",
            word32,
            vec![
                Predicate::Domain("ieee754_binary32".to_string()),
                Predicate::Arithmetic,
            ],
        );
        let props = derive_platform_properties(&float32);
        // Width(32) inherited from base via SubDag recursion
        assert_eq!(props.width, Some(32));
        assert_eq!(props.domain.as_deref(), Some("ieee754_binary32"));
        assert!(props.arithmetic);
        // Emits correctly
        assert_eq!(emit_type(&float32, Backend::Rust), "f32");
        assert_eq!(emit_type(&float32, Backend::Go), "float32");
        assert_eq!(emit_type(&float32, Backend::C), "float");
    }

    #[test]
    fn emit_type_refined_with_base_signed_integer() {
        use gunbc_ir::type_op::Predicate;
        // Byte has Width(8)
        let byte = gunbc_ir::type_lib::refined("Byte", vec![Predicate::Width(8)]);
        // Int8 = Byte where signed, arithmetic — inherits Width(8) from base
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
    fn resolve_and_emit_matches_map_abstract_type_for_known_types() {
        let registry = gunbc_ir::TypeRegistry::with_core_types();
        // String is in the registry — structural path should produce same result
        assert_eq!(
            resolve_and_emit("String", Some(&registry), Backend::Rust),
            map_abstract_type(&RUST_TYPE_MAPPING, "String")
        );
        assert_eq!(
            resolve_and_emit("Bool", Some(&registry), Backend::Rust),
            map_abstract_type(&RUST_TYPE_MAPPING, "Bool")
        );
    }
}
