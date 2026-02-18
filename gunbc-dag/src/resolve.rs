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

use std::collections::HashMap;

use daglang_lower::{CollectionOpKind, LoweredOp, ObligationCategory};
use gunbc_exec::{DynOp, ExecError, Executable};
use gunbc_ir::node::NodeBody;
use gunbc_ir::{Dag, Node, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv, PrepareFileReadOp, PrepareFileWriteOp};

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
}

impl DeferredCallableOp {
    fn new(module: &str, name: &str) -> Self {
        Self {
            module: module.to_string(),
            name: name.to_string(),
        }
    }
}

impl Executable for DeferredCallableOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        Err(ExecError::new(format!(
            "DSL callable `{}`.`{}` is not runtime-mapped yet",
            self.module, self.name
        )))
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
        resolved.add_node(node.clone().map_ops(&mut |_| dyn_op.clone()));
    }
    resolved.edges = dag.edges.clone();
    Ok(resolved)
}

// ============================================================================
// Node resolution
// ============================================================================

fn resolve_node(node: &Node<LoweredOp>) -> Result<DynOp, ResolveError> {
    let node_id = node.id.0.clone();
    match &node.body {
        NodeBody::Opaque(op) => resolve_op(&node_id, op),
        NodeBody::SubDag(_) => Err(ResolveError {
            node_id,
            reason: "SubDag nodes must be lowered before resolution".into(),
        }),
    }
}

fn resolve_op(node_id: &str, op: &LoweredOp) -> Result<DynOp, ResolveError> {
    match op {
        LoweredOp::Collection { kind, .. } => resolve_collection(kind),
        LoweredOp::Pipeline { module, name, .. } => Ok(deferred_callable(module, name)),
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
            resolve_domain(node_id, module, name)
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
        return Some(DynOp::new(FsEnv::new(filename::Scope::Write)));
    }

    // Content upsert pattern nodes (expanded from `content_upsert()` pattern)
    if let Some(suffix) = name.strip_prefix("content_upsert::") {
        if suffix.starts_with("prepare_read_") {
            return Some(DynOp::new(PrepareFileReadOp));
        }
        if suffix.starts_with("execute_read_") {
            return Some(DynOp::new(TransportOps::Execute));
        }
        if suffix.starts_with("compare_") && suffix.ends_with("_content") {
            return Some(DynOp::new(BlobOps::CompareContent));
        }
        if suffix.starts_with("prepare_write_") {
            return Some(DynOp::new(PrepareFileWriteOp));
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

fn resolve_domain(node_id: &str, module: &str, name: &str) -> Result<DynOp, ResolveError> {
    match module {
        "tools.pragma" => resolve_pragma(node_id, name),
        "tools.makegen" => resolve_makegen(node_id, name),
        "tools.build" => resolve_build(node_id, name),
        "tools.codegen" => resolve_codegen(node_id, name),
        "tools.bootstrap" => resolve_bootstrap(node_id, name),
        "tools.docgen" => resolve_docgen(node_id, name),
        "tools.testgen" => resolve_testgen(node_id, name),
        "tools.clippy" => resolve_clippy(node_id, name),
        "tools.deps" => resolve_deps(node_id, name),
        "pipelines.ci" | "shared.dag_util" | "std.patterns" | "std.resources" => {
            Ok(deferred_callable(module, name))
        }
        "services.shell"
        | "services.cargo"
        | "services.gcp.secret_manager"
        | "services.gcp.sts" => resolve_service_transport(node_id, module, name),
        _ => Err(unknown_callable(node_id, module, name)),
    }
}

fn resolve_pragma(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "render_clippy_toml" => Ok(DynOp::new(PragmaOp::RenderClippy)),
        "render_disallowed_methods_allowlist" => Ok(DynOp::new(PragmaOp::RenderAllowlist)),
        "render_pragma_lint_policy" => Ok(DynOp::new(PragmaOp::RenderLintPolicy)),
        "pragma" => Ok(deferred_callable("tools.pragma", "pragma")),
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

fn resolve_build(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "build_all" => Ok(deferred_callable("tools.build", "build_all")),
        _ => Err(unknown_callable(node_id, "tools.build", name)),
    }
}

fn resolve_codegen(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "codegen" => Ok(deferred_callable("tools.codegen", "codegen")),
        _ => Err(unknown_callable(node_id, "tools.codegen", name)),
    }
}

fn resolve_bootstrap(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "bootstrap" => Ok(deferred_callable("tools.bootstrap", "bootstrap")),
        "render_bootstrap_makefile" => Ok(deferred_callable(
            "tools.bootstrap",
            "render_bootstrap_makefile",
        )),
        "render_bootstrap_gitignore" => Ok(deferred_callable(
            "tools.bootstrap",
            "render_bootstrap_gitignore",
        )),
        _ => Err(unknown_callable(node_id, "tools.bootstrap", name)),
    }
}

fn resolve_docgen(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "docgen" => Ok(deferred_callable("tools.docgen", "docgen")),
        "render_ab_workflows_doc" => {
            Ok(deferred_callable("tools.docgen", "render_ab_workflows_doc"))
        }
        _ => Err(unknown_callable(node_id, "tools.docgen", name)),
    }
}

fn resolve_testgen(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "generate_tests" => Ok(deferred_callable("tools.testgen", "generate_tests")),
        "testgen" => Ok(deferred_callable("tools.testgen", "testgen")),
        _ => Err(unknown_callable(node_id, "tools.testgen", name)),
    }
}

fn resolve_clippy(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "clippy_lint" => Ok(deferred_callable("tools.clippy", "clippy_lint")),
        _ => Err(unknown_callable(node_id, "tools.clippy", name)),
    }
}

fn resolve_deps(node_id: &str, name: &str) -> Result<DynOp, ResolveError> {
    match name {
        "render_deps_toml" => Ok(deferred_callable("tools.deps", "render_deps_toml")),
        "select_platform_deps" => Ok(deferred_callable("tools.deps", "select_platform_deps")),
        "deps_install" => Ok(deferred_callable("tools.deps", "deps_install")),
        "deps_generate" => Ok(deferred_callable("tools.deps", "deps_generate")),
        _ => Err(unknown_callable(node_id, "tools.deps", name)),
    }
}

fn resolve_service_transport(
    node_id: &str,
    module: &str,
    name: &str,
) -> Result<DynOp, ResolveError> {
    if name.starts_with("service_transport::execute::") {
        return Ok(DynOp::new(TransportOps::Execute));
    }
    if name.starts_with("service_transport::prepare::")
        || name.starts_with("service_transport::parse::")
    {
        return Ok(deferred_callable(module, name));
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

fn deferred_callable(module: &str, name: &str) -> DynOp {
    DynOp::new(DeferredCallableOp::new(module, name))
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
        assert!(format!("{:?}", result).contains("PrepareFileReadOp"));
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
        assert!(format!("{:?}", result).contains("PrepareFileWriteOp"));
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
    fn resolve_unknown_module_fails() {
        let node = callable_node(
            "unknown_op",
            "tools.unknown",
            "do_something",
            ObligationCategory::None,
        );
        let err = resolve_node(&node).unwrap_err();
        assert!(err.reason.contains("unknown callable"));
        assert!(err.reason.contains("tools.unknown"));
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
