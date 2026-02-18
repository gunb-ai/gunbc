//! DSL-backed graph builder for the build pipeline.

use crate::dsl_builder::build_build_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Cardinality, Dag, WorkflowSignature};

/// Runtime op type for build graphs.
pub type BuildGraphOp = DynOp;

/// Get the declared signature for the build workflow.
pub fn build_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_output("overall_success", "Bool", Cardinality::ONE)
        .with_output("report", "String", Cardinality::ONE)
}

/// Build the build graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "build",
    builder = "build_build_graph().unwrap()"
)]
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    build_build_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_build_graph_from_dsl() {
        let dag = build_build_graph().expect("build DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
