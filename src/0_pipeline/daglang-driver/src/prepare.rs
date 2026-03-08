use std::path::Path;

use super::*;

use daglang_emit::rust_exec_runtime::{
    DependencyPaths as ExecRuntimeDependencyPaths, EmitConfig as ExecRuntimeEmitConfig,
};
use gunbc_ir::WorkspaceLayout;

/// Context prepared for the stage-ordered compile runner.
///
/// This packages the context-sensitive setup that still depends on driver
/// inputs (profile discovery, target-module selection, path validation,
/// deterministic source digest seeding) so the main stage runner can focus on
/// the compile pipeline itself.
#[derive(Debug)]
pub struct PreparedCompileContext {
    pub(crate) module_graph: ModuleGraph,
    pub(crate) callable_scope: Option<HashSet<String>>,
    pub(crate) entry_module_name: Option<String>,
    pub(crate) target_module_name: Option<String>,
    pub(crate) lossy_fn_bodies: Vec<String>,
    pub(crate) source_digest: Option<String>,
    pub(crate) exec_runtime_emit_config: ExecRuntimeEmitConfig,
}

/// Prepare a discovered module graph for stage-ordered compilation.
///
/// This is the impure edge of the driver pipeline. It may augment the module
/// graph from the filesystem (profile modules), canonicalize target paths for
/// matching, validate module/filename consistency, and seed the deterministic
/// source digest from the already loaded module sources.
pub fn prepare_compile_context(
    context: &DriverContext,
    mut module_graph: ModuleGraph,
    options: &CompileOptions,
) -> Result<PreparedCompileContext, CompileError> {
    include_profile_modules(
        &mut module_graph,
        &context.roots,
        options.profile.as_deref(),
    )?;
    validate_module_path_consistency(
        &module_graph,
        &context.roots,
        context.target_file.as_deref(),
    )?;
    let callable_scope_result = callable_scope_for_context(context, &module_graph)?;
    let (callable_scope, entry_module_name) = match callable_scope_result {
        Some((scope, entry)) => (Some(scope), Some(entry)),
        None => (None, None),
    };

    Ok(PreparedCompileContext {
        source_digest: Some(compute_source_digest_from_module_graph(&module_graph)),
        target_module_name: target_module_name_for_context(context, &module_graph)?,
        lossy_fn_bodies: collect_lossy_fn_bodies(&module_graph),
        exec_runtime_emit_config: prepare_exec_runtime_emit_config(options)?,
        module_graph,
        callable_scope,
        entry_module_name,
    })
}

pub(crate) fn prepare_exec_runtime_emit_config(
    options: &CompileOptions,
) -> Result<ExecRuntimeEmitConfig, CompileError> {
    let dependency_paths = match options.output_dir.as_deref() {
        Some(output_dir) => dependency_paths_for_output_dir(output_dir)?,
        None => ExecRuntimeDependencyPaths::default(),
    };
    Ok(ExecRuntimeEmitConfig { dependency_paths })
}

fn dependency_paths_for_output_dir(
    output_dir: &Path,
) -> Result<ExecRuntimeDependencyPaths, CompileError> {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            CompileError::from(format!(
                "failed to resolve workspace layout for exec-runtime emission: {error}"
            ))
        })?;
    Ok(ExecRuntimeDependencyPaths {
        gunbc_ir: relative_workspace_dependency_path(&layout, output_dir, "gunbc-ir")?,
        gunbc_exec: relative_workspace_dependency_path(&layout, output_dir, "gunbc-exec")?,
        gunbc_lib_transport: relative_workspace_dependency_path(
            &layout,
            output_dir,
            "gunbc-lib-transport",
        )?,
    })
}

fn relative_workspace_dependency_path(
    layout: &WorkspaceLayout,
    output_dir: &Path,
    crate_name: &str,
) -> Result<String, CompileError> {
    let dep_dir = layout.crate_dir(crate_name).ok_or_else(|| {
        CompileError::from(format!(
            "workspace layout missing crate `{crate_name}` for exec-runtime emission"
        ))
    })?;
    Ok(normalize_dep_path(
        &layout.relative_path(output_dir, dep_dir),
    ))
}

fn normalize_dep_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
