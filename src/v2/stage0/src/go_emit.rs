use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub static GO_TYPE_MAP: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("String".to_string(), "string".to_string()), ("Int".to_string(), "int64".to_string()), ("Float".to_string(), "float64".to_string()), ("Bool".to_string(), "bool".to_string()), ("Bytes".to_string(), "[]byte".to_string()), ("Unit".to_string(), "struct{}".to_string()), ("Secret".to_string(), "string".to_string()), ("Json".to_string(), "interface{}".to_string())])
});

pub static GO_KEYWORDS: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("true".to_string(), "true".to_string()), ("false".to_string(), "false".to_string()), ("null".to_string(), "nil".to_string()), ("and".to_string(), "&&".to_string()), ("or".to_string(), "||".to_string()), ("not".to_string(), "!".to_string()), ("div".to_string(), "/".to_string())])
});

pub static GO_CONTAINER_TEMPLATES: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("list".to_string(), "[]{0}".to_string()), ("set".to_string(), "map[{0}]struct{}".to_string()), ("optional".to_string(), "*{0}".to_string()), ("map".to_string(), "map[{0}]{1}".to_string())])
});

pub static GO_RESERVED: &[&str] = &[
    "break",
    "case",
    "chan",
    "const",
    "continue",
    "default",
    "defer",
    "else",
    "fallthrough",
    "for",
    "func",
    "go",
    "goto",
    "if",
    "import",
    "interface",
    "map",
    "package",
    "range",
    "return",
    "select",
    "struct",
    "switch",
    "type",
    "var",
    "bool",
    "byte",
    "complex64",
    "complex128",
    "error",
    "float32",
    "float64",
    "int",
    "int8",
    "int16",
    "int32",
    "int64",
    "rune",
    "string",
    "uint",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "uintptr",
    "true",
    "false",
    "iota",
    "nil",
    "append",
    "cap",
    "close",
    "complex",
    "copy",
    "delete",
    "imag",
    "len",
    "make",
    "new",
    "panic",
    "print",
    "println",
    "real",
    "recover"
];

pub static GO_RESERVED_ESCAPE_SUFFIX: &str = "_";

pub static GO_SOURCE_EXTENSION: &str = ".go";

pub static GO_MANIFEST_FILE: &str = "go.mod";

