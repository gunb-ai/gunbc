//! Generic DSL entrypoint builders for tool graphs.

use std::collections::{BTreeMap, HashMap};

use daglang_emit::EmbeddedData;
use gunbc_codegen::makegen::{
    registry::{ToolInfo, ToolRegistry},
    shared::render_makefile,
};
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
pub fn bootstrap_signature() -> Result<WorkflowSignature, BuilderError> {
    build_bootstrap_graph().map(|dag| infer_signature(&dag))
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
pub fn codegen_signature() -> Result<WorkflowSignature, BuilderError> {
    build_codegen_graph().map(|dag| infer_signature(&dag))
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

// ============================================================================
// Makegen graph builders + content generation
// ============================================================================

/// Runtime op type for makegen graphs.
pub type MakegenGraphOp = DynOp;

/// Get the declared signature for the makegen workflow (auto-derived from DAG).
pub fn makegen_signature() -> Result<WorkflowSignature, BuilderError> {
    build_makegen_graph().map(|dag| infer_signature(&dag))
}

/// Build makegen graph from the DSL source.
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    crate::dsl_builder::build_dsl_graph_for_entrypoint("tools/makegen.dag", Some("makegen"))
}

/// Embedded asset key for precomputed makegen content.
pub const MAKEGEN_ASSET_KEY: &str = "tools.makegen::makefile";

/// Build embedded asset map for compile-time codegen.
pub fn build_embedded_data() -> Result<HashMap<String, EmbeddedData>, String> {
    let mut data = HashMap::new();
    data.insert(MAKEGEN_ASSET_KEY.to_string(), makegen_embedded_data()?);
    Ok(data)
}

/// Embedded makegen content payload.
pub fn makegen_embedded_data() -> Result<EmbeddedData, String> {
    Ok(EmbeddedData {
        module: "tools.makegen".to_string(),
        layer1_file_path: "src/embedded_makefile.txt".to_string(),
        layer2_ident: "makegen_content".to_string(),
        content: compute_makegen_content()?,
    })
}

/// Compute makegen content by rendering from discovered tools.
pub fn compute_makegen_content() -> Result<String, String> {
    let registry = default_registry_enriched()?;
    render_makefile(&registry).map_err(|err| err.to_string())
}

// ============================================================================
// Registry enrichment (stays in gunbc-dag due to gunbc_testgen_registry dep)
// ============================================================================

/// Build the default registry enriched with live-secret requirements.
///
/// Wraps `ToolRegistry::default_registry()` with testgen-registry enrichment
/// that can't live in `gunbc-codegen` due to dependency cycles.
pub fn default_registry_enriched() -> Result<ToolRegistry, String> {
    let mut registry = ToolRegistry::default_registry()?;
    enrich_live_secrets(&mut registry.tools);
    registry
        .tools
        .sort_by(|a, b| a.short_name.cmp(&b.short_name));
    Ok(registry)
}

/// Enrich tool entries with live-secret requirements from DagSpec registrations.
fn enrich_live_secrets(tools: &mut [ToolInfo]) {
    let mut secrets_by_tool: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for spec in gunbc_testgen_registry::iter_dag_specs() {
        if let Some(tool_name) = spec.meta.tool_name {
            let entry = secrets_by_tool.entry(tool_name).or_default();
            if let Some(required) = spec.testgen.live_required {
                for secret in required {
                    let s = secret.to_string();
                    if !entry.contains(&s) {
                        entry.push(s);
                    }
                }
            }
            if let Some(groups) = spec.testgen.live_required_any_of {
                for group in groups {
                    for secret in *group {
                        let s = secret.to_string();
                        if !entry.contains(&s) {
                            entry.push(s);
                        }
                    }
                }
            }
        }
    }

    for tool in tools.iter_mut() {
        if let Some(secrets) = secrets_by_tool.remove(tool.short_name.as_str()) {
            tool.live_secrets = secrets;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enriched_registry_derives_tools_from_dsl() {
        let registry = default_registry_enriched().expect("enriched registry should succeed");
        assert!(registry.tools.iter().any(|tool| tool.short_name == "deps"));
        assert!(registry
            .tools
            .iter()
            .any(|tool| tool.short_name == "makegen"));
        assert!(registry
            .tools
            .iter()
            .any(|tool| tool.short_name == "pragma"));
    }

    #[test]
    fn enriched_registry_has_unique_short_names() {
        let registry = default_registry_enriched().expect("enriched registry should succeed");
        let mut seen = std::collections::BTreeSet::new();
        for tool in &registry.tools {
            assert!(
                seen.insert(tool.short_name.clone()),
                "duplicate tool short_name in makegen registry: {}",
                tool.short_name
            );
        }
    }
}
