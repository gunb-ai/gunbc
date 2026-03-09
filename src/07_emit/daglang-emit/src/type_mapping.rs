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
pub struct PrimitiveMapping {
    pub dsl_names: &'static [&'static str],
    pub target: &'static str,
}

/// Language-specific type mapping configuration.
///
/// Backends construct a `&'static DslTypeMapping` with their tables and pass
/// it to [`map_abstract_type`] to resolve an abstract type string.
pub struct DslTypeMapping {
    /// Primitive type mappings (checked in order; first match wins).
    pub primitives: &'static [PrimitiveMapping],
    /// Format wrapper for `List<T>` — receives the mapped inner type.
    /// Example: `"Vec<{}>"` (Rust), `"[]{}"` (Go).
    pub list_fmt: &'static str,
    /// Format wrapper for `Optional<T>`.
    /// Example: `"Option<{}>"` (Rust), `"*{}"` (Go).  `None` means no
    /// explicit wrapper (C backend).
    pub optional_fmt: Option<&'static str>,
    /// Format wrapper for `Map<K,V>`.
    /// Example: `"HashMap<{}, {}>"` (Rust), `"map[{}]{}"` (Go).  `None`
    /// means fall through to `fallback`.
    pub map_fmt: Option<&'static str>,
    /// The default type when no mapping matches (e.g. `"serde_json::Value"`,
    /// `"interface{}"`, or a sentinel the caller interprets).
    pub fallback: &'static str,
}

/// Look up a primitive type name in the mapping table.
///
/// Returns the mapped target name, or `None` if not found. Unlike
/// [`map_abstract_type`], this does NOT handle generics or fallback — callers
/// that handle generic structure themselves (e.g., `type_expr_to_rust`) should
/// use this to avoid double-wrapping.
pub fn lookup_primitive(mapping: &DslTypeMapping, name: &str) -> Option<&'static str> {
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
pub fn map_abstract_type(mapping: &DslTypeMapping, abstract_type: &str) -> String {
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
#[derive(Debug, Clone, Default)]
pub struct PlatformProperties {
    /// Bit width (from Width predicate).
    pub width: Option<u16>,
    /// Whether the type is signed (from Signed predicate).
    pub signed: Option<bool>,
    /// Whether the type supports arithmetic (from Arithmetic predicate).
    pub arithmetic: bool,
    /// Domain constraint (from Domain predicate, e.g., "ieee754_binary32").
    pub domain: Option<String>,
    /// Collection/string length constraint (from Length predicate).
    pub length: Option<u64>,
}

/// Derive platform properties from a type DAG by walking its predicate nodes.
pub fn derive_platform_properties(dag: &gunbc_ir::dag::Dag<gunbc_ir::type_op::TypeOp>) -> PlatformProperties {
    use gunbc_ir::node::NodeBody;
    use gunbc_ir::type_op::{Predicate, TypeOp};

    let mut props = PlatformProperties::default();

    for node in &dag.nodes {
        if let NodeBody::Opaque(TypeOp::Validate(pred)) = &node.body {
            match pred {
                Predicate::Width(w) => props.width = Some(*w),
                Predicate::Signed(_) => props.signed = Some(true),
                Predicate::Unsigned => props.signed = Some(false),
                Predicate::Arithmetic => props.arithmetic = true,
                Predicate::Domain(d) => props.domain = Some(d.clone()),
                Predicate::Length(l) => props.length = Some(*l),
                _ => {}
            }
        }
        // Recurse into SubDags
        if let NodeBody::SubDag(subdag, _) = &node.body {
            let inner = derive_platform_properties(subdag);
            if props.width.is_none() {
                props.width = inner.width;
            }
            if props.signed.is_none() {
                props.signed = inner.signed;
            }
            if !props.arithmetic {
                props.arithmetic = inner.arithmetic;
            }
            if props.domain.is_none() {
                props.domain = inner.domain;
            }
            if props.length.is_none() {
                props.length = inner.length;
            }
        }
    }

    props
}

/// Emit a native type string from a type DAG for the given backend.
///
/// Uses structural properties (width, signedness, domain) to determine
/// the native type. Falls back to string-based `map_abstract_type` for
/// types that don't carry structural predicates.
pub fn emit_type(dag: &gunbc_ir::dag::Dag<gunbc_ir::type_op::TypeOp>, backend: Backend) -> String {
    let props = derive_platform_properties(dag);

    // If we have structural width+signedness, emit a platform-native type
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

    // Fall back to the base type name from the DAG
    let base = gunbc_ir::contract::base_type(dag);
    let mapping = match backend {
        Backend::Rust => &RUST_TYPE_MAPPING,
        Backend::Go => &GO_TYPE_MAPPING,
        Backend::C => &RUST_TYPE_MAPPING, // C reuses Rust mapping as default
    };
    if let Some(base_name) = base {
        map_abstract_type(mapping, &base_name)
    } else {
        mapping.fallback.to_string()
    }
}

// =========================================================================
// Per-backend static tables
// =========================================================================

/// Rust backend type mapping table.
pub static RUST_TYPE_MAPPING: DslTypeMapping = DslTypeMapping {
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
pub static GO_TYPE_MAPPING: DslTypeMapping = DslTypeMapping {
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
}
