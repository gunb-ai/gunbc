use std::path::PathBuf;

use crate::path_utils;
use crate::pipeline::PipelineContext;
use daglang_driver::DriverContext;
use daglang_resolve::ModuleGraph;
use gunbc_exec::{BoundaryMocks, DynOp, ExecutionLog, ExecutionMode};
use gunbc_ir::Dag;
use serde::Deserialize;

use super::{CheckOutput, CompileError, CompileOptions, CompileOutput};

/// Builds compile pipeline context from CLI input.
///
/// Paths ending in `.dag` that are regular files are treated as single-file
/// targets. Directories named with a `.dag` suffix are rejected with an
/// explicit error — callers should pass the directory path without the
/// `.dag` suffix or reference a `.dag` file inside it.
pub fn build_context(
    cwd: &std::path::Path,
    input: Option<&String>,
) -> Result<PipelineContext, String> {
    build_context_with_default_roots(cwd, input, None)
}

#[allow(clippy::disallowed_methods)]
pub fn resolve_configured_roots(cwd: &std::path::Path) -> Result<Option<Vec<PathBuf>>, String> {
    let config_path = cwd.join("daglang.toml");
    if !config_path.exists() {
        return Ok(None);
    }
    let config_text = std::fs::read_to_string(&config_path)
        .map_err(|error| format!("failed to read {}: {error}", config_path.display()))?;
    let parsed: DaglangConfig = toml::from_str(&config_text)
        .map_err(|error| format!("failed to parse {}: {error}", config_path.display()))?;
    let Some(discovery) = parsed.discovery else {
        return Ok(None);
    };
    let Some(config_roots) = discovery.roots else {
        return Ok(None);
    };
    if config_roots.is_empty() {
        return Err("discovery.roots in daglang.toml must not be empty".to_string());
    }
    let mut normalized = config_roots
        .iter()
        .map(|root| path_utils::normalize_cli_path(cwd, &PathBuf::from(root)))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    Ok(Some(normalized))
}

pub fn resolve_default_roots(cwd: &std::path::Path) -> Result<Vec<PathBuf>, String> {
    match resolve_configured_roots(cwd)? {
        Some(config_roots) => Ok(config_roots),
        None => Ok(vec![path_utils::resolve_default_root(cwd)]),
    }
}

/// Build pipeline context from optional CLI input and optional pre-resolved
/// default roots. This is shared by compile/check/modules command flows.
pub fn build_context_with_default_roots(
    cwd: &std::path::Path,
    input: Option<&String>,
    default_roots: Option<&[PathBuf]>,
) -> Result<PipelineContext, String> {
    let parsed = input.map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    if let Some(ref path) = parsed {
        if let Some(error) = path_utils::check_dag_directory_conflict(path) {
            return Err(error);
        }
    }
    let (roots, target_file) = match parsed {
        Some(path) if path_utils::is_single_file_target(&path) => {
            let root = path_utils::resolve_single_file_root(cwd, &path);
            (vec![root], Some(path))
        }
        Some(path) => (vec![path], None),
        None => (
            default_roots
                .map(|roots| roots.to_vec())
                .unwrap_or_else(|| vec![path_utils::resolve_default_root(cwd)]),
            None,
        ),
    };

    Ok(PipelineContext { roots, target_file })
}

#[derive(Debug, Deserialize)]
struct DaglangConfig {
    discovery: Option<DiscoveryConfig>,
}

#[derive(Debug, Deserialize)]
struct DiscoveryConfig {
    roots: Option<Vec<String>>,
}

pub fn compile_from_context(context: &PipelineContext) -> Result<CompileOutput, CompileError> {
    compile_from_context_with_options(context, CompileOptions::default())
}

pub fn compile_from_context_with_options(
    context: &PipelineContext,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    daglang_driver::compile_from_context_with_options(
        &DriverContext {
            roots: context.roots.clone(),
            target_file: context.target_file.clone(),
        },
        options,
    )
}

pub fn check_from_context(context: &PipelineContext) -> Result<CheckOutput, CompileError> {
    daglang_driver::check_from_context(&DriverContext {
        roots: context.roots.clone(),
        target_file: context.target_file.clone(),
    })
}

pub fn check_from_module_graph(module_graph: ModuleGraph) -> Result<CheckOutput, CompileError> {
    daglang_driver::check_from_module_graph(module_graph)
}

pub fn execute_resolved_dag(
    dag: &Dag<DynOp>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, CompileError> {
    gunbc_exec::execute_with_mode_and_inputs(dag, mode, input_mocks)
        .map_err(|error| CompileError::from(format!("execution error: {error}")))
}

pub fn compile_resolve_execute_from_context(
    context: &PipelineContext,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, CompileError> {
    let output = compile_from_context(context)?;
    let resolved = gunbc_resolve::resolve_lowered_dag_with(
        &output.lowered_dag,
        gunbc_app::extern_ops::gunbc_runtime_bindings(),
    )
    .map_err(|error| CompileError::from(format!("resolve error: {error}")))?;
    execute_resolved_dag(&resolved, mode, input_mocks)
}

/// Compile from a pre-built module graph, skipping discovery (DL5).
pub fn compile_from_module_graph_with_options(
    context: &PipelineContext,
    module_graph: ModuleGraph,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    daglang_driver::compile_from_module_graph_with_options(
        &DriverContext {
            roots: context.roots.clone(),
            target_file: context.target_file.clone(),
        },
        module_graph,
        options,
    )
}
