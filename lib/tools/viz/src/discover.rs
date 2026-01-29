//! Generic DAG discovery.
//!
//! Discovers all DAGs by iterating the tool registry from gunbc-makegen.
//! This ensures the registry is the single source of truth for what tools exist.

use crate::export::{export_dag, VizGraph};
use gunbc_makegen::registry::ToolRegistry;

/// Get the exported DAG for a tool by crate name.
/// 
/// Returns None if the tool doesn't have a known DAG builder
/// (e.g., tools that are defined but not yet implemented).
fn get_viz_graph_for_tool(crate_name: &str) -> Option<VizGraph> {
    match crate_name {
        "gunbc-gist" => {
            let dag = gunbc_gist::build_gist_graph(vec![], false).ok()?;
            Some(export_dag(&dag, crate_name))
        }
        "gunbc-buck2" => {
            let dag = gunbc_buck2::build_buck2_graph().ok()?;
            Some(export_dag(&dag, crate_name))
        }
        "gunbc-makegen" => {
            let dag = gunbc_makegen::build_makegen_graph().ok()?;
            Some(export_dag(&dag, crate_name))
        }
        "gunbc-deps" => {
            let dag = gunbc_deps::build_deps_graph().ok()?;
            Some(export_dag(&dag, crate_name))
        }
        "gunbc-ci" => {
            let dag = gunbc_ci::build_ci_graph().ok()?;
            Some(export_dag(&dag, crate_name))
        }
        "gunbc-bootstrap" => {
            let dag = gunbc_bootstrap::build_bootstrap_graph().ok()?;
            Some(export_dag(&dag, crate_name))
        }
        "gunbc-viz" => {
            let dag = crate::graph::build_viz_graph().ok()?;
            Some(export_dag(&dag, crate_name))
        }
        // NOTE: prep tool has been removed - functionality consolidated into CI
        _ => None, // Unknown tool - skip it
    }
}

/// Discover all DAGs from the tool registry.
/// 
/// Iterates the registry (single source of truth) and calls
/// the appropriate DAG builder for each tool.
pub fn discover_all_dags() -> Vec<VizGraph> {
    let registry = ToolRegistry::default_registry();
    
    registry
        .tools
        .iter()
        .filter_map(|tool| get_viz_graph_for_tool(&tool.crate_name))
        .collect()
}

/// Check if a tool has a known DAG builder.
fn has_dag_builder(crate_name: &str) -> bool {
    matches!(
        crate_name,
        "gunbc-gist"
            | "gunbc-buck2"
            | "gunbc-makegen"
            | "gunbc-deps"
            | "gunbc-ci"
            | "gunbc-bootstrap"
            | "gunbc-viz"
    )
}

/// Get a list of tools from the registry that don't have DAG builders yet.
/// Useful for identifying what needs to be implemented.
pub fn missing_dag_builders() -> Vec<String> {
    let registry = ToolRegistry::default_registry();
    
    registry
        .tools
        .iter()
        .filter(|tool| !has_dag_builder(&tool.crate_name))
        .map(|tool| tool.crate_name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_finds_all_registered_tools() {
        let registry = ToolRegistry::default_registry();
        let dags = discover_all_dags();
        
        // Should have a DAG for each tool in the registry
        // (unless some tools don't have builders yet)
        let missing = missing_dag_builders();
        let expected_count = registry.tools.len() - missing.len();
        
        assert_eq!(
            dags.len(), 
            expected_count,
            "Expected {} DAGs (registry has {} tools, {} missing builders)",
            expected_count,
            registry.tools.len(),
            missing.len()
        );
    }

    #[test]
    fn test_dag_names_match_registry() {
        let registry = ToolRegistry::default_registry();
        let dags = discover_all_dags();
        
        // Every discovered DAG should have a name matching a registry entry
        for dag in &dags {
            let in_registry = registry.tools.iter().any(|t| t.crate_name == dag.name);
            assert!(in_registry, "DAG '{}' not in registry", dag.name);
        }
    }

    #[test]
    fn test_discovered_dags_have_structure() {
        let dags = discover_all_dags();
        
        for dag in &dags {
            assert!(!dag.nodes.is_empty(), "{} has no nodes", dag.name);
            assert!(dag.meta.node_count > 0);
        }
    }

    #[test]
    fn test_no_missing_builders() {
        let missing = missing_dag_builders();
        assert!(
            missing.is_empty(),
            "Some tools are missing DAG builders: {:?}",
            missing
        );
    }
}
