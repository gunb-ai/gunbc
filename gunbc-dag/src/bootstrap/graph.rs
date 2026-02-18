//! DSL-backed graph builder for the bootstrap tool.

use crate::dsl_builder::build_bootstrap_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Cardinality, Dag, WorkflowSignature};

/// Runtime op type for bootstrap graphs.
pub type BootstrapGraphOp = DynOp;

/// Get the declared signature for the bootstrap workflow.
pub fn bootstrap_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_input("check_mode", "OptionalBool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        .with_output(
            "makefile_response",
            "TransportResponse",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "makefile_written_path",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "makefile_content",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "gitignore_response",
            "TransportResponse",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "gitignore_written_path",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "gitignore_content",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output("fresh", "Bool", Cardinality::ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("crate_count", "Int", Cardinality::ONE)
}

/// Build bootstrap graph from the DSL source.
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    build_bootstrap_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_bootstrap_graph_from_dsl() {
        let dag = build_bootstrap_graph().expect("bootstrap DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
