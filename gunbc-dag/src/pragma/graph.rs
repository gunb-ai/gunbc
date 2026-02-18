//! DSL-backed graph builder for the pragma tool.

use crate::dsl_builder::build_pragma_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Cardinality, Dag, WorkflowSignature};

/// Runtime op type for pragma graphs.
pub type PragmaGraphOp = DynOp;

/// Get the declared signature for the pragma workflow.
pub fn pragma_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_input("check_mode", "OptionalBool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        .with_output(
            "clippy_response",
            "TransportResponse",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "clippy_written_path",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output("clippy_content", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output(
            "allowlist_response",
            "TransportResponse",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "allowlist_written_path",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "allowlist_content",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "policy_response",
            "TransportResponse",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "policy_written_path",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output("policy_content", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("fresh", "Bool", Cardinality::ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "OptionalString", Cardinality::ZERO_OR_ONE)
}

/// Build pragma graph from the DSL source.
pub fn build_pragma_graph() -> Result<Dag<PragmaGraphOp>, BuilderError> {
    build_pragma_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pragma_graph_from_dsl() {
        let dag = build_pragma_graph().expect("pragma DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
