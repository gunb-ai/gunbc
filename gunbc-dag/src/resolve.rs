//! Central resolver: `LoweredOp` → `DynOp` via existing domain ops.
//!
//! Maps each lowered operation from a compiled `.dag` file to its concrete
//! `Executable` implementation, wrapped in `DynOp`. This eliminates the need
//! for per-module union enums (`PragmaGraphOp`, `WorkspaceOp`, etc.).
//!
//! # Architecture
//!
//! Resolution has two layers:
//!
//! 1. **Infrastructure** (cross-module): Typed lowered primitive nodes
//!    (`LoweredOp::Primitive`) map to shared primitive/transport ops.
//!
//! 2. **Domain** (per-module): Module-specific callables (e.g., `tools.pragma`
//!    / `render_clippy_toml`) map to their domain op variants.
//!
//! # Adding a new module
//!
//! To wire a new `.dag` module:
//! 1. Add a match arm in `resolve_domain()` for the module path
//! 2. Map each callable name to its domain op via `DynOp::new(...)`
//! 3. Infrastructure nodes (content_upsert, fs_env) are handled automatically

use std::collections::HashMap;

use daglang_lower::{
    CollectionOpKind, LoweredOp, ObligationCategory, PrimitiveLiteral, PrimitiveOpKind,
};
use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::node::NodeBody;
use gunbc_ir::resource::AccessMode;
use gunbc_ir::transport::{
    FileOp, FileRequest, RestRequest, ShellRequest, TransportRequest, TransportResponse,
};
use gunbc_ir::{Cardinality, Dag, Edge, Node, Port, SecretString, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};

use crate::bootstrap::ops::BootstrapOp;
use crate::makegen::ops::MakegenOp;
use crate::pragma::ops::PragmaOp;

// ============================================================================
// Error type
// ============================================================================

/// Error resolving a `LoweredOp` to a concrete `DynOp`.
#[derive(Debug, Clone)]
pub struct ResolveError {
    pub node_id: String,
    pub reason: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resolve error at `{}`: {}", self.node_id, self.reason)
    }
}

impl std::error::Error for ResolveError {}

fn declared_output_names(outputs: &[Port]) -> Vec<String> {
    outputs.iter().map(|p| p.name.0.clone()).collect()
}

fn execute_with_declared_output_passthrough(
    output_port_names: &[String],
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut outputs = HashMap::new();
    for (key, value) in &inputs {
        outputs.insert(key.clone(), value.clone());
    }
    for port_name in output_port_names {
        outputs.entry(port_name.clone()).or_insert(Value::Skipped);
    }
    Ok(outputs)
}

#[derive(Debug, Clone)]
enum BuildToolOp {
    BuildAll { output_port_names: Vec<String> },
}

impl Executable for BuildToolOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Self::BuildAll { output_port_names } => {
                execute_with_declared_output_passthrough(output_port_names, inputs)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum DocgenToolOp {
    Docgen { output_port_names: Vec<String> },
    RenderAbWorkflowsDoc { output_port_names: Vec<String> },
}

impl Executable for DocgenToolOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let output_port_names = match self {
            Self::Docgen { output_port_names }
            | Self::RenderAbWorkflowsDoc { output_port_names } => output_port_names,
        };
        execute_with_declared_output_passthrough(output_port_names, inputs)
    }
}

#[derive(Debug, Clone)]
enum TestgenToolOp {
    GenerateTests { output_port_names: Vec<String> },
    Testgen { output_port_names: Vec<String> },
}

impl Executable for TestgenToolOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let output_port_names = match self {
            Self::GenerateTests { output_port_names } | Self::Testgen { output_port_names } => {
                output_port_names
            }
        };
        execute_with_declared_output_passthrough(output_port_names, inputs)
    }
}

#[derive(Debug, Clone)]
enum ClippyToolOp {
    ClippyLint { output_port_names: Vec<String> },
}

impl Executable for ClippyToolOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Self::ClippyLint { output_port_names } => {
                execute_with_declared_output_passthrough(output_port_names, inputs)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum DepsToolOp {
    RenderDepsToml { output_port_names: Vec<String> },
    SelectPlatformDeps { output_port_names: Vec<String> },
    DepsInstall { output_port_names: Vec<String> },
    DepsGenerate { output_port_names: Vec<String> },
}

impl Executable for DepsToolOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let output_port_names = match self {
            Self::RenderDepsToml { output_port_names }
            | Self::SelectPlatformDeps { output_port_names }
            | Self::DepsInstall { output_port_names }
            | Self::DepsGenerate { output_port_names } => output_port_names,
        };
        execute_with_declared_output_passthrough(output_port_names, inputs)
    }
}

#[derive(Debug, Clone)]
enum PipelineCiOp {
    Ci { output_port_names: Vec<String> },
}

impl Executable for PipelineCiOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Self::Ci { output_port_names } => {
                execute_with_declared_output_passthrough(output_port_names, inputs)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum SharedDagUtilOp {
    AggregateResults { output_port_names: Vec<String> },
    AllSucceeded { output_port_names: Vec<String> },
    FormatReport { output_port_names: Vec<String> },
    StageResult { output_port_names: Vec<String> },
    SkippedStage { output_port_names: Vec<String> },
    StageFromOutput { output_port_names: Vec<String> },
    GeneratedHeader { output_port_names: Vec<String> },
    RenderAndUpsert { output_port_names: Vec<String> },
}

impl Executable for SharedDagUtilOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let output_port_names = match self {
            Self::AggregateResults { output_port_names }
            | Self::AllSucceeded { output_port_names }
            | Self::FormatReport { output_port_names }
            | Self::StageResult { output_port_names }
            | Self::SkippedStage { output_port_names }
            | Self::StageFromOutput { output_port_names }
            | Self::GeneratedHeader { output_port_names }
            | Self::RenderAndUpsert { output_port_names } => output_port_names,
        };
        execute_with_declared_output_passthrough(output_port_names, inputs)
    }
}

#[derive(Debug, Clone)]
enum SharedGistModesOp {
    BranchContext { output_port_names: Vec<String> },
    ResolveRecentBase { output_port_names: Vec<String> },
    GistFilename { output_port_names: Vec<String> },
    GistUpload { output_port_names: Vec<String> },
    ShareContent { output_port_names: Vec<String> },
    DetectRuntime { output_port_names: Vec<String> },
}

impl Executable for SharedGistModesOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let output_port_names = match self {
            Self::BranchContext { output_port_names }
            | Self::ResolveRecentBase { output_port_names }
            | Self::GistFilename { output_port_names }
            | Self::GistUpload { output_port_names }
            | Self::ShareContent { output_port_names }
            | Self::DetectRuntime { output_port_names } => output_port_names,
        };
        execute_with_declared_output_passthrough(output_port_names, inputs)
    }
}

#[derive(Debug, Clone)]
enum StdPatternsOp {
    FileContentMatches { output_port_names: Vec<String> },
    ClassifyFiles { output_port_names: Vec<String> },
    ReadTextFiles { output_port_names: Vec<String> },
    AcquireSubjectToken { output_port_names: Vec<String> },
    OptionalImpersonation { output_port_names: Vec<String> },
    Ensure { output_port_names: Vec<String> },
    Upsert { output_port_names: Vec<String> },
    ContentUpsert { output_port_names: Vec<String> },
    CredentialChain { output_port_names: Vec<String> },
    Transaction { output_port_names: Vec<String> },
    Retry { output_port_names: Vec<String> },
}

impl Executable for StdPatternsOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let output_port_names = match self {
            Self::FileContentMatches { output_port_names }
            | Self::ClassifyFiles { output_port_names }
            | Self::ReadTextFiles { output_port_names }
            | Self::AcquireSubjectToken { output_port_names }
            | Self::OptionalImpersonation { output_port_names }
            | Self::Ensure { output_port_names }
            | Self::Upsert { output_port_names }
            | Self::ContentUpsert { output_port_names }
            | Self::CredentialChain { output_port_names }
            | Self::Transaction { output_port_names }
            | Self::Retry { output_port_names } => output_port_names,
        };
        execute_with_declared_output_passthrough(output_port_names, inputs)
    }
}

/// Simple identity callable adapter for DSL entrypoint wrappers.
#[derive(Debug, Clone)]
struct IdentityCallableOp;

impl Executable for IdentityCallableOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        Ok(inputs)
    }
}

/// Typed error op for nodes that exist in topology but must not be executed.
///
/// Use this instead of identity/no-op placeholders so that accidental execution
/// fails immediately with a clear message rather than silently producing wrong outputs.
#[derive(Debug, Clone)]
struct UnsupportedOp {
    callable: String,
}

impl Executable for UnsupportedOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        Err(ExecError::new(format!(
            "unsupported operation `{}`: must be lowered away before execution",
            self.callable
        )))
    }
}

/// Constant literal source adapter generated by lowering for literal call args.
#[derive(Debug, Clone)]
struct LiteralSourceOp {
    output_port: String,
    value: Value,
}

impl Executable for LiteralSourceOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        OutputMap::new()
            .value(self.output_port.as_str(), self.value.clone())
            .ok()
    }
}

/// Resource lifecycle acquire adapter for `std.resources`.
///
/// Produces a resource handle value appropriate for the resource kind.
/// In production, these will be real handle acquisitions; for now, they
/// produce cross-platform default handles for dry-run/test execution.
#[derive(Debug, Clone)]
struct ResourceAcquireOp {
    resource_kind: &'static str,
}

impl Executable for ResourceAcquireOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let handle: Value = match self.resource_kind {
            "Filesystem" => {
                filename::FilesystemHandle::cross_platform(filename::Scope::Write).into()
            }
            "Network" => Value::Str("network:default".to_string()),
            "Clock" => Value::Str("clock:monotonic".to_string()),
            "AuthContext" => Value::Str("auth:deferred".to_string()),
            other => Value::Str(format!("resource:{other}")),
        };
        // Output port name matches the lowered graph convention.
        OutputMap::new().value("resource_handle", handle).ok()
    }
}

/// Resource lifecycle release adapter for `std.resources`.
///
/// No-op: releases resource handles (currently a passthrough).
#[derive(Debug, Clone)]
struct ResourceReleaseOp;

impl Executable for ResourceReleaseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        Ok(inputs)
    }
}

/// Filesystem env adapter for DSL graphs.
///
/// Lowered DAGs currently use different fs output port names (`file:write`
/// and/or `FilesystemHandle`). Emit both for compatibility.
#[derive(Debug, Clone)]
struct DslFsEnvOp;

impl Executable for DslFsEnvOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let fs: Value = filename::FilesystemHandle::cross_platform(filename::Scope::Write).into();
        OutputMap::new()
            .value(FsEnv::WRITE_PORT, fs.clone())
            .value("FilesystemHandle", fs)
            .ok()
    }
}

/// Terminal adapter for `tools.pragma::pragma`.
///
/// The lowered DSL function aggregates write transport responses via `__deps`.
/// This adapter computes the three `*_written` booleans expected by the
/// function signature.
#[derive(Debug, Clone)]
struct PragmaEntrypointOp;

impl Executable for PragmaEntrypointOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut clippy_written = false;
        let mut allowlist_written = false;
        let mut policy_written = false;

        if let Some(deps) = inputs.get("__deps").and_then(Value::as_list) {
            for dep in deps {
                let Value::Response(TransportResponse::File(file)) = dep else {
                    continue;
                };
                if file.operation != FileOp::Write || !file.success {
                    continue;
                }
                match file.path.as_str() {
                    "clippy.toml" => clippy_written = true,
                    "tools/disallowed-methods-allowlist.txt" => allowlist_written = true,
                    "tools/pragma-lint-policy.txt" => policy_written = true,
                    _ => {}
                }
            }
        }

        OutputMap::new()
            .bool("clippy_written", clippy_written)
            .bool("allowlist_written", allowlist_written)
            .bool("policy_written", policy_written)
            .ok()
    }
}

/// services.shell.Codegen.Check prepare adapter.
#[derive(Debug, Clone)]
struct ServiceShellCodegenCheckPrepareOp;

impl Executable for ServiceShellCodegenCheckPrepareOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        OutputMap::new()
            .request(
                "request",
                TransportRequest::Shell(
                    ShellRequest::new("test").args(["-f", "target/codegen/.stamp"]),
                ),
            )
            .bool("skip", false)
            .ok()
    }
}

/// services.shell.Codegen.Check parse adapter.
#[derive(Debug, Clone)]
struct ServiceShellCodegenCheckParseOp;

impl Executable for ServiceShellCodegenCheckParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let needed = match inputs.get("response") {
            Some(Value::Response(TransportResponse::Shell(shell))) => shell.success(),
            Some(Value::Skipped) | None => false,
            Some(other) => {
                return Err(ExecError::new(format!(
                    "expected Shell response for Codegen.Check parse, got {:?}",
                    std::mem::discriminant(other)
                )))
            }
        };
        OutputMap::new().bool("needed", needed).ok()
    }
}

/// services.shell.Codegen.Run prepare adapter.
#[derive(Debug, Clone)]
struct ServiceShellCodegenRunPrepareOp;

impl Executable for ServiceShellCodegenRunPrepareOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        OutputMap::new()
            .request(
                "request",
                TransportRequest::Shell(ShellRequest::new("cargo").args([
                    "run",
                    "-p",
                    "gunbc-dag",
                    "--bin",
                    "gunbc-codegen",
                    "--",
                    "codegen",
                ])),
            )
            .bool("skip", false)
            .ok()
    }
}

/// services.shell.Codegen.Run parse adapter.
#[derive(Debug, Clone)]
struct ServiceShellCodegenRunParseOp;

impl Executable for ServiceShellCodegenRunParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Shell(shell))) => OutputMap::new()
                .bool("success", shell.success())
                .str("stdout", shell.stdout.clone())
                .str("stderr", shell.stderr.clone())
                .ok(),
            Some(Value::Skipped) | None => OutputMap::new()
                .bool("success", false)
                .str("stdout", String::new())
                .str("stderr", String::new())
                .ok(),
            Some(other) => Err(ExecError::new(format!(
                "expected Shell response for Codegen.Run parse, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}

/// services.cargo.Build.Build prepare adapter.
#[derive(Debug, Clone)]
struct ServiceCargoBuildPrepareOp;

impl Executable for ServiceCargoBuildPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let all_targets = inputs
            .get("all_targets")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mut args = vec!["build".to_string()];
        if all_targets {
            args.push("--all-targets".to_string());
        }
        OutputMap::new()
            .request(
                "request",
                TransportRequest::Shell(ShellRequest::new("cargo").args(args)),
            )
            .ok()
    }
}

/// services.cargo.Build.Test prepare adapter.
#[derive(Debug, Clone)]
struct ServiceCargoTestPrepareOp;

impl Executable for ServiceCargoTestPrepareOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        OutputMap::new()
            .request(
                "request",
                TransportRequest::Shell(ShellRequest::new("cargo").arg("test")),
            )
            .ok()
    }
}

/// services.cargo.Build.Clippy prepare adapter.
#[derive(Debug, Clone)]
struct ServiceCargoClippyPrepareOp;

impl Executable for ServiceCargoClippyPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let all_targets = inputs
            .get("all_targets")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mut args = vec!["clippy".to_string()];
        if all_targets {
            args.push("--all-targets".to_string());
        }
        args.extend(["--".to_string(), "-D".to_string(), "warnings".to_string()]);
        OutputMap::new()
            .request(
                "request",
                TransportRequest::Shell(ShellRequest::new("cargo").args(args)),
            )
            .ok()
    }
}

/// services.cargo parse adapter for Build/Test/Clippy operations.
#[derive(Debug, Clone)]
struct ServiceCargoParseOp;

impl Executable for ServiceCargoParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Shell(shell))) => OutputMap::new()
                .bool("success", shell.success())
                .str("stdout", shell.stdout.clone())
                .str("stderr", shell.stderr.clone())
                .ok(),
            Some(Value::Skipped) | None => OutputMap::new()
                .bool("success", false)
                .str("stdout", String::new())
                .str("stderr", String::new())
                .ok(),
            Some(other) => Err(ExecError::new(format!(
                "expected Shell response for cargo parse, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}

/// services.gcp.STS.Exchange prepare adapter.
#[derive(Debug, Clone)]
struct ServiceGcpStsExchangePrepareOp;

impl Executable for ServiceGcpStsExchangePrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let audience = inputs
            .get("audience")
            .and_then(Value::as_str)
            .unwrap_or("(unresolved)");
        let subject_token = value_as_string_or_default(inputs.get("subject_token"));
        let body = serde_json::json!({
            "audience": audience,
            "grant_type": "urn:ietf:params:oauth:grant-type:token-exchange",
            "requested_token_type": "urn:ietf:params:oauth:token-type:access_token",
            "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
            "subject_token": subject_token,
        });
        OutputMap::new()
            .request(
                "request",
                TransportRequest::Rest(
                    RestRequest::post("https://sts.googleapis.com/v1/token").json(body),
                ),
            )
            .ok()
    }
}

/// services.gcp.STS.Exchange parse adapter.
#[derive(Debug, Clone)]
struct ServiceGcpStsExchangeParseOp;

impl Executable for ServiceGcpStsExchangeParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Rest(rest))) => {
                if !rest.is_success() {
                    return Err(ExecError::new(format!(
                        "STS exchange failed (status {})",
                        rest.status
                    )));
                }
                let access_token = rest
                    .body
                    .get("access_token")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| ExecError::new("missing access_token in STS response"))?;
                let expires_in = rest
                    .body
                    .get("expires_in")
                    .and_then(|value| value.as_i64())
                    .unwrap_or(0);
                OutputMap::new()
                    .secret("access_token", SecretString::new(access_token))
                    .int("expires_in", expires_in)
                    .ok()
            }
            Some(Value::Skipped) | None => OutputMap::new()
                .secret("access_token", SecretString::new(""))
                .int("expires_in", 0)
                .ok(),
            Some(other) => Err(ExecError::new(format!(
                "expected REST response for STS parse, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}

/// services.gcp.SecretManager.AccessVersion prepare adapter.
#[derive(Debug, Clone)]
struct ServiceGcpSecretManagerAccessVersionPrepareOp;

impl Executable for ServiceGcpSecretManagerAccessVersionPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let project = inputs
            .get("project")
            .and_then(Value::as_str)
            .unwrap_or("(unresolved)");
        let secret = inputs
            .get("secret")
            .and_then(Value::as_str)
            .unwrap_or("(unresolved)");
        let version = inputs
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("latest");
        let url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{project}/secrets/{secret}/versions/{version}:access"
        );
        OutputMap::new()
            .request("request", TransportRequest::Rest(RestRequest::get(url)))
            .ok()
    }
}

/// services.gcp.SecretManager.AccessVersion parse adapter.
#[derive(Debug, Clone)]
struct ServiceGcpSecretManagerAccessVersionParseOp;

impl Executable for ServiceGcpSecretManagerAccessVersionParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match inputs.get("response") {
            Some(Value::Response(TransportResponse::Rest(rest))) => {
                if !rest.is_success() {
                    return Err(ExecError::new(format!(
                        "Secret Manager access failed (status {})",
                        rest.status
                    )));
                }
                let name = rest
                    .body
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                let payload_b64 = rest
                    .body
                    .get("payload")
                    .and_then(|payload| payload.get("data"))
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| ExecError::new("missing payload.data in secret response"))?;
                let bytes = base64_decode(payload_b64)
                    .map_err(|error| ExecError::new(format!("base64 decode failed: {error}")))?;
                let payload = Value::List(
                    bytes
                        .into_iter()
                        .map(|byte| Value::Int(byte as i64))
                        .collect(),
                );
                OutputMap::new()
                    .value("payload", payload)
                    .str("name", name)
                    .ok()
            }
            Some(Value::Skipped) | None => OutputMap::new()
                .value("payload", Value::List(Vec::new()))
                .str("name", String::new())
                .ok(),
            Some(other) => Err(ExecError::new(format!(
                "expected REST response for SecretManager parse, got {:?}",
                std::mem::discriminant(other)
            ))),
        }
    }
}

/// services.shell.Find.ListDirs prepare adapter.
#[derive(Debug, Clone)]
struct ServiceShellFindListDirsPrepareOp;

impl Executable for ServiceShellFindListDirsPrepareOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            ExecError::new(
                "PrepareShellFindListDirs: missing required `path` input (String/FilePath)",
            )
        })?;

        let max_depth = inputs.get("max_depth").and_then(Value::as_int).unwrap_or(1);
        let min_depth = inputs.get("min_depth").and_then(Value::as_int).unwrap_or(1);

        let request = TransportRequest::Shell(ShellRequest::new("find").args(vec![
            path.to_string(),
            "-maxdepth".to_string(),
            max_depth.to_string(),
            "-mindepth".to_string(),
            min_depth.to_string(),
            "-type".to_string(),
            "d".to_string(),
        ]));

        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
    }
}

/// services.shell.Find.ListDirs parse adapter.
#[derive(Debug, Clone)]
struct ServiceShellFindListDirsParseOp;

impl Executable for ServiceShellFindListDirsParseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut dirs = Vec::new();

        if let Some(Value::Response(TransportResponse::Shell(shell))) = inputs.get("response") {
            if shell.success() {
                dirs = shell
                    .stdout
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect();
            }
        }

        OutputMap::new().str_list("dirs", dirs).ok()
    }
}

/// File-read prepare adapter for DSL content-upsert chains.
///
/// Requires a `path` input. Missing `path` is a wiring bug and returns
/// an error so the issue surfaces at execution time rather than silently
/// producing a placeholder request.
#[derive(Debug, Clone)]
struct PrepareFileReadCompatOp;

impl Executable for PrepareFileReadCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if matches!(inputs.get("path"), Some(Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            ExecError::new(
                "PrepareFileRead: missing required `path` input — check content-upsert wiring",
            )
        })?;
        OutputMap::new()
            .request("request", TransportRequest::File(FileRequest::read(path)))
            .bool("skip", false)
            .ok()
    }
}

/// File-write prepare adapter for DSL content-upsert chains.
///
/// Requires `path` and content inputs. Content is looked up under `content`,
/// `return`, or `expected_content` because the DSL lowering pipeline uses
/// different port names depending on the call path (callable return values
/// are named `return`, content-upsert compare nodes use `expected_content`,
/// and direct wiring uses `content`).
#[derive(Debug, Clone)]
struct PrepareFileWriteCompatOp;

impl Executable for PrepareFileWriteCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if matches!(inputs.get("path"), Some(Value::Skipped)) {
            return OutputMap::new().value("request", Value::Skipped).ok();
        }
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            ExecError::new(
                "PrepareFileWrite: missing required `path` input — check content-upsert wiring",
            )
        })?;
        let content_value = inputs
            .get("content")
            .or_else(|| inputs.get("return"))
            .or_else(|| inputs.get("expected_content"));
        if matches!(content_value, Some(Value::Skipped)) {
            return OutputMap::new().value("request", Value::Skipped).ok();
        }
        let content = content_value
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new(
                    "PrepareFileWrite: missing content input (expected `content`, `return`, or `expected_content`)",
                )
            })?;
        OutputMap::new()
            .request(
                "request",
                TransportRequest::File(FileRequest::write(path, content)),
            )
            .bool("skip", false)
            .ok()
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Resolve a lowered DAG into an executable `Dag<DynOp>`.
///
/// Each `LoweredOp` node is replaced with its concrete domain op wrapped
/// in `DynOp`. Edges and ports are preserved unchanged.
pub fn resolve_lowered_dag(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
    let mut resolved = Dag::new();
    for node in &dag.nodes {
        let dyn_op = resolve_node(node)?;
        let mut resolved_node = node.clone().map_ops(&mut |_| dyn_op.clone());
        normalize_release_resource_inputs(&mut resolved_node);
        if let Some(mode) = needs_transport_resource(node, &resolved_node) {
            resolved_node
                .inputs
                .push(Port::resource("res:file", "FilesystemHandle", mode));
        }
        resolved.add_node(resolved_node);
    }
    resolved.edges = dag.edges.clone();
    wire_missing_filesystem_resources(&mut resolved);
    Ok(resolved)
}

fn normalize_release_resource_inputs(node: &mut Node<DynOp>) {
    if !node.id.0.starts_with("release_resource_") {
        return;
    }
    for input in &mut node.inputs {
        if input.name.0 == "resource_handle" {
            input.cardinality = Cardinality::ZERO_OR_MORE;
        }
    }
}

// ============================================================================
// Node resolution
// ============================================================================

fn resolve_node(node: &Node<LoweredOp>) -> Result<DynOp, ResolveError> {
    let node_id = node.id.0.clone();
    match &node.body {
        NodeBody::Opaque(op) => resolve_op(&node_id, op, &node.outputs),
        NodeBody::SubDag(_) => Err(ResolveError {
            node_id,
            reason: "SubDag nodes must be lowered before resolution".into(),
        }),
    }
}

fn resolve_op(node_id: &str, op: &LoweredOp, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match op {
        LoweredOp::Collection { kind, .. } => resolve_collection(kind),
        LoweredOp::Pipeline { module, name, .. } => Ok(DynOp::new(UnsupportedOp {
            callable: format!("Pipeline::{module}::{name}"),
        })),
        LoweredOp::Primitive { kind, .. } => resolve_primitive(kind, outputs),
        LoweredOp::Callable { module, name, .. } => resolve_domain(node_id, module, name, outputs),
    }
}

// ============================================================================
// Infrastructure resolution (cross-module patterns)
// ============================================================================

/// Resolve typed lowered primitive nodes shared across all modules.
fn resolve_primitive(kind: &PrimitiveOpKind, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match kind {
        PrimitiveOpKind::FsEnv => Ok(DynOp::new(DslFsEnvOp)),
        PrimitiveOpKind::CallParamSource { .. } => Ok(DynOp::new(IdentityCallableOp)),
        PrimitiveOpKind::CallLiteralSource { literal } => {
            let output_port = outputs
                .first()
                .map(|port| port.name.0.clone())
                .unwrap_or_else(|| "value".to_string());
            let value = match literal {
                PrimitiveLiteral::String(value) => Value::Str(value.clone()),
                PrimitiveLiteral::Int(value) => Value::Int(*value),
                PrimitiveLiteral::Bool(value) => Value::Bool(*value),
                PrimitiveLiteral::None => Value::Unit,
            };
            Ok(DynOp::new(LiteralSourceOp { output_port, value }))
        }
        PrimitiveOpKind::ContentUpsertPrepareRead => Ok(DynOp::new(PrepareFileReadCompatOp)),
        PrimitiveOpKind::ContentUpsertExecuteRead => Ok(DynOp::new(TransportOps::Execute)),
        PrimitiveOpKind::ContentUpsertCompareContent => Ok(DynOp::new(BlobOps::CompareContent)),
        PrimitiveOpKind::ContentUpsertPrepareWrite => Ok(DynOp::new(PrepareFileWriteCompatOp)),
        PrimitiveOpKind::ContentUpsertExecuteTransport => Ok(DynOp::new(TransportOps::Execute)),
    }
}

// ============================================================================
// Domain resolution (per-module callables)
// ============================================================================

fn resolve_domain(
    node_id: &str,
    module: &str,
    name: &str,
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
    match module {
        "tools.pragma" => resolve_pragma(node_id, name),
        "tools.makegen" => resolve_makegen(node_id, name),
        "tools.build" => resolve_build(node_id, name, outputs),
        "tools.codegen" => resolve_codegen(node_id, name),
        "tools.bootstrap" => resolve_bootstrap(node_id, name, outputs),
        "tools.docgen" => resolve_docgen(node_id, name, outputs),
        "tools.testgen" => resolve_testgen(node_id, name, outputs),
        "tools.clippy" => resolve_clippy(node_id, name, outputs),
        "tools.deps" => resolve_deps(node_id, name, outputs),
        "pipelines.ci" => resolve_pipeline_ci(node_id, name, outputs),
        "shared.dag_util" => resolve_shared_dag_util(node_id, name, outputs),
        "shared.gist_modes" => resolve_shared_gist_modes(node_id, name, outputs),
        "std.patterns" => resolve_std_patterns(node_id, name, outputs),
        "std.resources" => resolve_std_resources(name),
        _ if module.starts_with("services.") || module.starts_with("workspace.") => {
            resolve_service_transport(node_id, module, name)
        }
        _ => Err(unknown_callable(node_id, module, name)),
    }
}

fn resolve_pragma(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "render_clippy_toml" => Ok(DynOp::new(PragmaOp::RenderClippy)),
        "render_disallowed_methods_allowlist" => Ok(DynOp::new(PragmaOp::RenderAllowlist)),
        "render_pragma_lint_policy" => Ok(DynOp::new(PragmaOp::RenderLintPolicy)),
        "pragma" => Ok(DynOp::new(PragmaEntrypointOp)),
        _ => Err(unknown_callable(node_id, "tools.pragma", name)),
    }
}

fn resolve_makegen(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "load_registry" => Ok(DynOp::new(MakegenOp::LoadRegistry)),
        "render_makefile" => Ok(DynOp::new(MakegenOp::RenderMakefile)),
        "makegen" => Ok(DynOp::new(MakegenOp::Entrypoint)),
        _ => Err(unknown_callable(node_id, "tools.makegen", name)),
    }
}

fn resolve_build(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "build_all" => Ok(DynOp::new(BuildToolOp::BuildAll {
            output_port_names: declared_output_names(outputs),
        })),
        _ => Err(unknown_callable(node_id, "tools.build", name)),
    }
}

fn resolve_codegen(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "codegen" => Ok(DynOp::new(IdentityCallableOp)),
        _ => Err(unknown_callable(node_id, "tools.codegen", name)),
    }
}

fn resolve_bootstrap(node_id: &str, name: &str, _outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        // The DSL `func bootstrap(...)` wrapper only aggregates upstream values.
        "bootstrap" => Ok(DynOp::new(IdentityCallableOp)),
        "render_bootstrap_makefile" => Ok(DynOp::new(BootstrapOp::GenerateMakefile)),
        "render_bootstrap_gitignore" => Ok(DynOp::new(BootstrapOp::GenerateGitignore)),
        _ => Err(unknown_callable(node_id, "tools.bootstrap", name)),
    }
}

fn resolve_docgen(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "docgen" => Ok(DynOp::new(DocgenToolOp::Docgen {
            output_port_names: declared_output_names(outputs),
        })),
        "render_ab_workflows_doc" => Ok(DynOp::new(DocgenToolOp::RenderAbWorkflowsDoc {
            output_port_names: declared_output_names(outputs),
        })),
        _ => Err(unknown_callable(node_id, "tools.docgen", name)),
    }
}

fn resolve_testgen(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "generate_tests" => Ok(DynOp::new(TestgenToolOp::GenerateTests {
            output_port_names: declared_output_names(outputs),
        })),
        "testgen" => Ok(DynOp::new(TestgenToolOp::Testgen {
            output_port_names: declared_output_names(outputs),
        })),
        _ => Err(unknown_callable(node_id, "tools.testgen", name)),
    }
}

fn resolve_clippy(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "clippy_lint" => Ok(DynOp::new(ClippyToolOp::ClippyLint {
            output_port_names: declared_output_names(outputs),
        })),
        _ => Err(unknown_callable(node_id, "tools.clippy", name)),
    }
}

fn resolve_deps(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "render_deps_toml" => Ok(DynOp::new(DepsToolOp::RenderDepsToml {
            output_port_names: declared_output_names(outputs),
        })),
        "select_platform_deps" => Ok(DynOp::new(DepsToolOp::SelectPlatformDeps {
            output_port_names: declared_output_names(outputs),
        })),
        "deps_install" => Ok(DynOp::new(DepsToolOp::DepsInstall {
            output_port_names: declared_output_names(outputs),
        })),
        "deps_generate" => Ok(DynOp::new(DepsToolOp::DepsGenerate {
            output_port_names: declared_output_names(outputs),
        })),
        _ => Err(unknown_callable(node_id, "tools.deps", name)),
    }
}

fn resolve_pipeline_ci(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "ci" => Ok(DynOp::new(PipelineCiOp::Ci {
            output_port_names: declared_output_names(outputs),
        })),
        _ => Err(unknown_callable(node_id, "pipelines.ci", name)),
    }
}

fn resolve_shared_dag_util(
    node_id: &str,
    name: &str,
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
    let output_port_names = declared_output_names(outputs);
    match name {
        "aggregate_results" => Ok(DynOp::new(SharedDagUtilOp::AggregateResults {
            output_port_names,
        })),
        "all_succeeded" => Ok(DynOp::new(SharedDagUtilOp::AllSucceeded {
            output_port_names,
        })),
        "format_report" => Ok(DynOp::new(SharedDagUtilOp::FormatReport {
            output_port_names,
        })),
        "stage_result" => Ok(DynOp::new(SharedDagUtilOp::StageResult {
            output_port_names,
        })),
        "skipped_stage" => Ok(DynOp::new(SharedDagUtilOp::SkippedStage {
            output_port_names,
        })),
        "stage_from_output" => Ok(DynOp::new(SharedDagUtilOp::StageFromOutput {
            output_port_names,
        })),
        "generated_header" => Ok(DynOp::new(SharedDagUtilOp::GeneratedHeader {
            output_port_names,
        })),
        "render_and_upsert" => Ok(DynOp::new(SharedDagUtilOp::RenderAndUpsert {
            output_port_names,
        })),
        _ => Err(unknown_callable(node_id, "shared.dag_util", name)),
    }
}

fn resolve_shared_gist_modes(
    node_id: &str,
    name: &str,
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
    let output_port_names = declared_output_names(outputs);
    match name {
        "branch_context" => Ok(DynOp::new(SharedGistModesOp::BranchContext {
            output_port_names,
        })),
        "resolve_recent_base" => Ok(DynOp::new(SharedGistModesOp::ResolveRecentBase {
            output_port_names,
        })),
        "gist_filename" => Ok(DynOp::new(SharedGistModesOp::GistFilename {
            output_port_names,
        })),
        "gist_upload" => Ok(DynOp::new(SharedGistModesOp::GistUpload {
            output_port_names,
        })),
        "share_content" => Ok(DynOp::new(SharedGistModesOp::ShareContent {
            output_port_names,
        })),
        "detect_runtime" => Ok(DynOp::new(SharedGistModesOp::DetectRuntime {
            output_port_names,
        })),
        _ => Err(unknown_callable(node_id, "shared.gist_modes", name)),
    }
}

fn resolve_std_patterns(
    node_id: &str,
    name: &str,
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
    let output_port_names = declared_output_names(outputs);
    match name {
        "file_content_matches" => Ok(DynOp::new(StdPatternsOp::FileContentMatches {
            output_port_names,
        })),
        "classify_files" => Ok(DynOp::new(StdPatternsOp::ClassifyFiles {
            output_port_names,
        })),
        "read_text_files" => Ok(DynOp::new(StdPatternsOp::ReadTextFiles {
            output_port_names,
        })),
        "acquire_subject_token" => Ok(DynOp::new(StdPatternsOp::AcquireSubjectToken {
            output_port_names,
        })),
        "optional_impersonation" => Ok(DynOp::new(StdPatternsOp::OptionalImpersonation {
            output_port_names,
        })),
        "ensure" => Ok(DynOp::new(StdPatternsOp::Ensure { output_port_names })),
        "upsert" => Ok(DynOp::new(StdPatternsOp::Upsert { output_port_names })),
        "content_upsert" => Ok(DynOp::new(StdPatternsOp::ContentUpsert {
            output_port_names,
        })),
        "credential_chain" => Ok(DynOp::new(StdPatternsOp::CredentialChain {
            output_port_names,
        })),
        "transaction" => Ok(DynOp::new(StdPatternsOp::Transaction { output_port_names })),
        "retry" => Ok(DynOp::new(StdPatternsOp::Retry { output_port_names })),
        _ => Err(unknown_callable(node_id, "std.patterns", name)),
    }
}

fn resolve_std_resources(name: &str) -> Result<DynOp, ResolveError> {
    // Resource lifecycle acquire/release nodes from the DSL resource system.
    // Names follow the pattern: `resource_lifecycle::acquire::ResourceName`
    // or `resource_lifecycle::release::ResourceName`.
    if let Some(resource_name) = name.strip_prefix("resource_lifecycle::acquire::") {
        let kind = match resource_name {
            "Filesystem" => "Filesystem",
            "Network" => "Network",
            "Clock" => "Clock",
            "AuthContext" => "AuthContext",
            _ => "unknown",
        };
        return Ok(DynOp::new(ResourceAcquireOp {
            resource_kind: kind,
        }));
    }
    if name.starts_with("resource_lifecycle::release::") {
        return Ok(DynOp::new(ResourceReleaseOp));
    }
    // Other std.resources callables pass through as identity.
    Ok(DynOp::new(IdentityCallableOp))
}

fn resolve_service_transport(
    node_id: &str,
    module: &str,
    name: &str,
) -> Result<DynOp, ResolveError> {
    if module == "services.gcp.sts" {
        match name {
            "service_transport::prepare::gcp.STS::Exchange" => {
                return Ok(DynOp::new(ServiceGcpStsExchangePrepareOp));
            }
            "service_transport::parse::gcp.STS::Exchange" => {
                return Ok(DynOp::new(ServiceGcpStsExchangeParseOp));
            }
            _ => {}
        }
    }

    if module == "services.gcp.secret_manager" {
        match name {
            "service_transport::prepare::gcp.SecretManager::AccessVersion" => {
                return Ok(DynOp::new(ServiceGcpSecretManagerAccessVersionPrepareOp));
            }
            "service_transport::parse::gcp.SecretManager::AccessVersion" => {
                return Ok(DynOp::new(ServiceGcpSecretManagerAccessVersionParseOp));
            }
            _ => {}
        }
    }

    if module == "services.cargo" {
        match name {
            "service_transport::prepare::cargo.Build::Build" => {
                return Ok(DynOp::new(ServiceCargoBuildPrepareOp));
            }
            "service_transport::prepare::cargo.Build::Test" => {
                return Ok(DynOp::new(ServiceCargoTestPrepareOp));
            }
            "service_transport::prepare::cargo.Build::Clippy" => {
                return Ok(DynOp::new(ServiceCargoClippyPrepareOp));
            }
            "service_transport::parse::cargo.Build::Build"
            | "service_transport::parse::cargo.Build::Test"
            | "service_transport::parse::cargo.Build::Clippy" => {
                return Ok(DynOp::new(ServiceCargoParseOp));
            }
            _ => {}
        }
    }

    if module == "services.shell" {
        match name {
            "service_transport::prepare::shell.Find::ListDirs" => {
                return Ok(DynOp::new(ServiceShellFindListDirsPrepareOp));
            }
            "service_transport::parse::shell.Find::ListDirs" => {
                return Ok(DynOp::new(ServiceShellFindListDirsParseOp));
            }
            // tools.codegen service transport adapters → existing domain ops
            "service_transport::prepare::shell.Codegen::Check" => {
                return Ok(DynOp::new(ServiceShellCodegenCheckPrepareOp));
            }
            "service_transport::parse::shell.Codegen::Check" => {
                return Ok(DynOp::new(ServiceShellCodegenCheckParseOp));
            }
            "service_transport::prepare::shell.Codegen::Run" => {
                return Ok(DynOp::new(ServiceShellCodegenRunPrepareOp));
            }
            "service_transport::parse::shell.Codegen::Run" => {
                return Ok(DynOp::new(ServiceShellCodegenRunParseOp));
            }
            _ => {}
        }
    }

    if name.starts_with("service_transport::execute::") {
        return Ok(DynOp::new(TransportOps::Execute));
    }
    Err(unknown_callable(node_id, module, name))
}

// ============================================================================
// Collection resolution
// ============================================================================

/// Resolve collection ops to typed error executables.
///
/// Collection nodes exist in DAG topology for progress/parity visibility,
/// but must not be executed at runtime until dedicated collection executors
/// land. Attempting to execute these nodes fails immediately with a clear
/// message rather than silently passing data through.
fn resolve_collection(kind: &CollectionOpKind) -> Result<DynOp, ResolveError> {
    let label = match kind {
        CollectionOpKind::Map => "Collection::Map",
        CollectionOpKind::FlatMap => "Collection::FlatMap",
        CollectionOpKind::Filter => "Collection::Filter",
        CollectionOpKind::Fold => "Collection::Fold",
        CollectionOpKind::Join => "Collection::Join",
    };
    Ok(DynOp::new(UnsupportedOp {
        callable: label.to_string(),
    }))
}

// ============================================================================
// Helpers
// ============================================================================

fn value_as_string_or_default(value: Option<&Value>) -> String {
    match value {
        Some(Value::Str(s)) => s.clone(),
        Some(Value::Secret(secret)) => secret.expose().to_string(),
        _ => "(unresolved)".to_string(),
    }
}

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

    if !sextets.len().is_multiple_of(4) {
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

fn unknown_callable(node_id: &str, module: &str, name: &str) -> ResolveError {
    ResolveError {
        node_id: node_id.to_string(),
        reason: format!("unknown callable `{module}.{name}`"),
    }
}

/// Check if a transport execute node needs a filesystem resource input added.
///
/// Returns `Some(AccessMode)` if the node is a transport execute node
/// (content_upsert or service_transport) that doesn't already have a
/// filesystem resource input. The resource system requires all transport
/// execute nodes to declare their resource access.
fn needs_transport_resource(
    lowered: &Node<LoweredOp>,
    resolved: &Node<DynOp>,
) -> Option<AccessMode> {
    let mode = match &lowered.body {
        NodeBody::Opaque(LoweredOp::Primitive {
            kind: PrimitiveOpKind::ContentUpsertExecuteTransport,
            ..
        }) => AccessMode::Write,
        NodeBody::Opaque(LoweredOp::Primitive {
            kind: PrimitiveOpKind::ContentUpsertExecuteRead,
            ..
        }) => AccessMode::Read,
        NodeBody::Opaque(LoweredOp::Callable {
            name, obligation, ..
        }) if matches!(obligation, ObligationCategory::ServiceTransportExecute)
            || name.starts_with("service_transport::execute::") =>
        {
            // Service transport execute nodes need filesystem access.
            AccessMode::Read
        }
        _ => return None,
    };

    // Only add if not already present.
    let already_has = resolved.inputs.iter().any(|port| {
        port.type_id.0 == "FilesystemHandle"
            && (port.name.0 == "res:file" || port.name.0.starts_with("res:file:"))
    });
    if already_has {
        None
    } else {
        Some(mode)
    }
}

fn wire_missing_filesystem_resources(dag: &mut Dag<DynOp>) {
    let mut pending = Vec::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            let is_filesystem_resource_port = port.type_id.0 == "FilesystemHandle"
                && (port.name.0 == "res:file" || port.name.0.starts_with("res:file:"));
            if !is_filesystem_resource_port {
                continue;
            }
            let connected = dag
                .edges
                .iter()
                .any(|edge| edge.to_node == node.id && edge.to_port == port.name);
            if !connected {
                pending.push((node.id.0.clone(), port.name.0.clone()));
            }
        }
    }
    if pending.is_empty() {
        return;
    }

    let fs_node_id = "fs_env".to_string();
    let fs_output_port = if let Some(existing) = dag.get_node(&fs_node_id.clone().into()) {
        existing
            .outputs
            .iter()
            .find(|port| port.type_id.0 == "FilesystemHandle")
            .map(|port| port.name.0.clone())
            .unwrap_or_else(|| "FilesystemHandle".to_string())
    } else {
        dag.add_node(Node::opaque(
            fs_node_id.as_str(),
            vec![],
            vec![Port::new("FilesystemHandle", "FilesystemHandle")],
            DynOp::new(DslFsEnvOp),
        ));
        "FilesystemHandle".to_string()
    };

    for (node_id, port_name) in pending {
        let already_connected = dag.edges.iter().any(|edge| {
            edge.from_node.0 == fs_node_id
                && edge.from_port.0 == fs_output_port
                && edge.to_node.0 == node_id
                && edge.to_port.0 == port_name
        });
        if !already_connected {
            dag.add_edge(Edge::new(
                fs_node_id.clone(),
                fs_output_port.clone(),
                node_id,
                port_name,
            ));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{CallableKind, PrimitiveLiteral, PrimitiveOpKind};
    use gunbc_ir::{Node, Port};

    fn callable_node(
        id: &str,
        module: &str,
        name: &str,
        obligation: ObligationCategory,
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Callable {
                module: module.to_string(),
                kind: CallableKind::Fn,
                name: name.to_string(),
                obligation,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        )
    }

    fn collection_node(id: &str, kind: CollectionOpKind) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![Port::new("items", "String")],
            vec![Port::new("items", "String")],
            LoweredOp::Collection {
                module: "test".to_string(),
                callable: "test_fn".to_string(),
                kind,
            },
        )
    }

    fn primitive_node(
        id: &str,
        module: &str,
        name: &str,
        kind: PrimitiveOpKind,
        obligation: ObligationCategory,
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Primitive {
                module: module.to_string(),
                name: name.to_string(),
                kind,
                obligation,
            },
        )
    }

    #[test]
    fn resolve_pragma_render_ops() {
        let cases = [
            ("render_clippy_toml", "RenderClippy"),
            ("render_disallowed_methods_allowlist", "RenderAllowlist"),
            ("render_pragma_lint_policy", "RenderLintPolicy"),
        ];
        for (name, expected_debug) in cases {
            let node = callable_node(name, "tools.pragma", name, ObligationCategory::None);
            let result = resolve_node(&node).expect(name);
            assert!(
                format!("{:?}", result).contains(expected_debug),
                "expected {expected_debug} for {name}, got {:?}",
                result
            );
        }
    }

    #[test]
    fn resolve_makegen_ops() {
        let node = callable_node(
            "load_registry",
            "tools.makegen",
            "load_registry",
            ObligationCategory::None,
        );
        let result = resolve_node(&node).expect("load_registry");
        assert!(format!("{:?}", result).contains("LoadRegistry"));

        let node = callable_node(
            "render_makefile",
            "tools.makegen",
            "render_makefile",
            ObligationCategory::None,
        );
        let result = resolve_node(&node).expect("render_makefile");
        assert!(format!("{:?}", result).contains("RenderMakefile"));
    }

    #[test]
    fn resolve_services_shell_codegen_transport_ops() {
        let cases = [
            (
                "service_transport::prepare::shell.Codegen::Check",
                "ServiceShellCodegenCheckPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Check",
                "ServiceShellCodegenCheckParseOp",
            ),
            (
                "service_transport::prepare::shell.Codegen::Run",
                "ServiceShellCodegenRunPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Run",
                "ServiceShellCodegenRunParseOp",
            ),
        ];

        for (name, expected_debug) in cases {
            let node = callable_node(name, "services.shell", name, ObligationCategory::None);
            let result = resolve_node(&node).expect(name);
            assert!(
                format!("{:?}", result).contains(expected_debug),
                "expected {expected_debug} for {name}, got {:?}",
                result
            );
        }
    }

    #[test]
    fn resolve_tools_codegen_entrypoint_identity() {
        let node = callable_node(
            "codegen",
            "tools.codegen",
            "codegen",
            ObligationCategory::None,
        );
        let result = resolve_node(&node).expect("tools.codegen::codegen");
        assert!(format!("{:?}", result).contains("IdentityCallableOp"));
    }

    #[test]
    fn resolve_fs_env() {
        let node = primitive_node(
            "fs_env",
            "tools.makegen",
            "fs_env",
            PrimitiveOpKind::FsEnv,
            ObligationCategory::ResourceProvide,
        );
        let result = resolve_node(&node).expect("fs_env");
        assert!(format!("{:?}", result).contains("FsEnv"));
    }

    #[test]
    fn resolve_content_upsert_prepare_read() {
        let node = primitive_node(
            "prepare_read_clippy",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
            PrimitiveOpKind::ContentUpsertPrepareRead,
            ObligationCategory::ServiceTransportPrepare,
        );
        let result = resolve_node(&node).expect("prepare_read");
        assert!(format!("{:?}", result).contains("PrepareFileRead"));
    }

    #[test]
    fn resolve_content_upsert_execute_read() {
        let node = primitive_node(
            "execute_read_clippy",
            "tools.pragma",
            "content_upsert::execute_read_clippy",
            PrimitiveOpKind::ContentUpsertExecuteRead,
            ObligationCategory::ServiceTransportExecute,
        );
        let result = resolve_node(&node).expect("execute_read");
        assert!(format!("{:?}", result).contains("Execute"));
    }

    #[test]
    fn resolve_content_upsert_compare() {
        let node = primitive_node(
            "compare_clippy_content",
            "tools.pragma",
            "content_upsert::compare_clippy_content",
            PrimitiveOpKind::ContentUpsertCompareContent,
            ObligationCategory::InterfaceContractVerification,
        );
        let result = resolve_node(&node).expect("compare");
        assert!(format!("{:?}", result).contains("CompareContent"));
    }

    #[test]
    fn resolve_content_upsert_prepare_write() {
        let node = primitive_node(
            "prepare_write_clippy",
            "tools.pragma",
            "content_upsert::prepare_write_clippy",
            PrimitiveOpKind::ContentUpsertPrepareWrite,
            ObligationCategory::ServiceTransportPrepare,
        );
        let result = resolve_node(&node).expect("prepare_write");
        assert!(format!("{:?}", result).contains("PrepareFileWrite"));
    }

    #[test]
    fn resolve_content_upsert_execute_transport() {
        let node = primitive_node(
            "execute_clippy_transport",
            "tools.pragma",
            "content_upsert::execute_clippy_transport",
            PrimitiveOpKind::ContentUpsertExecuteTransport,
            ObligationCategory::ServiceTransportExecute,
        );
        let result = resolve_node(&node).expect("execute_transport");
        assert!(format!("{:?}", result).contains("Execute"));
    }

    #[test]
    fn resolve_literal_source_op_emits_constant_output() {
        let node = Node::opaque(
            "literal_path",
            vec![],
            vec![Port::new("path", "String")],
            LoweredOp::Primitive {
                module: "tools.bootstrap".to_string(),
                name: "call_literal_source::strhex:637261746573".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::String("crates".to_string()),
                },
                obligation: ObligationCategory::ServiceParamSource,
            },
        );
        let result = resolve_node(&node).expect("literal source should resolve");
        let outputs = result
            .execute(HashMap::new())
            .expect("literal source executes");
        assert_eq!(
            outputs.get("path").and_then(Value::as_str),
            Some("crates"),
            "literal source should decode and emit the string constant"
        );
    }

    #[test]
    fn resolve_param_source_op_passthroughs_input() {
        let node = Node::opaque(
            "param_source_path",
            vec![Port::new("path", "String")],
            vec![Port::new("path", "String")],
            LoweredOp::Primitive {
                module: "tools.makegen".to_string(),
                name: "call_param_source::makegen::path".to_string(),
                kind: PrimitiveOpKind::CallParamSource {
                    callable: "makegen".to_string(),
                    param: "path".to_string(),
                },
                obligation: ObligationCategory::ServiceParamSource,
            },
        );
        let result = resolve_node(&node).expect("param source should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("tmp/out.mk".to_string()));
        let outputs = result.execute(inputs).expect("param source should execute");
        assert_eq!(
            outputs.get("path").and_then(Value::as_str),
            Some("tmp/out.mk"),
            "param source should pass through input port value"
        );
    }

    #[test]
    fn resolve_services_gcp_transport_ops() {
        let cases = [
            (
                "services.gcp.sts",
                "service_transport::prepare::gcp.STS::Exchange",
                "ServiceGcpStsExchangePrepareOp",
            ),
            (
                "services.gcp.sts",
                "service_transport::parse::gcp.STS::Exchange",
                "ServiceGcpStsExchangeParseOp",
            ),
            (
                "services.gcp.secret_manager",
                "service_transport::prepare::gcp.SecretManager::AccessVersion",
                "ServiceGcpSecretManagerAccessVersionPrepareOp",
            ),
            (
                "services.gcp.secret_manager",
                "service_transport::parse::gcp.SecretManager::AccessVersion",
                "ServiceGcpSecretManagerAccessVersionParseOp",
            ),
        ];
        for (module, name, expected_debug) in cases {
            let node = callable_node(name, module, name, ObligationCategory::None);
            let result = resolve_node(&node).expect(name);
            assert!(
                format!("{:?}", result).contains(expected_debug),
                "expected {expected_debug} for {name}, got {:?}",
                result
            );
        }
    }

    #[test]
    fn resolve_collection_map() {
        let node = collection_node("map_items", CollectionOpKind::Map);
        let result = resolve_node(&node).expect("map");
        assert!(format!("{:?}", result).contains("UnsupportedOp"));
    }

    #[test]
    fn resolve_unknown_module_fails_closed() {
        let node = callable_node(
            "unknown_op",
            "tools.unknown",
            "do_something",
            ObligationCategory::None,
        );
        let err = resolve_node(&node).expect_err("unknown modules should fail closed");
        assert!(err.reason.contains("unknown callable"));
    }

    #[test]
    fn resolve_unknown_callable_fails() {
        let node = callable_node(
            "bad_op",
            "tools.pragma",
            "nonexistent_op",
            ObligationCategory::None,
        );
        let err = resolve_node(&node).unwrap_err();
        assert!(err.reason.contains("unknown callable"));
    }

    #[test]
    fn resolve_unknown_service_transport_prepare_fails() {
        let node = callable_node(
            "bad_service_prepare",
            "services.gcp.sts",
            "service_transport::prepare::gcp.STS::Refresh",
            ObligationCategory::ServiceTransportPrepare,
        );
        let err = resolve_node(&node).unwrap_err();
        assert!(err.reason.contains("unknown callable"));
    }

    #[test]
    fn resolve_full_dag_preserves_edges() {
        let mut dag = Dag::new();
        dag.add_node(callable_node(
            "render",
            "tools.pragma",
            "render_clippy_toml",
            ObligationCategory::None,
        ));
        dag.add_node(primitive_node(
            "prepare_read",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
            PrimitiveOpKind::ContentUpsertPrepareRead,
            ObligationCategory::ServiceTransportPrepare,
        ));
        dag.edges.push(gunbc_ir::Edge {
            from_node: "render".into(),
            from_port: "content".into(),
            to_node: "prepare_read".into(),
            to_port: "content".into(),
            index: 0,
            kind: gunbc_ir::EdgeKind::DataFlow,
        });

        let resolved = resolve_lowered_dag(&dag).expect("resolve dag");
        assert_eq!(resolved.nodes.len(), 2);
        assert_eq!(resolved.edges.len(), 1);
        assert_eq!(resolved.edges[0].from_node.0, "render");
        assert_eq!(resolved.edges[0].to_node.0, "prepare_read");
    }

    #[test]
    fn needs_transport_resource_respects_existing_res_file_port() {
        let lowered = primitive_node(
            "execute_read_makegen",
            "tools.makegen",
            "content_upsert::execute_read_makegen",
            PrimitiveOpKind::ContentUpsertExecuteRead,
            ObligationCategory::ServiceTransportExecute,
        );
        let resolved = Node::opaque(
            "execute_read_makegen",
            vec![Port::resource("file", "FilesystemHandle", AccessMode::Read)],
            vec![Port::new("response", "TransportResponse")],
            DynOp::new(DslFsEnvOp),
        );

        let mode = needs_transport_resource(&lowered, &resolved);
        assert!(
            mode.is_none(),
            "existing `res:file` input should prevent duplicate resource port injection"
        );
    }

    #[test]
    fn wire_missing_filesystem_resources_wires_res_file_port() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "fs_env",
            vec![],
            vec![Port::new("FilesystemHandle", "FilesystemHandle")],
            DynOp::new(DslFsEnvOp),
        ));
        dag.add_node(Node::opaque(
            "execute_read_makegen",
            vec![Port::resource("file", "FilesystemHandle", AccessMode::Read)],
            vec![Port::new("response", "TransportResponse")],
            DynOp::new(DslFsEnvOp),
        ));

        wire_missing_filesystem_resources(&mut dag);

        let has_edge = dag.edges.iter().any(|edge| {
            edge.from_node.0 == "fs_env"
                && edge.from_port.0 == "FilesystemHandle"
                && edge.to_node.0 == "execute_read_makegen"
                && edge.to_port.0 == "res:file"
        });
        assert!(
            has_edge,
            "filesystem resource edge should be auto-wired for `res:file` inputs"
        );
    }

    #[test]
    fn prepare_file_write_propagates_skipped_content_to_request() {
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("foo.txt".to_string()));
        inputs.insert("content".to_string(), Value::Skipped);

        let outputs = PrepareFileWriteCompatOp
            .execute(inputs)
            .expect("skipped content should not error");
        assert_eq!(
            outputs.get("request"),
            Some(&Value::Skipped),
            "prepare_write should propagate skipped content via skipped request"
        );
    }

    #[test]
    fn normalize_release_resource_inputs_uses_list_cardinality() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "release_resource_std_resources_Filesystem",
            vec![Port::new("resource_handle", "ResourceHandle")],
            vec![Port::new("released", "Bool")],
            LoweredOp::Callable {
                module: "std.resources".to_string(),
                kind: CallableKind::Pattern,
                name: "resource_lifecycle::release::Filesystem".to_string(),
                obligation: ObligationCategory::ResourceRelease,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));

        let resolved = resolve_lowered_dag(&dag).expect("release node should resolve");
        let release_node = resolved
            .get_node(&"release_resource_std_resources_Filesystem".into())
            .expect("release node should exist");
        let handle_port = release_node
            .inputs
            .iter()
            .find(|port| port.name.0 == "resource_handle")
            .expect("release node should keep resource_handle input");
        assert_eq!(
            handle_port.cardinality,
            Cardinality::ZERO_OR_MORE,
            "release resource_handle input should accept fan-in without scalar conflicts"
        );
    }
}
