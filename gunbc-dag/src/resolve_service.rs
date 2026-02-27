//! Generic service operation interpreters driven by `ServiceOperationSpec`.
//!
//! These interpreters replace per-service adapter structs. A single
//! `GenericRestPrepareOp` handles *all* REST service operations;
//! a single `GenericShellPrepareOp` handles *all* shell service operations.
//! The behaviour is parameterised by the spec extracted from `.dag` annotations.

use std::collections::{BTreeSet, HashMap};

use daglang_lower::{
    ArgvSegment, BodyEntry, FieldSpec, FileOperationSpec, LocalOperationSpec, OutputFieldSpec,
    RestOperationSpec, ShellOperationSpec, ShellOutputParsing,
};
use gunbc_exec::{
    decorate_service_failure, AuthContext, ExecError, Executable, OutputMap, ServiceCallMetadata,
    TransportContext,
};
use gunbc_ir::transport::{
    FileRequest, LocalRequest, RestRequest, ShellRequest, TransportRequest, TransportResponse,
};
use gunbc_ir::{SecretString, Value};

// ============================================================================
// REST
// ============================================================================

/// Generic REST prepare: builds a `RestRequest` from a `RestOperationSpec`.
#[derive(Debug, Clone)]
pub struct GenericRestPrepareOp {
    pub spec: RestOperationSpec,
}

impl Executable for GenericRestPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Propagate skip from upstream (e.g., non-selected match branches).
        if inputs.values().any(|v| matches!(v, Value::Skipped)) {
            return OutputMap::new().value("request", Value::Skipped).ok();
        }

        // Skip when required non-config inputs are missing (e.g., param_source
        // nodes in non-taken match branches whose data edges were never wired).
        // Config inputs are excluded — missing config is a real user error.
        let has_missing_required = self.spec.input_fields.iter().any(|field| {
            field.default.is_none()
                && !field.name.starts_with("config.")
                && !inputs.contains_key(&field.name)
        });
        if has_missing_required {
            return OutputMap::new().value("request", Value::Skipped).ok();
        }

        ensure_required_profile_config_inputs(&self.spec, &inputs)?;

        // 1. Interpolate path parameters.
        let mut path = self.spec.path_template.clone();
        for field in &self.spec.input_fields {
            if field.is_path_param {
                let placeholder = format!("{{{}}}", field.name);
                let value = input_as_string(&inputs, &field.name, field.default.as_deref());
                path = path.replace(&placeholder, &value);
            }
        }

        // 2. Build full URL.
        let url = if self.spec.endpoint.is_empty() {
            path
        } else {
            format!("{}{}", self.spec.endpoint.trim_end_matches('/'), path)
        };

        // 3. Create request with correct HTTP method.
        let mut request = match self.spec.method.as_str() {
            "GET" => RestRequest::get(&url),
            "POST" => RestRequest::post(&url),
            "PUT" => RestRequest::put(&url),
            "PATCH" => RestRequest::patch(&url),
            "DELETE" => RestRequest::delete(&url),
            _ => RestRequest::post(&url),
        };

        // 3b. Coerce List<String> → String when the declared field type is "String".
        // This handles e.g. `content: listing.files` where a List<String> flows into a
        // service input declared as String — join with newlines for the HTTP body.
        let coercions: Vec<(String, String)> = self
            .spec
            .input_fields
            .iter()
            .filter(|f| f.type_id == "String")
            .filter_map(|f| {
                if let Some(Value::List(items)) = inputs.get(&f.name) {
                    let joined = items
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    Some((f.name.clone(), joined))
                } else {
                    None
                }
            })
            .collect();
        let mut inputs = inputs;
        for (name, joined) in coercions {
            inputs.insert(name, Value::Str(joined));
        }

        // 4. Build JSON body.
        if self.spec.method != "GET" {
            let body = if let Some(template) = &self.spec.body_template {
                // Explicit body template: use literal constants + input refs + nested objects.
                let map = build_body_from_template(template, &inputs);
                serde_json::Value::Object(map)
            } else {
                // Auto-build body from all non-path, non-header-only input fields.
                let header_fields: BTreeSet<String> = self
                    .spec
                    .headers
                    .iter()
                    .flat_map(|(_, v)| collect_template_placeholders(v))
                    .collect();
                let mut map = serde_json::Map::new();
                for field in &self.spec.input_fields {
                    if field.is_path_param || header_fields.contains(&field.name) {
                        continue;
                    }
                    if let Some(value) = inputs.get(&field.name) {
                        insert_value_as_json(&mut map, &field.name, value);
                    } else if let Some(default) = &field.default {
                        map.insert(
                            field.name.clone(),
                            serde_json::Value::String(default.clone()),
                        );
                    }
                }
                if map.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::Object(map)
                }
            };

            if !body.is_null() {
                request = request.json(body);
            }
        }

        // 5. Add custom headers.
        for (key, value) in &self.spec.headers {
            let header_value = interpolate_template(value, &inputs, &self.spec.input_fields);
            request = request.header(key, header_value);
        }

        // 6. Mark request as requiring auth when scheme is declared.
        // Execute handler will fail-closed if res:credential is missing.
        if self.spec.auth_scheme.is_some() {
            request.requires_auth = true;
        }

        OutputMap::new()
            .request("request", TransportRequest::Rest(request))
            .ok()
    }
}

/// Generic REST parse: extracts output fields from a `RestResponse`.
#[derive(Debug, Clone)]
pub struct GenericRestParseOp {
    pub spec: RestOperationSpec,
    pub service_name: String,
    pub operation_name: String,
    pub auth_scheme: String,
    pub permissions: Vec<String>,
}

impl Executable for GenericRestParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Rest(rest))) => {
                if !rest.is_success() {
                    let body_excerpt = match &rest.body {
                        serde_json::Value::Object(map) => map
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        _ => String::new(),
                    };
                    let detail = if body_excerpt.is_empty() {
                        format!(
                            "{} {} failed (status {})",
                            self.spec.method, self.spec.path_template, rest.status
                        )
                    } else {
                        format!(
                            "{} {} failed (status {}): {}",
                            self.spec.method, self.spec.path_template, rest.status, body_excerpt
                        )
                    };
                    // Determine auth scheme + credential ref.
                    // Prefer explicit @auth scheme; fall back to inferring from
                    // @headers Authorization pattern.
                    let (scheme, cred_ref) = if !self.auth_scheme.is_empty() {
                        (self.auth_scheme.clone(), None)
                    } else {
                        infer_auth_from_headers(&self.spec.headers)
                    };
                    // Attach auth context when we have auth metadata OR when
                    // the HTTP status implies auth failure.
                    let is_auth_status = rest.status == 401 || rest.status == 403;
                    let auth =
                        if !scheme.is_empty() || is_auth_status || !self.permissions.is_empty() {
                            Some(AuthContext {
                                scheme: if scheme.is_empty() {
                                    None
                                } else {
                                    Some(scheme)
                                },
                                credential_ref: cred_ref,
                                required_permissions: self.permissions.clone(),
                                lock_target: rest_lock_target(
                                    &self.spec.method,
                                    &self.spec.endpoint,
                                    &self.spec.path_template,
                                ),
                            })
                        } else {
                            None
                        };

                    let err = decorate_service_failure(
                        ExecError::new(detail),
                        ServiceCallMetadata {
                            provider: self.service_name.clone(),
                            operation: self.operation_name.clone(),
                        },
                        TransportContext::Rest {
                            endpoint: self.spec.endpoint.clone(),
                            method: self.spec.method.clone(),
                            status_code: rest.status,
                            reason: if body_excerpt.is_empty() {
                                None
                            } else {
                                Some(body_excerpt.clone())
                            },
                        },
                        auth,
                    );
                    return Err(err);
                }

                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    let value = extract_output_field(field, &rest.body)?;
                    out = out.value(&field.name, value);
                }
                out.ok()
            }
            Some(Value::Skipped) | None => {
                // Produce default/empty values for all output fields.
                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    out = out.value(&field.name, default_output_value(field));
                }
                out.ok()
            }
            Some(other) => Err(ExecError::new(format!(
                "expected REST response for {} parse, got {:?}",
                self.spec.path_template,
                std::mem::discriminant(other)
            ))),
        }
    }
}

/// Extract a single output field value from a JSON response body.
fn extract_output_field(
    field: &OutputFieldSpec,
    body: &serde_json::Value,
) -> Result<Value, ExecError> {
    // raw_body: entire body as string.
    if field.is_raw_body {
        let text = match body {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        return if field.is_secret {
            Ok(Value::Secret(SecretString::new(text)))
        } else {
            Ok(Value::Str(text))
        };
    }

    // Navigate JSON path (supports dot-separated nested paths like "payload.data").
    let json_val = navigate_json_path(body, &field.json_path);

    // Type-specific conversion.
    match field.type_id.as_str() {
        "Secret" => {
            let s = json_val.and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value::Secret(SecretString::new(s)))
        }
        "Int" => {
            let n = json_val.and_then(|v| v.as_i64()).unwrap_or(0);
            Ok(Value::Int(n))
        }
        "Bool" => {
            let b = json_val.and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(Value::Bool(b))
        }
        "Bytes" => {
            // Base64-encoded payload (e.g., GCP SecretManager).
            // Tries direct field first, then nested .data path.
            let b64 = json_val
                .and_then(|v| {
                    v.as_str().map(|s| s.to_string()).or_else(|| {
                        v.get("data")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string())
                    })
                })
                .unwrap_or_default();
            let bytes = base64_decode(&b64)
                .map_err(|e| ExecError::new(format!("base64 decode for {}: {e}", field.name)))?;
            Ok(Value::List(
                bytes.into_iter().map(|b| Value::Int(b as i64)).collect(),
            ))
        }
        "Json" => {
            let v = json_val.cloned().unwrap_or(serde_json::Value::Null);
            Ok(Value::Json(v))
        }
        // String, Url, GistId, NonEmptyStr, etc. → all as String.
        _ => {
            let s = json_val.and_then(|v| v.as_str()).unwrap_or("");
            if field.is_secret {
                Ok(Value::Secret(SecretString::new(s)))
            } else {
                Ok(Value::Str(s.to_string()))
            }
        }
    }
}

/// Navigate a dot-separated JSON path (e.g., "payload.data").
fn navigate_json_path<'a>(
    body: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    let mut current = body;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// Produce a default/empty value for an output field (used for skipped responses).
fn default_output_value(field: &OutputFieldSpec) -> Value {
    match field.type_id.as_str() {
        "Secret" => Value::Secret(SecretString::new("")),
        "Int" => Value::Int(0),
        "Bool" => Value::Bool(false),
        "Bytes" => Value::List(Vec::new()),
        "Json" => Value::Json(serde_json::Value::Null),
        _ => {
            if field.is_secret {
                Value::Secret(SecretString::new(""))
            } else {
                Value::Str(String::new())
            }
        }
    }
}

// ============================================================================
// Shell
// ============================================================================

/// Generic Shell prepare: builds a `ShellRequest` from a `ShellOperationSpec`.
#[derive(Debug, Clone)]
pub struct GenericShellPrepareOp {
    pub spec: ShellOperationSpec,
}

impl Executable for GenericShellPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Propagate skip from upstream (e.g., non-selected match branches).
        if inputs.values().any(|v| matches!(v, Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }

        // Skip when required inputs are missing (non-taken branch param sources).
        let has_missing_required = self
            .spec
            .input_fields
            .iter()
            .any(|field| field.default.is_none() && !inputs.contains_key(&field.name));
        if has_missing_required {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }

        let mut argv: Vec<String> = Vec::new();

        for segment in &self.spec.argv_template {
            match segment {
                ArgvSegment::Literal(s) => {
                    // Handle complex interpolation: e.g., "{base}...{head}"
                    if s.contains('{') {
                        let interpolated =
                            interpolate_template(s, &inputs, &self.spec.input_fields);
                        argv.push(interpolated);
                    } else {
                        argv.push(s.clone());
                    }
                }
                ArgvSegment::InputRef(name) => {
                    let value = input_as_string_for_shell(&inputs, name, &self.spec.input_fields);
                    argv.push(value);
                }
            }
        }

        // Append List<String> input fields not already in argv (e.g., `args` in cargo.Build.Run).
        for field in &self.spec.input_fields {
            if field.is_path_param {
                continue; // Already handled in argv template.
            }
            if field.type_id.starts_with("List<") || field.type_id.starts_with("List ") {
                if let Some(Value::List(items)) = inputs.get(&field.name) {
                    for item in items {
                        if let Some(s) = item.as_str() {
                            argv.push(s.to_string());
                        }
                    }
                }
            }
        }

        if argv.is_empty() {
            return Err(ExecError::new("shell spec produced empty argv"));
        }

        let command = argv.remove(0);
        let mut request = ShellRequest::new(command).args(argv);

        for (key, value) in &self.spec.env {
            request = request.env(key, value);
        }

        OutputMap::new()
            .request("request", TransportRequest::Shell(request))
            .bool("skip", false)
            .ok()
    }
}

/// Wrap shell stdout as `Value::Secret` or `Value::Str` based on the output field spec.
fn shell_trim_value(field: Option<&OutputFieldSpec>, text: String) -> Value {
    let is_secret = field
        .map(|f| f.type_id == "Secret" || f.is_secret)
        .unwrap_or(false);
    if is_secret {
        Value::Secret(SecretString::new(text))
    } else {
        Value::Str(text)
    }
}

/// Generic Shell parse: extracts output fields from a `ShellResponse`.
#[derive(Debug, Clone)]
pub struct GenericShellParseOp {
    pub spec: ShellOperationSpec,
    pub service_name: String,
    pub operation_name: String,
}

impl Executable for GenericShellParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Shell(shell))) => {
                match self.spec.output_parsing {
                    ShellOutputParsing::SuccessStdoutStderr => OutputMap::new()
                        .bool("success", shell.success())
                        .str("stdout", shell.stdout.clone())
                        .str("stderr", shell.stderr.clone())
                        .ok(),

                    ShellOutputParsing::ExitCodeBool => {
                        // Use the first output field name (e.g., "needed", "exists", "ok").
                        let field_name = self
                            .spec
                            .output_fields
                            .first()
                            .map(|f| f.name.as_str())
                            .unwrap_or("success");
                        OutputMap::new().bool(field_name, shell.success()).ok()
                    }

                    ShellOutputParsing::SplitLines => {
                        let lines: Vec<String> = shell
                            .stdout
                            .lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty())
                            .map(|line| line.to_string())
                            .collect();
                        let field_name = self
                            .spec
                            .output_fields
                            .first()
                            .map(|f| f.name.as_str())
                            .unwrap_or("lines");
                        OutputMap::new().str_list(field_name, lines).ok()
                    }

                    ShellOutputParsing::TrimStdout => {
                        let field = self.spec.output_fields.first();
                        let field_name = field.map(|f| f.name.as_str()).unwrap_or("output");
                        let text = shell.stdout.trim().to_string();
                        OutputMap::new()
                            .value(field_name, shell_trim_value(field, text))
                            .ok()
                    }
                }
            }
            Some(Value::Skipped) | None => {
                // Produce defaults based on parsing mode.
                match self.spec.output_parsing {
                    ShellOutputParsing::SuccessStdoutStderr => OutputMap::new()
                        .bool("success", false)
                        .str("stdout", String::new())
                        .str("stderr", String::new())
                        .ok(),
                    ShellOutputParsing::ExitCodeBool => {
                        let field_name = self
                            .spec
                            .output_fields
                            .first()
                            .map(|f| f.name.as_str())
                            .unwrap_or("success");
                        OutputMap::new().bool(field_name, false).ok()
                    }
                    ShellOutputParsing::SplitLines => {
                        let field_name = self
                            .spec
                            .output_fields
                            .first()
                            .map(|f| f.name.as_str())
                            .unwrap_or("lines");
                        OutputMap::new().str_list(field_name, Vec::new()).ok()
                    }
                    ShellOutputParsing::TrimStdout => {
                        let field = self.spec.output_fields.first();
                        let field_name = field.map(|f| f.name.as_str()).unwrap_or("output");
                        OutputMap::new()
                            .value(field_name, shell_trim_value(field, String::new()))
                            .ok()
                    }
                }
            }
            Some(other) => Err(decorate_service_failure(
                ExecError::new(format!(
                    "expected Shell response for parse, got {:?}",
                    std::mem::discriminant(other)
                )),
                ServiceCallMetadata {
                    provider: self.service_name.clone(),
                    operation: self.operation_name.clone(),
                },
                TransportContext::Shell {
                    exit_code: Some(-1),
                    command: self
                        .spec
                        .argv_template
                        .iter()
                        .filter_map(|s| match s {
                            ArgvSegment::Literal(l) => Some(l.as_str()),
                            _ => None,
                        })
                        .next()
                        .unwrap_or("unknown")
                        .to_string(),
                },
                None,
            )),
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Build a stable lock target for REST acquisition diagnostics.
///
/// Includes method + endpoint + path template so operations under a shared
/// endpoint do not collapse into the same lock identity.
fn rest_lock_target(method: &str, endpoint: &str, path_template: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    let path = path_template.trim();
    if endpoint.is_empty() {
        return format!("{method} {path}");
    }
    if path.is_empty() {
        return format!("{method} {endpoint}");
    }
    if path.starts_with('/') {
        format!("{method} {endpoint}{path}")
    } else {
        format!("{method} {endpoint}/{path}")
    }
}

/// Infer auth scheme and credential reference from `@headers` annotation.
///
/// Looks for an `Authorization` header and extracts the scheme (Bearer, Basic)
/// and the credential input name from template interpolation (`{auth_token}`).
/// Returns `(scheme, Option<credential_ref>)`.
fn infer_auth_from_headers(headers: &[(String, String)]) -> (String, Option<String>) {
    for (key, value) in headers {
        if key.eq_ignore_ascii_case("authorization") {
            let scheme = if value.starts_with("Bearer ") {
                "BearerToken"
            } else if value.starts_with("Basic ") {
                "BasicAuth"
            } else {
                "Header"
            };
            // Extract input name from template: "Bearer {auth_token}" → "auth_token"
            let cred_ref = value.find('{').and_then(|start| {
                value[start + 1..]
                    .find('}')
                    .map(|end| value[start + 1..start + 1 + end].to_string())
            });
            return (scheme.to_string(), cred_ref);
        }
    }
    (String::new(), None)
}

/// Extract an input value as a string, handling Secret and defaults.
#[allow(clippy::disallowed_methods)] // Approved: transport boundary — secret values marshalled into service requests
fn input_as_string(inputs: &HashMap<String, Value>, name: &str, default: Option<&str>) -> String {
    match inputs.get(name) {
        Some(Value::Str(s)) => s.clone(),
        Some(Value::Secret(secret)) => secret.expose_plaintext_for_transport().to_string(),
        Some(Value::Int(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => default.unwrap_or("(unresolved)").to_string(),
    }
}

/// Extract an input value as a string for shell argv interpolation.
fn input_as_string_for_shell(
    inputs: &HashMap<String, Value>,
    name: &str,
    fields: &[FieldSpec],
) -> String {
    let default = fields
        .iter()
        .find(|f| f.name == name)
        .and_then(|f| f.default.as_deref());
    input_as_string(inputs, name, default)
}

/// Interpolate `{name}` placeholders in a template string.
fn interpolate_template(
    template: &str,
    inputs: &HashMap<String, Value>,
    fields: &[FieldSpec],
) -> String {
    let mut result = template.to_string();
    for field in fields {
        let placeholder = format!("{{{}}}", field.name);
        if result.contains(&placeholder) {
            let value = input_as_string(inputs, &field.name, field.default.as_deref());
            result = result.replace(&placeholder, &value);
        }
    }
    result
}

fn ensure_required_profile_config_inputs(
    spec: &RestOperationSpec,
    inputs: &HashMap<String, Value>,
) -> Result<(), ExecError> {
    let mut placeholders = collect_template_placeholders(&spec.path_template);
    for (_, value) in &spec.headers {
        placeholders.extend(collect_template_placeholders(value));
    }
    if let Some(body_template) = &spec.body_template {
        collect_body_template_input_refs(body_template, &mut placeholders);
    }

    for placeholder in placeholders {
        if !placeholder.starts_with("config.") {
            continue;
        }
        let field = spec
            .input_fields
            .iter()
            .find(|field| field.name == placeholder);
        let default = field.and_then(|field| field.default.as_deref());
        match inputs.get(&placeholder) {
            Some(Value::Str(_))
            | Some(Value::Secret(_))
            | Some(Value::Int(_))
            | Some(Value::Bool(_)) => {}
            Some(_) if default.is_some() => {}
            Some(_) => {
                return Err(ExecError::new(format!(
                    "profile config input `{placeholder}` has unsupported value kind for REST template interpolation"
                )));
            }
            None if default.is_some() => {}
            None => {
                return Err(ExecError::new(format!(
                    "missing required profile config input `{placeholder}` for REST transport template interpolation"
                )));
            }
        }
    }
    Ok(())
}

fn collect_template_placeholders(template: &str) -> BTreeSet<String> {
    let mut placeholders = BTreeSet::new();
    let mut current = String::new();
    let mut in_placeholder = false;
    for ch in template.chars() {
        if ch == '{' {
            current.clear();
            in_placeholder = true;
            continue;
        }
        if ch == '}' {
            if in_placeholder && !current.trim().is_empty() {
                placeholders.insert(current.trim().to_string());
            }
            current.clear();
            in_placeholder = false;
            continue;
        }
        if in_placeholder {
            current.push(ch);
        }
    }
    placeholders
}

/// Build a JSON object from a body template, resolving input refs and nested entries.
/// Recursively collect input field names referenced in a body template.
fn collect_body_template_input_refs(entries: &[BodyEntry], out: &mut BTreeSet<String>) {
    for entry in entries {
        match entry {
            BodyEntry::InputRef(_, input_name) => {
                out.insert(input_name.clone());
            }
            BodyEntry::Nested(_, inner) => {
                collect_body_template_input_refs(inner, out);
            }
            BodyEntry::Literal(_, _) => {}
        }
    }
}

fn build_body_from_template(
    entries: &[BodyEntry],
    inputs: &HashMap<String, Value>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for entry in entries {
        match entry {
            BodyEntry::Literal(key, value) => {
                map.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
            BodyEntry::InputRef(key, field_name) => {
                if let Some(value) = inputs.get(field_name.as_str()) {
                    insert_value_as_json(&mut map, key, value);
                }
            }
            BodyEntry::Nested(key, inner_entries) => {
                let inner = build_body_from_template(inner_entries, inputs);
                map.insert(key.clone(), serde_json::Value::Object(inner));
            }
        }
    }
    map
}

/// Insert a `Value` into a JSON map with appropriate conversion.
fn insert_value_as_json(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &Value,
) {
    match value {
        Value::Str(s) => {
            map.insert(key.to_string(), serde_json::Value::String(s.clone()));
        }
        #[allow(clippy::disallowed_methods)]
        // Approved: transport boundary — secret serialized for service request
        Value::Secret(secret) => {
            map.insert(
                key.to_string(),
                serde_json::Value::String(secret.expose_plaintext_for_transport().to_string()),
            );
        }
        Value::Int(n) => {
            map.insert(key.to_string(), serde_json::json!(*n));
        }
        Value::Bool(b) => {
            map.insert(key.to_string(), serde_json::Value::Bool(*b));
        }
        Value::Json(j) => {
            map.insert(key.to_string(), j.clone());
        }
        Value::List(items) => {
            let arr: Vec<serde_json::Value> = items.iter().filter_map(value_to_json).collect();
            map.insert(key.to_string(), serde_json::Value::Array(arr));
        }
        Value::Map(entries) => {
            let mut inner = serde_json::Map::new();
            for (k, v) in entries {
                insert_value_as_json(&mut inner, k, v);
            }
            map.insert(key.to_string(), serde_json::Value::Object(inner));
        }
        _ => {
            // Unit, Skipped, Request, Response — skip.
        }
    }
}

/// Convert a single `Value` to a `serde_json::Value` (for list items and nested maps).
#[allow(clippy::disallowed_methods)] // Approved: transport boundary — secret serialized for JSON payload
fn value_to_json(value: &Value) -> Option<serde_json::Value> {
    match value {
        Value::Str(s) => Some(serde_json::Value::String(s.clone())),
        Value::Secret(secret) => Some(serde_json::Value::String(
            secret.expose_plaintext_for_transport().to_string(),
        )),
        Value::Int(n) => Some(serde_json::json!(*n)),
        Value::Bool(b) => Some(serde_json::Value::Bool(*b)),
        Value::Json(j) => Some(j.clone()),
        Value::Map(entries) => {
            let mut inner = serde_json::Map::new();
            for (k, v) in entries {
                insert_value_as_json(&mut inner, k, v);
            }
            Some(serde_json::Value::Object(inner))
        }
        Value::List(items) => {
            let arr: Vec<serde_json::Value> = items.iter().filter_map(value_to_json).collect();
            Some(serde_json::Value::Array(arr))
        }
        _ => None,
    }
}

/// Minimal base64 decoder (no external dependency).
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut sextets: Vec<u8> = Vec::with_capacity(input.len());
    for &byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' => sextets.push(byte - b'A'),
            b'a'..=b'z' => sextets.push(byte - b'a' + 26),
            b'0'..=b'9' => sextets.push(byte - b'0' + 52),
            b'+' => sextets.push(62),
            b'/' => sextets.push(63),
            b'=' => sextets.push(64),
            b' ' | b'\n' | b'\r' | b'\t' => {}
            other => {
                return Err(format!("invalid base64 char 0x{other:02x}"));
            }
        }
    }

    if sextets.is_empty() {
        return Ok(Vec::new());
    }

    if sextets.len() & 3 != 0 {
        return Err("invalid base64 length".to_string());
    }

    let chunks = sextets.len() / 4;
    let mut out = Vec::with_capacity(chunks * 3);
    for (idx, chunk) in sextets.chunks(4).enumerate() {
        let v0 = chunk[0];
        let v1 = chunk[1];
        let v2 = chunk[2];
        let v3 = chunk[3];
        if v0 == 64 || v1 == 64 {
            return Err("invalid base64 padding".to_string());
        }
        if v2 == 64 && v3 != 64 {
            return Err("invalid base64 padding".to_string());
        }
        let pad = if v2 == 64 {
            2
        } else if v3 == 64 {
            1
        } else {
            0
        };
        if pad > 0 && idx != chunks.saturating_sub(1) {
            return Err("invalid base64 padding".to_string());
        }
        out.push((v0 << 2) | (v1 >> 4));
        if v2 != 64 {
            out.push(((v1 & 0x0f) << 4) | (v2 >> 2));
        }
        if v3 != 64 {
            out.push(((v2 & 0x03) << 6) | v3);
        }
    }
    Ok(out)
}

// ============================================================================
// FILE
// ============================================================================

/// Generic file prepare: builds a `FileRequest` from a `FileOperationSpec`.
#[derive(Debug, Clone)]
pub struct GenericFilePrepareOp {
    pub spec: FileOperationSpec,
}

impl Executable for GenericFilePrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if inputs.values().any(|v| matches!(v, Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            ExecError::new("GenericFilePrepare: missing required `path` input".to_string())
        })?;
        let request = match self.spec.operation.as_str() {
            "READ" => FileRequest::read(path),
            "READ_BYTES" => FileRequest::read_bytes(path),
            "WRITE" => {
                let content = inputs
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                FileRequest::write(path, content)
            }
            "APPEND" => {
                let content = inputs
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                FileRequest::append(path, content)
            }
            "DELETE" => FileRequest::delete(path),
            "EXISTS" => FileRequest::exists(path),
            "CREATE_DIR" => FileRequest::create_dir(path),
            "GLOB" => FileRequest::glob(path),
            other => {
                return Err(ExecError::new(format!(
                    "GenericFilePrepare: unknown file operation `{other}`"
                )))
            }
        };
        OutputMap::new()
            .request("request", TransportRequest::File(request))
            .bool("skip", false)
            .ok()
    }
}

/// Generic file parse: extracts content from a `FileResponse`.
#[derive(Debug, Clone)]
pub struct GenericFileParseOp {
    pub spec: FileOperationSpec,
}

impl Executable for GenericFileParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::File(file_resp))) => {
                if !file_resp.success {
                    let err = file_resp.error.as_deref().unwrap_or("unknown file error");
                    return Err(ExecError::new(format!(
                        "File operation failed on `{}`: {err}",
                        file_resp.path
                    )));
                }
                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    match field.name.as_str() {
                        "content" => {
                            let content = file_resp.content.as_deref().unwrap_or_default();
                            out = out.str("content", content);
                        }
                        "written" | "deleted" | "created" => {
                            out = out.bool(&field.name, file_resp.success);
                        }
                        "exists" => {
                            out = out.bool("exists", file_resp.exists.unwrap_or(false));
                        }
                        "paths" => {
                            // Glob results: newline-separated list → List<String>
                            let content = file_resp.content.as_deref().unwrap_or_default();
                            let paths: Vec<Value> = content
                                .lines()
                                .filter(|l| !l.is_empty())
                                .map(|l| Value::Str(l.to_string()))
                                .collect();
                            out = out.value("paths", Value::List(paths));
                        }
                        other => {
                            let content = file_resp.content.as_deref().unwrap_or_default();
                            out = out.str(other, content);
                        }
                    }
                }
                out.ok()
            }
            Some(Value::Skipped) | None => {
                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    out = out.value(&field.name, Value::Skipped);
                }
                out.ok()
            }
            other => Err(ExecError::new(format!(
                "GenericFileParse: expected File response, got {other:?}"
            ))),
        }
    }
}

// ============================================================================
// Local (pure computation, no I/O)
// ============================================================================

/// Generic local prepare: packages inputs as a JSON body in a ShellRequest.
/// In DryRun mode, the execute node returns mock data; in real mode, a local
/// computation executor would interpret the request.
#[derive(Debug, Clone)]
pub struct GenericLocalPrepareOp {
    pub spec: LocalOperationSpec,
}

impl Executable for GenericLocalPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if inputs.values().any(|v| matches!(v, Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let mut body = serde_json::Map::new();
        for field in &self.spec.input_fields {
            if let Some(val) = inputs.get(&field.name) {
                if let Some(json_val) = value_to_json(val) {
                    body.insert(field.name.clone(), json_val);
                }
            }
        }
        let request = LocalRequest {
            inputs: serde_json::Value::Object(body),
        };
        OutputMap::new()
            .request("request", TransportRequest::Local(request))
            .bool("skip", false)
            .ok()
    }
}

/// Generic local parse: extracts output fields from a ShellResponse JSON body.
#[derive(Debug, Clone)]
pub struct GenericLocalParseOp {
    pub spec: LocalOperationSpec,
}

impl Executable for GenericLocalParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Local(local_resp))) => {
                let parsed = &local_resp.outputs;
                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    let val = parsed.get(&field.name);
                    out = match val {
                        Some(serde_json::Value::String(s)) => out.str(&field.name, s.clone()),
                        Some(serde_json::Value::Bool(b)) => out.bool(&field.name, *b),
                        Some(other) => out.str(&field.name, other.to_string()),
                        None => out.str(&field.name, String::new()),
                    };
                }
                out.ok()
            }
            // Backward compat: accept Shell responses from the old echo-based carrier.
            Some(Value::Response(TransportResponse::Shell(shell))) => {
                let parsed: serde_json::Value =
                    serde_json::from_str(shell.stdout.trim()).unwrap_or_default();
                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    let val = parsed.get(&field.name);
                    out = match val {
                        Some(serde_json::Value::String(s)) => out.str(&field.name, s.clone()),
                        Some(serde_json::Value::Bool(b)) => out.bool(&field.name, *b),
                        Some(other) => out.str(&field.name, other.to_string()),
                        None => out.str(&field.name, String::new()),
                    };
                }
                out.ok()
            }
            Some(Value::Skipped) | None => {
                let mut out = OutputMap::new();
                for field in &self.spec.output_fields {
                    out = out.value(&field.name, Value::Skipped);
                }
                out.ok()
            }
            Some(other) => Err(ExecError::new(format!(
                "GenericLocalParse: expected Local or Shell response, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}

// ============================================================================
// InterfaceStub (IS-6): stub ops for interface capabilities without profile
// ============================================================================

/// Interface stub prepare: packages inputs into a `TransportRequest` for
/// structural transport detection (DryRun boundary interception).
#[derive(Debug, Clone)]
pub struct InterfaceStubPrepareOp {
    pub interface: String,
    pub capability: String,
}

impl Executable for InterfaceStubPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if inputs.values().any(|v| matches!(v, Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        // Package inputs as a Local request for structural transport detection.
        // Redact secrets — stub transport should never expose plaintext.
        let mut body = serde_json::Map::new();
        for (key, val) in &inputs {
            if matches!(val, Value::Secret(_)) {
                body.insert(key.clone(), serde_json::Value::String("***".to_string()));
            } else if let Some(json_val) = value_to_json(val) {
                body.insert(key.clone(), json_val);
            }
        }
        let request = gunbc_ir::transport::LocalRequest {
            inputs: serde_json::Value::Object(body),
        };
        OutputMap::new()
            .request("request", TransportRequest::Local(request))
            .bool("skip", false)
            .ok()
    }
}

/// Interface stub execute: errors in Real mode ("requires --profile"),
/// auto-mocked in DryRun (boundary mocks supply typed outputs).
#[derive(Debug, Clone)]
pub struct InterfaceStubExecuteOp {
    pub interface: String,
    pub capability: String,
}

impl Executable for InterfaceStubExecuteOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        Err(ExecError::new(format!(
            "interface stub `{}.{}` requires --profile: no active profile bindings \
             (this call would be auto-mocked in DryRun mode)",
            self.interface, self.capability
        )))
    }
}

/// Interface stub parse: identity passthrough. Forwards typed capability
/// outputs from execute (or from DryRun mocks) unchanged.
#[derive(Debug, Clone)]
pub struct InterfaceStubParseOp {
    pub interface: String,
    pub capability: String,
}

impl Executable for InterfaceStubParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Identity passthrough: forward all inputs as outputs.
        Ok(inputs)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests: secret inspection is inherent to testing service resolution
mod tests {
    use super::*;
    use gunbc_ir::transport::{RestResponse, ShellResponse};

    fn rest_spec_simple() -> RestOperationSpec {
        RestOperationSpec {
            endpoint: "https://api.example.com".to_string(),
            method: "POST".to_string(),
            path_template: "/v1/things".to_string(),
            input_fields: vec![FieldSpec {
                name: "name".to_string(),
                type_id: "String".to_string(),
                default: None,
                is_secret: false,
                is_path_param: false,
            }],
            output_fields: vec![OutputFieldSpec {
                name: "id".to_string(),
                type_id: "String".to_string(),
                json_path: "id".to_string(),
                is_secret: false,
                is_raw_body: false,
            }],
            body_template: None,
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            error_mappings: vec![],
        }
    }

    fn rest_spec_with_path_params() -> RestOperationSpec {
        RestOperationSpec {
            endpoint: "https://secretmanager.googleapis.com".to_string(),
            method: "GET".to_string(),
            path_template: "/v1/projects/{project}/secrets/{secret}/versions/{version}:access"
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
                FieldSpec {
                    name: "version".to_string(),
                    type_id: "String".to_string(),
                    default: Some("latest".to_string()),
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
            auth_scheme: None,
            auth_input: None,
            error_mappings: vec![],
        }
    }

    fn shell_spec_simple() -> ShellOperationSpec {
        ShellOperationSpec {
            argv_template: vec![
                ArgvSegment::Literal("git".to_string()),
                ArgvSegment::Literal("rev-parse".to_string()),
                ArgvSegment::Literal("--abbrev-ref".to_string()),
                ArgvSegment::Literal("HEAD".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![OutputFieldSpec {
                name: "branch".to_string(),
                type_id: "String".to_string(),
                json_path: "branch".to_string(),
                is_secret: false,
                is_raw_body: false,
            }],
            output_parsing: ShellOutputParsing::TrimStdout,
            env: vec![],
        }
    }

    fn shell_spec_exit_code() -> ShellOperationSpec {
        ShellOperationSpec {
            argv_template: vec![
                ArgvSegment::Literal("test".to_string()),
                ArgvSegment::Literal("-f".to_string()),
                ArgvSegment::Literal("target/codegen/.stamp".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![OutputFieldSpec {
                name: "needed".to_string(),
                type_id: "Bool".to_string(),
                json_path: "needed".to_string(),
                is_secret: false,
                is_raw_body: false,
            }],
            output_parsing: ShellOutputParsing::ExitCodeBool,
            env: vec![],
        }
    }

    #[test]
    fn rest_prepare_simple_post() {
        let op = GenericRestPrepareOp {
            spec: rest_spec_simple(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Str("hello".to_string()));

        let outputs = op.execute(inputs).unwrap();
        let req = outputs.get("request").unwrap();
        match req {
            Value::Request(TransportRequest::Rest(r)) => {
                assert_eq!(r.url, "https://api.example.com/v1/things");
                assert!(r.body.is_some());
                let body = r.body.as_ref().unwrap();
                assert_eq!(body["name"], "hello");
            }
            other => panic!("expected REST request, got {other:?}"),
        }
    }

    #[test]
    fn rest_prepare_path_interpolation() {
        let op = GenericRestPrepareOp {
            spec: rest_spec_with_path_params(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("project".to_string(), Value::Str("my-project".to_string()));
        inputs.insert("secret".to_string(), Value::Str("my-secret".to_string()));

        let outputs = op.execute(inputs).unwrap();
        let req = outputs.get("request").unwrap();
        match req {
            Value::Request(TransportRequest::Rest(r)) => {
                assert!(r.url.contains("my-project"));
                assert!(r.url.contains("my-secret"));
                assert!(r.url.contains("latest")); // default
                assert!(r.body.is_none()); // GET request
            }
            other => panic!("expected REST request, got {other:?}"),
        }
    }

    #[test]
    fn rest_prepare_fails_closed_when_required_config_placeholder_is_missing() {
        let spec = RestOperationSpec {
            endpoint: "https://api.example.com".to_string(),
            method: "GET".to_string(),
            path_template: "/repos/{config.owner}/issues/42".to_string(),
            input_fields: vec![FieldSpec {
                name: "config.owner".to_string(),
                type_id: "String".to_string(),
                default: None,
                is_secret: false,
                is_path_param: true,
            }],
            output_fields: vec![],
            body_template: None,
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            error_mappings: vec![],
        };
        let op = GenericRestPrepareOp { spec };
        let error = op
            .execute(HashMap::new())
            .expect_err("missing config placeholder input should fail closed");
        assert!(
            error
                .to_string()
                .contains("missing required profile config input `config.owner`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rest_parse_extracts_fields() {
        let op = GenericRestParseOp {
            spec: rest_spec_simple(),
            service_name: String::new(),
            operation_name: String::new(),
            auth_scheme: String::new(),
            permissions: vec![],
        };
        let response = RestResponse::ok(serde_json::json!({ "id": "abc-123" }));
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("id").and_then(Value::as_str), Some("abc-123"));
    }

    #[test]
    fn rest_parse_secret_field() {
        let spec = RestOperationSpec {
            endpoint: "https://sts.googleapis.com".to_string(),
            method: "POST".to_string(),
            path_template: "/v1/token".to_string(),
            input_fields: vec![],
            output_fields: vec![
                OutputFieldSpec {
                    name: "access_token".to_string(),
                    type_id: "Secret".to_string(),
                    json_path: "access_token".to_string(),
                    is_secret: true,
                    is_raw_body: false,
                },
                OutputFieldSpec {
                    name: "expires_in".to_string(),
                    type_id: "Int".to_string(),
                    json_path: "expires_in".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
            ],
            body_template: None,
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            error_mappings: vec![],
        };
        let op = GenericRestParseOp {
            spec,
            service_name: String::new(),
            operation_name: String::new(),
            auth_scheme: String::new(),
            permissions: vec![],
        };
        let response = RestResponse::ok(serde_json::json!({
            "access_token": "ya29.secret-token",
            "expires_in": 3600,
        }));
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        match outputs.get("access_token") {
            Some(Value::Secret(s)) => {
                assert_eq!(s.expose_plaintext_for_transport(), "ya29.secret-token")
            }
            other => panic!("expected Secret, got {other:?}"),
        }
        assert_eq!(outputs.get("expires_in"), Some(&Value::Int(3600)));
    }

    #[test]
    fn rest_parse_bytes_base64() {
        let op = GenericRestParseOp {
            spec: rest_spec_with_path_params(),
            service_name: String::new(),
            operation_name: String::new(),
            auth_scheme: String::new(),
            permissions: vec![],
        };
        let response = RestResponse::ok(serde_json::json!({
            "name": "projects/p/secrets/s/versions/1",
            "payload": { "data": "SGVsbG8=" },  // "Hello" in base64
        }));
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        match outputs.get("payload") {
            Some(Value::List(bytes)) => {
                let decoded: Vec<u8> = bytes.iter().map(|v| v.as_int().unwrap() as u8).collect();
                assert_eq!(decoded, b"Hello");
            }
            other => panic!("expected List of bytes, got {other:?}"),
        }
    }

    #[test]
    fn shell_prepare_simple() {
        let op = GenericShellPrepareOp {
            spec: shell_spec_simple(),
        };

        let outputs = op.execute(HashMap::new()).unwrap();
        let req = outputs.get("request").unwrap();
        match req {
            Value::Request(TransportRequest::Shell(s)) => {
                assert_eq!(s.command, "git");
                assert_eq!(s.args, vec!["rev-parse", "--abbrev-ref", "HEAD"]);
            }
            other => panic!("expected Shell request, got {other:?}"),
        }
    }

    #[test]
    fn shell_prepare_injects_env_from_spec() {
        let op = GenericShellPrepareOp {
            spec: ShellOperationSpec {
                argv_template: vec![
                    ArgvSegment::Literal("cargo".to_string()),
                    ArgvSegment::Literal("build".to_string()),
                ],
                input_fields: vec![],
                output_fields: vec![],
                output_parsing: ShellOutputParsing::SuccessStdoutStderr,
                env: vec![("RUSTFLAGS".to_string(), "-D warnings".to_string())],
            },
        };

        let outputs = op.execute(HashMap::new()).unwrap();
        match outputs.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(s)) => {
                assert_eq!(s.command, "cargo");
                assert_eq!(s.args, vec!["build"]);
                assert_eq!(
                    s.env.get("RUSTFLAGS"),
                    Some(&"-D warnings".to_string()),
                    "env from spec should be injected into ShellRequest"
                );
            }
            other => panic!("expected Shell request, got {other:?}"),
        }
    }

    #[test]
    fn shell_prepare_empty_env_by_default() {
        let op = GenericShellPrepareOp {
            spec: shell_spec_simple(),
        };

        let outputs = op.execute(HashMap::new()).unwrap();
        match outputs.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(s)) => {
                assert!(
                    s.env.is_empty(),
                    "shell spec with empty env should produce no env vars"
                );
            }
            other => panic!("expected Shell request, got {other:?}"),
        }
    }

    #[test]
    fn shell_parse_trim_stdout() {
        let op = GenericShellParseOp {
            spec: shell_spec_simple(),
            service_name: String::new(),
            operation_name: String::new(),
        };
        let response = ShellResponse::ok("  main  \n");
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("branch").and_then(Value::as_str), Some("main"));
    }

    #[test]
    fn shell_parse_exit_code_bool() {
        let op = GenericShellParseOp {
            spec: shell_spec_exit_code(),
            service_name: String::new(),
            operation_name: String::new(),
        };
        let response = ShellResponse::ok("");
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("needed"), Some(&Value::Bool(true)));
    }

    #[test]
    fn shell_parse_split_lines() {
        let spec = ShellOperationSpec {
            argv_template: vec![
                ArgvSegment::Literal("git".to_string()),
                ArgvSegment::Literal("branch".to_string()),
                ArgvSegment::Literal("-r".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![OutputFieldSpec {
                name: "branches".to_string(),
                type_id: "List<String>".to_string(),
                json_path: "branches".to_string(),
                is_secret: false,
                is_raw_body: false,
            }],
            output_parsing: ShellOutputParsing::SplitLines,
            env: vec![],
        };
        let op = GenericShellParseOp {
            spec,
            service_name: String::new(),
            operation_name: String::new(),
        };
        let response = ShellResponse::ok("origin/main\norigin/dev\n\n");
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        match outputs.get("branches") {
            Some(Value::List(items)) => {
                let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
                assert_eq!(strs, vec!["origin/main", "origin/dev"]);
            }
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn shell_parse_success_stdout_stderr() {
        let spec = ShellOperationSpec {
            argv_template: vec![
                ArgvSegment::Literal("cargo".to_string()),
                ArgvSegment::Literal("build".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![
                OutputFieldSpec {
                    name: "success".to_string(),
                    type_id: "Bool".to_string(),
                    json_path: "success".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
                OutputFieldSpec {
                    name: "stdout".to_string(),
                    type_id: "String".to_string(),
                    json_path: "stdout".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
                OutputFieldSpec {
                    name: "stderr".to_string(),
                    type_id: "String".to_string(),
                    json_path: "stderr".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
            ],
            output_parsing: ShellOutputParsing::SuccessStdoutStderr,
            env: vec![],
        };
        let op = GenericShellParseOp {
            spec,
            service_name: String::new(),
            operation_name: String::new(),
        };
        let mut response = ShellResponse::ok("compiled ok");
        response.stderr = "warning: unused var".to_string();
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("success"), Some(&Value::Bool(true)));
        assert_eq!(
            outputs.get("stdout").and_then(Value::as_str),
            Some("compiled ok")
        );
        assert_eq!(
            outputs.get("stderr").and_then(Value::as_str),
            Some("warning: unused var")
        );
    }

    #[test]
    fn rest_prepare_body_template() {
        let spec = RestOperationSpec {
            endpoint: "https://sts.googleapis.com".to_string(),
            method: "POST".to_string(),
            path_template: "/v1/token".to_string(),
            input_fields: vec![
                FieldSpec {
                    name: "audience".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: false,
                },
                FieldSpec {
                    name: "subject_token".to_string(),
                    type_id: "Secret".to_string(),
                    default: None,
                    is_secret: true,
                    is_path_param: false,
                },
            ],
            output_fields: vec![],
            body_template: Some(vec![
                BodyEntry::InputRef("audience".to_string(), "audience".to_string()),
                BodyEntry::Literal(
                    "grant_type".to_string(),
                    "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
                ),
                BodyEntry::InputRef("subject_token".to_string(), "subject_token".to_string()),
            ]),
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            error_mappings: vec![],
        };

        let op = GenericRestPrepareOp { spec };
        let mut inputs = HashMap::new();
        inputs.insert(
            "audience".to_string(),
            Value::Str("my-audience".to_string()),
        );
        inputs.insert(
            "subject_token".to_string(),
            Value::Secret(SecretString::new("tok123")),
        );

        let outputs = op.execute(inputs).unwrap();
        let req = outputs.get("request").unwrap();
        match req {
            Value::Request(TransportRequest::Rest(r)) => {
                let body = r.body.as_ref().unwrap();
                assert_eq!(body["audience"], "my-audience");
                assert_eq!(
                    body["grant_type"],
                    "urn:ietf:params:oauth:grant-type:token-exchange"
                );
                assert_eq!(body["subject_token"], "tok123");
            }
            other => panic!("expected REST request, got {other:?}"),
        }
    }

    #[test]
    fn rest_prepare_propagates_skip() {
        let op = GenericRestPrepareOp {
            spec: rest_spec_simple(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Skipped);

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("request"), Some(&Value::Skipped));
    }

    #[test]
    fn shell_prepare_propagates_skip() {
        let op = GenericShellPrepareOp {
            spec: shell_spec_simple(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("some_input".to_string(), Value::Skipped);

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("request"), Some(&Value::Skipped));
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    // ── File ops ─────────────────────────────────────────────────────

    fn file_read_spec() -> FileOperationSpec {
        FileOperationSpec {
            operation: "READ".to_string(),
            path_template: "{path}".to_string(),
            input_fields: vec![FieldSpec {
                name: "path".to_string(),
                type_id: "String".to_string(),
                default: None,
                is_secret: false,
                is_path_param: true,
            }],
            output_fields: vec![OutputFieldSpec {
                name: "content".to_string(),
                type_id: "String".to_string(),
                json_path: "content".to_string(),
                is_secret: false,
                is_raw_body: false,
            }],
        }
    }

    #[test]
    fn file_prepare_builds_read_request() {
        let op = GenericFilePrepareOp {
            spec: file_read_spec(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("/tmp/test.txt".to_string()));

        let outputs = op.execute(inputs).unwrap();
        let req = outputs.get("request").unwrap();
        match req {
            Value::Request(TransportRequest::File(r)) => {
                assert_eq!(r.path, "/tmp/test.txt");
                assert_eq!(r.operation, gunbc_ir::transport::FileOp::Read);
            }
            other => panic!("expected File request, got {other:?}"),
        }
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
    }

    #[test]
    fn file_prepare_propagates_skip() {
        let op = GenericFilePrepareOp {
            spec: file_read_spec(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Skipped);

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("request"), Some(&Value::Skipped));
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn file_parse_extracts_content() {
        use gunbc_ir::transport::file::FileResponse;
        let op = GenericFileParseOp {
            spec: file_read_spec(),
        };
        let resp = FileResponse::read_ok("/tmp/test.txt", "hello world");
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(resp)),
        );

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(
            outputs.get("content"),
            Some(&Value::Str("hello world".to_string()))
        );
    }

    #[test]
    fn file_parse_propagates_skip() {
        let op = GenericFileParseOp {
            spec: file_read_spec(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Skipped);

        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("content"), Some(&Value::Skipped));
    }

    // ── InterfaceStub ops (IS-6 / IS-8) ──────────────────────────────

    #[test]
    fn interface_stub_prepare_packages_inputs_as_local_request() {
        let op = InterfaceStubPrepareOp {
            interface: "IssueProvider".to_string(),
            capability: "list_issues".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("project".to_string(), Value::Str("my-project".to_string()));
        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
        match outputs.get("request") {
            Some(Value::Request(TransportRequest::Local(local))) => {
                assert!(local.inputs.get("project").is_some());
            }
            other => panic!("expected Local request, got {other:?}"),
        }
    }

    #[test]
    fn interface_stub_prepare_propagates_skip() {
        let op = InterfaceStubPrepareOp {
            interface: "IssueProvider".to_string(),
            capability: "list_issues".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("project".to_string(), Value::Skipped);
        let outputs = op.execute(inputs).unwrap();
        assert_eq!(outputs.get("request"), Some(&Value::Skipped));
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn interface_stub_execute_errors_in_real_mode() {
        let op = InterfaceStubExecuteOp {
            interface: "IssueProvider".to_string(),
            capability: "list_issues".to_string(),
        };
        let error = op
            .execute(HashMap::new())
            .expect_err("stub execute should error in Real mode");
        let msg = error.to_string();
        assert!(
            msg.contains("IssueProvider.list_issues"),
            "error should name the interface.capability: {msg}"
        );
        assert!(
            msg.contains("--profile"),
            "error should mention --profile: {msg}"
        );
    }

    #[test]
    fn interface_stub_parse_is_identity_passthrough() {
        let op = InterfaceStubParseOp {
            interface: "IssueProvider".to_string(),
            capability: "list_issues".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert("issues".to_string(), Value::Str("issue-1".to_string()));
        inputs.insert("count".to_string(), Value::Int(1));
        let outputs = op.execute(inputs).unwrap();
        assert_eq!(
            outputs.get("issues"),
            Some(&Value::Str("issue-1".to_string()))
        );
        assert_eq!(outputs.get("count"), Some(&Value::Int(1)));
    }

    #[test]
    fn interface_stub_prepare_redacts_secrets() {
        let op = InterfaceStubPrepareOp {
            interface: "CredentialProvider".to_string(),
            capability: "get_token".to_string(),
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            "api_key".to_string(),
            Value::Secret(SecretString::new("super-secret-key")),
        );
        inputs.insert("scope".to_string(), Value::Str("read".to_string()));
        let outputs = op.execute(inputs).unwrap();
        match outputs.get("request") {
            Some(Value::Request(TransportRequest::Local(local))) => {
                // Secret should be redacted to "***"
                assert_eq!(
                    local.inputs.get("api_key"),
                    Some(&serde_json::Value::String("***".to_string())),
                    "secrets must be redacted in stub transport"
                );
                // Non-secret values should be passed through
                assert_eq!(
                    local.inputs.get("scope"),
                    Some(&serde_json::Value::String("read".to_string()))
                );
            }
            other => panic!("expected Local request, got {other:?}"),
        }
    }

    fn shell_spec_secret_output() -> ShellOperationSpec {
        ShellOperationSpec {
            argv_template: vec![
                ArgvSegment::Literal("gcloud".to_string()),
                ArgvSegment::Literal("auth".to_string()),
                ArgvSegment::Literal("print-access-token".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![OutputFieldSpec {
                name: "access_token".to_string(),
                type_id: "Secret".to_string(),
                json_path: "access_token".to_string(),
                is_secret: true,
                is_raw_body: false,
            }],
            output_parsing: ShellOutputParsing::TrimStdout,
            env: vec![],
        }
    }

    #[test]
    fn shell_parse_trim_stdout_secret() {
        let op = GenericShellParseOp {
            spec: shell_spec_secret_output(),
            service_name: String::new(),
            operation_name: String::new(),
        };
        let response = ShellResponse::ok("  ya29.secret-token  \n");
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        match outputs.get("access_token") {
            Some(Value::Secret(s)) => {
                assert_eq!(s.expose_plaintext_for_transport(), "ya29.secret-token");
            }
            other => panic!("expected Secret, got {other:?}"),
        }
    }

    #[test]
    fn shell_parse_trim_stdout_secret_skipped() {
        let op = GenericShellParseOp {
            spec: shell_spec_secret_output(),
            service_name: String::new(),
            operation_name: String::new(),
        };
        // No response → Skipped/None path
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Skipped);

        let outputs = op.execute(inputs).unwrap();
        match outputs.get("access_token") {
            Some(Value::Secret(s)) => {
                assert!(s.is_empty(), "skipped secret should be empty");
            }
            other => panic!("expected Secret, got {other:?}"),
        }
    }

    #[test]
    fn shell_parse_trim_stdout_empty_secret() {
        let op = GenericShellParseOp {
            spec: shell_spec_secret_output(),
            service_name: String::new(),
            operation_name: String::new(),
        };
        let response = ShellResponse::ok("");
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(response)),
        );

        let outputs = op.execute(inputs).unwrap();
        match outputs.get("access_token") {
            Some(Value::Secret(s)) => {
                assert!(s.is_empty(), "empty stdout should produce empty secret");
            }
            other => panic!("expected Secret, got {other:?}"),
        }
    }

    #[test]
    fn rest_lock_target_includes_path_template() {
        let target = rest_lock_target("GET", "https://api.example.com", "/v1/items/{id}");
        assert_eq!(target, "GET https://api.example.com/v1/items/{id}");
    }

    #[test]
    fn rest_lock_target_supports_relative_path_templates() {
        let target = rest_lock_target("POST", "https://api.example.com/", "v1/items");
        assert_eq!(target, "POST https://api.example.com/v1/items");
    }
}
