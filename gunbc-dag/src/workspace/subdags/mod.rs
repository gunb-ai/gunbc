//! SubDag builders for each tool.
//!
//! Each tool has a `build_*_subdag()` function that returns `Node<WorkspaceOp>`,
//! enabling fractal composition into the Workspace DAG.

pub mod bootstrap;
pub mod build;
pub mod ci;
pub mod clippy;
pub mod codegen;
pub mod dag_viz;
pub mod deps;
pub mod docgen;
pub mod gist;
pub mod languages;
pub mod makegen;
pub mod pragma;
pub mod testgen;

use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Dag};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Build the Workspace DAG containing all tool and language SubDags.
///
/// This is the root DAG that composes all functionality in the workspace
/// using the fractal SubDag pattern.
///
/// # Structure
///
/// ```text
/// Workspace DAG
/// ├── makegen SubDag (Makefile generation)
/// ├── languages SubDag
/// │   ├── rust
/// │   ├── makefile
/// │   ├── gitignore
/// │   └── ...
/// └── (more tools as they're migrated)
/// ```
pub fn build_workspace_dag() -> Result<Dag<WorkspaceOp>, BuilderError> {
    let mut dag = Dag::new();

    // Hard-cut discovery: workspace composition is sourced directly from dsl/tools.
    let tool_names = discover_dsl_tool_names()?;
    validate_required_dsl_tools(&tool_names)?;
    validate_dsl_tool_coverage(&tool_names)?;
    add_discovered_tool_subdags(&mut dag, &tool_names)?;

    // CI and language subdags are repo-level orchestration and are always present.
    dag.add_node(ci::build_ci_subdag());
    dag.add_node(languages::build_languages_subdag());
    Ok(dag)
}

fn discover_dsl_tool_names() -> Result<BTreeSet<String>, BuilderError> {
    let root = dsl_tools_root();
    let entries = fs::read_dir(&root).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to read DSL tool discovery root {}: {error}",
            root.display()
        ))
    })?;
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to read entry in DSL tool discovery root {}: {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("dag") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                BuilderError::InternalInvariant(format!(
                    "failed to parse UTF-8 module stem for {}",
                    path.display()
                ))
            })?;
        names.insert(stem.to_string());
    }
    Ok(names)
}

fn dsl_tools_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl/tools")
}

fn validate_required_dsl_tools(tool_names: &BTreeSet<String>) -> Result<(), BuilderError> {
    const REQUIRED: &[&str] = &[
        "makegen",
        "clippy",
        "deps",
        "bootstrap",
        "gist",
        "build",
        "codegen",
        "dag_viz",
        "docgen",
        "pragma",
        "testgen",
    ];
    let missing: Vec<&str> = REQUIRED
        .iter()
        .copied()
        .filter(|name| !tool_names.contains(*name))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(BuilderError::InternalInvariant(format!(
        "missing required DSL tool modules for workspace DAG: {}",
        missing.join(", ")
    )))
}

fn validate_dsl_tool_coverage(tool_names: &BTreeSet<String>) -> Result<(), BuilderError> {
    const COVERED: &[&str] = &[
        "makegen",
        "clippy",
        "deps",
        "bootstrap",
        "gist",
        "build",
        "codegen",
        "dag_viz",
        "docgen",
        "pragma",
        "testgen",
    ];
    const EXCLUDED: &[&str] = &[];

    let unknown: Vec<String> = tool_names
        .iter()
        .filter(|name| !COVERED.contains(&name.as_str()) && !EXCLUDED.contains(&name.as_str()))
        .cloned()
        .collect();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(BuilderError::InternalInvariant(format!(
        "unmapped DSL tool modules in workspace DAG discovery: {} (add mapping in workspace/subdags or explicit exclusion)",
        unknown.join(", ")
    )))
}

fn add_discovered_tool_subdags(
    dag: &mut Dag<WorkspaceOp>,
    tool_names: &BTreeSet<String>,
) -> Result<(), BuilderError> {
    if tool_names.contains("build") {
        dag.add_node(build::build_build_subdag()?);
    }
    if tool_names.contains("makegen") {
        dag.add_node(makegen::build_makegen_subdag());
    }
    if tool_names.contains("clippy") {
        dag.add_node(clippy::build_clippy_lint_all_subdag());
    }
    if tool_names.contains("deps") {
        dag.add_node(deps::build_deps_install_subdag()?);
        dag.add_node(deps::build_deps_generate_subdag()?);
    }
    if tool_names.contains("bootstrap") {
        dag.add_node(bootstrap::build_bootstrap_subdag()?);
    }
    if tool_names.contains("codegen") {
        dag.add_node(codegen::build_codegen_subdag()?);
    }
    if tool_names.contains("dag_viz") {
        dag.add_node(dag_viz::build_dag_viz_subdag()?);
    }
    if tool_names.contains("docgen") {
        dag.add_node(docgen::build_docgen_subdag()?);
    }
    if tool_names.contains("gist") {
        dag.add_node(gist::build_gist_rust_subdag());
    }
    if tool_names.contains("pragma") {
        dag.add_node(pragma::build_pragma_subdag()?);
    }
    if tool_names.contains("testgen") {
        dag.add_node(testgen::build_testgen_subdag()?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_workspace_dag_structure() {
        let dag = build_workspace_dag().expect("workspace DAG should build");

        // Should have all tool subdags plus languages
        let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(node_ids.contains(&"build"));
        assert!(node_ids.contains(&"makegen"));
        assert!(node_ids.contains(&"clippy"));
        assert!(node_ids.contains(&"deps_install"));
        assert!(node_ids.contains(&"deps_generate"));
        assert!(node_ids.contains(&"bootstrap"));
        assert!(node_ids.contains(&"codegen"));
        assert!(node_ids.contains(&"ci"));
        assert!(node_ids.contains(&"dag_viz"));
        assert!(node_ids.contains(&"docgen"));
        assert!(node_ids.contains(&"gist"));
        assert!(node_ids.contains(&"pragma"));
        assert!(node_ids.contains(&"testgen"));
        assert!(node_ids.contains(&"languages"));
    }

    #[test]
    fn test_workspace_dag_nodes_are_subdags() {
        let dag = build_workspace_dag().expect("workspace DAG should build");

        for node in &dag.nodes {
            assert!(node.is_subdag(), "Node {} should be a SubDag", node.id.0);
        }
    }

    #[test]
    fn test_registered_tool_subdag_mapping() {
        let tool_names: BTreeSet<String> =
            [
                "build",
                "makegen",
                "clippy",
                "deps",
                "bootstrap",
                "codegen",
                "dag_viz",
                "docgen",
                "gist",
                "pragma",
                "testgen",
            ]
                .into_iter()
                .map(|name| name.to_string())
                .collect();
        let mut dag = Dag::new();
        add_discovered_tool_subdags(&mut dag, &tool_names)
            .expect("registered mapping should build");

        let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(node_ids.contains(&"build"));
        assert!(node_ids.contains(&"makegen"));
        assert!(node_ids.contains(&"clippy"));
        assert!(node_ids.contains(&"deps_install"));
        assert!(node_ids.contains(&"deps_generate"));
        assert!(node_ids.contains(&"bootstrap"));
        assert!(node_ids.contains(&"codegen"));
        assert!(node_ids.contains(&"dag_viz"));
        assert!(node_ids.contains(&"docgen"));
        assert!(node_ids.contains(&"gist"));
        assert!(node_ids.contains(&"pragma"));
        assert!(node_ids.contains(&"testgen"));
    }

    #[test]
    fn test_required_registered_tools_validation() {
        let mut tool_names = BTreeSet::new();
        tool_names.insert("makegen".to_string());
        tool_names.insert("deps".to_string());

        let error = validate_required_dsl_tools(&tool_names)
            .expect_err("missing required modules should fail");
        assert!(
            error
                .to_string()
                .contains("missing required DSL tool modules for workspace DAG"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn test_dsl_tool_discovery_contains_core_modules() {
        let discovered = discover_dsl_tool_names().expect("dsl tool discovery should succeed");
        assert!(discovered.contains("build"));
        assert!(discovered.contains("makegen"));
        assert!(discovered.contains("clippy"));
        assert!(discovered.contains("deps"));
        assert!(discovered.contains("bootstrap"));
        assert!(discovered.contains("codegen"));
        assert!(discovered.contains("dag_viz"));
        assert!(discovered.contains("docgen"));
        assert!(discovered.contains("gist"));
        assert!(discovered.contains("pragma"));
        assert!(discovered.contains("testgen"));
    }

    #[test]
    fn test_dsl_tools_root_exists() {
        assert!(
            dsl_tools_root().is_dir(),
            "dsl tools root should exist at {}",
            dsl_tools_root().display()
        );
    }

    #[test]
    fn test_registered_tool_coverage_validation_rejects_unknown() {
        let tool_names: BTreeSet<String> = [
            "build",
            "makegen",
            "clippy",
            "deps",
            "bootstrap",
            "codegen",
            "dag_viz",
            "docgen",
            "gist",
            "pragma",
            "testgen",
            "unknown_new_tool",
        ]
        .into_iter()
        .map(|name| name.to_string())
        .collect();

        let error = validate_dsl_tool_coverage(&tool_names)
            .expect_err("missing required registrations should fail");
        assert!(
            error.to_string().contains("unmapped DSL tool modules"),
            "unexpected validation error: {error}"
        );
    }
}
