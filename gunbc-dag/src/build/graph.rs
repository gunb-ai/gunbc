//! DSL-backed graph builder for the build pipeline.

use crate::dsl_builder::build_build_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for build graphs.
pub type BuildGraphOp = DynOp;

/// Get the declared signature for the build workflow (auto-derived from DAG).
pub fn build_signature() -> WorkflowSignature {
    match build_build_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build build DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
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
