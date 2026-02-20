use std::path::PathBuf;

use crate::path_utils;
use crate::pipeline::PipelineContext;
use daglang_driver::DriverContext;
use daglang_resolve::ModuleGraph;
use gunbc_exec::{BoundaryMocks, DynOp, ExecutionLog, ExecutionMode};
use gunbc_ir::Dag;

use super::{resolve_lowered_dag, CheckOutput, CompileError, CompileOptions, CompileOutput};

/// Builds compile pipeline context from CLI input.
///
/// Paths ending in `.dag` that are regular files are treated as single-file
/// targets. Directories named with a `.dag` suffix are rejected with an
/// explicit error — callers should pass the directory path without the
/// `.dag` suffix or reference a `.dag` file inside it.
pub fn build_context(cwd: &std::path::Path, input: Option<&String>) -> PipelineContext {
    let parsed = input.map(|value| path_utils::normalize_cli_path(cwd, &PathBuf::from(value)));
    if let Some(ref path) = parsed {
        if let Some(error) = path_utils::check_dag_directory_conflict(path) {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
    let (roots, target_file) = match parsed {
        Some(path) if path_utils::is_single_file_target(&path) => {
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
    let resolved = resolve_lowered_dag(&output.lowered_dag)
        .map_err(|error| CompileError::from(format!("resolve error: {error}")))?;
    execute_resolved_dag(&resolved, mode, input_mocks)
}
