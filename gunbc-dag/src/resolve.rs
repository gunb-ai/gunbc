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
//! - **Passthrough callables** (forward inputs to outputs): add an entry
//!   to `PASSTHROUGH_CALLABLES` — no new types or functions needed.
//! - **Custom callables**: add a match arm in `resolve_domain()` for the
//!   module path and map each callable to its `DynOp`.
//! - Infrastructure nodes (content_upsert, fs_env) are handled automatically.

use std::collections::HashMap;

use daglang_lower::{
    CollectionOpKind, LoweredOp, ObligationCategory, PrimitiveLiteral, PrimitiveOpKind,
    ServiceCallMetadata, ServiceOperationSpec,
};
use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::node::NodeBody;
use gunbc_ir::resource::AccessMode;
use gunbc_ir::transport::{FileOp, FileRequest, TransportRequest, TransportResponse};
use gunbc_ir::{Cardinality, Dag, Edge, Node, Port, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};

use crate::bootstrap::ops::BootstrapOp;
use crate::makegen::ops::MakegenOp;
use crate::pragma::ops::PragmaOp;
use crate::resolve_service::{
    GenericRestParseOp, GenericRestPrepareOp, GenericShellParseOp, GenericShellPrepareOp,
};

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

/// Single passthrough op that replaces all `domain_passthrough_op!` macro
/// instances.  Every registered callable gets the same behavior: forward
/// all inputs to outputs, filling any declared output port that has no
/// matching input with `Value::Skipped`.
#[derive(Debug, Clone)]
struct PassthroughOp {
    output_port_names: Vec<String>,
}

impl Executable for PassthroughOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        execute_with_declared_output_passthrough(&self.output_port_names, inputs)
    }
}

/// Centralized registry of `(module, &[callable_name])` pairs that use
/// passthrough dispatch.  Adding a new passthrough callable only requires
/// appending to this list — no new enum, impl, or resolver function.
const PASSTHROUGH_CALLABLES: &[(&str, &[&str])] = &[
    ("tools.build", &["build_all"]),
    ("tools.clippy", &["clippy_lint"]),
    ("tools.deps", &["render_deps_toml", "select_platform_deps", "deps_install", "deps_generate"]),
    ("tools.docgen", &["docgen", "render_ab_workflows_doc"]),
    ("tools.testgen", &["generate_tests", "testgen"]),
    ("pipelines.ci", &["ci"]),
    ("shared.dag_util", &[
        "aggregate_results", "all_succeeded", "format_report", "stage_result",
        "skipped_stage", "stage_from_output", "generated_header", "render_and_upsert",
    ]),
    ("shared.gist_modes", &[
        "branch_context", "resolve_recent_base", "gist_filename",
        "gist_upload", "share_content", "detect_runtime",
    ]),
    ("std.patterns", &[
        "file_content_matches", "classify_files", "read_text_files",
        "acquire_subject_token", "optional_impersonation", "ensure",
        "upsert", "content_upsert", "credential_chain", "transaction", "retry",
    ]),
];

/// Try to resolve a callable via the passthrough registry.
fn resolve_passthrough(
    node_id: &str,
    module: &str,
    name: &str,
    outputs: &[Port],
) -> Option<Result<DynOp, ResolveError>> {
    for &(mod_name, callables) in PASSTHROUGH_CALLABLES {
        if mod_name == module {
            if callables.contains(&name) {
                return Some(Ok(DynOp::new(PassthroughOp {
                    output_port_names: declared_output_names(outputs),
                })));
            }
            return Some(Err(unknown_callable(node_id, module, name)));
        }
    }
    None
}

#[derive(Debug, Clone)]
enum InfraToolOp {
    Infra,
}

impl Executable for InfraToolOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let environment = inputs
            .get("environment")
            .and_then(Value::as_str)
            .ok_or_else(|| ExecError::new("tools.infra.infra missing `environment` input"))?;
        let runtime = inputs
            .get("runtime")
            .and_then(Value::as_str)
            .ok_or_else(|| ExecError::new("tools.infra.infra missing `runtime` input"))?;
        let spec_targets = inputs
            .get("spec_targets")
            .and_then(Value::as_str_list)
            .ok_or_else(|| ExecError::new("tools.infra.infra missing `spec_targets` input"))?;
        let target = inputs
            .get("target")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let skip = inputs
            .get("skip")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let execute = inputs
            .get("execute")
            .and_then(Value::as_bool)
            .ok_or_else(|| ExecError::new("tools.infra.infra missing `execute` input"))?;

        let mut planned_targets: Vec<String> = if target.is_empty() {
            spec_targets.clone()
        } else {
            spec_targets
                .iter()
                .filter(|item| target.iter().any(|candidate| candidate == *item))
                .cloned()
                .collect()
        };
        planned_targets.retain(|item| !skip.iter().any(|candidate| candidate == item));

        let target_count = planned_targets.len() as i64;
        let applied_count = if execute { target_count } else { 0 };
        let mode = if execute { "apply" } else { "plan" };
        let report = format!(
            "infra {mode} (env={environment}, runtime={runtime}): {target_count} target(s)"
        );
        OutputMap::new()
            .str("environment", environment)
            .str("runtime", runtime)
            .str("mode", mode)
            .str_list("planned_targets", planned_targets)
            .int("target_count", target_count)
            .int("applied_count", applied_count)
            .str("report", report)
            .ok()
    }
}

fn resolve_infra(node_id: &str, name: &str, _outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "infra" => Ok(DynOp::new(InfraToolOp::Infra)),
        _ => Err(unknown_callable(node_id, "tools.infra", name)),
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

impl UnsupportedOp {
    fn new(callable: &str) -> Self {
        Self {
            callable: callable.to_string(),
        }
    }
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
/// The `resource_kind` is derived from the DSL callable name — no
/// hardcoded list of resource names needed in the resolver.
#[derive(Debug, Clone)]
struct ResourceAcquireOp {
    resource_kind: String,
}

impl Executable for ResourceAcquireOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let handle: Value = match self.resource_kind.as_str() {
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
        NodeBody::SubDag(_) => Ok(DynOp::new(UnsupportedOp::new("subdag_pattern"))),
    }
}

fn resolve_op(node_id: &str, op: &LoweredOp, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match op {
        LoweredOp::Collection { kind, .. } => resolve_collection(kind),
        LoweredOp::Pipeline { module, name, .. } => {
            Ok(DynOp::new(UnsupportedOp::new(&format!("{module}.{name}",))))
        }
        LoweredOp::Primitive { kind, .. } => resolve_primitive(kind, outputs),
        LoweredOp::Callable {
            module,
            name,
            service_metadata,
            ..
        } => resolve_domain(node_id, module, name, outputs, service_metadata.as_deref()),
        LoweredOp::LoopUnpack { .. }
        | LoweredOp::LoopPack { .. }
        | LoweredOp::BranchMerge { .. } => {
            Ok(DynOp::new(UnsupportedOp::new("pattern_internal")))
        }
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
                PrimitiveLiteral::Unit => Value::Unit,
            };
            Ok(DynOp::new(LiteralSourceOp { output_port, value }))
        }
        PrimitiveOpKind::IoPrepareFileRead => Ok(DynOp::new(PrepareFileReadCompatOp)),
        PrimitiveOpKind::IoExecuteFileRead => Ok(DynOp::new(TransportOps::Execute)),
        PrimitiveOpKind::CompareEquality => Ok(DynOp::new(BlobOps::CompareContent)),
        PrimitiveOpKind::IoPrepareFileWrite => Ok(DynOp::new(PrepareFileWriteCompatOp)),
        PrimitiveOpKind::IoExecuteFileWrite => Ok(DynOp::new(TransportOps::Execute)),
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
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    // 1. Modules with custom resolvers (non-passthrough behavior).
    match module {
        "tools.pragma" => return resolve_pragma(node_id, name),
        "tools.makegen" => return resolve_makegen(node_id, name),
        "tools.codegen" => return resolve_codegen(node_id, name),
        "tools.bootstrap" => return resolve_bootstrap(node_id, name, outputs),
        "tools.infra" => return resolve_infra(node_id, name, outputs),
        "std.resources" => return resolve_std_resources(name),
        _ => {}
    }
    // 2. Service/workspace modules use generic transport dispatch.
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(node_id, module, name, service_metadata);
    }
    // 3. Passthrough registry (replaces per-module domain_passthrough_op! macros).
    if let Some(result) = resolve_passthrough(node_id, module, name, outputs) {
        return result;
    }
    Err(unknown_callable(node_id, module, name))
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

fn resolve_std_resources(name: &str) -> Result<DynOp, ResolveError> {
    // Resource lifecycle acquire/release nodes from the DSL resource system.
    // Names follow the pattern: `resource_lifecycle::acquire::ResourceName`
    // or `resource_lifecycle::release::ResourceName`.
    // The resource name is taken directly from the DSL callable —
    // no hardcoded list needed. Adding a new resource to std/resources.dag
    // works without changing resolver code.
    if let Some(resource_name) = name.strip_prefix("resource_lifecycle::acquire::") {
        return Ok(DynOp::new(ResourceAcquireOp {
            resource_kind: resource_name.to_string(),
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
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    // Execute nodes are always the same transport executor.
    if name.starts_with("service_transport::execute::") {
        return Ok(DynOp::new(TransportOps::Execute));
    }

    // Generic dispatch: use the spec from service_metadata to select interpreter.
    if let Some(metadata) = service_metadata {
        if let Some(spec) = &metadata.spec {
            let is_prepare = name.starts_with("service_transport::prepare::");
            let is_parse = name.starts_with("service_transport::parse::");

            match (spec, is_prepare, is_parse) {
                (ServiceOperationSpec::Rest(rest_spec), true, _) => {
                    return Ok(DynOp::new(GenericRestPrepareOp {
                        spec: rest_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Rest(rest_spec), _, true) => {
                    return Ok(DynOp::new(GenericRestParseOp {
                        spec: rest_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Shell(shell_spec), true, _) => {
                    return Ok(DynOp::new(GenericShellPrepareOp {
                        spec: shell_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Shell(shell_spec), _, true) => {
                    return Ok(DynOp::new(GenericShellParseOp {
                        spec: shell_spec.clone(),
                    }));
                }
                _ => {}
            }
        }
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
            kind: PrimitiveOpKind::IoExecuteFileWrite,
            ..
        }) => AccessMode::Write,
        NodeBody::Opaque(LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileRead,
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
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Primitive {
                module: module.to_string(),
                name: name.to_string(),
                kind,
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

    /// Build a service transport node with metadata and spec for generic dispatch.
    fn service_callable_node(
        id: &str,
        module: &str,
        name: &str,
        obligation: ObligationCategory,
        metadata: ServiceCallMetadata,
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
                service_metadata: Some(Box::new(metadata)),
                is_interactive: false,
                resource_target: None,
            },
        )
    }

    fn codegen_check_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "shell.Codegen".to_string(),
            operation: "Check".to_string(),
            transport: ServiceTransportClass::ShellLocal,
            idempotent: false,
            readonly: true,
            permissions: vec![],
            spec: Some(ServiceOperationSpec::Shell(ShellOperationSpec {
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
            })),
        }
    }

    fn codegen_run_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "shell.Codegen".to_string(),
            operation: "Run".to_string(),
            transport: ServiceTransportClass::ShellLocal,
            idempotent: false,
            readonly: false,
            permissions: vec![],
            spec: Some(ServiceOperationSpec::Shell(ShellOperationSpec {
                argv_template: vec![
                    ArgvSegment::Literal("cargo".to_string()),
                    ArgvSegment::Literal("run".to_string()),
                    ArgvSegment::Literal("-p".to_string()),
                    ArgvSegment::Literal("gunbc-dag".to_string()),
                    ArgvSegment::Literal("--bin".to_string()),
                    ArgvSegment::Literal("gunbc-codegen".to_string()),
                    ArgvSegment::Literal("--".to_string()),
                    ArgvSegment::Literal("codegen".to_string()),
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
            })),
        }
    }

    #[test]
    fn resolve_services_shell_codegen_transport_ops() {
        // Prepare nodes use generic shell prepare/parse with spec from metadata.
        let cases = [
            (
                "service_transport::prepare::shell.Codegen::Check",
                codegen_check_metadata(),
                "GenericShellPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Check",
                codegen_check_metadata(),
                "GenericShellParseOp",
            ),
            (
                "service_transport::prepare::shell.Codegen::Run",
                codegen_run_metadata(),
                "GenericShellPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Run",
                codegen_run_metadata(),
                "GenericShellParseOp",
            ),
        ];

        for (name, metadata, expected_debug) in cases {
            let node = service_callable_node(
                name,
                "services.shell",
                name,
                ObligationCategory::None,
                metadata,
            );
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
        let node = primitive_node("fs_env", "tools.makegen", "fs_env", PrimitiveOpKind::FsEnv);
        let result = resolve_node(&node).expect("fs_env");
        assert!(format!("{:?}", result).contains("FsEnv"));
    }

    #[test]
    fn resolve_content_upsert_prepare_read() {
        let node = primitive_node(
            "prepare_read_clippy",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
            PrimitiveOpKind::IoPrepareFileRead,
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
            PrimitiveOpKind::IoExecuteFileRead,
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
            PrimitiveOpKind::CompareEquality,
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
            PrimitiveOpKind::IoPrepareFileWrite,
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
            PrimitiveOpKind::IoExecuteFileWrite,
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

    fn sts_exchange_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "gcp.STS".to_string(),
            operation: "Exchange".to_string(),
            transport: ServiceTransportClass::RestNetwork,
            idempotent: true,
            readonly: false,
            permissions: vec![],
            spec: Some(ServiceOperationSpec::Rest(RestOperationSpec {
                endpoint: "https://sts.googleapis.com".to_string(),
                method: "POST".to_string(),
                path_template: "/v1/token".to_string(),
                input_fields: vec![
                    FieldSpec {
                        name: "subject_token".to_string(),
                        type_id: "Secret".to_string(),
                        default: None,
                        is_secret: true,
                        is_path_param: false,
                    },
                    FieldSpec {
                        name: "audience".to_string(),
                        type_id: "NonEmptyStr".to_string(),
                        default: None,
                        is_secret: false,
                        is_path_param: false,
                    },
                ],
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
            })),
        }
    }

    fn secret_manager_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "gcp.SecretManager".to_string(),
            operation: "AccessVersion".to_string(),
            transport: ServiceTransportClass::RestNetwork,
            idempotent: true,
            readonly: true,
            permissions: vec!["secretmanager.versions.access".to_string()],
            spec: Some(ServiceOperationSpec::Rest(RestOperationSpec {
                endpoint: "https://secretmanager.googleapis.com".to_string(),
                method: "GET".to_string(),
                path_template: "/v1/projects/{project}/secrets/{secret}/versions/{version}:access"
                    .to_string(),
                input_fields: vec![
                    FieldSpec {
                        name: "project".to_string(),
                        type_id: "ProjectId".to_string(),
                        default: None,
                        is_secret: false,
                        is_path_param: true,
                    },
                    FieldSpec {
                        name: "secret".to_string(),
                        type_id: "NonEmptyStr".to_string(),
                        default: None,
                        is_secret: false,
                        is_path_param: true,
                    },
                    FieldSpec {
                        name: "version".to_string(),
                        type_id: "NonEmptyStr".to_string(),
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
            })),
        }
    }

    #[test]
    fn resolve_services_gcp_transport_ops() {
        let cases = [
            (
                "services.gcp.sts",
                "service_transport::prepare::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericRestPrepareOp",
            ),
            (
                "services.gcp.sts",
                "service_transport::parse::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericRestParseOp",
            ),
            (
                "services.gcp.secret_manager",
                "service_transport::prepare::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericRestPrepareOp",
            ),
            (
                "services.gcp.secret_manager",
                "service_transport::parse::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericRestParseOp",
            ),
        ];
        for (module, name, metadata, expected_debug) in cases {
            let node =
                service_callable_node(name, module, name, ObligationCategory::None, metadata);
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
    fn resolve_infra_callable() {
        let node = callable_node("infra", "tools.infra", "infra", ObligationCategory::None);
        let result = resolve_node(&node).expect("infra");
        assert!(
            !format!("{:?}", result).contains("UnsupportedOp"),
            "expected callable resolution for tools.infra.infra, got {:?}",
            result
        );
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
            PrimitiveOpKind::IoPrepareFileRead,
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
            PrimitiveOpKind::IoExecuteFileRead,
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
