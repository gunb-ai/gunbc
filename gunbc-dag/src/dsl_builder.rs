//! Shared helpers for DSL-backed graph builders (T3).

use daglang_driver::{compile_from_context, DriverContext};
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};
use std::collections::HashSet;

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

/// Compile a DSL module and resolve lowered ops into `Dag<DynOp>`.
pub(crate) fn build_dsl_graph(relative_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    let lowered = strip_pipeline_nodes(compile_lowered(relative_module)?);
    resolve_lowered_dag(&lowered).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}`: {error}"
        ))
    })
}

pub(crate) fn build_bootstrap_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/bootstrap.dag")
}

pub(crate) fn build_build_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/build.dag")
}

pub(crate) fn build_ci_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("pipelines/ci.dag")
}

pub(crate) fn build_codegen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/codegen.dag")
}

pub(crate) fn build_docgen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/docgen.dag")
}

pub(crate) fn build_infra_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/infra.dag")
}

pub(crate) fn build_makegen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/makegen.dag")
}

pub(crate) fn build_pragma_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/pragma.dag")
}

pub(crate) fn build_deps_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/deps.dag")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn builds_infra_dsl_graph() {
        let dag = build_infra_graph_dsl().expect("infra DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_clippy_dsl_graph() {
        let dag = build_dsl_graph("tools/clippy.dag").expect("clippy DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_deps_dsl_graph() {
        let dag = build_deps_graph_dsl().expect("deps DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_gist_dsl_graph() {
        let dag = build_dsl_graph("tools/gist.dag").expect("gist DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_review_dsl_graph() {
        let dag = build_dsl_graph("tools/review.dag").expect("review DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_ci_dsl_graph() {
        let dag = build_ci_graph_dsl().expect("ci DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
        assert!(
            !dag.nodes.iter().any(|node| node.id.0 == "pipelines.ci::ci"),
            "runtime CI graph should not include pipeline metadata nodes"
        );
    }
}
