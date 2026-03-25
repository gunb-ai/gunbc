use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub static RUST_TYPE_MAP: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("String".to_string(), "String".to_string()), ("Int".to_string(), "i64".to_string()), ("Float".to_string(), "f64".to_string()), ("Bool".to_string(), "bool".to_string()), ("Bytes".to_string(), "Vec<u8>".to_string()), ("Unit".to_string(), "()".to_string()), ("Secret".to_string(), "String".to_string()), ("Json".to_string(), "serde_json::Value".to_string())])
});

pub static RUST_KEYWORDS: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("true".to_string(), "true".to_string()), ("false".to_string(), "false".to_string()), ("null".to_string(), "None".to_string()), ("and".to_string(), "&&".to_string()), ("or".to_string(), "||".to_string()), ("not".to_string(), "!".to_string()), ("div".to_string(), "/".to_string())])
});

pub static RUST_CONTAINER_TEMPLATES: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("list".to_string(), "Vec<{0}>".to_string()), ("set".to_string(), "std::collections::BTreeSet<{0}>".to_string()), ("non_empty_list".to_string(), "NonEmptyVec<{0}>".to_string()), ("non_empty_set".to_string(), "NonEmptyBTreeSet<{0}>".to_string()), ("optional".to_string(), "Option<{0}>".to_string()), ("map".to_string(), "BTreeMap<{0}, {1}>".to_string())])
});

pub static RUST_RESERVED: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "yield",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "try",
    "typeof",
    "unsized",
    "virtual"
];

pub static RUST_RESERVED_ESCAPE_PREFIX: &str = "r#";

pub static RUST_STRUCT_DERIVES: &str = "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]";

pub static RUST_STRUCT_DERIVES_COPY: &str = "#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]";

pub static RUST_ENUM_DERIVES: &str = "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]";

pub static RUST_ENUM_DERIVES_COPY: &str = "#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]";

pub static RUST_SERDE_TAG: &str = "#[serde(tag = \"_variant\")]";

pub static RUST_SERDE_RENAME_TEMPLATE: &str = "#[serde(rename = \"{0}\")]";

pub static RUST_SOURCE_EXTENSION: &str = ".rs";

pub static RUST_SOURCE_DIR: &str = "src/";

pub static RUST_VISIBILITY: &str = "pub ";

pub static RT_FUNCTIONS: std::sync::LazyLock<std::collections::HashMap<String, bool>> = std::sync::LazyLock::new(|| {
    HashMap::from([("concat".to_string(), true), ("char_at".to_string(), true), ("string_length".to_string(), true), ("substring".to_string(), true), ("string_contains".to_string(), true), ("scan_while".to_string(), true), ("skip_horizontal_ws".to_string(), true), ("scan_to_eol".to_string(), true), ("scan_string_end".to_string(), true), ("code_point".to_string(), true), ("from_code_point".to_string(), true), ("lookup".to_string(), true), ("index_by".to_string(), true), ("empty_map".to_string(), true), ("map_insert".to_string(), true), ("map_merge".to_string(), true), ("list_concat".to_string(), true), ("str_eq".to_string(), true), ("filesystem_read".to_string(), true), ("list_push".to_string(), true), ("map_get".to_string(), true), ("map_keys".to_string(), true), ("map_values".to_string(), true), ("parse_int".to_string(), true), ("map_contains_key".to_string(), true), ("map_has".to_string(), true)])
});

pub static RT_REF_MAP_FUNCTIONS: std::sync::LazyLock<std::collections::HashMap<String, bool>> = std::sync::LazyLock::new(|| {
    HashMap::from([("map_get".to_string(), true), ("map_keys".to_string(), true), ("map_values".to_string(), true), ("lookup".to_string(), true), ("map_contains_key".to_string(), true), ("map_has".to_string(), true)])
});

