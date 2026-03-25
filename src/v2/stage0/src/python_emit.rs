use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub static PYTHON_TYPE_MAP: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("String".to_string(), "str".to_string()), ("Int".to_string(), "int".to_string()), ("Float".to_string(), "float".to_string()), ("Bool".to_string(), "bool".to_string()), ("Bytes".to_string(), "bytes".to_string()), ("Unit".to_string(), "None".to_string()), ("Secret".to_string(), "str".to_string()), ("Json".to_string(), "dict".to_string())])
});

pub static PYTHON_KEYWORDS: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("true".to_string(), "True".to_string()), ("false".to_string(), "False".to_string()), ("null".to_string(), "None".to_string()), ("and".to_string(), "and".to_string()), ("or".to_string(), "or".to_string()), ("not".to_string(), "not ".to_string()), ("div".to_string(), "//".to_string())])
});

pub static PYTHON_CONTAINER_TEMPLATES: std::sync::LazyLock<std::collections::HashMap<String, String>> = std::sync::LazyLock::new(|| {
    HashMap::from([("list".to_string(), "list[{0}]".to_string()), ("set".to_string(), "set[{0}]".to_string()), ("optional".to_string(), "Optional[{0}]".to_string()), ("map".to_string(), "dict[{0}, {1}]".to_string())])
});

pub static PYTHON_RESERVED: &[&str] = &[
    "False",
    "None",
    "True",
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "class",
    "continue",
    "def",
    "del",
    "elif",
    "else",
    "except",
    "finally",
    "for",
    "from",
    "global",
    "if",
    "import",
    "in",
    "is",
    "lambda",
    "nonlocal",
    "not",
    "or",
    "pass",
    "raise",
    "return",
    "try",
    "while",
    "with",
    "yield",
    "type",
    "match",
    "case"
];

pub static PYTHON_RESERVED_ESCAPE_SUFFIX: &str = "_";

pub static PYTHON_DERIVE_ATTRIBUTE: &str = "@dataclass";

pub static PYTHON_DEFAULT_VALUE: &str = "None";

pub static PYTHON_SOURCE_EXTENSION: &str = ".py";

pub static PYTHON_MODULE_INIT: &str = "__init__.py";

