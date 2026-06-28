pub use crate::std_emit_model::SimpleMethodSpec;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub fn go_keywords() -> Rc<HashMap<String, String>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, String>> = {
            let mut __m = HashMap::new();
            __m.insert("true".to_string(), "true".to_string());
            __m.insert("false".to_string(), "false".to_string());
            __m.insert("null".to_string(), "nil".to_string());
            __m.insert("and".to_string(), "&&".to_string());
            __m.insert("or".to_string(), "||".to_string());
            __m.insert("not".to_string(), "!".to_string());
            __m.insert("div".to_string(), "/".to_string());
            Rc::new(__m)
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, String>>| c.clone())
}

pub fn go_container_templates() -> Rc<HashMap<String, String>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, String>> = {
            let mut __m = HashMap::new();
            __m.insert("list".to_string(), "[]{0}".to_string());
            __m.insert("set".to_string(), "map[{0}]struct{}".to_string());
            __m.insert("optional".to_string(), "*{0}".to_string());
            __m.insert("map".to_string(), "map[{0}]{1}".to_string());
            __m.insert("free_monoid".to_string(), "[]{0}".to_string());
            __m.insert("partial_function".to_string(), "map[{0}]{1}".to_string());
            __m.insert("boolean_algebra".to_string(), "bool".to_string());
            Rc::new(__m)
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, String>>| c.clone())
}

pub fn go_reserved() -> Rc<Vec<String>> {
    thread_local! {
        static CACHED: Rc<Vec<String>> = {
            Rc::new(vec!["break".to_string(), "case".to_string(), "chan".to_string(), "const".to_string(), "continue".to_string(), "default".to_string(), "defer".to_string(), "else".to_string(), "fallthrough".to_string(), "for".to_string(), "func".to_string(), "go".to_string(), "goto".to_string(), "if".to_string(), "import".to_string(), "interface".to_string(), "map".to_string(), "package".to_string(), "range".to_string(), "return".to_string(), "select".to_string(), "struct".to_string(), "switch".to_string(), "type".to_string(), "var".to_string(), "bool".to_string(), "byte".to_string(), "complex64".to_string(), "complex128".to_string(), "error".to_string(), "float32".to_string(), "float64".to_string(), "int".to_string(), "int8".to_string(), "int16".to_string(), "int32".to_string(), "int64".to_string(), "rune".to_string(), "string".to_string(), "uint".to_string(), "uint8".to_string(), "uint16".to_string(), "uint32".to_string(), "uint64".to_string(), "uintptr".to_string(), "true".to_string(), "false".to_string(), "iota".to_string(), "nil".to_string(), "append".to_string(), "cap".to_string(), "close".to_string(), "complex".to_string(), "copy".to_string(), "delete".to_string(), "imag".to_string(), "len".to_string(), "make".to_string(), "new".to_string(), "panic".to_string(), "print".to_string(), "println".to_string(), "real".to_string(), "recover".to_string()])
        };
    }
    CACHED.with(|c: &Rc<Vec<String>>| c.clone())
}

pub fn go_reserved_escape_suffix() -> String {
    thread_local! {
        static CACHED: String = {
            "_".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_func_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "func".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_async_prefix() -> String {
    thread_local! {
        static CACHED: String = {
            "".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_struct_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "struct".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_enum_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "type".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_type_alias_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "type".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_param_separator() -> String {
    thread_local! {
        static CACHED: String = {
            ", ".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_return_arrow() -> String {
    thread_local! {
        static CACHED: String = {
            " ".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_param_type_sep() -> String {
    thread_local! {
        static CACHED: String = {
            " ".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_module_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "package".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_import_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "import".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_import_from_keyword() -> String {
    thread_local! {
        static CACHED: String = {
            "".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_manifest_file() -> String {
    thread_local! {
        static CACHED: String = {
            "go.mod".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_source_extension() -> String {
    thread_local! {
        static CACHED: String = {
            ".go".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_string_types() -> Rc<Vec<String>> {
    thread_local! {
        static CACHED: Rc<Vec<String>> = {
            Rc::new(vec!["String".to_string(), "Secret".to_string()])
        };
    }
    CACHED.with(|c: &Rc<Vec<String>>| c.clone())
}

pub fn go_simple_method_specs() -> Rc<Vec<Rc<SimpleMethodSpec>>> {
    thread_local! {
        static CACHED: Rc<Vec<Rc<SimpleMethodSpec>>> = {
            serde_json::from_value(serde_json::json!([{"method_name": "count", "template": "len({recv})", "wraps_result": false}, {"method_name": "join", "template": "strings.Join({recv}, {arg})", "wraps_result": false}, {"method_name": "split", "template": "strings.Split({recv}, {arg})", "wraps_result": false}, {"method_name": "string_contains", "template": "strings.Contains({recv}, {arg})", "wraps_result": false}, {"method_name": "filter", "template": "v2rt.Filter({recv}, {arg})", "wraps_result": false}, {"method_name": "any", "template": "v2rt.Any({recv}, {arg})", "wraps_result": false}, {"method_name": "all", "template": "v2rt.All({recv}, {arg})", "wraps_result": false}, {"method_name": "flat_map", "template": "v2rt.FlatMap({recv}, {arg})", "wraps_result": false}, {"method_name": "skip", "template": "{recv}[{arg}:]", "wraps_result": false}, {"method_name": "take", "template": "{recv}[:{arg}]", "wraps_result": false}, {"method_name": "sort_by", "template": "v2rt.SortBy({recv}, {arg})", "wraps_result": false}, {"method_name": "append", "template": "append({recv}, {arg})", "wraps_result": false}]))
                .expect("valid data definition")
        };
    }
    CACHED.with(|c: &Rc<Vec<Rc<SimpleMethodSpec>>>| c.clone())
}

pub fn go_method_templates_flat() -> Rc<HashMap<String, String>> {
    go_simple_method_specs().iter().cloned().fold(
        v1_rt::rc_empty_map::<String, String>(),
        |acc: Rc<HashMap<String, String>>, spec: Rc<SimpleMethodSpec>| {
            v1_rt::rc_map_insert(acc, spec.method_name.clone(), spec.template.clone())
        },
    )
}

pub fn go_lambda_template() -> String {
    thread_local! {
        static CACHED: String = {
            "func({0}) interface{} { {1} }".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_error_expr_template() -> String {
    thread_local! {
        static CACHED: String = {
            "panic({0})".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_list_literal_empty() -> String {
    thread_local! {
        static CACHED: String = {
            "[]interface{}{}".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_list_literal_template() -> String {
    thread_local! {
        static CACHED: String = {
            "[]interface{}{{0}}".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_null_coalesce_template() -> String {
    thread_local! {
        static CACHED: String = {
            "func() interface{} { if {0} != nil { return {0} }; return {1} }()".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_error_type_template() -> String {
    thread_local! {
        static CACHED: String = {
            "__EMIT_BUG_{0}__".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_type_arg_open() -> String {
    thread_local! {
        static CACHED: String = {
            "<".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_type_arg_close() -> String {
    thread_local! {
        static CACHED: String = {
            ">".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_void_type() -> String {
    thread_local! {
        static CACHED: String = {
            "".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_tuple_empty() -> String {
    thread_local! {
        static CACHED: String = {
            "struct{}".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_tuple_pair_template() -> String {
    thread_local! {
        static CACHED: String = {
            "struct{ First {0}; Second {1} }".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_tuple_multi_template() -> String {
    thread_local! {
        static CACHED: String = {
            "struct{ {0} }".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn go_tuple_separator() -> String {
    thread_local! {
        static CACHED: String = {
            "; ".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}
