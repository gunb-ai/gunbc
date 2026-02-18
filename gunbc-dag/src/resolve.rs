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
//! 1. **Infrastructure** (cross-module): Content upsert pattern nodes, FsEnv,
//!    and transport execute nodes are recognized by name pattern and obligation
//!    category. These map to shared primitive/transport ops.
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

use std::collections::{HashMap, HashSet};

use daglang_lower::{CollectionOpKind, LoweredOp, ObligationCategory};
use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::node::NodeBody;
use gunbc_ir::resource::AccessMode;
use gunbc_ir::transport::{
    FileOp, FileRequest, ShellRequest, ShellResponse, TransportRequest, TransportResponse,
};
use gunbc_ir::{value_backing_for_type_id, Dag, Edge, Node, Port, Value, ValueBacking};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};

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

/// Placeholder for lowered DSL callables not yet runtime-mapped.
#[derive(Debug, Clone)]
struct DeferredCallableOp {
    module: String,
    name: String,
    outputs: Vec<(String, Value)>,
}

impl DeferredCallableOp {
    fn new(module: &str, name: &str, outputs: &[Port]) -> Self {
        Self {
            module: module.to_string(),
            name: name.to_string(),
            outputs: outputs
                .iter()
                .map(|port| {
                    (
                        port.name.0.clone(),
                        default_value_for_output(port.type_id.0.as_str(), port.name.0.as_str()),
                    )
                })
                .collect(),
        }
    }
}

impl Executable for DeferredCallableOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let _ = (&self.module, &self.name);
        Ok(self.outputs.iter().cloned().collect())
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

/// File-read prepare adapter for DSL content-upsert chains.
///
/// Requires a `path` input. Missing `path` is a wiring bug and returns
/// an error so the issue surfaces at execution time rather than silently
/// producing a placeholder request.
#[derive(Debug, Clone)]
struct PrepareFileReadCompatOp;

impl Executable for PrepareFileReadCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
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
/// Requires a `path` input. Accepts content from `content`, `return`, or
/// `expected_content` (naming varies across lowered call paths). Missing
/// `path` is a wiring bug and returns an error.
#[derive(Debug, Clone)]
struct PrepareFileWriteCompatOp;

impl Executable for PrepareFileWriteCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            ExecError::new(
                "PrepareFileWrite: missing required `path` input — check content-upsert wiring",
            )
        })?;
        let content = inputs
            .get("content")
            .or_else(|| inputs.get("return"))
            .or_else(|| inputs.get("expected_content"))
            .and_then(Value::as_str)
            .unwrap_or("");
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
        if needs_content_upsert_write_resource(node, &resolved_node) {
            resolved_node.inputs.push(Port::resource(
                "res:file:*",
                "FilesystemHandle",
                AccessMode::Write,
            ));
        }
        resolved.add_node(resolved_node);
    }
    resolved.edges = dag.edges.clone();
    wire_missing_filesystem_resources(&mut resolved);
    dedupe_release_resource_edges(&mut resolved);
    Ok(resolved)
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
        LoweredOp::Pipeline { module, name, .. } => Ok(deferred_callable(module, name, outputs)),
        LoweredOp::Callable {
            module,
            name,
            obligation,
            ..
        } => {
            // Infrastructure patterns first (cross-module)
            if let Some(dyn_op) = resolve_infrastructure(name, obligation) {
                return Ok(dyn_op);
            }
            // Module-specific domain ops
            resolve_domain(node_id, module, name, outputs)
        }
    }
}

// ============================================================================
// Infrastructure resolution (cross-module patterns)
// ============================================================================

/// Resolve infrastructure nodes shared across all modules.
///
/// These are recognized by name pattern and obligation category:
/// - `fs_env` (ResourceProvide) → FsEnv with Write scope
/// - `content_upsert::prepare_read_*` → PrepareFileReadOp
/// - `content_upsert::execute_read_*` → TransportOps::Execute
/// - `content_upsert::compare_*_content` → BlobOps::CompareContent
/// - `content_upsert::prepare_write_*` → PrepareFileWriteOp
/// - `content_upsert::execute_*_transport` → TransportOps::Execute
fn resolve_infrastructure(name: &str, _obligation: &ObligationCategory) -> Option<DynOp> {
    // FsEnv resource provider
    if name == "fs_env" {
        return Some(DynOp::new(DslFsEnvOp));
    }

    // Content upsert pattern nodes (expanded from `content_upsert()` pattern)
    if let Some(suffix) = name.strip_prefix("content_upsert::") {
        if suffix.starts_with("prepare_read_") {
            return Some(DynOp::new(PrepareFileReadCompatOp));
        }
        if suffix.starts_with("execute_read_") {
            return Some(DynOp::new(TransportOps::Execute));
        }
        if suffix.starts_with("compare_") && suffix.ends_with("_content") {
            return Some(DynOp::new(BlobOps::CompareContent));
        }
        if suffix.starts_with("prepare_write_") {
            return Some(DynOp::new(PrepareFileWriteCompatOp));
        }
        if suffix.ends_with("_transport") {
            return Some(DynOp::new(TransportOps::Execute));
        }
    }

    None
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
        "pipelines.ci" | "shared.dag_util" | "shared.gist_modes" | "std.patterns"
        | "std.resources" => Ok(deferred_callable(module, name, outputs)),
        _ if module.starts_with("services.") || module.starts_with("workspace.") => {
            resolve_service_transport(node_id, module, name, outputs)
        }
        _ => Ok(deferred_callable(module, name, outputs)),
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
        "build_all" => Ok(deferred_callable("tools.build", "build_all", outputs)),
        _ => Err(unknown_callable(node_id, "tools.build", name)),
    }
}

fn resolve_codegen(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "codegen" => Ok(DynOp::new(IdentityCallableOp)),
        _ => Err(unknown_callable(node_id, "tools.codegen", name)),
    }
}

fn resolve_bootstrap(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "bootstrap" => Ok(deferred_callable("tools.bootstrap", "bootstrap", outputs)),
        "render_bootstrap_makefile" => Ok(deferred_callable(
            "tools.bootstrap",
            "render_bootstrap_makefile",
            outputs,
        )),
        "render_bootstrap_gitignore" => Ok(deferred_callable(
            "tools.bootstrap",
            "render_bootstrap_gitignore",
            outputs,
        )),
        _ => Err(unknown_callable(node_id, "tools.bootstrap", name)),
    }
}

fn resolve_docgen(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "docgen" => Ok(deferred_callable("tools.docgen", "docgen", outputs)),
        "render_ab_workflows_doc" => Ok(deferred_callable(
            "tools.docgen",
            "render_ab_workflows_doc",
            outputs,
        )),
        _ => Err(unknown_callable(node_id, "tools.docgen", name)),
    }
}

fn resolve_testgen(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "generate_tests" => Ok(deferred_callable(
            "tools.testgen",
            "generate_tests",
            outputs,
        )),
        "testgen" => Ok(deferred_callable("tools.testgen", "testgen", outputs)),
        _ => Err(unknown_callable(node_id, "tools.testgen", name)),
    }
}

fn resolve_clippy(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "clippy_lint" => Ok(deferred_callable("tools.clippy", "clippy_lint", outputs)),
        _ => Err(unknown_callable(node_id, "tools.clippy", name)),
    }
}

fn resolve_deps(node_id: &str, name: &str, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match name {
        "render_deps_toml" => Ok(deferred_callable("tools.deps", "render_deps_toml", outputs)),
        "select_platform_deps" => Ok(deferred_callable(
            "tools.deps",
            "select_platform_deps",
            outputs,
        )),
        "deps_install" => Ok(deferred_callable("tools.deps", "deps_install", outputs)),
        "deps_generate" => Ok(deferred_callable("tools.deps", "deps_generate", outputs)),
        _ => Err(unknown_callable(node_id, "tools.deps", name)),
    }
}

fn resolve_service_transport(
    node_id: &str,
    module: &str,
    name: &str,
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
    if module == "services.shell" {
        match name {
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
    if name.starts_with("service_transport::prepare::")
        || name.starts_with("service_transport::parse::")
    {
        return Ok(deferred_callable(module, name, outputs));
    }
    Err(unknown_callable(node_id, module, name))
}

// ============================================================================
// Collection resolution
// ============================================================================

/// Resolve collection ops to passthrough executables.
///
/// Collection execution is currently a structural scaffold — lowering emits
/// collection nodes for progress/parity visibility, but runtime semantics
/// remain unchanged until dedicated collection executors land.
fn resolve_collection(kind: &CollectionOpKind) -> Result<DynOp, ResolveError> {
    use gunbc_primitives::collection::{FilterOp, FoldOp, MapOp};

    Ok(match kind {
        CollectionOpKind::Map | CollectionOpKind::FlatMap => DynOp::new(MapOp::Identity),
        CollectionOpKind::Filter => DynOp::new(FilterOp::All),
        CollectionOpKind::Fold => DynOp::new(FoldOp::Count),
        CollectionOpKind::Join => DynOp::new(FoldOp::Join(String::new())),
    })
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

fn deferred_callable(module: &str, name: &str, outputs: &[Port]) -> DynOp {
    DynOp::new(DeferredCallableOp::new(module, name, outputs))
}

fn default_value_for_output(type_id: &str, port_name: &str) -> Value {
    match type_id {
        "TransportResponse" => {
            Value::Response(TransportResponse::Shell(ShellResponse::ok(String::new())))
        }
        "TransportRequest" => Value::Request(TransportRequest::Shell(ShellRequest::new("true"))),
        "Secret" => Value::Secret(gunbc_ir::SecretString::new("mock")),
        "FilesystemHandle" => {
            filename::FilesystemHandle::cross_platform(filename::Scope::Write).into()
        }
        _ => match value_backing_for_type_id(type_id) {
            ValueBacking::String => Value::Str("mock".to_string()),
            ValueBacking::Bool => {
                let lower = port_name.to_ascii_lowercase();
                let value = !(lower == "skip"
                    || lower.ends_with("_skip")
                    || lower.ends_with("_skipped")
                    || lower == "fresh");
                Value::Bool(value)
            }
            ValueBacking::Int | ValueBacking::Float => Value::Int(1),
            ValueBacking::Json => Value::Json(serde_json::json!({"mock": true})),
            ValueBacking::Map => Value::Map(std::collections::BTreeMap::new()),
            ValueBacking::List => Value::List(vec![Value::Str("mock".to_string())]),
            ValueBacking::Set => Value::Set(vec![Value::Str("mock".to_string())]),
            ValueBacking::Unit => Value::Unit,
            ValueBacking::Bytes => Value::List(vec![Value::Int(0)]),
        },
    }
}

fn needs_content_upsert_write_resource(lowered: &Node<LoweredOp>, resolved: &Node<DynOp>) -> bool {
    let NodeBody::Opaque(LoweredOp::Callable { name, .. }) = &lowered.body else {
        return false;
    };
    let Some(suffix) = name.strip_prefix("content_upsert::") else {
        return false;
    };
    if !suffix.ends_with("_transport") {
        return false;
    }
    !resolved.inputs.iter().any(|port| {
        port.type_id.0 == "FilesystemHandle"
            && port.name.0.starts_with("res:file:")
            && matches!(
                port.resource_access,
                Some(AccessMode::Write) | Some(AccessMode::Exclusive)
            )
    })
}

fn wire_missing_filesystem_resources(dag: &mut Dag<DynOp>) {
    let mut pending = Vec::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            if port.type_id.0 != "FilesystemHandle" || !port.name.0.starts_with("res:file:") {
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

fn dedupe_release_resource_edges(dag: &mut Dag<DynOp>) {
    let mut seen = HashSet::<(String, String)>::new();
    dag.edges.retain(|edge| {
        if !edge.to_node.0.starts_with("release_resource_") {
            return true;
        }
        seen.insert((edge.to_node.0.clone(), edge.to_port.0.clone()))
    });
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::CallableKind;
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
        let node = callable_node(
            "fs_env",
            "tools.pragma",
            "fs_env",
            ObligationCategory::ResourceProvide,
        );
        let result = resolve_node(&node).expect("fs_env");
        assert!(format!("{:?}", result).contains("FsEnv"));
    }

    #[test]
    fn resolve_content_upsert_prepare_read() {
        let node = callable_node(
            "prepare_read_clippy",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
            ObligationCategory::ServiceTransportPrepare,
        );
        let result = resolve_node(&node).expect("prepare_read");
        assert!(format!("{:?}", result).contains("PrepareFileRead"));
    }

    #[test]
    fn resolve_content_upsert_execute_read() {
        let node = callable_node(
            "execute_read_clippy",
            "tools.pragma",
            "content_upsert::execute_read_clippy",
            ObligationCategory::ServiceTransportExecute,
        );
        let result = resolve_node(&node).expect("execute_read");
        assert!(format!("{:?}", result).contains("Execute"));
    }

    #[test]
    fn resolve_content_upsert_compare() {
        let node = callable_node(
            "compare_clippy_content",
            "tools.pragma",
            "content_upsert::compare_clippy_content",
            ObligationCategory::InterfaceContractVerification,
        );
        let result = resolve_node(&node).expect("compare");
        assert!(format!("{:?}", result).contains("CompareContent"));
    }

    #[test]
    fn resolve_content_upsert_prepare_write() {
        let node = callable_node(
            "prepare_write_clippy",
            "tools.pragma",
            "content_upsert::prepare_write_clippy",
            ObligationCategory::ServiceTransportPrepare,
        );
        let result = resolve_node(&node).expect("prepare_write");
        assert!(format!("{:?}", result).contains("PrepareFileWrite"));
    }

    #[test]
    fn resolve_content_upsert_execute_transport() {
        let node = callable_node(
            "execute_clippy_transport",
            "tools.pragma",
            "content_upsert::execute_clippy_transport",
            ObligationCategory::ServiceTransportExecute,
        );
        let result = resolve_node(&node).expect("execute_transport");
        assert!(format!("{:?}", result).contains("Execute"));
    }

    #[test]
    fn resolve_collection_map() {
        let node = collection_node("map_items", CollectionOpKind::Map);
        let result = resolve_node(&node).expect("map");
        assert!(format!("{:?}", result).contains("Identity"));
    }

    #[test]
    fn resolve_unknown_module_uses_deferred_defaults() {
        let node = callable_node(
            "unknown_op",
            "tools.unknown",
            "do_something",
            ObligationCategory::None,
        );
        let result = resolve_node(&node).expect("unknown modules should defer");
        assert!(
            format!("{:?}", result).contains("DeferredCallableOp"),
            "expected deferred callable fallback, got {:?}",
            result
        );
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
    fn resolve_full_dag_preserves_edges() {
        let mut dag = Dag::new();
        dag.add_node(callable_node(
            "render",
            "tools.pragma",
            "render_clippy_toml",
            ObligationCategory::None,
        ));
        dag.add_node(callable_node(
            "prepare_read",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
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
}
