//! Thin wrapper: delegates DSL builder operations to `gunbc_resolve::builder`
//! with app-specific extern symbol resolution.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

pub use gunbc_resolve::builder::DslGraphResult;
pub use gunbc_resolve::BuildOpts;

use crate::resolve::GunbcExternResolver;

/// Compile a DSL module and resolve lowered ops into `Dag<DynOp>`.
pub fn build_dsl_graph(relative_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    Ok(gunbc_resolve::builder::build_dsl_graph(
        relative_module,
        &GunbcExternResolver,
        BuildOpts::default(),
    )?
    .dag)
}

/// Compile a DSL module and resolve lowered ops, also returning DSL type registry.
pub(crate) fn build_dsl_graph_with_types(
    relative_module: &str,
) -> Result<DslGraphResult, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph(
        relative_module,
        &GunbcExternResolver,
        BuildOpts::default(),
    )
}

/// Compile a DSL module with an active profile and resolve, returning full result (RT24).
pub(crate) fn build_dsl_graph_with_types_and_profile(
    relative_module: &str,
    profile: &str,
) -> Result<DslGraphResult, BuilderError> {
    gunbc_resolve::builder::build_dsl_graph(
        relative_module,
        &GunbcExternResolver,
        BuildOpts {
            profile: Some(profile),
            ..BuildOpts::default()
        },
    )
}

/// Build a DSL graph with an active profile (PT-4).
pub fn build_dsl_graph_with_profile(
    relative_module: &str,
    profile: &str,
) -> Result<Dag<DynOp>, BuilderError> {
    Ok(gunbc_resolve::builder::build_dsl_graph(
        relative_module,
        &GunbcExternResolver,
        BuildOpts {
            profile: Some(profile),
            ..BuildOpts::default()
        },
    )?
    .dag)
}

/// Compile a DSL module and resolve to `Dag<DynOp>` by selecting an inferred entrypoint.
///
/// When `profile` is `Some`, the named profile's interface bindings are activated.
pub fn build_dsl_graph_for_entrypoint(
    relative_module: &str,
    entry_func: Option<&str>,
    profile: Option<&str>,
) -> Result<Dag<DynOp>, BuilderError> {
    Ok(gunbc_resolve::builder::build_dsl_graph(
        relative_module,
        &GunbcExternResolver,
        BuildOpts {
            entry_func,
            profile,
        },
    )?
    .dag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_makegen_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/makegen.dag", Some("makegen"), None)
            .expect("makegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_pragma_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/pragma.dag", Some("pragma"), None)
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
        let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist"), None)
            .expect("gist DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_ci_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/ci.dag", Some("ci"), None)
            .expect("ci DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_readme_dsl_graph() {
        let dag = build_dsl_graph_for_entrypoint("tools/readme.dag", Some("readme"), None)
            .expect("readme DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }
}
