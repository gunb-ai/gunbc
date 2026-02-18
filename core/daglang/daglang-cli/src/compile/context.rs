use std::path::PathBuf;

use crate::path_utils;
use crate::pipeline::PipelineContext;
use daglang_driver::DriverContext;
use gunbc_exec::{BoundaryMocks, ExecutionLog, ExecutionMode};
use gunbc_ir::Dag;

use super::{
    resolve_lowered_dag, CheckOutput, CompileError, CompileOptions, CompileOutput, ResolvedOp,
};

/// Builds compile pipeline context from CLI input.
///
/// Compatibility note: paths ending in `.dag` are always treated as
/// single-file targets, even when they point to a directory.
/// This only applies to the strict lowercase `.dag` extension.
/// Wrong-cased dag-like extensions (`.DAG`, `.DaG`, etc.) are handled by
/// higher-level CLI validation and are not treated as single-file targets.
pub fn build_context(cwd: &std::path::Path, input: Option<&String>) -> PipelineContext {
    let parsed = input.map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    let (roots, target_file) = match parsed {
        Some(path) if path_utils::is_single_file_target(&path, true) => {
            let root = path_utils::resolve_single_file_root(cwd, &path);
            (vec![root], Some(path))
        }
        Some(path) => (vec![path], None),
        None => (vec![path_utils::resolve_default_root(cwd)], None),
    };

    PipelineContext { roots, target_file }
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

pub fn execute_resolved_dag(
    dag: &Dag<ResolvedOp>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, CompileError> {
    daglang_exec_bridge::execute_resolved_dag(dag, mode, input_mocks)
        .map_err(|error| CompileError::from(format!("execution error: {error}")))
}

pub fn compile_resolve_execute_from_context(
    context: &PipelineContext,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, CompileError> {
    let output = compile_from_context(context)?;
    let resolved = resolve_lowered_dag(&output.lowered_dag)
        .map_err(|error| CompileError::from(format!("resolve error: {error}")))?;
    execute_resolved_dag(&resolved, mode, input_mocks)
}
