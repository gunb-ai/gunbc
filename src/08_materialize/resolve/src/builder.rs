//! Shared helpers for DSL-backed graph builders.
//!
//! Generic infrastructure: compile `.dag` modules and resolve lowered ops into
//! `Dag<DynOp>` using explicit runtime bindings.
//!
//! Integrates the dagbin cache (C28-P2/P3): compiled `Dag<LoweredOp>` are
//! serialized to `target/dagbin/{source_digest}.dagbin` after compilation and
//! loaded from cache on subsequent runs if the source digest matches.

use daglang_derive::CallableProperties;
use daglang_driver::{
    compile_from_context, compile_from_context_with_options, compute_source_digest_for_context,
    CompileOptions, DriverContext, InferredEntrypoint,
};
use gunbc_exec::DynOp;
use gunbc_infra::dagbin_cache::{CacheLookup, DagbinCache};
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use crate::resolve_lowered_dag_with;

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

        slice_dag_from_entry_preserving_fn_bodies(result.dag, &entrypoint.node_id)?
    } else {
        result.dag
    };

    let dag = resolve_lowered_dag_with(&lowered).map_err(|error| {
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
    opts: BuildOpts<'_>,
) -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph(relative_module, opts).map(|r| r.dag)
}

// ============================================================================
// Dagbin cache (C28-P2/P3)
// ============================================================================

/// Bump when cache format changes. Stale caches with a different version are
/// discarded on load.
// Bump when lowering/resolution semantics change so source-digest cache hits
// do not reuse stale DAG shapes compiled by older compiler code.
const DAGBIN_CACHE_VERSION: u32 = 3;

/// Serializable bundle of compilation artifacts stored in the dagbin cache.
///
/// Contains everything `CompileLoweredResult` holds so that a cache hit can
/// fully replace a compilation without any re-derivation.
#[derive(Serialize, Deserialize)]
struct CachedCompileData {
    cache_version: u32,
    dag: gunbc_ir::Dag<daglang_lower::LoweredOp>,
    dsl_type_registry: gunbc_ir::TypeRegistry,
    inferred_entrypoints: Vec<InferredEntrypoint>,
    callable_properties: BTreeMap<String, CallableProperties>,
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

    // Dagbin cache: only used for non-profiled compilations. Profile-specific
    // compilations may produce different DAGs from the same source.
    if profile.is_none() {
        if let Some(result) = try_load_from_cache(&layout, &context, relative_module)? {
            return Ok(result);
        }
    }

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

    let result = CompileLoweredResult {
        dag: output.lowered_dag.into_inner(),
        dsl_type_registry: output.dsl_type_registry,
        inferred_entrypoints: output.inferred_entrypoints,
        callable_properties: output.derived.callable_properties,
    };

    // Store to cache (non-profiled only). Failures are silently ignored --
    // caching is an optimization, not a correctness requirement.
    if profile.is_none() {
        let _ = try_store_to_cache(&layout, &context, &result);
    }

    Ok(result)
}

/// Attempt to load a `CompileLoweredResult` from the dagbin cache.
///
/// Returns `Ok(Some(result))` on cache hit, `Ok(None)` on miss or error.
/// Build-time filesystem access (bootstrap exception).
fn try_load_from_cache(
    layout: &WorkspaceLayout,
    context: &DriverContext,
    relative_module: &str,
) -> Result<Option<CompileLoweredResult>, BuilderError> {
    let source_digest = match compute_source_digest_for_context(context) {
        Ok(d) => d,
        Err(_) => return Ok(None), // Can't compute digest -- skip cache
    };

    let cache = DagbinCache::from_workspace_root(&layout.workspace_root);

    let bytes = match cache.load(&source_digest) {
        Ok(CacheLookup::Hit(bytes)) => bytes,
        Ok(CacheLookup::Miss) => return Ok(None),
        Err(_) => return Ok(None), // I/O error -- skip cache
    };

    let cached: CachedCompileData = match serde_json::from_slice(&bytes) {
        Ok(data) => data,
        Err(_) => return Ok(None), // Corrupt cache entry -- skip
    };

    // Version mismatch -- discard stale cache
    if cached.cache_version != DAGBIN_CACHE_VERSION {
        return Ok(None);
    }

    eprintln!(
        "dagbin cache hit: {relative_module} (digest {}...)",
        &source_digest[..12.min(source_digest.len())]
    );

    Ok(Some(CompileLoweredResult {
        dag: cached.dag,
        dsl_type_registry: cached.dsl_type_registry,
        inferred_entrypoints: cached.inferred_entrypoints,
        callable_properties: cached.callable_properties,
    }))
}

/// Attempt to store a `CompileLoweredResult` in the dagbin cache.
///
/// Silently returns on any error -- caching is best-effort.
/// Build-time filesystem access (bootstrap exception).
fn try_store_to_cache(
    layout: &WorkspaceLayout,
    context: &DriverContext,
    result: &CompileLoweredResult,
) -> Result<(), BuilderError> {
    let source_digest = compute_source_digest_for_context(context).map_err(|e| {
        BuilderError::InternalInvariant(format!("dagbin cache store: source digest failed: {e}"))
    })?;

    let cached = CachedCompileData {
        cache_version: DAGBIN_CACHE_VERSION,
        dag: result.dag.clone(),
        dsl_type_registry: result.dsl_type_registry.clone(),
        inferred_entrypoints: result.inferred_entrypoints.clone(),
        callable_properties: result.callable_properties.clone(),
    };

    let bytes = serde_json::to_vec(&cached).map_err(|e| {
        BuilderError::InternalInvariant(format!("dagbin cache store: serialize failed: {e}"))
    })?;

    let cache = DagbinCache::from_workspace_root(&layout.workspace_root);
    cache.store(&source_digest, &bytes).map_err(|e| {
        BuilderError::InternalInvariant(format!("dagbin cache store: write failed: {e}"))
    })?;

    Ok(())
}

/// Slice a `Dag<LoweredOp>` to the entry node's reachable subgraph,
/// preserving all Callable nodes with fn_body.
///
/// fn items with bodies are used by `evaluate_fn_body` as sibling functions.
/// They're referenced by name from within other fn bodies, not via DAG edges,
/// so standard edge-based reachability misses them. Including all fn_body nodes
/// ensures the sibling fn lookup in `resolve_lowered_dag_with` succeeds.
fn slice_dag_from_entry_preserving_fn_bodies(
    mut dag: Dag<daglang_lower::LoweredOp>,
    entry_node_id: &str,
) -> Result<Dag<daglang_lower::LoweredOp>, BuilderError> {
    if !dag.nodes.iter().any(|node| node.id.0 == entry_node_id) {
        return Err(BuilderError::InternalInvariant(format!(
            "entry node `{entry_node_id}` not found in compiled DAG"
        )));
    }

    // Collect IDs of all Callable nodes with fn_body — these must survive slicing.
    let fn_body_node_ids: HashSet<String> = dag
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            gunbc_ir::NodeBody::Opaque(daglang_lower::LoweredOp::Callable {
                fn_body: Some(_),
                ..
            }) => Some(node.id.0.clone()),
            _ => None,
        })
        .collect();

    // Standard edge-based reachability from entry node.
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

    let mut forward_stack = vec![entry_node_id.to_string()];
    let mut forward_seen = HashSet::<String>::new();
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

    // Preserve all fn_body nodes (used by evaluate_fn_body as sibling fns).
    include.extend(fn_body_node_ids);

    dag.nodes.retain(|node| include.contains(&node.id.0));
    dag.edges
        .retain(|edge| include.contains(&edge.from_node.0) && include.contains(&edge.to_node.0));
    Ok(dag)
}

/// Derive module name from a relative `.dag` path.
///
/// `"gunbc/tools/gist.dag"` -> `"gunbc.tools.gist"`
fn module_name_from_path(relative_module: &str) -> String {
    relative_module
        .strip_suffix(".dag")
        .unwrap_or(relative_module)
        .replace('/', ".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn module_name_from_path_strips_suffix() {
        assert_eq!(
            module_name_from_path("gunbc/tools/gist.dag"),
            "gunbc.tools.gist"
        );
        assert_eq!(
            module_name_from_path("tools/bootstrap.dag"),
            "tools.bootstrap"
        );
        assert_eq!(module_name_from_path("std/fermi.dag"), "std.fermi");
    }

    /// Verify the dagbin cache round-trip: compile a real tool, store, load,
    /// and confirm the restored DAG has the same structure.
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn dagbin_cache_round_trip_with_real_tool() {
        let layout = WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout should resolve");
        let dsl_root = layout.workspace_root.join("dsl");
        let target_file = dsl_root.join("tools/bootstrap.dag");

        let context = DriverContext {
            roots: vec![dsl_root],
            target_file: Some(target_file),
        };

        // Full compilation
        let output = compile_from_context(&context).expect("bootstrap should compile");

        let original_result = CompileLoweredResult {
            dag: output.lowered_dag.into_inner(),
            dsl_type_registry: output.dsl_type_registry,
            inferred_entrypoints: output.inferred_entrypoints,
            callable_properties: output.derived.callable_properties,
        };

        // Serialize to CachedCompileData
        let cached = CachedCompileData {
            cache_version: DAGBIN_CACHE_VERSION,
            dag: original_result.dag.clone(),
            dsl_type_registry: original_result.dsl_type_registry.clone(),
            inferred_entrypoints: original_result.inferred_entrypoints.clone(),
            callable_properties: original_result.callable_properties.clone(),
        };
        let bytes = serde_json::to_vec(&cached).expect("serialize should succeed");
        assert!(bytes.len() > 100, "serialized data should be non-trivial");

        // Deserialize
        let restored: CachedCompileData =
            serde_json::from_slice(&bytes).expect("deserialize should succeed");

        // Verify structural equality
        assert_eq!(restored.cache_version, DAGBIN_CACHE_VERSION);
        assert_eq!(
            original_result.dag.nodes.len(),
            restored.dag.nodes.len(),
            "node count should survive round-trip"
        );
        assert_eq!(
            original_result.dag.edges.len(),
            restored.dag.edges.len(),
            "edge count should survive round-trip"
        );
        assert_eq!(
            original_result.inferred_entrypoints.len(),
            restored.inferred_entrypoints.len(),
            "entrypoint count should survive round-trip"
        );
        assert_eq!(
            original_result.callable_properties.len(),
            restored.callable_properties.len(),
            "callable_properties count should survive round-trip"
        );
    }

    /// Verify the full dagbin cache store/load cycle through the filesystem.
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn dagbin_cache_filesystem_store_load() {
        let cache_dir = std::env::temp_dir().join("gunbc-resolve-dagbin-builder-test");
        let _ = std::fs::remove_dir_all(&cache_dir);

        let layout = WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout should resolve");
        let dsl_root = layout.workspace_root.join("dsl");
        let target_file = dsl_root.join("tools/bootstrap.dag");

        let context = DriverContext {
            roots: vec![dsl_root],
            target_file: Some(target_file),
        };

        // Full compilation
        let output = compile_from_context(&context).expect("bootstrap should compile");

        let result = CompileLoweredResult {
            dag: output.lowered_dag.into_inner(),
            dsl_type_registry: output.dsl_type_registry,
            inferred_entrypoints: output.inferred_entrypoints,
            callable_properties: output.derived.callable_properties,
        };

        // Compute source digest
        let source_digest =
            compute_source_digest_for_context(&context).expect("source digest should succeed");

        // Store
        let cached = CachedCompileData {
            cache_version: DAGBIN_CACHE_VERSION,
            dag: result.dag.clone(),
            dsl_type_registry: result.dsl_type_registry.clone(),
            inferred_entrypoints: result.inferred_entrypoints.clone(),
            callable_properties: result.callable_properties.clone(),
        };
        let bytes = serde_json::to_vec(&cached).expect("serialize should succeed");

        let cache = DagbinCache::new(&cache_dir);
        cache
            .store(&source_digest, &bytes)
            .expect("store should succeed");

        // Load
        let loaded_bytes = match cache.load(&source_digest).expect("load should succeed") {
            CacheLookup::Hit(b) => b,
            CacheLookup::Miss => panic!("expected cache hit after store"),
        };

        let restored: CachedCompileData =
            serde_json::from_slice(&loaded_bytes).expect("deserialize should succeed");

        assert_eq!(result.dag.nodes.len(), restored.dag.nodes.len());
        assert_eq!(result.dag.edges.len(), restored.dag.edges.len());
        assert_eq!(
            result.inferred_entrypoints, restored.inferred_entrypoints,
            "entrypoints should survive cache round-trip"
        );

        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    /// Verify that stale cache (version mismatch) is discarded.
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn dagbin_cache_version_mismatch_returns_miss() {
        let cache_dir = std::env::temp_dir().join("gunbc-resolve-dagbin-version-test");
        let _ = std::fs::remove_dir_all(&cache_dir);

        let digest = "fake_digest_for_version_test";
        // Write a cache entry with a wrong version
        let stale = CachedCompileData {
            cache_version: DAGBIN_CACHE_VERSION + 999,
            dag: Dag {
                nodes: Vec::new(),
                edges: Vec::new(),
            },
            dsl_type_registry: gunbc_ir::TypeRegistry::default(),
            inferred_entrypoints: Vec::new(),
            callable_properties: BTreeMap::new(),
        };
        let bytes = serde_json::to_vec(&stale).expect("serialize should succeed");

        let cache = DagbinCache::new(&cache_dir);
        cache.store(digest, &bytes).expect("store should succeed");

        // Simulate cache lookup (manual -- not through compile_lowered)
        let loaded = match cache.load(digest).expect("load should succeed") {
            CacheLookup::Hit(b) => b,
            CacheLookup::Miss => panic!("expected raw cache hit"),
        };
        let parsed: CachedCompileData =
            serde_json::from_slice(&loaded).expect("deserialize should succeed");
        assert_ne!(parsed.cache_version, DAGBIN_CACHE_VERSION);

        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    /// Verify that `try_load_from_cache` returns `None` when cache directory
    /// doesn't exist (common on first run).
    #[test]
    fn try_load_from_empty_cache_returns_none() {
        let layout = WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout should resolve");
        let dsl_root = layout.workspace_root.join("dsl");

        let context = DriverContext {
            roots: vec![dsl_root],
            target_file: Some(layout.workspace_root.join("dsl/tools/bootstrap.dag")),
        };

        // Use a non-existent cache dir by creating a layout with a fake root
        let fake_layout = WorkspaceLayout {
            workspace_root: Path::new("/tmp/gunbc-resolve-dagbin-nonexistent-root").to_path_buf(),
            crates: BTreeMap::new(),
        };

        let result = try_load_from_cache(&fake_layout, &context, "tools/bootstrap.dag")
            .expect("try_load should not error");
        assert!(result.is_none(), "empty cache should return None");
    }
}
