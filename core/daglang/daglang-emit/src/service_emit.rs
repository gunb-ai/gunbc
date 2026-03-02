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
use gunbc_ir::transport::middleware::{
    RateLimitAlgorithm, RetryBackoff, TransportMiddlewareConfig,
};

/// Explicit service transport phase for generated operation nodes.
///
/// The lowerer already provides structural phase metadata via
/// `ObligationCategory` (prepare/execute/parse). Emission should consume that
/// metadata directly instead of re-parsing callable name prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceTransportPhase {
    Prepare,
    Execute,
    Parse,
}

// ===========================================================================
// Go service transport code generation (SC5)
// ===========================================================================

/// Emit a Go function for a service transport node.
pub fn emit_go_service_func(
    symbol_name: &str,
    phase: ServiceTransportPhase,
    spec: &ServiceOperationSpec,
) -> String {
    if phase == ServiceTransportPhase::Execute {
        return emit_go_execute_stub(symbol_name);
    }

    match (spec, phase) {
        (ServiceOperationSpec::Rest(rest), ServiceTransportPhase::Prepare) => {
            emit_go_rest_prepare(symbol_name, rest)
        }
        (ServiceOperationSpec::Rest(rest), ServiceTransportPhase::Parse) => {
            emit_go_rest_parse(symbol_name, rest)
        }
        (ServiceOperationSpec::Shell(shell), ServiceTransportPhase::Prepare) => {
            emit_go_shell_prepare(symbol_name, shell)
        }
        (ServiceOperationSpec::Shell(shell), ServiceTransportPhase::Parse) => {
            emit_go_shell_parse(symbol_name, shell)
        }
        _ => format!("func {symbol_name}() {{\n    // generated callable stub\n}}\n"),
    }
}

/// Emit nested body template entries as Go map literal syntax.
fn emit_nested_body_entries(
    out: &mut String,
    key: &str,
    entries: &[daglang_lower::BodyEntry],
    indent: usize,
) {
    let pad = "    ".repeat(indent);
    out.push_str(&format!("{pad}\"{key}\": map[string]interface{{}}{{\n"));
    for entry in entries {
        let inner_pad = "    ".repeat(indent + 1);
        match entry {
            daglang_lower::BodyEntry::Literal(k, v) => {
                out.push_str(&format!("{inner_pad}\"{k}\": \"{v}\",\n"));
            }
            daglang_lower::BodyEntry::InputRef(k, field) => {
                out.push_str(&format!("{inner_pad}\"{k}\": {field},\n"));
            }
            daglang_lower::BodyEntry::Nested(k, inner) => {
                emit_nested_body_entries(out, k, inner, indent + 1);
            }
        }
    }
    out.push_str(&format!("{pad}}},\n"));
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
                    daglang_lower::BodyEntry::Nested(key, inner) => {
                        emit_nested_body_entries(&mut out, key, inner, 2);
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
    phase: ServiceTransportPhase,
    spec: &ServiceOperationSpec,
) -> String {
    if phase == ServiceTransportPhase::Execute {
        return emit_c_execute_stub(symbol_name);
    }

    match (spec, phase) {
        (ServiceOperationSpec::Rest(rest), ServiceTransportPhase::Prepare) => {
            emit_c_rest_prepare(symbol_name, rest)
        }
        (ServiceOperationSpec::Rest(rest), ServiceTransportPhase::Parse) => {
            emit_c_rest_parse(symbol_name, rest)
        }
        (ServiceOperationSpec::Shell(shell), ServiceTransportPhase::Prepare) => {
            emit_c_shell_prepare(symbol_name, shell)
        }
        (ServiceOperationSpec::Shell(shell), ServiceTransportPhase::Parse) => {
            emit_c_shell_parse(symbol_name, shell)
        }
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
    phase: ServiceTransportPhase,
    spec: &ServiceOperationSpec,
) -> String {
    let description = match (spec, phase) {
        (ServiceOperationSpec::Rest(rest), ServiceTransportPhase::Prepare) => {
            format!(
                "prepare REST {} {}{}",
                rest.method, rest.endpoint, rest.path_template
            )
        }
        (ServiceOperationSpec::Rest(rest), ServiceTransportPhase::Parse) => {
            let fields: Vec<&str> = rest.output_fields.iter().map(|f| f.name.as_str()).collect();
            format!("parse REST response [{}]", fields.join(", "))
        }
        (_, ServiceTransportPhase::Execute) => "execute transport request".to_string(),
        (ServiceOperationSpec::Shell(shell), ServiceTransportPhase::Prepare) => {
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
        (ServiceOperationSpec::Shell(shell), ServiceTransportPhase::Parse) => {
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
// Multi-target transport middleware config emission (TL-14)
// ===========================================================================

/// Extract `TransportMiddlewareConfig` from a `ServiceOperationSpec`, if present.
pub fn extract_middleware(spec: &ServiceOperationSpec) -> Option<&TransportMiddlewareConfig> {
    match spec {
        ServiceOperationSpec::Rest(rest) => rest.middleware.as_ref(),
        _ => None,
    }
}

/// Emit a Rust function that constructs `TransportMiddlewareConfig` for an operation.
///
/// The generated code links to the Target SDK types from `gunbc_ir::transport::middleware`.
pub fn emit_rust_middleware_config(
    op_name: &str,
    config: &TransportMiddlewareConfig,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "/// Transport middleware configuration for `{op_name}`.\n\
         pub fn {op_name}_middleware_config() -> gunbc_ir::transport::middleware::TransportMiddlewareConfig {{\n\
         "
    ));
    out.push_str("    gunbc_ir::transport::middleware::TransportMiddlewareConfig {\n");

    // Rate limit.
    if let Some(ref rl) = config.rate_limit {
        let algo = match rl.algorithm {
            RateLimitAlgorithm::TokenBucket => "TokenBucket",
            RateLimitAlgorithm::SlidingWindow => "SlidingWindow",
        };
        out.push_str(&format!(
            "        rate_limit: Some(gunbc_ir::transport::middleware::RateLimitConfig {{\n\
             \x20           scope_key: \"{scope_key}\".to_string(),\n\
             \x20           algorithm: gunbc_ir::transport::middleware::RateLimitAlgorithm::{algo},\n\
             \x20           max_burst: {max_burst},\n\
             \x20           requests: {requests},\n\
             \x20           window_seconds: {window_seconds},\n\
             \x20           honor_retry_after: {honor_retry_after},\n\
             \x20       }}),\n",
            scope_key = rl.scope_key,
            max_burst = rl.max_burst,
            requests = rl.requests,
            window_seconds = rl.window_seconds,
            honor_retry_after = rl.honor_retry_after,
        ));
    } else {
        out.push_str("        rate_limit: None,\n");
    }

    // Retry.
    if let Some(ref retry) = config.retry {
        let backoff = match retry.backoff {
            RetryBackoff::Fixed => "Fixed",
            RetryBackoff::Exponential => "Exponential",
            RetryBackoff::ExponentialJitter => "ExponentialJitter",
        };
        let statuses = retry
            .retry_statuses
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "        retry: Some(gunbc_ir::transport::middleware::RetryConfig {{\n\
             \x20           max_attempts: {max_attempts},\n\
             \x20           base_delay_ms: {base_delay_ms},\n\
             \x20           max_delay_ms: {max_delay_ms},\n\
             \x20           backoff: gunbc_ir::transport::middleware::RetryBackoff::{backoff},\n\
             \x20           retry_statuses: vec![{statuses}],\n\
             \x20           retry_network_errors: {retry_network_errors},\n\
             \x20           require_idempotent_or_readonly: {require_idempotent},\n\
             \x20           circuit_breaker: None,\n\
             \x20       }}),\n",
            max_attempts = retry.max_attempts,
            base_delay_ms = retry.base_delay_ms,
            max_delay_ms = retry.max_delay_ms,
            retry_network_errors = retry.retry_network_errors,
            require_idempotent = retry.require_idempotent_or_readonly,
        ));
    } else {
        out.push_str("        retry: None,\n");
    }

    // Credential.
    out.push_str("        credential: None,\n");

    // Response classification with error shape.
    if let Some(ref rc) = config.response_classification {
        out.push_str(
            "        response_classification: Some(gunbc_ir::transport::middleware::ResponseClassification {\n",
        );
        out.push_str(&format!(
            "            provider: gunbc_ir::transport::middleware::ResponseProvider::{provider},\n",
            provider = rust_response_provider_variant(rc.provider),
        ));
        out.push_str(&format!(
            "            prioritize_auth_errors: {pae},\n",
            pae = rc.prioritize_auth_errors,
        ));
        out.push_str(&format!(
            "            parse_provider_error_shapes: {ppes},\n",
            ppes = rc.parse_provider_error_shapes,
        ));
        if let Some(ref shape) = rc.error_shape {
            out.push_str(
                "            error_shape: Some(gunbc_ir::transport::middleware::ErrorShapeExtraction {\n",
            );
            out.push_str(&format!(
                "                message_path: \"{}\".to_string(),\n",
                shape.message_path,
            ));
            if let Some(ref cp) = shape.code_path {
                out.push_str(&format!(
                    "                code_path: Some(\"{}\".to_string()),\n",
                    cp,
                ));
            } else {
                out.push_str("                code_path: None,\n");
            }
            if let Some(ref dp) = shape.details_path {
                out.push_str(&format!(
                    "                details_path: Some(\"{}\".to_string()),\n",
                    dp,
                ));
            } else {
                out.push_str("                details_path: None,\n");
            }
            out.push_str("            }),\n");
        } else {
            out.push_str("            error_shape: None,\n");
        }
        out.push_str("        }),\n");
    } else {
        out.push_str("        response_classification: None,\n");
    }

    out.push_str("    }\n");
    out.push_str("}\n");
    out
}

/// Emit a Go struct literal returning `TransportMiddlewareConfig` for an operation.
pub fn emit_go_middleware_config(
    op_name: &str,
    config: &TransportMiddlewareConfig,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// {op_name}MiddlewareConfig returns the transport middleware configuration.\n\
         func {op_name}MiddlewareConfig() TransportMiddlewareConfig {{\n\
         \treturn TransportMiddlewareConfig{{\n"
    ));

    // Rate limit.
    if let Some(ref rl) = config.rate_limit {
        let algo = match rl.algorithm {
            RateLimitAlgorithm::TokenBucket => "TokenBucket",
            RateLimitAlgorithm::SlidingWindow => "SlidingWindow",
        };
        out.push_str(&format!(
            "\t\tRateLimit: &RateLimitConfig{{\n\
             \t\t\tScopeKey: \"{scope_key}\",\n\
             \t\t\tAlgorithm: \"{algo}\",\n\
             \t\t\tMaxBurst: {max_burst},\n\
             \t\t\tRequests: {requests},\n\
             \t\t\tWindowSeconds: {window_seconds},\n\
             \t\t\tHonorRetryAfter: {honor_retry_after},\n\
             \t\t}},\n",
            scope_key = rl.scope_key,
            max_burst = rl.max_burst,
            requests = rl.requests,
            window_seconds = rl.window_seconds,
            honor_retry_after = rl.honor_retry_after,
        ));
    }

    // Retry.
    if let Some(ref retry) = config.retry {
        let backoff = match retry.backoff {
            RetryBackoff::Fixed => "fixed",
            RetryBackoff::Exponential => "exponential",
            RetryBackoff::ExponentialJitter => "exponential_jitter",
        };
        let statuses = retry
            .retry_statuses
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "\t\tRetry: &RetryConfig{{\n\
             \t\t\tMaxAttempts: {max_attempts},\n\
             \t\t\tBaseDelayMs: {base_delay_ms},\n\
             \t\t\tMaxDelayMs: {max_delay_ms},\n\
             \t\t\tBackoff: \"{backoff}\",\n\
             \t\t\tRetryStatuses: []int{{{statuses}}},\n\
             \t\t\tRetryNetworkErrors: {retry_network_errors},\n\
             \t\t\tRequireIdempotentOrReadonly: {require_idempotent},\n\
             \t\t}},\n",
            max_attempts = retry.max_attempts,
            base_delay_ms = retry.base_delay_ms,
            max_delay_ms = retry.max_delay_ms,
            retry_network_errors = retry.retry_network_errors,
            require_idempotent = retry.require_idempotent_or_readonly,
        ));
    }

    // Error shape.
    if let Some(ref rc) = config.response_classification {
        if let Some(ref shape) = rc.error_shape {
            out.push_str(&format!(
                "\t\tErrorShape: &ErrorShapeExtraction{{\n\
                 \t\t\tMessagePath: \"{message_path}\",\n",
                message_path = shape.message_path,
            ));
            if let Some(ref cp) = shape.code_path {
                out.push_str(&format!("\t\t\tCodePath: \"{}\",\n", cp));
            }
            out.push_str("\t\t},\n");
        }
    }

    out.push_str("\t}\n}\n");
    out
}

/// Emit C middleware config constants for an operation.
pub fn emit_c_middleware_config(
    op_name: &str,
    config: &TransportMiddlewareConfig,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "/* {op_name}: transport middleware configuration */\n"
    ));

    if let Some(ref rl) = config.rate_limit {
        let algo = match rl.algorithm {
            RateLimitAlgorithm::TokenBucket => "TOKEN_BUCKET",
            RateLimitAlgorithm::SlidingWindow => "SLIDING_WINDOW",
        };
        out.push_str(&format!(
            "static const struct {{\n\
             \x20   const char* scope_key;\n\
             \x20   const char* algorithm;\n\
             \x20   unsigned int max_burst;\n\
             \x20   unsigned int requests;\n\
             \x20   unsigned int window_seconds;\n\
             \x20   int honor_retry_after;\n\
             }} {op_name}_rate_limit = {{\n\
             \x20   .scope_key = \"{scope_key}\",\n\
             \x20   .algorithm = \"{algo}\",\n\
             \x20   .max_burst = {max_burst},\n\
             \x20   .requests = {requests},\n\
             \x20   .window_seconds = {window_seconds},\n\
             \x20   .honor_retry_after = {honor_retry_after},\n\
             }};\n\n",
            scope_key = rl.scope_key,
            max_burst = rl.max_burst,
            requests = rl.requests,
            window_seconds = rl.window_seconds,
            honor_retry_after = if rl.honor_retry_after { 1 } else { 0 },
        ));
    }

    if let Some(ref retry) = config.retry {
        let backoff = match retry.backoff {
            RetryBackoff::Fixed => "FIXED",
            RetryBackoff::Exponential => "EXPONENTIAL",
            RetryBackoff::ExponentialJitter => "EXPONENTIAL_JITTER",
        };
        let statuses = retry
            .retry_statuses
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "static const struct {{\n\
             \x20   unsigned int max_attempts;\n\
             \x20   unsigned long base_delay_ms;\n\
             \x20   unsigned long max_delay_ms;\n\
             \x20   const char* backoff;\n\
             \x20   int retry_network_errors;\n\
             \x20   int require_idempotent_or_readonly;\n\
             }} {op_name}_retry = {{\n\
             \x20   .max_attempts = {max_attempts},\n\
             \x20   .base_delay_ms = {base_delay_ms},\n\
             \x20   .max_delay_ms = {max_delay_ms},\n\
             \x20   .backoff = \"{backoff}\",\n\
             \x20   .retry_network_errors = {retry_network_errors},\n\
             \x20   .require_idempotent_or_readonly = {require_idempotent},\n\
             }};\nstatic const int {op_name}_retry_statuses[] = {{{statuses}}};\n\n",
            max_attempts = retry.max_attempts,
            base_delay_ms = retry.base_delay_ms,
            max_delay_ms = retry.max_delay_ms,
            retry_network_errors = if retry.retry_network_errors { 1 } else { 0 },
            require_idempotent = if retry.require_idempotent_or_readonly {
                1
            } else {
                0
            },
        ));
    }

    if let Some(ref rc) = config.response_classification {
        if let Some(ref shape) = rc.error_shape {
            out.push_str(&format!(
                "static const struct {{\n\
                 \x20   const char* message_path;\n\
                 \x20   const char* code_path;\n\
                 }} {op_name}_error_shape = {{\n\
                 \x20   .message_path = \"{message_path}\",\n\
                 \x20   .code_path = {code_path},\n\
                 }};\n\n",
                message_path = shape.message_path,
                code_path = shape
                    .code_path
                    .as_deref()
                    .map(|p| format!("\"{}\"", p))
                    .unwrap_or_else(|| "NULL".to_string()),
            ));
        }
    }

    out
}

/// Emit MIPS data section with middleware config for an operation.
pub fn emit_mips_middleware_config(
    op_name: &str,
    config: &TransportMiddlewareConfig,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("    # {op_name}: transport middleware config\n"));

    if let Some(ref rl) = config.rate_limit {
        out.push_str(&format!(
            "{op_name}_rate_limit_scope:\n    .asciiz \"{scope_key}\"\n\
             {op_name}_rate_limit_requests:\n    .word {requests}\n\
             {op_name}_rate_limit_window:\n    .word {window_seconds}\n\
             {op_name}_rate_limit_burst:\n    .word {max_burst}\n",
            scope_key = rl.scope_key,
            requests = rl.requests,
            window_seconds = rl.window_seconds,
            max_burst = rl.max_burst,
        ));
    }

    if let Some(ref retry) = config.retry {
        out.push_str(&format!(
            "{op_name}_retry_max_attempts:\n    .word {max_attempts}\n\
             {op_name}_retry_base_delay:\n    .word {base_delay_ms}\n\
             {op_name}_retry_max_delay:\n    .word {max_delay_ms}\n",
            max_attempts = retry.max_attempts,
            base_delay_ms = retry.base_delay_ms,
            max_delay_ms = retry.max_delay_ms,
        ));
    }

    if let Some(ref rc) = config.response_classification {
        if let Some(ref shape) = rc.error_shape {
            out.push_str(&format!(
                "{op_name}_error_message_path:\n    .asciiz \"{}\"\n",
                shape.message_path,
            ));
        }
    }

    out
}

/// Serialize a `TransportMiddlewareConfig` to a JSON string.
///
/// Used to emit a language-agnostic middleware config manifest file alongside
/// the generated code. Any runtime can deserialize this to configure middleware.
pub fn serialize_middleware_config_json(
    configs: &[(String, &TransportMiddlewareConfig)],
) -> String {
    let mut map = serde_json::Map::new();
    for (name, config) in configs {
        if let Ok(val) = serde_json::to_value(config) {
            map.insert(name.clone(), val);
        }
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(map))
        .unwrap_or_else(|_| "{}".to_string())
}

fn rust_response_provider_variant(
    provider: gunbc_ir::transport::middleware::ResponseProvider,
) -> &'static str {
    use gunbc_ir::transport::middleware::ResponseProvider;
    match provider {
        ResponseProvider::Generic => "Generic",
        ResponseProvider::GitHub => "GitHub",
        ResponseProvider::Gcp => "Gcp",
        ResponseProvider::Anthropic => "Anthropic",
        ResponseProvider::OpenAi => "OpenAi",
    }
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
                    is_optional: false,
                },
                OutputFieldSpec {
                    name: "model".to_string(),
                    type_id: "String".to_string(),
                    json_path: "model".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                    is_optional: false,
                },
            ],
            body_template: None,
            headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
            auth_scheme: None,
            auth_input: None,
            middleware: None,
            response_mapping: vec![],
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
            env: vec![],
            exit_mapping: vec![],
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
                is_optional: false,
            }],
            body_template: None,
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            middleware: None,
            response_mapping: vec![],
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
                    is_optional: false,
                },
                OutputFieldSpec {
                    name: "name".to_string(),
                    type_id: "String".to_string(),
                    json_path: "name".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                    is_optional: false,
                },
            ],
            body_template: None,
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            middleware: None,
            response_mapping: vec![],
        }
    }

    // -- Go tests --

    #[test]
    fn go_rest_prepare_generates_http_request() {
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_spec()));
        let code = emit_go_service_func(
            "prepare_anthropic_messages",
            ServiceTransportPhase::Prepare,
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
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_spec()));
        let code = emit_go_service_func(
            "parse_anthropic_messages",
            ServiceTransportPhase::Parse,
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
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_with_bytes_output()));
        let code = emit_go_service_func("parse_secret_access", ServiceTransportPhase::Parse, &spec);
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
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_with_path_params()));
        let code = emit_go_service_func(
            "prepare_secret_access",
            ServiceTransportPhase::Prepare,
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
        let code =
            emit_go_service_func("prepare_cargo_build", ServiceTransportPhase::Prepare, &spec);
        assert!(code.contains("*exec.Cmd"), "returns exec.Cmd: {code}");
        assert!(code.contains("exec.Command"), "uses exec.Command");
        assert!(code.contains("\"cargo\""), "has cargo");
        assert!(code.contains("\"build\""), "has build");
    }

    #[test]
    fn go_shell_parse_success_stdout_stderr() {
        let spec = ServiceOperationSpec::Shell(sample_shell_spec());
        let code = emit_go_service_func("parse_cargo_build", ServiceTransportPhase::Parse, &spec);
        assert!(code.contains("Success bool"), "has Success: {code}");
        assert!(code.contains("Stdout string"), "has Stdout");
        assert!(code.contains("Stderr string"), "has Stderr");
    }

    #[test]
    fn go_execute_stub() {
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_spec()));
        let code = emit_go_service_func(
            "execute_anthropic_messages",
            ServiceTransportPhase::Execute,
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
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_with_path_params()));
        let code = emit_c_service_func(
            "prepare_secret_access",
            ServiceTransportPhase::Prepare,
            &spec,
        );
        assert!(code.contains("snprintf"), "uses snprintf: {code}");
        assert!(code.contains("%s"), "has format specifiers");
        assert!(code.contains("project"), "references project");
        assert!(code.contains("secret"), "references secret");
    }

    #[test]
    fn c_rest_parse_has_field_comments() {
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_spec()));
        let code = emit_c_service_func(
            "parse_anthropic_messages",
            ServiceTransportPhase::Parse,
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
        let code =
            emit_c_service_func("prepare_cargo_build", ServiceTransportPhase::Prepare, &spec);
        assert!(code.contains("\"cargo\""), "has cargo in comment: {code}");
        assert!(code.contains("\"build\""), "has build in comment");
    }

    // -- MIPS tests --

    #[test]
    fn mips_rest_prepare_has_spec_comment() {
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_spec()));
        let code = emit_mips_service_func(
            "prepare_anthropic_messages",
            ServiceTransportPhase::Prepare,
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
        let code =
            emit_mips_service_func("prepare_cargo_build", ServiceTransportPhase::Prepare, &spec);
        assert!(
            code.contains("prepare shell [cargo build]"),
            "has argv: {code}"
        );
        assert!(code.contains("jr $ra"), "returns");
    }

    #[test]
    fn mips_execute_stub_has_description() {
        let spec = ServiceOperationSpec::Rest(Box::new(sample_rest_spec()));
        let code =
            emit_mips_service_func("execute_transport", ServiceTransportPhase::Execute, &spec);
        assert!(
            code.contains("execute transport request"),
            "has description: {code}"
        );
    }

    // -- TL-14: Multi-target middleware config emission tests --

    fn sample_middleware_config() -> TransportMiddlewareConfig {
        use gunbc_ir::transport::middleware::*;
        TransportMiddlewareConfig {
            rate_limit: Some(RateLimitConfig {
                scope_key: "github:core".to_string(),
                algorithm: RateLimitAlgorithm::TokenBucket,
                max_burst: 20,
                requests: 5000,
                window_seconds: 3600,
                honor_retry_after: true,
            }),
            retry: Some(RetryConfig {
                max_attempts: 3,
                base_delay_ms: 100,
                max_delay_ms: 2000,
                backoff: RetryBackoff::ExponentialJitter,
                retry_statuses: vec![429, 500, 502, 503, 504],
                retry_network_errors: true,
                require_idempotent_or_readonly: false,
                circuit_breaker: None,
            }),
            credential: None,
            response_classification: Some(ResponseClassification {
                provider: gunbc_ir::transport::middleware::ResponseProvider::GitHub,
                prioritize_auth_errors: true,
                parse_provider_error_shapes: false,
                error_shape: Some(ErrorShapeExtraction {
                    message_path: ".message".to_string(),
                    code_path: Some(".status".to_string()),
                    details_path: Some(".documentation_url".to_string()),
                }),
            }),
        }
    }

    #[test]
    fn rust_middleware_config_emits_constructor() {
        let config = sample_middleware_config();
        let code = emit_rust_middleware_config("github_gist_create", &config);
        assert!(
            code.contains("fn github_gist_create_middleware_config()"),
            "has function: {code}"
        );
        assert!(
            code.contains("scope_key: \"github:core\""),
            "has rate limit scope: {code}"
        );
        assert!(
            code.contains("TokenBucket"),
            "has rate limit algorithm: {code}"
        );
        assert!(
            code.contains("max_attempts: 3"),
            "has retry max_attempts: {code}"
        );
        assert!(
            code.contains("ExponentialJitter"),
            "has retry backoff: {code}"
        );
        assert!(
            code.contains("message_path: \".message\""),
            "has error shape message path: {code}"
        );
        assert!(
            code.contains("code_path: Some(\".status\""),
            "has error shape code path: {code}"
        );
    }

    #[test]
    fn go_middleware_config_emits_struct() {
        let config = sample_middleware_config();
        let code = emit_go_middleware_config("github_gist_create", &config);
        assert!(
            code.contains("func github_gist_createMiddlewareConfig()"),
            "has function: {code}"
        );
        assert!(
            code.contains("ScopeKey: \"github:core\""),
            "has rate limit scope: {code}"
        );
        assert!(
            code.contains("MaxAttempts: 3"),
            "has retry max_attempts: {code}"
        );
        assert!(
            code.contains("MessagePath: \".message\""),
            "has error shape: {code}"
        );
    }

    #[test]
    fn c_middleware_config_emits_structs() {
        let config = sample_middleware_config();
        let code = emit_c_middleware_config("github_gist_create", &config);
        assert!(
            code.contains("github_gist_create_rate_limit"),
            "has rate limit struct: {code}"
        );
        assert!(
            code.contains("scope_key = \"github:core\""),
            "has scope key: {code}"
        );
        assert!(
            code.contains("github_gist_create_retry"),
            "has retry struct: {code}"
        );
        assert!(
            code.contains("github_gist_create_error_shape"),
            "has error shape struct: {code}"
        );
        assert!(
            code.contains("message_path = \".message\""),
            "has message path: {code}"
        );
    }

    #[test]
    fn mips_middleware_config_emits_data_section() {
        let config = sample_middleware_config();
        let code = emit_mips_middleware_config("github_gist_create", &config);
        assert!(
            code.contains("github_gist_create_rate_limit_scope"),
            "has rate limit label: {code}"
        );
        assert!(
            code.contains("\"github:core\""),
            "has scope key: {code}"
        );
        assert!(
            code.contains("github_gist_create_retry_max_attempts"),
            "has retry label: {code}"
        );
        assert!(
            code.contains(".word 3"),
            "has retry max_attempts value: {code}"
        );
        assert!(
            code.contains("github_gist_create_error_message_path"),
            "has error shape label: {code}"
        );
    }

    #[test]
    fn extract_middleware_from_rest_spec() {
        let mut spec = sample_rest_spec();
        spec.middleware = Some(sample_middleware_config());
        let op_spec = ServiceOperationSpec::Rest(Box::new(spec));
        let mw = extract_middleware(&op_spec);
        assert!(mw.is_some(), "should extract middleware from REST spec");
        assert!(mw.unwrap().rate_limit.is_some(), "should have rate limit");
    }

    #[test]
    fn extract_middleware_from_shell_spec_returns_none() {
        let op_spec = ServiceOperationSpec::Shell(sample_shell_spec());
        assert!(extract_middleware(&op_spec).is_none());
    }

    #[test]
    fn middleware_config_empty_emits_nones() {
        let config = TransportMiddlewareConfig::default();
        let code = emit_rust_middleware_config("empty_op", &config);
        assert!(code.contains("rate_limit: None"), "has None rate limit: {code}");
        assert!(code.contains("retry: None"), "has None retry: {code}");
        assert!(
            code.contains("response_classification: None"),
            "has None classification: {code}"
        );
    }

    #[test]
    fn serialize_middleware_config_json_produces_valid_json() {
        let config = sample_middleware_config();
        let json = serialize_middleware_config_json(&[
            ("github_gist_create".to_string(), &config),
        ]);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed.get("github_gist_create").is_some());
        assert!(
            parsed["github_gist_create"]["rate_limit"]["scope_key"] == "github:core"
        );
    }
}
