//! Shared helpers for DSL-backed graph builders (T3).

use daglang_driver::{compile_from_context, DriverContext};
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};

use crate::resolve_lowered_dag;

fn workspace_layout() -> Result<WorkspaceLayout, BuilderError> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to resolve workspace layout for DSL builder: {error}"
            ))
        })
}

fn compile_lowered(relative_module: &str) -> Result<Dag<daglang_lower::LoweredOp>, BuilderError> {
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
    Ok(output.lowered_dag)
}

/// Compile a DSL module and resolve lowered ops into `Dag<DynOp>`.
pub fn build_dsl_graph(relative_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    let lowered = compile_lowered(relative_module)?;
    resolve_lowered_dag(&lowered).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}`: {error}"
        ))
    })
}

pub fn build_bootstrap_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/bootstrap.dag")
}

pub fn build_build_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/build.dag")
}

pub fn build_ci_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("pipelines/ci.dag")
}

pub fn build_codegen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/codegen.dag")
}

pub fn build_docgen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/docgen.dag")
}

pub fn build_makegen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/makegen.dag")
}

pub fn build_pragma_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/pragma.dag")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::build_codegen_graph;
    use crate::wire_fs_env_write_mock;
    use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionLog, ExecutionMode};

    fn find_bool_output(log: &ExecutionLog, key: &str) -> Option<bool> {
        log.entries
            .iter()
            .rev()
            .find_map(|entry| entry.outputs.get(key).and_then(|value| value.as_bool()))
    }

    #[test]
    fn builds_makegen_dsl_graph() {
        let dag = build_makegen_graph_dsl().expect("makegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_pragma_dsl_graph() {
        let dag = build_pragma_graph_dsl().expect("pragma DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_bootstrap_dsl_graph() {
        let dag = build_bootstrap_graph_dsl().expect("bootstrap DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_build_dsl_graph() {
        let dag = build_build_graph_dsl().expect("build DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_codegen_dsl_graph() {
        let dag = build_codegen_graph_dsl().expect("codegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_docgen_dsl_graph() {
        let dag = build_docgen_graph_dsl().expect("docgen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_ci_dsl_graph() {
        let dag = build_ci_graph_dsl().expect("ci DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn codegen_dsl_matches_hand_built_dry_run_behavior() {
        let hand_built = build_codegen_graph().expect("hand-built codegen graph should build");
        let dsl = build_codegen_graph_dsl().expect("DSL codegen graph should resolve");

        let mut hand_mocks = BoundaryMocks::new();
        wire_fs_env_write_mock(&hand_built, &mut hand_mocks);
        let hand_log = execute_with_mode(&hand_built, ExecutionMode::DryRun(hand_mocks))
            .expect("hand-built dry-run should execute");

        let mut dsl_mocks = BoundaryMocks::new();
        wire_fs_env_write_mock(&dsl, &mut dsl_mocks);
        let dsl_log = execute_with_mode(&dsl, ExecutionMode::DryRun(dsl_mocks))
            .expect("DSL dry-run should execute");

        let hand_success =
            find_bool_output(&hand_log, "prep_success").expect("hand-built prep_success missing");
        let hand_ran =
            find_bool_output(&hand_log, "codegen_ran").expect("hand-built codegen_ran missing");

        let dsl_success =
            find_bool_output(&dsl_log, "success").expect("DSL success output missing");
        let dsl_ran = find_bool_output(&dsl_log, "ran").expect("DSL ran output missing");

        assert_eq!(hand_success, dsl_success, "success behavior diverged");
        assert_eq!(hand_ran, dsl_ran, "ran behavior diverged");
    }
}
