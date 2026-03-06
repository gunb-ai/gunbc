//! Shared helpers for DSL-backed graph builders.
//!
//! Generic infrastructure: compile `.dag` modules and resolve lowered ops into
//! `Dag<DynOp>` using explicit runtime bindings.

use daglang_derive::CallableProperties;
use daglang_driver::{
    compile_from_context, compile_from_context_with_options, CompileOptions, DriverContext,
    InferredEntrypoint,
};
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};
use std::collections::{BTreeMap, HashSet};

use crate::{resolve_lowered_dag_with, RuntimeBindings};

/// Options for `build_dsl_graph`.
#[derive(Debug, Default)]
pub struct BuildOpts<'a> {
    /// Active profile for interface binding resolution (PT-4).
    pub profile: Option<&'a str>,
    /// Slice DAG to the named entrypoint's reachable subgraph.
    /// When `None`, the full module DAG is returned.
    pub entry_func: Option<&'a str>,
}

/// Result of compiling and resolving a DSL module.
pub struct DslGraphResult {
    /// The resolved DAG.
    pub dag: Dag<DynOp>,
    /// Type registry extracted from DSL-defined sum/product types.
    pub dsl_type_registry: gunbc_ir::TypeRegistry,
    /// Per-callable structural properties derived from graph traversal.
    pub callable_properties: BTreeMap<String, CallableProperties>,
}

/// Compile a DSL module and resolve lowered ops into `Dag<DynOp>`.
///
/// This is the single entry point for all DSL graph building. Use `BuildOpts`
/// to control profile selection and entrypoint slicing.
pub fn build_dsl_graph(
    relative_module: &str,
    bindings: &RuntimeBindings,
    opts: BuildOpts<'_>,
) -> Result<DslGraphResult, BuilderError> {
    let result = compile_lowered(relative_module, opts.profile)?;

    let lowered = if let Some(entry_func) = opts.entry_func {
        let module_name = module_name_from_path(relative_module);
        let module_entrypoints: Vec<&InferredEntrypoint> = result
            .inferred_entrypoints
            .iter()
            .filter(|ep| ep.module == module_name)
            .collect();

        let entrypoint = module_entrypoints
            .iter()
            .find(|ep| ep.func_name == entry_func)
            .ok_or_else(|| {
                BuilderError::InternalInvariant(format!(
                    "entrypoint `{entry_func}` not found in `{relative_module}` (available: {:?})",
                    module_entrypoints
                        .iter()
                        .map(|ep| ep.func_name.as_str())
                        .collect::<Vec<_>>()
                ))
            })?;

        slice_dag_from_entry(result.dag, &entrypoint.node_id)?
    } else {
        result.dag
    };

    let dag = resolve_lowered_dag_with(&lowered, bindings).map_err(|error| {
        let ctx = match (opts.profile, opts.entry_func) {
            (Some(p), Some(e)) => format!(" (profile={p}, entry={e})"),
            (Some(p), None) => format!(" (profile={p})"),
            (None, Some(e)) => format!(" (entry={e})"),
            (None, None) => String::new(),
        };
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}`{ctx}: {error}"
        ))
    })?;

    Ok(DslGraphResult {
        dag,
        dsl_type_registry: result.dsl_type_registry,
        callable_properties: result.callable_properties,
    })
}

/// Convenience wrapper: compile + resolve a DSL module and return just the DAG.
///
/// Used by generated CLI binaries to avoid a redundant closure around
/// `build_dsl_graph(...).map(|r| r.dag)`.
pub fn build_dsl_graph_dag(
    relative_module: &str,
    bindings: &RuntimeBindings,
    opts: BuildOpts<'_>,
) -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph(relative_module, bindings, opts).map(|r| r.dag)
}

// ============================================================================
// Internal helpers
// ============================================================================

fn workspace_layout() -> Result<WorkspaceLayout, BuilderError> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to resolve workspace layout for DSL builder: {error}"
            ))
        })
}

struct CompileLoweredResult {
    dag: Dag<daglang_lower::LoweredOp>,
    dsl_type_registry: gunbc_ir::TypeRegistry,
    inferred_entrypoints: Vec<InferredEntrypoint>,
    callable_properties: BTreeMap<String, CallableProperties>,
}

fn compile_lowered(
    relative_module: &str,
    profile: Option<&str>,
) -> Result<CompileLoweredResult, BuilderError> {
    let layout = workspace_layout()?;
    let dsl_root = layout.workspace_root.join("dsl");
    let target_file = dsl_root.join(relative_module);

    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(target_file),
    };

    let output = if let Some(profile) = profile {
        let options = CompileOptions {
            profile: Some(profile.to_string()),
            ..CompileOptions::default()
        };
        compile_from_context_with_options(&context, options).map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to compile DSL module `{relative_module}` with profile `{profile}`: {error}"
            ))
        })?
    } else {
        compile_from_context(&context).map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to compile DSL module `{relative_module}`: {error}"
            ))
        })?
    };

    Ok(CompileLoweredResult {
        dag: output.lowered_dag.into_inner(),
        dsl_type_registry: output.dsl_type_registry,
        inferred_entrypoints: output.inferred_entrypoints,
        callable_properties: output.derived.callable_properties,
    })
}

fn slice_dag_from_entry<T>(mut dag: Dag<T>, entry_node_id: &str) -> Result<Dag<T>, BuilderError> {
    if !dag.nodes.iter().any(|node| node.id.0 == entry_node_id) {
        return Err(BuilderError::InternalInvariant(format!(
            "entry node `{entry_node_id}` not found in compiled DAG"
        )));
    }

    let mut include = HashSet::<String>::new();
    let mut backward_stack = vec![entry_node_id.to_string()];
    while let Some(node_id) = backward_stack.pop() {
        if !include.insert(node_id.clone()) {
            continue;
        }
        for edge in &dag.edges {
            if edge.to_node.0 == node_id {
                backward_stack.push(edge.from_node.0.clone());
            }
        }
    }

    let mut forward_seen = HashSet::<String>::new();
    let mut forward_stack = vec![entry_node_id.to_string()];
    while let Some(node_id) = forward_stack.pop() {
        if !forward_seen.insert(node_id.clone()) {
            continue;
        }
        include.insert(node_id.clone());
        for edge in &dag.edges {
            if edge.from_node.0 == node_id {
                forward_stack.push(edge.to_node.0.clone());
            }
        }
    }

    dag.nodes.retain(|node| include.contains(&node.id.0));
    dag.edges
        .retain(|edge| include.contains(&edge.from_node.0) && include.contains(&edge.to_node.0));
    Ok(dag)
}

/// Derive module name from a relative `.dag` path.
///
/// `"tools/makegen.dag"` -> `"tools.makegen"`
fn module_name_from_path(relative_module: &str) -> String {
    relative_module
        .strip_suffix(".dag")
        .unwrap_or(relative_module)
        .replace('/', ".")
}
