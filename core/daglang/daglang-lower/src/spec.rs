//! Service operation spec types.
//!
//! Type definitions for protocol-specific operation specifications extracted
//! from `.dag` service declarations. Each spec variant parameterizes a generic
//! protocol interpreter (REST, Shell, File, Local).

use serde::Serialize;

/// Complete specification for a service operation, extracted from `.dag` declarations.
/// Each variant parameterizes a generic protocol interpreter (REST, Shell, File).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ServiceOperationSpec {
    Rest(RestOperationSpec),
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
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
    /// True if the output type is optional (`T?` / `Option<T>`).
    /// Used by shell TrimStdout to decide whether non-zero exit returns Skipped or errors.
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
