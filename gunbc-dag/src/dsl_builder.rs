//! Thin wrapper: delegates DSL builder operations to `gunbc_resolve::builder`
//! with app-specific extern symbol resolution.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

pub use gunbc_resolve::builder::DslGraphResult;

use crate::resolve::GunbcExternResolver;

/// Compile a DSL module and resolve lowered ops into `Dag<DynOp>`.
pub fn build_dsl_graph(relative_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph(relative_module, &GunbcExternResolver)
}

/// Convention-based tool graph builder.
///
/// `build_tool_graph("bootstrap")` -> `build_dsl_graph("tools/bootstrap.dag")`.
pub fn build_tool_graph(tool_name: &str) -> Result<Dag<DynOp>, BuilderError> {
    gunbc_resolve::builder::build_tool_graph(tool_name, &GunbcExternResolver)
}

/// Infer the workflow signature for a convention-based tool.
pub fn tool_signature(
    tool_name: &str,
) -> Result<gunbc_ir::WorkflowSignature, BuilderError> {
    gunbc_resolve::builder::tool_signature(tool_name, &GunbcExternResolver)
}

/// Compile a DSL module and resolve lowered ops, also returning DSL type registry.
pub(crate) fn build_dsl_graph_with_types(
    relative_module: &str,
) -> Result<DslGraphResult, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph_with_types(relative_module, &GunbcExternResolver)
}

/// Compile a DSL module with an active profile and resolve, returning full result (RT24).
pub(crate) fn build_dsl_graph_with_types_and_profile(
    relative_module: &str,
    profile: &str,
) -> Result<DslGraphResult, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph_with_types_and_profile(
        relative_module,
        profile,
        &GunbcExternResolver,
    )
}

/// Build a DSL graph with an active profile (PT-4).
pub fn build_dsl_graph_with_profile(
    relative_module: &str,
    profile: &str,
) -> Result<Dag<DynOp>, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph_with_profile(
        relative_module,
        profile,
        &GunbcExternResolver,
    )
}

pub fn build_dsl_graph_for_entry(
    relative_module: &str,
    entry_node_id: &str,
) -> Result<Dag<DynOp>, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph_for_entry(
        relative_module,
        entry_node_id,
        &GunbcExternResolver,
    )
}

/// Compile a DSL module and resolve to `Dag<DynOp>` by selecting an inferred entrypoint.
///
/// - `entry_func: None` — use the sole inferred entrypoint (errors if multiple)
/// - `entry_func: Some("name")` — select the named entrypoint
pub fn build_dsl_graph_for_entrypoint(
    relative_module: &str,
    entry_func: Option<&str>,
) -> Result<Dag<DynOp>, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph_for_entrypoint(
        relative_module,
        entry_func,
        &GunbcExternResolver,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_makegen_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/makegen.dag", Some("makegen"))
            .expect("makegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_pragma_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/pragma.dag", Some("pragma"))
            .expect("pragma DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_bootstrap_dsl_graph() {
        let dag =
            build_dsl_graph("tools/bootstrap.dag").expect("bootstrap DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_codegen_dsl_graph() {
        let dag = build_dsl_graph("tools/codegen.dag").expect("codegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_infra_dsl_graph() {
        let dag = build_dsl_graph("tools/infra.dag").expect("infra DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_clippy_dsl_graph() {
        let dag = build_dsl_graph("tools/clippy.dag").expect("clippy DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_deps_dsl_graph() {
        let dag = build_dsl_graph("tools/deps.dag").expect("deps DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_review_dsl_graph() {
        let dag = build_dsl_graph("tools/review.dag").expect("review DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_dimension_review_dsl_graph() {
        let dag = build_dsl_graph("funcs/review_pipeline.dag")
            .expect("dimension review DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_gist_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist"))
            .expect("gist DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_ci_dsl_graph() {
        let dag = build_dsl_graph("pipelines/ci.dag").expect("ci DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
        assert!(
            !dag.nodes.iter().any(|node| node.id.0 == "pipelines.ci::ci"),
            "runtime CI graph should not include pipeline metadata nodes"
        );
    }
}
