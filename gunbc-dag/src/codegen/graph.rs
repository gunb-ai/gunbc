//! DSL-backed graph builder for the codegen prep tool.

use crate::dsl_builder::build_codegen_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::resource::ExecMode;
use gunbc_ir::{BuilderError, Cardinality, Dag, WorkflowSignature};

/// Runtime op type for codegen graphs.
pub type CodegenGraphOp = DynOp;

/// Get the declared signature for the codegen workflow.
pub fn codegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_output("codegen_ran", "Bool", Cardinality::ONE)
        .with_output("prep_message", "String", Cardinality::ONE)
        .with_output("response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
}

/// Build the codegen graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "codegen",
    builder = "build_codegen_graph().unwrap()"
)]
pub fn build_codegen_graph() -> Result<Dag<CodegenGraphOp>, BuilderError> {
    build_codegen_graph_dsl()
}

/// Build the codegen graph with a compatibility mode parameter.
///
/// Mode-specific behavior is controlled by runtime `check_mode` inputs.
pub fn build_codegen_graph_with_mode(mode: ExecMode) -> Result<Dag<CodegenGraphOp>, BuilderError> {
    let _ = mode;
    build_codegen_graph()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_codegen_graph_from_dsl() {
        let dag = build_codegen_graph().expect("codegen DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }
}
