//! Generic DSL entrypoint builders for tool graphs.

use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for bootstrap graphs.
pub type BootstrapGraphOp = DynOp;
/// Runtime op type for build graphs.
pub type BuildGraphOp = DynOp;
/// Runtime op type for codegen graphs.
pub type CodegenGraphOp = DynOp;
/// Runtime op type for deps graphs.
pub type DepsGraphOp = DynOp;
/// Runtime op type for infra graphs.
pub type InfraGraphOp = DynOp;

/// Get the declared signature for the bootstrap workflow.
pub fn bootstrap_signature() -> WorkflowSignature {
    match build_bootstrap_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build bootstrap DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build bootstrap graph from the DSL source.
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/bootstrap.dag", Some("bootstrap"))
}

/// Get the declared signature for the build workflow.
pub fn build_signature() -> Result<WorkflowSignature, BuilderError> {
    build_build_graph().map(|dag| infer_signature(&dag))
}

/// Build build graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "build",
    builder = "crate::build_build_graph().unwrap()"
)]
pub fn build_build_graph() -> Result<Dag<BuildGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/build.dag", Some("build_all"))
}

/// Get the declared signature for the codegen workflow.
pub fn codegen_signature() -> WorkflowSignature {
    match build_codegen_graph() {
        Ok(dag) => infer_signature(&dag),
        Err(err) => {
            eprintln!("warning: failed to build codegen DAG for signature: {err}");
            WorkflowSignature::default()
        }
    }
}

/// Build codegen graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "codegen",
    builder = "crate::build_codegen_graph().unwrap()"
)]
pub fn build_codegen_graph() -> Result<Dag<CodegenGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/codegen.dag", Some("codegen"))
}

/// Build deps graph from the DSL source.
pub fn build_deps_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/deps.dag", Some("deps"))
}

/// Build infra graph from the DSL source.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "infra",
    builder = "crate::build_infra_graph().unwrap()"
)]
pub fn build_infra_graph() -> Result<Dag<InfraGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/infra.dag", Some("infra"))
}
