//! Tool registry — delegates to gunbc_codegen::makegen::registry.
//!
//! `enrich_live_secrets` stays here because it requires `gunbc_testgen_registry`
//! which would create a circular dependency if added to `gunbc-codegen`.

pub use gunbc_codegen::makegen::registry::{
    default_build_config, BuildCommand, BuildConfig, BuildSystem, EntrypointParam, ExtraTarget,
    ToolInfo, ToolRegistry, WorkflowKind, WorkflowSpec,
};

use std::collections::BTreeMap;

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
