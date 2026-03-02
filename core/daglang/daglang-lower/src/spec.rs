//! Service operation spec types.
//!
//! Type definitions for protocol-specific operation specifications extracted
//! from `.dag` service declarations. Each spec variant parameterizes a generic
//! protocol interpreter (REST, Shell, File, Local).

use gunbc_ir::transport::middleware::TransportMiddlewareConfig;
use serde::Serialize;

/// Complete specification for a service operation, extracted from `.dag` declarations.
/// Each variant parameterizes a generic protocol interpreter (REST, Shell, File).
/// Note: RestOperationSpec is boxed to avoid large_enum_variant clippy warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ServiceOperationSpec {
    Rest(Box<RestOperationSpec>),
    Shell(ShellOperationSpec),
    File(FileOperationSpec),
    Local(LocalOperationSpec),
    /// Stub spec for interface capabilities compiled without a profile binding.
    /// Carries the interface and capability names for diagnostic messages.
    /// `spec.is_some()` is true, so resolver routing treats stubs as concrete
    /// endpoints (design decision D2 from interface-stub-transport.md).
    InterfaceStub {
        interface: String,
        capability: String,
    },
}

impl ServiceOperationSpec {
    /// Returns the input fields for any spec variant.
    ///
    /// This collapses the 4 near-identical match arms that previously appeared
    /// in `service_prepare_ports()` and similar functions (FC-11).
    pub fn input_fields(&self) -> &[FieldSpec] {
        match self {
            Self::Rest(spec) => &spec.input_fields,
            Self::Shell(spec) => &spec.input_fields,
            Self::File(spec) => &spec.input_fields,
            Self::Local(spec) => &spec.input_fields,
            Self::InterfaceStub { .. } => &[],
        }
    }

    /// Returns the output fields for any spec variant.
    pub fn output_fields(&self) -> &[OutputFieldSpec] {
        match self {
            Self::Rest(spec) => &spec.output_fields,
            Self::Shell(spec) => &spec.output_fields,
            Self::File(spec) => &spec.output_fields,
            Self::Local(spec) => &spec.output_fields,
            Self::InterfaceStub { .. } => &[],
        }
    }

    /// Extract authentication requirements from this spec (CT-7).
    ///
    /// Returns the auth scheme and input field name if the operation requires
    /// authentication credentials. CI pipelines use this to derive the set of
    /// secrets they must provision.
    pub fn auth_requirement(&self) -> Option<AuthRequirement> {
        match self {
            Self::Rest(spec) => {
                let scheme = spec.auth_scheme.as_deref()?;
                Some(AuthRequirement {
                    scheme: scheme.to_string(),
                    input_field: spec
                        .auth_input
                        .clone()
                        .unwrap_or_else(|| "auth_token".to_string()),
                    endpoint: spec.endpoint.clone(),
                })
            }
            _ => None,
        }
    }
}

/// Authentication requirement derived from DSL service config (CT-7).
///
/// Produced by scanning service operation specs. CI pipelines use these to
/// determine which secrets to provision without hardcoded inventory linkage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthRequirement {
    /// Auth scheme (e.g., "BearerToken", "ApiKey").
    pub scheme: String,
    /// Input field name that carries the credential (e.g., "auth_token", "api_key").
    pub input_field: String,
    /// Service endpoint this credential is used for.
    pub endpoint: String,
}

/// File protocol specification: operation type + path template.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FileOperationSpec {
    /// File operation kind, parsed from `transport file { op: OP }`.
    pub operation: gunbc_ir::transport::FileOp,
    /// Path template from `transport file { path: "{path}" }`.
    pub path_template: String,
    /// Input fields from `input { ... }`.
    pub input_fields: Vec<FieldSpec>,
    /// Output fields from `output { ... }`.
    pub output_fields: Vec<OutputFieldSpec>,
}

/// Local operation specification: pure computation, no I/O transport.
/// Used for local services whose operations are domain-specific functions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct LocalOperationSpec {
    pub input_fields: Vec<FieldSpec>,
    pub output_fields: Vec<OutputFieldSpec>,
}

/// REST protocol specification: endpoint + method + path + body + response.
///
/// Note: `PartialOrd`/`Ord` are derived only for fields that support it.
/// The `middleware` field is excluded from ordering comparisons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RestOperationSpec {
    /// Base URL from `config { endpoint: "https://..." }` on the service.
    pub endpoint: String,
    /// HTTP method from `transport rest { method: METHOD }`.
    pub method: String,
    /// URL path template from `transport rest { path: "/path/{param}" }`.
    pub path_template: String,
    /// Input fields from `input { ... }`.
    pub input_fields: Vec<FieldSpec>,
    /// Output fields from `output { ... }` with optional `from "json_key"` renames.
    pub output_fields: Vec<OutputFieldSpec>,
    /// Explicit body template, if present.
    /// When None, body is built from all non-path input fields.
    pub body_template: Option<Vec<BodyEntry>>,
    /// Extra HTTP headers.
    pub headers: Vec<(String, String)>,
    /// Auth scheme from `config { auth: BearerToken }`. Desugars to a `res:credential`
    /// input on the execute node; `Credential::apply()` uses this scheme to set
    /// the correct HTTP header at transport execution time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_scheme: Option<String>,
    /// Name of the input field that carries the authentication credential.
    /// When set, the lowerer wires this field to `res:credential` on the execute
    /// node instead of including it in the prepare body. Declared via
    /// `config { auth_input: field_name }` in the DSL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_input: Option<String>,
    /// Transport middleware configuration (rate limit, retry, credential, response).
    /// Populated from `rate_limit {}`, `retry {}`, etc. blocks in the service config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middleware: Option<TransportMiddlewareConfig>,
    /// Response contract: maps HTTP status codes to response types.
    /// Compiled from `response { STATUS => TYPE }` blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_mapping: Vec<ResponseMappingEntry>,
}

/// Shell protocol specification: argv template + output parsing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ShellOperationSpec {
    /// Command + args template from `transport shell { argv: ["cmd", "arg", "{param}"] }`.
    pub argv_template: Vec<ArgvSegment>,
    /// Input fields from `input { ... }`.
    pub input_fields: Vec<FieldSpec>,
    /// Output fields from `output { ... }`.
    pub output_fields: Vec<OutputFieldSpec>,
    /// How to parse the shell response.
    pub output_parsing: ShellOutputParsing,
    /// Environment variables for the shell process.
    /// Resolved from `env: Map<String, String>` input defaults at compile time.
    pub env: Vec<(String, String)>,
    /// Exit code contract: maps exit codes to output types.
    /// Compiled from `exit { CODE => TYPE }` blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exit_mapping: Vec<ExitMappingEntry>,
}

/// Specification for an input field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FieldSpec {
    pub name: String,
    pub type_id: String,
    pub default: Option<String>,
    pub is_secret: bool,
    /// True if this field appears as `{name}` in the path/argv template.
    pub is_path_param: bool,
}

/// Specification for an output field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct OutputFieldSpec {
    pub name: String,
    pub type_id: String,
    /// JSON pointer path for extraction (from `@json("key")` or field name).
    pub json_path: String,
    pub is_secret: bool,
    /// True if this field uses `@raw_body` (response body as raw string).
    pub is_raw_body: bool,
    /// True if the field type is optional (`T?` / `Option<T>`).
    pub is_optional: bool,
}

/// Body template entry: a literal constant, an input field reference, or nested entries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum BodyEntry {
    /// Literal JSON key-value: `"grant_type": "urn:ietf:..."`.
    Literal(String, String),
    /// Reference to an input field: `"audience": audience`.
    InputRef(String, String),
    /// Nested object: `files: { "filename.md": { content: content } }`.
    Nested(String, Vec<BodyEntry>),
}

/// Argv segment in a shell command template.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ArgvSegment {
    /// Literal string: `"cargo"`, `"--all-targets"`.
    Literal(String),
    /// Input field interpolation: `"{package}"`.
    InputRef(String),
}

/// How to parse shell command output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ShellOutputParsing {
    /// Single string: `trim(stdout)`.
    TrimStdout,
    /// List of strings: `split(trim(stdout), "\n")`.
    SplitLines,
    /// Standard triple: `(success: Bool, stdout: String, stderr: String)`.
    SuccessStdoutStderr,
    /// Bool from exit code: `success = exit_code == 0`.
    ExitCodeBool,
}

/// HTTP response contract entry: maps status codes to response types.
/// Compiled from `response { STATUS => TYPE }` blocks in `.dag` files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ResponseMappingEntry {
    /// Status pattern (exact code or wildcard).
    pub status: ResponseStatusPattern,
    /// Type name for the response body.
    pub response_type: String,
    /// Optional description for documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// HTTP status code pattern for response contracts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ResponseStatusPattern {
    /// Exact status code: 200, 201, 404, etc.
    Exact(u16),
    /// 2xx wildcard: any 2xx status.
    Success2xx,
    /// 3xx wildcard: redirects.
    Redirect3xx,
    /// 4xx wildcard: client errors.
    ClientError4xx,
    /// 5xx wildcard: server errors.
    ServerError5xx,
}

/// Shell exit code contract entry: maps exit codes to output types.
/// Compiled from `exit { CODE => TYPE }` blocks in `.dag` files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ExitMappingEntry {
    /// Exit code pattern (exact code or wildcard).
    pub code: ExitCodePattern,
    /// Type name for the output on this exit code.
    pub output_type: String,
    /// Optional description for documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Shell exit code pattern for exit contracts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ExitCodePattern {
    /// Exact exit code: 0, 1, 128, etc.
    Exact(i32),
    /// Non-zero wildcard: any non-zero exit code.
    NonZero,
}

/// Response contract completeness warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseCompletenessWarning {
    pub service: String,
    pub operation: String,
    pub message: String,
}

/// Check response mapping for completeness (SL-11).
///
/// A complete response mapping should have:
/// - At least one success entry (2xx or exact 200/201/etc.)
/// - At least one error entry (4xx/5xx or specific error codes)
///
/// Returns a list of warnings for incomplete response blocks.
pub fn check_response_completeness(
    response_mapping: &[ResponseMappingEntry],
    service: &str,
    operation: &str,
) -> Vec<ResponseCompletenessWarning> {
    // If no response mapping is declared, don't warn (backward compatibility).
    if response_mapping.is_empty() {
        return vec![];
    }

    let mut warnings = Vec::new();

    // Check for success entry (2xx)
    let has_success = response_mapping.iter().any(|entry| match entry.status {
        ResponseStatusPattern::Success2xx => true,
        ResponseStatusPattern::Exact(code) => (200..300).contains(&code),
        _ => false,
    });

    // Check for error entry (4xx or 5xx)
    let has_error = response_mapping.iter().any(|entry| match entry.status {
        ResponseStatusPattern::ClientError4xx | ResponseStatusPattern::ServerError5xx => true,
        ResponseStatusPattern::Exact(code) => (400..600).contains(&code),
        _ => false,
    });

    if !has_success {
        warnings.push(ResponseCompletenessWarning {
            service: service.to_string(),
            operation: operation.to_string(),
            message: "response block missing success entry (200, 201, or 2xx)".to_string(),
        });
    }

    if !has_error {
        warnings.push(ResponseCompletenessWarning {
            service: service.to_string(),
            operation: operation.to_string(),
            message: "response block missing error entry (4xx or 5xx)".to_string(),
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_response_mapping_produces_no_warnings() {
        let mapping = vec![
            ResponseMappingEntry {
                status: ResponseStatusPattern::Exact(200),
                response_type: "Json".to_string(),
                description: None,
            },
            ResponseMappingEntry {
                status: ResponseStatusPattern::ClientError4xx,
                response_type: "Error".to_string(),
                description: None,
            },
        ];
        let warnings = check_response_completeness(&mapping, "test.Service", "Op");
        assert!(warnings.is_empty());
    }

    #[test]
    fn missing_success_entry_produces_warning() {
        let mapping = vec![ResponseMappingEntry {
            status: ResponseStatusPattern::ServerError5xx,
            response_type: "Error".to_string(),
            description: None,
        }];
        let warnings = check_response_completeness(&mapping, "test.Service", "Op");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("success entry"));
    }

    #[test]
    fn missing_error_entry_produces_warning() {
        let mapping = vec![ResponseMappingEntry {
            status: ResponseStatusPattern::Exact(201),
            response_type: "Created".to_string(),
            description: None,
        }];
        let warnings = check_response_completeness(&mapping, "test.Service", "Op");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("error entry"));
    }

    #[test]
    fn empty_mapping_produces_no_warnings() {
        let warnings = check_response_completeness(&[], "test.Service", "Op");
        assert!(
            warnings.is_empty(),
            "empty mapping should not produce warnings"
        );
    }

    #[test]
    fn wildcard_patterns_satisfy_completeness() {
        let mapping = vec![
            ResponseMappingEntry {
                status: ResponseStatusPattern::Success2xx,
                response_type: "Json".to_string(),
                description: None,
            },
            ResponseMappingEntry {
                status: ResponseStatusPattern::ClientError4xx,
                response_type: "ClientError".to_string(),
                description: None,
            },
            ResponseMappingEntry {
                status: ResponseStatusPattern::ServerError5xx,
                response_type: "ServerError".to_string(),
                description: None,
            },
        ];
        let warnings = check_response_completeness(&mapping, "test.Service", "Op");
        assert!(warnings.is_empty());
    }

    // CT-7: Auth requirement derivation tests

    #[test]
    fn rest_spec_auth_requirement_extracts_scheme_and_field() {
        let spec = ServiceOperationSpec::Rest(Box::new(RestOperationSpec {
            endpoint: "https://api.github.com".to_string(),
            method: "POST".to_string(),
            path_template: "/gists".to_string(),
            input_fields: vec![],
            output_fields: vec![],
            body_template: None,
            headers: vec![],
            auth_scheme: Some("BearerToken".to_string()),
            auth_input: Some("auth_token".to_string()),
            middleware: None,
            response_mapping: vec![],
        }));
        let req = spec
            .auth_requirement()
            .expect("should have auth requirement");
        assert_eq!(req.scheme, "BearerToken");
        assert_eq!(req.input_field, "auth_token");
        assert_eq!(req.endpoint, "https://api.github.com");
    }

    #[test]
    fn rest_spec_without_auth_returns_none() {
        let spec = ServiceOperationSpec::Rest(Box::new(RestOperationSpec {
            endpoint: "http://localhost:8080".to_string(),
            method: "GET".to_string(),
            path_template: "/health".to_string(),
            input_fields: vec![],
            output_fields: vec![],
            body_template: None,
            headers: vec![],
            auth_scheme: None,
            auth_input: None,
            middleware: None,
            response_mapping: vec![],
        }));
        assert!(spec.auth_requirement().is_none());
    }

    #[test]
    fn shell_spec_auth_requirement_is_none() {
        let spec = ServiceOperationSpec::Shell(ShellOperationSpec {
            argv_template: vec![],
            input_fields: vec![],
            output_fields: vec![],
            output_parsing: ShellOutputParsing::TrimStdout,
            env: vec![],
            exit_mapping: vec![],
        });
        assert!(spec.auth_requirement().is_none());
    }
}
