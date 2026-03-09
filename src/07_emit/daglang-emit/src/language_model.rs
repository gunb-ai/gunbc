//! Backend language models: hierarchical target modeling via compilation target chains.
//!
//! Each language model is a collection of entries that describe how a target
//! language represents types. The structural resolver matches source type
//! properties against model entries to produce native syntax — replacing
//! the hardcoded match tables in `emit_identity_type`.
//!
//! Hierarchy (following compilation target chains):
//! - ISA layer: scalar widths, signedness, IEEE 754 domains
//! - C layer (ISO/IEC 9899:2018): int8_t..int64_t, float/double, struct, pointer
//! - Rust/Go/C++ layer: language-specific syntax over the same ISA scalars
//!
//! Ref: DESIGN-syllogistic-types.md, "Backend Language Models" section.

use super::type_mapping::Backend;

/// A scalar type entry: structural predicates → native syntax.
#[derive(Debug, Clone)]
pub struct ScalarEntry {
    pub width: Option<u16>,
    pub signed: Option<bool>,
    pub domain: Option<&'static str>,
    pub arithmetic: bool,
    pub syntax: &'static str,
}

/// A named type entry: type name → native syntax.
/// Covers types that are resolved by name (String, Bool, Json, etc.)
/// until their structural shapes are fully resolved.
#[derive(Debug, Clone)]
pub struct NamedEntry {
    pub names: &'static [&'static str],
    pub syntax: &'static str,
}

/// A container type entry: container kind → syntax template.
#[derive(Debug, Clone)]
pub struct ContainerEntry {
    pub kind: ContainerKind,
    pub syntax_template: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    Optional,
    List,
    Set,
    Map,
}

/// A transport operation mapping: abstract name → backend-specific function.
#[derive(Debug, Clone)]
pub struct TransportEntry {
    pub abstract_name: &'static str,
    pub native_call: &'static str,
}

/// A complete language model for one compilation target.
#[derive(Debug, Clone)]
pub struct LanguageModel {
    pub name: &'static str,
    pub scalars: &'static [ScalarEntry],
    pub named: &'static [NamedEntry],
    pub containers: &'static [ContainerEntry],
    pub transport: &'static [TransportEntry],
    pub unit_syntax: &'static str,
    pub opaque_fallback: &'static str,
}

// =========================================================================
// ISA-level scalar entries (shared by all targets)
// Ref: x86-64 SDM Vol. 1 §3.1, IEEE 754-2019 §3.3
// =========================================================================

macro_rules! int_entry {
    ($w:expr, $signed:expr, $syntax:expr) => {
        ScalarEntry {
            width: Some($w),
            signed: Some($signed),
            domain: None,
            arithmetic: true,
            syntax: $syntax,
        }
    };
}

macro_rules! float_entry {
    ($w:expr, $domain:expr, $syntax:expr) => {
        ScalarEntry {
            width: Some($w),
            signed: None,
            domain: Some($domain),
            arithmetic: true,
            syntax: $syntax,
        }
    };
}

// =========================================================================
// Rust language model
// Ref: The Rust Reference §6.1.1 (integers), §6.1.2 (floats)
// =========================================================================

static RUST_SCALARS: &[ScalarEntry] = &[
    float_entry!(32, "ieee754_binary32", "f32"),
    float_entry!(64, "ieee754_binary64", "f64"),
    int_entry!(8, true, "i8"),
    int_entry!(16, true, "i16"),
    int_entry!(32, true, "i32"),
    int_entry!(64, true, "i64"),
    int_entry!(8, false, "u8"),
    int_entry!(16, false, "u16"),
    int_entry!(32, false, "u32"),
    int_entry!(64, false, "u64"),
];

static RUST_NAMED: &[NamedEntry] = &[
    NamedEntry { names: &["String", "Path", "NonEmptyStr", "Url", "GistId", "ProjectId",
                           "ServiceAccountEmail", "FilePath", "Secret"], syntax: "String" },
    NamedEntry { names: &["Bool", "bool"], syntax: "bool" },
    NamedEntry { names: &["Int", "i64", "I64"], syntax: "i64" },
    NamedEntry { names: &["Float", "f64"], syntax: "f64" },
    NamedEntry { names: &["Char"], syntax: "char" },
    NamedEntry { names: &["Bytes"], syntax: "Vec<u8>" },
    NamedEntry { names: &["Json", "ToolRegistry"], syntax: "serde_json::Value" },
    NamedEntry { names: &["TransportRequest"], syntax: "TransportRequest" },
    NamedEntry { names: &["TransportResponse"], syntax: "TransportResponse" },
    NamedEntry { names: &["FilesystemHandle"], syntax: "PathBuf" },
    NamedEntry { names: &["Record"], syntax: "serde_json::Value" },
];

static RUST_CONTAINERS: &[ContainerEntry] = &[
    ContainerEntry { kind: ContainerKind::Optional, syntax_template: "Option<{T}>" },
    ContainerEntry { kind: ContainerKind::List, syntax_template: "Vec<{T}>" },
    ContainerEntry { kind: ContainerKind::Set, syntax_template: "HashSet<{T}>" },
    ContainerEntry { kind: ContainerKind::Map, syntax_template: "HashMap<{K}, {V}>" },
];

static RUST_TRANSPORT: &[TransportEntry] = &[
    TransportEntry { abstract_name: "prepare_file_read", native_call: "FileRequest::read" },
    TransportEntry { abstract_name: "execute_file_read", native_call: "execute_transport" },
    TransportEntry { abstract_name: "parse_file_read_response", native_call: "parse_file_response" },
    TransportEntry { abstract_name: "prepare_file_write", native_call: "FileRequest::write" },
    TransportEntry { abstract_name: "execute_file_write", native_call: "execute_transport" },
    TransportEntry { abstract_name: "parse_file_write_response", native_call: "parse_file_response" },
    TransportEntry { abstract_name: "prepare_file_exists", native_call: "FileRequest::exists" },
    TransportEntry { abstract_name: "execute_file_exists", native_call: "execute_transport" },
    TransportEntry { abstract_name: "prepare_shell_exec", native_call: "ShellRequest::new" },
    TransportEntry { abstract_name: "execute_shell_exec", native_call: "execute_transport" },
    TransportEntry { abstract_name: "parse_shell_exec_response", native_call: "parse_shell_response" },
    TransportEntry { abstract_name: "prepare_http_request", native_call: "RestRequest::new" },
    TransportEntry { abstract_name: "execute_http_request", native_call: "execute_transport" },
    TransportEntry { abstract_name: "prepare_directory_list", native_call: "FileRequest::list_dir" },
    TransportEntry { abstract_name: "execute_directory_list", native_call: "execute_transport" },
    TransportEntry { abstract_name: "acquire_resource", native_call: "acquire_resource_handle" },
];

pub static RUST_MODEL: LanguageModel = LanguageModel {
    name: "Rust",
    scalars: RUST_SCALARS,
    named: RUST_NAMED,
    containers: RUST_CONTAINERS,
    transport: RUST_TRANSPORT,
    unit_syntax: "()",
    opaque_fallback: "serde_json::Value",
};

// =========================================================================
// Go language model
// Ref: The Go Programming Language Specification §Numeric types
// =========================================================================

static GO_SCALARS: &[ScalarEntry] = &[
    float_entry!(32, "ieee754_binary32", "float32"),
    float_entry!(64, "ieee754_binary64", "float64"),
    int_entry!(8, true, "int8"),
    int_entry!(16, true, "int16"),
    int_entry!(32, true, "int32"),
    int_entry!(64, true, "int64"),
    int_entry!(8, false, "uint8"),
    int_entry!(16, false, "uint16"),
    int_entry!(32, false, "uint32"),
    int_entry!(64, false, "uint64"),
];

static GO_NAMED: &[NamedEntry] = &[
    NamedEntry { names: &["String", "Path", "NonEmptyStr", "Url", "GistId", "ProjectId",
                           "ServiceAccountEmail", "FilePath", "Secret", "FilesystemHandle"], syntax: "string" },
    NamedEntry { names: &["Bool", "bool"], syntax: "bool" },
    NamedEntry { names: &["Int", "i64", "I64"], syntax: "int64" },
    NamedEntry { names: &["Float", "f64"], syntax: "float64" },
    NamedEntry { names: &["Char"], syntax: "rune" },
    NamedEntry { names: &["Bytes"], syntax: "[]byte" },
    NamedEntry { names: &["Json", "ToolRegistry"], syntax: "interface{}" },
    NamedEntry { names: &["TransportRequest"], syntax: "transport.Request" },
    NamedEntry { names: &["TransportResponse"], syntax: "transport.Response" },
    NamedEntry { names: &["Record"], syntax: "interface{}" },
];

static GO_CONTAINERS: &[ContainerEntry] = &[
    ContainerEntry { kind: ContainerKind::Optional, syntax_template: "*{T}" },
    ContainerEntry { kind: ContainerKind::List, syntax_template: "[]{T}" },
    ContainerEntry { kind: ContainerKind::Set, syntax_template: "map[{T}]struct{}" },
    ContainerEntry { kind: ContainerKind::Map, syntax_template: "map[{K}]{V}" },
];

static GO_TRANSPORT: &[TransportEntry] = &[
    TransportEntry { abstract_name: "prepare_file_read", native_call: "transport.NewFileReadRequest" },
    TransportEntry { abstract_name: "execute_file_read", native_call: "transport.Execute" },
    TransportEntry { abstract_name: "parse_file_read_response", native_call: "transport.ParseFileResponse" },
    TransportEntry { abstract_name: "prepare_file_write", native_call: "transport.NewFileWriteRequest" },
    TransportEntry { abstract_name: "execute_file_write", native_call: "transport.Execute" },
    TransportEntry { abstract_name: "parse_file_write_response", native_call: "transport.ParseFileResponse" },
    TransportEntry { abstract_name: "prepare_file_exists", native_call: "transport.NewFileExistsRequest" },
    TransportEntry { abstract_name: "execute_file_exists", native_call: "transport.Execute" },
    TransportEntry { abstract_name: "prepare_shell_exec", native_call: "transport.NewShellRequest" },
    TransportEntry { abstract_name: "execute_shell_exec", native_call: "transport.Execute" },
    TransportEntry { abstract_name: "parse_shell_exec_response", native_call: "transport.ParseShellResponse" },
    TransportEntry { abstract_name: "prepare_http_request", native_call: "transport.NewHTTPRequest" },
    TransportEntry { abstract_name: "execute_http_request", native_call: "transport.Execute" },
    TransportEntry { abstract_name: "prepare_directory_list", native_call: "transport.NewDirListRequest" },
    TransportEntry { abstract_name: "execute_directory_list", native_call: "transport.Execute" },
    TransportEntry { abstract_name: "acquire_resource", native_call: "resource.Acquire" },
];

pub static GO_MODEL: LanguageModel = LanguageModel {
    name: "Go",
    scalars: GO_SCALARS,
    named: GO_NAMED,
    containers: GO_CONTAINERS,
    transport: GO_TRANSPORT,
    unit_syntax: "struct{}",
    opaque_fallback: "interface{}",
};

// =========================================================================
// C language model
// Ref: ISO/IEC 9899:2018 §7.20.1.1 (exact-width integers), §6.2.5 (floats)
// =========================================================================

static C_SCALARS: &[ScalarEntry] = &[
    float_entry!(32, "ieee754_binary32", "float"),
    float_entry!(64, "ieee754_binary64", "double"),
    int_entry!(8, true, "int8_t"),
    int_entry!(16, true, "int16_t"),
    int_entry!(32, true, "int32_t"),
    int_entry!(64, true, "int64_t"),
    int_entry!(8, false, "uint8_t"),
    int_entry!(16, false, "uint16_t"),
    int_entry!(32, false, "uint32_t"),
    int_entry!(64, false, "uint64_t"),
];

static C_NAMED: &[NamedEntry] = &[
    NamedEntry { names: &["String", "Path", "NonEmptyStr", "Url", "GistId", "ProjectId",
                           "ServiceAccountEmail", "FilePath", "Secret", "FilesystemHandle"], syntax: "const char*" },
    NamedEntry { names: &["Bool", "bool"], syntax: "bool" },
    NamedEntry { names: &["Int", "i64", "I64"], syntax: "int64_t" },
    NamedEntry { names: &["Float", "f64"], syntax: "double" },
    NamedEntry { names: &["Char"], syntax: "char" },
    NamedEntry { names: &["Bytes"], syntax: "uint8_t*" },
    NamedEntry { names: &["Json", "ToolRegistry", "Record"], syntax: "void*" },
    NamedEntry { names: &["TransportRequest"], syntax: "TransportRequest" },
    NamedEntry { names: &["TransportResponse"], syntax: "TransportResponse" },
];

static C_CONTAINERS: &[ContainerEntry] = &[
    ContainerEntry { kind: ContainerKind::Optional, syntax_template: "{T}*" },
    ContainerEntry { kind: ContainerKind::List, syntax_template: "{T}*" },
    ContainerEntry { kind: ContainerKind::Set, syntax_template: "{T}*" },
    ContainerEntry { kind: ContainerKind::Map, syntax_template: "{V}*" },
];

static C_TRANSPORT: &[TransportEntry] = &[
    TransportEntry { abstract_name: "prepare_file_read", native_call: "gunbc_file_read_request" },
    TransportEntry { abstract_name: "execute_file_read", native_call: "gunbc_transport_execute" },
    TransportEntry { abstract_name: "prepare_file_write", native_call: "gunbc_file_write_request" },
    TransportEntry { abstract_name: "execute_file_write", native_call: "gunbc_transport_execute" },
    TransportEntry { abstract_name: "prepare_file_exists", native_call: "gunbc_file_exists_request" },
    TransportEntry { abstract_name: "execute_file_exists", native_call: "gunbc_transport_execute" },
    TransportEntry { abstract_name: "prepare_shell_exec", native_call: "gunbc_shell_request" },
    TransportEntry { abstract_name: "execute_shell_exec", native_call: "gunbc_transport_execute" },
    TransportEntry { abstract_name: "prepare_http_request", native_call: "gunbc_http_request" },
    TransportEntry { abstract_name: "execute_http_request", native_call: "gunbc_transport_execute" },
    TransportEntry { abstract_name: "prepare_directory_list", native_call: "gunbc_dir_list_request" },
    TransportEntry { abstract_name: "execute_directory_list", native_call: "gunbc_transport_execute" },
    TransportEntry { abstract_name: "acquire_resource", native_call: "gunbc_acquire_resource" },
];

pub static C_MODEL: LanguageModel = LanguageModel {
    name: "C",
    scalars: C_SCALARS,
    named: C_NAMED,
    containers: C_CONTAINERS,
    transport: C_TRANSPORT,
    unit_syntax: "void",
    opaque_fallback: "void*",
};

// =========================================================================
// Structural resolver
// =========================================================================

/// Look up the language model for a backend.
pub fn model_for_backend(backend: Backend) -> &'static LanguageModel {
    match backend {
        Backend::Rust => &RUST_MODEL,
        Backend::Go => &GO_MODEL,
        Backend::C => &C_MODEL,
    }
}

/// Resolve a type name through the language model's named entries.
pub fn resolve_named(name: &str, model: &LanguageModel) -> Option<&'static str> {
    for entry in model.named {
        if entry.names.contains(&name) {
            return Some(entry.syntax);
        }
    }
    None
}

/// Resolve structural properties against the language model's scalar entries.
pub fn resolve_scalar(props: &gunbc_ir::StructuralProperties, model: &LanguageModel) -> Option<&'static str> {
    // Float types: match domain first (most specific)
    if let Some(domain) = &props.domain {
        for entry in model.scalars {
            if let Some(ed) = entry.domain {
                if (domain.starts_with(ed.split('_').next().unwrap_or(ed)) || domain == ed)
                    && (entry.width.is_none() || entry.width == props.width)
                {
                    return Some(entry.syntax);
                }
            }
        }
        // Partial domain match: find best width match
        for entry in model.scalars {
            if entry.domain.is_some() && entry.width == props.width {
                return Some(entry.syntax);
            }
        }
    }

    // Integer types: match width + signedness
    if let Some(width) = props.width {
        let signed = props.signed.unwrap_or(true);
        for entry in model.scalars {
            if entry.domain.is_none() && entry.width == Some(width) && entry.signed == Some(signed) {
                return Some(entry.syntax);
            }
        }
    }

    None
}

/// Resolve a container shape against the language model.
pub fn resolve_container(
    kind: ContainerKind,
    inner: &str,
    key: Option<&str>,
    model: &LanguageModel,
) -> Option<String> {
    for entry in model.containers {
        if entry.kind == kind {
            let result = entry.syntax_template
                .replace("{T}", inner)
                .replace("{V}", inner);
            let result = if let Some(k) = key {
                result.replace("{K}", k)
            } else {
                result
            };
            return Some(result);
        }
    }
    None
}

/// Resolve a transport operation name through the language model.
pub fn resolve_transport(name: &str, model: &LanguageModel) -> Option<&'static str> {
    for entry in model.transport {
        if entry.abstract_name == name {
            return Some(entry.native_call);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_model_resolves_scalars() {
        let props = gunbc_ir::StructuralProperties {
            width: Some(32), signed: Some(true), arithmetic: true, ..Default::default()
        };
        assert_eq!(resolve_scalar(&props, &RUST_MODEL), Some("i32"));

        let props = gunbc_ir::StructuralProperties {
            width: Some(64), signed: Some(false), arithmetic: true, ..Default::default()
        };
        assert_eq!(resolve_scalar(&props, &RUST_MODEL), Some("u64"));
    }

    #[test]
    fn go_model_resolves_floats() {
        let props = gunbc_ir::StructuralProperties {
            width: Some(64), domain: Some("ieee754_binary64".to_string()),
            arithmetic: true, ..Default::default()
        };
        assert_eq!(resolve_scalar(&props, &GO_MODEL), Some("float64"));
    }

    #[test]
    fn c_model_resolves_integers() {
        let props = gunbc_ir::StructuralProperties {
            width: Some(8), signed: Some(false), arithmetic: true, ..Default::default()
        };
        assert_eq!(resolve_scalar(&props, &C_MODEL), Some("uint8_t"));
    }

    #[test]
    fn named_resolution_across_backends() {
        assert_eq!(resolve_named("String", &RUST_MODEL), Some("String"));
        assert_eq!(resolve_named("String", &GO_MODEL), Some("string"));
        assert_eq!(resolve_named("String", &C_MODEL), Some("const char*"));
    }

    #[test]
    fn container_resolution() {
        assert_eq!(
            resolve_container(ContainerKind::List, "i32", None, &RUST_MODEL),
            Some("Vec<i32>".to_string())
        );
        assert_eq!(
            resolve_container(ContainerKind::Optional, "int64", None, &GO_MODEL),
            Some("*int64".to_string())
        );
        assert_eq!(
            resolve_container(ContainerKind::Map, "int64_t", Some("const char*"), &C_MODEL),
            Some("int64_t*".to_string())
        );
    }

    #[test]
    fn unknown_named_returns_none() {
        assert_eq!(resolve_named("CompletelyUnknown", &RUST_MODEL), None);
    }
}
