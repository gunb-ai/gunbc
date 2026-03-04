//! Shared helpers for DSL-backed graph builders.
//!
//! Generic infrastructure: compile `.dag` modules and resolve lowered ops into
//! `Dag<DynOp>` using a pluggable `ExternResolver`.

use daglang_derive::CallableProperties;
use daglang_driver::{
    compile_from_context, compile_from_context_with_options, CompileOptions, DriverContext,
    InferredEntrypoint,
};
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};
use std::collections::{BTreeMap, HashSet};

use crate::{resolve_lowered_dag_with, ExternResolver};

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

fn compile_lowered(relative_module: &str) -> Result<CompileLoweredResult, BuilderError> {
    let layout = workspace_layout()?;
    let dsl_root = layout.workspace_root.join("dsl");
    let target_file = dsl_root.join(relative_module);

    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(target_file),
    };

    let output = compile_from_context(&context).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to compile DSL module `{relative_module}`: {error}"
        ))
    })?;
    Ok(CompileLoweredResult {
        dag: output.lowered_dag,
        dsl_type_registry: output.dsl_type_registry,
        inferred_entrypoints: output.inferred_entrypoints,
        callable_properties: output.derived.callable_properties,
    })
}

/// Compile a DSL module with an active profile (PT-4).
///
/// Threads the profile name through `CompileOptions`, which causes the lowerer
/// to resolve interface bindings via the profile's bind declarations instead of
/// using stub transport.
fn compile_lowered_with_profile(
    relative_module: &str,
    profile: &str,
) -> Result<CompileLoweredResult, BuilderError> {
    let layout = workspace_layout()?;
    let dsl_root = layout.workspace_root.join("dsl");
    let target_file = dsl_root.join(relative_module);

    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(target_file),
    };
    let options = CompileOptions {
        profile: Some(profile.to_string()),
        ..CompileOptions::default()
    };

    let output = compile_from_context_with_options(&context, options).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to compile DSL module `{relative_module}` with profile `{profile}`: {error}"
        ))
    })?;
    Ok(CompileLoweredResult {
        dag: output.lowered_dag,
        dsl_type_registry: output.dsl_type_registry,
        inferred_entrypoints: output.inferred_entrypoints,
        callable_properties: output.derived.callable_properties,
    })
}

fn strip_pipeline_nodes(mut dag: Dag<daglang_lower::LoweredOp>) -> Dag<daglang_lower::LoweredOp> {
    let pipeline_ids: HashSet<String> = dag
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            gunbc_ir::node::NodeBody::Opaque(daglang_lower::LoweredOp::Pipeline { .. }) => {
                Some(node.id.0.clone())
            }
            _ => None,
        })
        .collect();

    if pipeline_ids.is_empty() {
        return dag;
    }

    dag.nodes.retain(|node| !pipeline_ids.contains(&node.id.0));
    dag.edges.retain(|edge| {
        !pipeline_ids.contains(&edge.from_node.0) && !pipeline_ids.contains(&edge.to_node.0)
    });
    dag
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
pub fn build_dsl_graph(
    relative_module: &str,
    resolver: &dyn ExternResolver,
) -> Result<Dag<DynOp>, BuilderError> {
    Ok(build_dsl_graph_with_types(relative_module, resolver)?.dag)
}

/// Convention-based tool graph builder.
///
/// `build_tool_graph("bootstrap", resolver)` -> `build_dsl_graph("tools/bootstrap.dag", resolver)`.
/// Replaces per-tool wrapper modules (RT81).
pub fn build_tool_graph(
    tool_name: &str,
    resolver: &dyn ExternResolver,
) -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph(&format!("tools/{tool_name}.dag"), resolver)
}

/// Infer the workflow signature for a convention-based tool.
///
/// Compiles the tool's DSL module and derives the signature from graph structure.
pub fn tool_signature(
    tool_name: &str,
    resolver: &dyn ExternResolver,
) -> Result<gunbc_ir::WorkflowSignature, BuilderError> {
    build_tool_graph(tool_name, resolver).map(|dag| gunbc_ir::infer_signature(&dag))
}

/// Compile a DSL module and resolve lowered ops, also returning DSL type registry.
pub fn build_dsl_graph_with_types(
    relative_module: &str,
    resolver: &dyn ExternResolver,
) -> Result<DslGraphResult, BuilderError> {
    let result = compile_lowered(relative_module)?;
    let lowered = strip_pipeline_nodes(result.dag);
    let dag = resolve_lowered_dag_with(&lowered, resolver).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}`: {error}"
        ))
    })?;
    Ok(DslGraphResult {
        dag,
        dsl_type_registry: result.dsl_type_registry,
        callable_properties: result.callable_properties,
    })
}

/// Compile a DSL module with an active profile and resolve, returning full result (RT24).
///
/// Like `build_dsl_graph_with_types()` but threads a profile through compilation
/// so interface bindings resolve via the profile's bind declarations.
pub fn build_dsl_graph_with_types_and_profile(
    relative_module: &str,
    profile: &str,
    resolver: &dyn ExternResolver,
) -> Result<DslGraphResult, BuilderError> {
    let result = compile_lowered_with_profile(relative_module, profile)?;
    let lowered = strip_pipeline_nodes(result.dag);
    let dag = resolve_lowered_dag_with(&lowered, resolver).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}` with profile `{profile}`: {error}"
        ))
    })?;
    Ok(DslGraphResult {
        dag,
        dsl_type_registry: result.dsl_type_registry,
        callable_properties: result.callable_properties,
    })
}

/// Build a DSL graph with an active profile (PT-4).
///
/// This is the compilation path for per-profile live tests. The profile
/// resolves interface bindings to concrete service implementations.
pub fn build_dsl_graph_with_profile(
    relative_module: &str,
    profile: &str,
    resolver: &dyn ExternResolver,
) -> Result<Dag<DynOp>, BuilderError> {
    let result = compile_lowered_with_profile(relative_module, profile)?;
    let lowered = strip_pipeline_nodes(result.dag);
    resolve_lowered_dag_with(&lowered, resolver).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}` with profile `{profile}`: {error}"
        ))
    })
}

/// Compile a DSL module and resolve to `Dag<DynOp>` by selecting an inferred entrypoint.
///
/// - `entry_func: None` — use the sole inferred entrypoint (errors if multiple)
/// - `entry_func: Some("name")` — select the named entrypoint
pub fn build_dsl_graph_for_entrypoint(
    relative_module: &str,
    entry_func: Option<&str>,
    resolver: &dyn ExternResolver,
) -> Result<Dag<DynOp>, BuilderError> {
    let result = compile_lowered(relative_module)?;
    let module_name = module_name_from_path(relative_module);

    // Filter entrypoints to this module only
    let module_entrypoints: Vec<&InferredEntrypoint> = result
        .inferred_entrypoints
        .iter()
        .filter(|ep| ep.module == module_name)
        .collect();

    let entrypoint = match entry_func {
        Some(name) => module_entrypoints
            .iter()
            .find(|ep| ep.func_name == name)
            .ok_or_else(|| {
                BuilderError::InternalInvariant(format!(
                    "entrypoint `{name}` not found in `{relative_module}` (available: {:?})",
                    module_entrypoints
                        .iter()
                        .map(|ep| ep.func_name.as_str())
                        .collect::<Vec<_>>()
                ))
            })?,
        None => {
            if module_entrypoints.len() == 1 {
                &module_entrypoints[0]
            } else {
                return Err(BuilderError::InternalInvariant(format!(
                    "`{relative_module}` has {} entrypoints — disambiguate with entry_func: {:?}",
                    module_entrypoints.len(),
                    module_entrypoints
                        .iter()
                        .map(|ep| ep.func_name.as_str())
                        .collect::<Vec<_>>()
                )));
            }
        }
    };

    let entry_node_id = &entrypoint.node_id;
    let lowered = strip_pipeline_nodes(result.dag);
    let lowered = slice_dag_from_entry(lowered, entry_node_id)?;
    resolve_lowered_dag_with(&lowered, resolver).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}` entry `{entry_node_id}`: {error}"
        ))
    })
}
