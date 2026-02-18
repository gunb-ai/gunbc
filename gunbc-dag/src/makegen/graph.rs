//! DSL-backed graph builder for the makegen tool.

use crate::dsl_builder::build_makegen_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Cardinality, Dag, WorkflowSignature};

/// Runtime op type for makegen graphs.
pub type MakegenGraphOp = DynOp;

/// Get the declared signature for the makegen workflow.
pub fn makegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_input("check_mode", "OptionalBool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        .with_output(
            "makegen_response",
            "TransportResponse",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "makegen_written_path",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output(
            "makegen_content",
            "OptionalString",
            Cardinality::ZERO_OR_ONE,
        )
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("fresh", "Bool", Cardinality::ONE)
        .with_output("tool_count", "Int", Cardinality::ONE)
        .with_output("tool_names", "NonEmptyStringList", Cardinality::ONE_OR_MORE)
}

/// Build makegen graph from the DSL source.
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    build_makegen_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_makegen_graph_from_dsl() {
        let dag = build_makegen_graph().expect("makegen DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
