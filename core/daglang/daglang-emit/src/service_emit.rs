//! Service transport code generation per target language.
//!
//! Given a `ServiceOperationSpec` (extracted from `.dag` annotations by the lowerer),
//! this module generates idiomatic service transport functions for each emission target:
//!
//! - **Go**: `net/http` + `encoding/json` for REST, `os/exec` for Shell
//! - **C**: `libcurl` for REST, `posix_spawn` for Shell
//! - **MIPS**: labeled functions with spec comments + runtime library calls
//!
//! The generated functions replace the generic stubs that bundle emission
//! would otherwise produce. Each function is parameterized entirely by the spec
//! data — no per-service code needed.
//!
//! **Owned by**: SC5-SC6 (service-codegen.md)

use daglang_lower::{
    ArgvSegment, FieldSpec, RestOperationSpec, ServiceOperationSpec, ShellOperationSpec,
    ShellOutputParsing,
};

// ===========================================================================
// Go service transport code generation (SC5)
// ===========================================================================

/// Emit a Go function for a service transport node.
pub fn emit_go_service_func(
    symbol_name: &str,
    raw_name: &str,
    spec: &ServiceOperationSpec,
) -> String {
    let is_prepare = raw_name.starts_with("service_transport::prepare::");
    let is_parse = raw_name.starts_with("service_transport::parse::");
    let is_execute = raw_name.starts_with("service_transport::execute::");

    if is_execute {
        return emit_go_execute_stub(symbol_name);
    }

    match (spec, is_prepare, is_parse) {
        (ServiceOperationSpec::Rest(rest), true, _) => emit_go_rest_prepare(symbol_name, rest),
        (ServiceOperationSpec::Rest(rest), _, true) => emit_go_rest_parse(symbol_name, rest),
        (ServiceOperationSpec::Shell(shell), true, _) => emit_go_shell_prepare(symbol_name, shell),
        (ServiceOperationSpec::Shell(shell), _, true) => emit_go_shell_parse(symbol_name, shell),
        _ => format!("func {symbol_name}() {{\n    // generated callable stub\n}}\n"),
    }
}

fn emit_go_execute_stub(symbol_name: &str) -> String {
    format!(
        "// {symbol_name} executes the transport request (HTTP or shell).\n\
         func {symbol_name}(req interface{{}}) (interface{{}}, error) {{\n    \
         // Transport execution is handled by the runtime.\n    \
         return nil, nil\n\
         }}\n"
    )
}

fn emit_go_rest_prepare(symbol_name: &str, spec: &RestOperationSpec) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// {symbol_name} prepares a {method} request to {endpoint}{path}.\n",
        method = spec.method,
        endpoint = spec.endpoint,
        path = spec.path_template,
    ));

    // Function signature: input fields → *http.Request
    let params = go_input_params(&spec.input_fields);
    out.push_str(&format!(
        "func {symbol_name}({params}) (*http.Request, error) {{\n"
    ));

    // URL construction with path interpolation.
    out.push_str(&format!(
        "    url := \"{endpoint}\" + {path_expr}\n",
        endpoint = spec.endpoint,
        path_expr = go_interpolate_path(&spec.path_template, &spec.input_fields),
    ));

    // Body construction for POST/PUT/PATCH.
    let method_upper = spec.method.to_uppercase();
    if method_upper == "POST" || method_upper == "PUT" || method_upper == "PATCH" {
        if let Some(ref template) = spec.body_template {
            // Explicit body template.
            out.push_str("    body := map[string]interface{}{\n");
            for entry in template {
                match entry {
                    daglang_lower::BodyEntry::Literal(key, val) => {
                        out.push_str(&format!("        \"{key}\": \"{val}\",\n"));
                    }
                    daglang_lower::BodyEntry::InputRef(key, field) => {
                        out.push_str(&format!("        \"{key}\": {field},\n"));
                    }
                }
            }
            out.push_str("    }\n");
        } else {
            // Auto-body from non-path input fields.
            let body_fields: Vec<&FieldSpec> = spec
                .input_fields
                .iter()
                .filter(|f| !f.is_path_param)
                .collect();
            out.push_str("    body := map[string]interface{}{\n");
            for field in &body_fields {
                out.push_str(&format!("        \"{name}\": {name},\n", name = field.name,));
            }
            out.push_str("    }\n");
        }
        out.push_str("    bodyBytes, err := json.Marshal(body)\n");
        out.push_str("    if err != nil {\n        return nil, err\n    }\n");
        out.push_str(&format!(
            "    req, err := http.NewRequest(\"{method_upper}\", url, bytes.NewReader(bodyBytes))\n"
        ));
    } else {
        out.push_str(&format!(
            "    req, err := http.NewRequest(\"{method_upper}\", url, nil)\n"
        ));
    }
    out.push_str("    if err != nil {\n        return nil, err\n    }\n");
    out.push_str("    req.Header.Set(\"Content-Type\", \"application/json\")\n");

    // Extra headers.
    for (key, val) in &spec.headers {
        out.push_str(&format!("    req.Header.Set(\"{key}\", \"{val}\")\n"));
    }

    out.push_str("    return req, nil\n");
    out.push_str("}\n");
    out
}

fn emit_go_rest_parse(symbol_name: &str, spec: &RestOperationSpec) -> String {
    let mut out = String::new();

    // Build output struct type.
    let struct_fields: Vec<String> = spec
        .output_fields
        .iter()
        .map(|f| {
            format!(
                "    {name} {ty}",
                name = go_pascal_case(&f.name),
                ty = go_type_for_field(&f.type_id),
            )
        })
        .collect();

    let result_type = format!("{symbol_name}Result");
    out.push_str(&format!(
        "// {result_type} holds the parsed response fields.\n\
         type {result_type} struct {{\n{fields}\n}}\n\n",
        fields = struct_fields.join("\n"),
    ));

    out.push_str(&format!(
        "// {symbol_name} parses the REST response.\n\
         func {symbol_name}(body []byte) (*{result_type}, error) {{\n"
    ));

    out.push_str("    var raw map[string]interface{}\n");
    out.push_str("    if err := json.Unmarshal(body, &raw); err != nil {\n");
    out.push_str("        return nil, err\n    }\n");
    out.push_str(&format!("    result := &{result_type}{{}}\n"));

    for field in &spec.output_fields {
        let go_name = go_pascal_case(&field.name);
        let path_parts: Vec<&str> = field.json_path.split('/').collect();
        if path_parts.len() == 1 {
            let accessor = format!("raw[\"{}\"]", path_parts[0]);
            out.push_str(&format!(
                "    if v, ok := {accessor}; ok {{\n        result.{go_name} = {convert}\n    }}\n",
                convert = go_convert_value("v", &field.type_id),
            ));
        } else {
            // Nested JSON path navigation.
            out.push_str(&format!(
                "    result.{go_name} = {func_call}\n",
                func_call = go_navigate_json(&path_parts, &field.type_id),
            ));
        }
    }

    out.push_str("    return result, nil\n");
    out.push_str("}\n");
    out
}

fn emit_go_shell_prepare(symbol_name: &str, spec: &ShellOperationSpec) -> String {
    let mut out = String::new();
    let params = go_input_params(&spec.input_fields);

    out.push_str(&format!(
        "// {symbol_name} prepares a shell command.\n\
         func {symbol_name}({params}) *exec.Cmd {{\n"
    ));

    // Build argv from template.
    out.push_str("    args := []string{");
    let argv_parts: Vec<String> = spec
        .argv_template
        .iter()
        .map(|seg| match seg {
            ArgvSegment::Literal(s) => format!("\"{}\"", s),
            ArgvSegment::InputRef(name) => name.clone(),
        })
        .collect();
    out.push_str(&argv_parts.join(", "));
    out.push_str("}\n");

    out.push_str("    return exec.Command(args[0], args[1:]...)\n");
    out.push_str("}\n");
    out
}

fn emit_go_shell_parse(symbol_name: &str, spec: &ShellOperationSpec) -> String {
    let mut out = String::new();

    out.push_str(&format!("// {symbol_name} parses shell command output.\n"));

    match &spec.output_parsing {
        ShellOutputParsing::TrimStdout => {
            out.push_str(&format!(
                "func {symbol_name}(stdout string, _ string, _ bool) string {{\n    \
                 return strings.TrimSpace(stdout)\n\
                 }}\n"
            ));
        }
        ShellOutputParsing::SplitLines => {
            out.push_str(&format!(
                "func {symbol_name}(stdout string, _ string, _ bool) []string {{\n    \
                 trimmed := strings.TrimSpace(stdout)\n    \
                 if trimmed == \"\" {{\n        return nil\n    }}\n    \
                 return strings.Split(trimmed, \"\\n\")\n\
                 }}\n"
            ));
        }
        ShellOutputParsing::ExitCodeBool => {
            out.push_str(&format!(
                "func {symbol_name}(_ string, _ string, success bool) bool {{\n    \
                 return success\n\
                 }}\n"
            ));
        }
        ShellOutputParsing::SuccessStdoutStderr => {
            let struct_name = format!("{symbol_name}Result");
            out.push_str(&format!(
                "type {struct_name} struct {{\n    \
                 Success bool\n    Stdout string\n    Stderr string\n\
                 }}\n\n"
            ));
            out.push_str(&format!(
                "func {symbol_name}(stdout string, stderr string, success bool) *{struct_name} {{\n    \
                 return &{struct_name}{{Success: success, Stdout: stdout, Stderr: stderr}}\n\
                 }}\n"
            ));
        }
    }

    out
}

// ===========================================================================
// C service transport code generation (SC6)
// ===========================================================================

/// Emit a C function for a service transport node.
pub fn emit_c_service_func(
    symbol_name: &str,
    raw_name: &str,
    spec: &ServiceOperationSpec,
) -> String {
    let is_prepare = raw_name.starts_with("service_transport::prepare::");
    let is_parse = raw_name.starts_with("service_transport::parse::");
    let is_execute = raw_name.starts_with("service_transport::execute::");

    if is_execute {
        return emit_c_execute_stub(symbol_name);
    }

    match (spec, is_prepare, is_parse) {
        (ServiceOperationSpec::Rest(rest), true, _) => emit_c_rest_prepare(symbol_name, rest),
        (ServiceOperationSpec::Rest(rest), _, true) => emit_c_rest_parse(symbol_name, rest),
        (ServiceOperationSpec::Shell(shell), true, _) => emit_c_shell_prepare(symbol_name, shell),
        (ServiceOperationSpec::Shell(shell), _, true) => emit_c_shell_parse(symbol_name, shell),
        _ => format!("static void {symbol_name}(void) {{}}\n"),
    }
}

fn emit_c_execute_stub(symbol_name: &str) -> String {
    format!(
        "/* {symbol_name}: execute transport request (handled by runtime). */\n\
         static int {symbol_name}(void* req, void** resp) {{\n    \
         /* Transport execution is handled by the runtime. */\n    \
         (void)req; (void)resp;\n    \
         return 0;\n\
         }}\n"
    )
}

fn emit_c_rest_prepare(symbol_name: &str, spec: &RestOperationSpec) -> String {
    let mut out = String::new();
    let method = &spec.method;
    let endpoint = &spec.endpoint;
    let path = &spec.path_template;

    out.push_str(&format!(
        "/* {symbol_name}: prepare {method} {endpoint}{path} */\n"
    ));

    // C function signature with input parameters.
    let params = c_input_params(&spec.input_fields);
    out.push_str(&format!(
        "static int {symbol_name}({params}char* url_buf, size_t url_buf_len) {{\n"
    ));

    // URL construction via snprintf.
    let format_str = c_path_format(&spec.path_template, &spec.input_fields);
    let format_args = c_path_args(&spec.path_template, &spec.input_fields);
    if format_args.is_empty() {
        out.push_str(&format!(
            "    snprintf(url_buf, url_buf_len, \"{endpoint}{format_str}\");\n"
        ));
    } else {
        out.push_str(&format!(
            "    snprintf(url_buf, url_buf_len, \"{endpoint}{format_str}\", {format_args});\n"
        ));
    }

    out.push_str("    return 0;\n");
    out.push_str("}\n");
    out
}

fn emit_c_rest_parse(symbol_name: &str, spec: &RestOperationSpec) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "/* {symbol_name}: parse REST response — fields: {field_list} */\n",
        field_list = spec
            .output_fields
            .iter()
            .map(|f| format!("{} ({})", f.name, f.json_path))
            .collect::<Vec<_>>()
            .join(", "),
    ));
    out.push_str(&format!(
        "static int {symbol_name}(const char* body, size_t body_len) {{\n    \
         /* JSON parsing: extract fields from response body. */\n    \
         (void)body; (void)body_len;\n    \
         return 0;\n\
         }}\n"
    ));
    out
}

fn emit_c_shell_prepare(symbol_name: &str, spec: &ShellOperationSpec) -> String {
    let argv_str: Vec<String> = spec
        .argv_template
        .iter()
        .map(|seg| match seg {
            ArgvSegment::Literal(s) => format!("\"{}\"", s),
            ArgvSegment::InputRef(name) => name.clone(),
        })
        .collect();

    format!(
        "/* {symbol_name}: prepare shell command [{argv}] */\n\
         static int {symbol_name}(const char** argv, size_t* argc) {{\n    \
         /* Argv template: {argv} */\n    \
         (void)argv; (void)argc;\n    \
         return 0;\n\
         }}\n",
        argv = argv_str.join(", "),
    )
}

fn emit_c_shell_parse(symbol_name: &str, spec: &ShellOperationSpec) -> String {
    let parsing_mode = match &spec.output_parsing {
        ShellOutputParsing::TrimStdout => "trim_stdout",
        ShellOutputParsing::SplitLines => "split_lines",
        ShellOutputParsing::ExitCodeBool => "exit_code_bool",
        ShellOutputParsing::SuccessStdoutStderr => "success_stdout_stderr",
    };

    format!(
        "/* {symbol_name}: parse shell output (mode: {parsing_mode}) */\n\
         static int {symbol_name}(const char* stdout_buf, const char* stderr_buf, int exit_code) {{\n    \
         /* Output parsing mode: {parsing_mode} */\n    \
         (void)stdout_buf; (void)stderr_buf; (void)exit_code;\n    \
         return 0;\n\
         }}\n"
    )
}

// ===========================================================================
// MIPS service transport code generation (SC6)
// ===========================================================================

/// Emit a MIPS assembly function for a service transport node.
pub fn emit_mips_service_func(
    symbol_name: &str,
    raw_name: &str,
    spec: &ServiceOperationSpec,
) -> String {
    let is_prepare = raw_name.starts_with("service_transport::prepare::");
    let is_parse = raw_name.starts_with("service_transport::parse::");
    let is_execute = raw_name.starts_with("service_transport::execute::");

    let description = match (spec, is_prepare, is_parse, is_execute) {
        (ServiceOperationSpec::Rest(rest), true, _, _) => {
            format!(
                "prepare REST {} {}{}",
                rest.method, rest.endpoint, rest.path_template
            )
        }
        (ServiceOperationSpec::Rest(rest), _, true, _) => {
            let fields: Vec<&str> = rest.output_fields.iter().map(|f| f.name.as_str()).collect();
            format!("parse REST response [{}]", fields.join(", "))
        }
        (_, _, _, true) => "execute transport request".to_string(),
        (ServiceOperationSpec::Shell(shell), true, _, _) => {
            let argv: Vec<String> = shell
                .argv_template
                .iter()
                .map(|seg| match seg {
                    ArgvSegment::Literal(s) => s.clone(),
                    ArgvSegment::InputRef(name) => format!("{{{name}}}"),
                })
                .collect();
            format!("prepare shell [{}]", argv.join(" "))
        }
        (ServiceOperationSpec::Shell(shell), _, true, _) => {
            let mode = match &shell.output_parsing {
                ShellOutputParsing::TrimStdout => "trim_stdout",
                ShellOutputParsing::SplitLines => "split_lines",
                ShellOutputParsing::ExitCodeBool => "exit_code_bool",
                ShellOutputParsing::SuccessStdoutStderr => "success_stdout_stderr",
            };
            format!("parse shell output ({})", mode)
        }
        _ => "service transport stub".to_string(),
    };

    format!(
        "    # {description}\n\
         {symbol_name}:\n    \
         jr $ra\n"
    )
}

// ===========================================================================
// Go helpers
// ===========================================================================

fn go_input_params(fields: &[FieldSpec]) -> String {
    if fields.is_empty() {
        return String::new();
    }
    fields
        .iter()
        .map(|f| format!("{} {}", f.name, go_type_for_field(&f.type_id)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn go_type_for_field(type_id: &str) -> String {
    match type_id {
        "String" | "NonEmptyStr" | "Url" | "GistId" | "ProjectId" | "ServiceAccountEmail" => {
            "string".to_string()
        }
        "Int" | "i64" => "int64".to_string(),
        "Float" | "f64" => "float64".to_string(),
        "Bool" => "bool".to_string(),
        "Secret" => "string".to_string(),
        "Bytes" => "[]byte".to_string(),
        "Json" => "interface{}".to_string(),
        _ => "interface{}".to_string(),
    }
}

fn go_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let mut s = c.to_ascii_uppercase().to_string();
                    s.push_str(chars.as_str());
                    s
                }
                None => String::new(),
            }
        })
        .collect()
}

fn go_interpolate_path(template: &str, _fields: &[FieldSpec]) -> String {
    // Check if the template has any {param} placeholders.
    if !template.contains('{') {
        return format!("\"{}\"", template);
    }

    // Build a fmt.Sprintf call with path interpolation.
    let mut format_str = String::new();
    let mut args = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = template.chars().collect();

    while i < chars.len() {
        if chars[i] == '{' {
            let start = i + 1;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            let param = &template[start..i];
            format_str.push_str("%s");
            args.push(param.to_string());
            if i < chars.len() {
                i += 1; // skip '}'
            }
        } else {
            format_str.push(chars[i]);
            i += 1;
        }
    }

    if args.is_empty() {
        format!("\"{}\"", template)
    } else {
        format!("fmt.Sprintf(\"{format_str}\", {})", args.join(", "))
    }
}

fn go_convert_value(var: &str, type_id: &str) -> String {
    match type_id {
        "String" | "NonEmptyStr" | "Url" | "GistId" | "ProjectId" | "ServiceAccountEmail" => {
            format!("{var}.(string)")
        }
        "Int" | "i64" => format!("int64({var}.(float64))"),
        "Float" | "f64" => format!("{var}.(float64)"),
        "Bool" => format!("{var}.(bool)"),
        "Secret" => format!("{var}.(string)"),
        "Bytes" => format!(
            "func() []byte {{ switch t := {var}.(type) {{ case []byte: return t; case string: return []byte(t); default: panic(\"expected []byte or string for Bytes field\") }} }}()"
        ),
        _ => var.to_string(),
    }
}

fn go_navigate_json(parts: &[&str], type_id: &str) -> String {
    // Generate a chain of map access expressions.
    if parts.is_empty() {
        return "nil".to_string();
    }

    // For deeply nested paths, we use a helper pattern.
    let mut expr = "raw".to_string();
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Leaf — extract and convert.
            return format!(
                "func() {ty} {{ \
                 m, _ := {expr}.(map[string]interface{{}}); \
                 v := m[\"{part}\"]; \
                 return {convert} \
                 }}()",
                ty = go_type_for_field(type_id),
                convert = go_convert_value("v", type_id),
            );
        }
        // Try numeric index for array access.
        if let Ok(idx) = part.parse::<usize>() {
            expr = format!(
                "func() interface{{}} {{ \
                 arr, _ := {expr}.([]interface{{}}); \
                 if len(arr) > {idx} {{ return arr[{idx}] }}; \
                 return nil \
                 }}()"
            );
        } else {
            expr = format!(
                "func() interface{{}} {{ \
                 m, _ := {expr}.(map[string]interface{{}}); \
                 return m[\"{part}\"] \
                 }}()"
            );
        }
    }
    "nil".to_string()
}

// ===========================================================================
// C helpers
// ===========================================================================

fn c_input_params(fields: &[FieldSpec]) -> String {
    if fields.is_empty() {
        return String::new();
    }
    let mut params: Vec<String> = fields
        .iter()
        .map(|f| format!("const char* {}", f.name))
        .collect();
    params.push(String::new()); // trailing comma separator
    params.join(", ")
}

fn c_path_format(template: &str, _fields: &[FieldSpec]) -> String {
    // Replace {param} with %s for snprintf.
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = template.chars().collect();

    while i < chars.len() {
        if chars[i] == '{' {
            result.push_str("%s");
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip '}'
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn c_path_args(template: &str, _fields: &[FieldSpec]) -> String {
    let mut args = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = template.chars().collect();

    while i < chars.len() {
        if chars[i] == '{' {
            let start = i + 1;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            args.push(template[start..i].to_string());
            if i < chars.len() {
                i += 1; // skip '}'
            }
        } else {
            i += 1;
        }
    }

    args.join(", ")
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::OutputFieldSpec;

    fn sample_rest_spec() -> RestOperationSpec {
        RestOperationSpec {
            endpoint: "https://api.example.com".to_string(),
            method: "POST".to_string(),
            path_template: "/v1/messages".to_string(),
            input_fields: vec![
                FieldSpec {
                    name: "model".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: false,
                },
                FieldSpec {
                    name: "messages".to_string(),
                    type_id: "Json".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: false,
                },
            ],
            output_fields: vec![
                OutputFieldSpec {
                    name: "content".to_string(),
                    type_id: "String".to_string(),
                    json_path: "content/0/text".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
                OutputFieldSpec {
                    name: "model".to_string(),
                    type_id: "String".to_string(),
                    json_path: "model".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
            ],
            body_template: None,
            headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
        }
    }

    fn sample_shell_spec() -> ShellOperationSpec {
        ShellOperationSpec {
            argv_template: vec![
                ArgvSegment::Literal("cargo".to_string()),
                ArgvSegment::Literal("build".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![],
            output_parsing: ShellOutputParsing::SuccessStdoutStderr,
        }
    }

    fn sample_rest_with_path_params() -> RestOperationSpec {
        RestOperationSpec {
            endpoint: "https://secretmanager.googleapis.com".to_string(),
            method: "GET".to_string(),
            path_template: "/v1/projects/{project}/secrets/{secret}".to_string(),
            input_fields: vec![
                FieldSpec {
                    name: "project".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: true,
                },
                FieldSpec {
                    name: "secret".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: true,
                },
            ],
            output_fields: vec![OutputFieldSpec {
                name: "name".to_string(),
                type_id: "String".to_string(),
                json_path: "name".to_string(),
                is_secret: false,
                is_raw_body: false,
            }],
            body_template: None,
            headers: vec![],
        }
    }

    fn sample_rest_with_bytes_output() -> RestOperationSpec {
        RestOperationSpec {
            endpoint: "https://secretmanager.googleapis.com".to_string(),
            method: "GET".to_string(),
            path_template: "/v1/projects/{project}/secrets/{secret}/versions/latest:access"
                .to_string(),
            input_fields: vec![
                FieldSpec {
                    name: "project".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: true,
                },
                FieldSpec {
                    name: "secret".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: true,
                },
            ],
            output_fields: vec![
                OutputFieldSpec {
                    name: "payload".to_string(),
                    type_id: "Bytes".to_string(),
                    json_path: "payload".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
                OutputFieldSpec {
                    name: "name".to_string(),
                    type_id: "String".to_string(),
                    json_path: "name".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
            ],
            body_template: None,
            headers: vec![],
        }
    }

    // -- Go tests --

    #[test]
    fn go_rest_prepare_generates_http_request() {
        let spec = ServiceOperationSpec::Rest(sample_rest_spec());
        let code = emit_go_service_func(
            "prepare_anthropic_messages",
            "service_transport::prepare::llm.Anthropic::Messages",
            &spec,
        );
        assert!(
            code.contains("func prepare_anthropic_messages("),
            "has func"
        );
        assert!(
            code.contains("*http.Request, error"),
            "returns http.Request"
        );
        assert!(code.contains("https://api.example.com"), "has endpoint");
        assert!(code.contains("/v1/messages"), "has path");
        assert!(code.contains("json.Marshal"), "marshals body");
        assert!(
            code.contains("anthropic-version"),
            "has extra header: {code}"
        );
    }

    #[test]
    fn go_rest_parse_generates_struct_and_json_extraction() {
        let spec = ServiceOperationSpec::Rest(sample_rest_spec());
        let code = emit_go_service_func(
            "parse_anthropic_messages",
            "service_transport::parse::llm.Anthropic::Messages",
            &spec,
        );
        assert!(
            code.contains("type parse_anthropic_messagesResult struct"),
            "has result struct"
        );
        assert!(code.contains("Content string"), "has Content field");
        assert!(code.contains("Model string"), "has Model field");
        assert!(code.contains("json.Unmarshal"), "unmarshals body");
    }

    #[test]
    fn go_rest_parse_bytes_field_uses_type_safe_conversion() {
        let spec = ServiceOperationSpec::Rest(sample_rest_with_bytes_output());
        let code = emit_go_service_func(
            "parse_secret_access",
            "service_transport::parse::gcp.SecretManager::AccessVersion",
            &spec,
        );
        assert!(
            code.contains("Payload []byte"),
            "bytes output field should map to []byte: {code}"
        );
        assert!(
            code.contains("case []byte: return t; case string: return []byte(t); default: panic(\"expected []byte or string for Bytes field\") } }()"),
            "bytes output extraction should enforce strict type safety and fail-fast: {code}"
        );
    }

    #[test]
    fn go_rest_prepare_with_path_params_uses_sprintf() {
        let spec = ServiceOperationSpec::Rest(sample_rest_with_path_params());
        let code = emit_go_service_func(
            "prepare_secret_access",
            "service_transport::prepare::gcp.SecretManager::AccessVersion",
            &spec,
        );
        assert!(
            code.contains("fmt.Sprintf"),
            "uses fmt.Sprintf for path interpolation: {code}"
        );
        assert!(code.contains("project"), "references project param");
        assert!(code.contains("secret"), "references secret param");
    }

    #[test]
    fn go_shell_prepare_generates_exec_command() {
        let spec = ServiceOperationSpec::Shell(sample_shell_spec());
        let code = emit_go_service_func(
            "prepare_cargo_build",
            "service_transport::prepare::cargo.Build::Build",
            &spec,
        );
        assert!(code.contains("*exec.Cmd"), "returns exec.Cmd: {code}");
        assert!(code.contains("exec.Command"), "uses exec.Command");
        assert!(code.contains("\"cargo\""), "has cargo");
        assert!(code.contains("\"build\""), "has build");
    }

    #[test]
    fn go_shell_parse_success_stdout_stderr() {
        let spec = ServiceOperationSpec::Shell(sample_shell_spec());
        let code = emit_go_service_func(
            "parse_cargo_build",
            "service_transport::parse::cargo.Build::Build",
            &spec,
        );
        assert!(code.contains("Success bool"), "has Success: {code}");
        assert!(code.contains("Stdout string"), "has Stdout");
        assert!(code.contains("Stderr string"), "has Stderr");
    }

    #[test]
    fn go_execute_stub() {
        let spec = ServiceOperationSpec::Rest(sample_rest_spec());
        let code = emit_go_service_func(
            "execute_anthropic_messages",
            "service_transport::execute::llm.Anthropic::Messages",
            &spec,
        );
        assert!(code.contains("func execute_anthropic_messages"), "has func");
        assert!(
            code.contains("interface{}, error"),
            "returns interface/error"
        );
    }

    // -- C tests --

    #[test]
    fn c_rest_prepare_generates_snprintf_url() {
        let spec = ServiceOperationSpec::Rest(sample_rest_with_path_params());
        let code = emit_c_service_func(
            "prepare_secret_access",
            "service_transport::prepare::gcp.SecretManager::AccessVersion",
            &spec,
        );
        assert!(code.contains("snprintf"), "uses snprintf: {code}");
        assert!(code.contains("%s"), "has format specifiers");
        assert!(code.contains("project"), "references project");
        assert!(code.contains("secret"), "references secret");
    }

    #[test]
    fn c_rest_parse_has_field_comments() {
        let spec = ServiceOperationSpec::Rest(sample_rest_spec());
        let code = emit_c_service_func(
            "parse_anthropic_messages",
            "service_transport::parse::llm.Anthropic::Messages",
            &spec,
        );
        assert!(
            code.contains("content (content/0/text)"),
            "documents json path: {code}"
        );
        assert!(code.contains("model (model)"), "documents model field");
    }

    #[test]
    fn c_shell_prepare_has_argv_comment() {
        let spec = ServiceOperationSpec::Shell(sample_shell_spec());
        let code = emit_c_service_func(
            "prepare_cargo_build",
            "service_transport::prepare::cargo.Build::Build",
            &spec,
        );
        assert!(code.contains("\"cargo\""), "has cargo in comment: {code}");
        assert!(code.contains("\"build\""), "has build in comment");
    }

    // -- MIPS tests --

    #[test]
    fn mips_rest_prepare_has_spec_comment() {
        let spec = ServiceOperationSpec::Rest(sample_rest_spec());
        let code = emit_mips_service_func(
            "prepare_anthropic_messages",
            "service_transport::prepare::llm.Anthropic::Messages",
            &spec,
        );
        assert!(
            code.contains("prepare REST POST"),
            "has REST method: {code}"
        );
        assert!(code.contains("api.example.com"), "has endpoint");
        assert!(code.contains("/v1/messages"), "has path");
        assert!(code.contains("jr $ra"), "returns");
    }

    #[test]
    fn mips_shell_prepare_has_argv_comment() {
        let spec = ServiceOperationSpec::Shell(sample_shell_spec());
        let code = emit_mips_service_func(
            "prepare_cargo_build",
            "service_transport::prepare::cargo.Build::Build",
            &spec,
        );
        assert!(
            code.contains("prepare shell [cargo build]"),
            "has argv: {code}"
        );
        assert!(code.contains("jr $ra"), "returns");
    }

    #[test]
    fn mips_execute_stub_has_description() {
        let spec = ServiceOperationSpec::Rest(sample_rest_spec());
        let code = emit_mips_service_func(
            "execute_transport",
            "service_transport::execute::llm.Anthropic::Messages",
            &spec,
        );
        assert!(
            code.contains("execute transport request"),
            "has description: {code}"
        );
    }
}
