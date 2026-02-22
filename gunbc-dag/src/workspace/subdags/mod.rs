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
pub mod infra;
pub mod languages;
pub mod makegen;
pub mod pragma;
pub mod testgen;

use crate::workspace::WorkspaceOp;
use crate::WorkspaceBinary;
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};
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
    let tool_names = discover_dsl_tool_names()?;
    let pipeline_names = discover_dsl_pipeline_names()?;
    build_workspace_dag_from_discovery(&tool_names, &pipeline_names)
}

/// Build a workspace DAG from already-discovered tool and pipeline module names.
///
/// This entrypoint is pure and deterministic for a fixed discovery set.
pub fn build_workspace_dag_from_discovery(
    tool_names: &BTreeSet<String>,
    pipeline_names: &BTreeSet<String>,
) -> Result<Dag<WorkspaceOp>, BuilderError> {
    let mut dag = Dag::new();
    let required_tools = required_dsl_tool_modules();
    let required_pipelines = required_dsl_pipeline_modules();
    let covered_tools = covered_dsl_tool_modules();
    let covered_pipelines = covered_dsl_pipeline_modules();
    validate_required("tool", tool_names, &required_tools)?;
    validate_required("pipeline", pipeline_names, &required_pipelines)?;
    validate_coverage("tool", tool_names, &covered_tools)?;
    validate_coverage("pipeline", pipeline_names, &covered_pipelines)?;
    add_discovered_tool_subdags(&mut dag, tool_names)?;
    add_discovered_pipeline_subdags(&mut dag, pipeline_names)?;

    // Language subdags are repo-level orchestration and are always present.
    dag.add_node(languages::build_languages_subdag());
    Ok(dag)
}

fn discover_dsl_tool_names() -> Result<BTreeSet<String>, BuilderError> {
    discover_dsl_module_names(dsl_tools_root()?, "tool")
}

fn discover_dsl_pipeline_names() -> Result<BTreeSet<String>, BuilderError> {
    discover_dsl_module_names(dsl_pipelines_root()?, "pipeline")
}

#[allow(clippy::disallowed_methods)] // Build-time DSL module discovery (not runtime I/O)
fn discover_dsl_module_names(
    root: PathBuf,
    module_kind: &str,
) -> Result<BTreeSet<String>, BuilderError> {
    let entries = fs::read_dir(&root).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to read DSL {module_kind} discovery root {}: {error}",
            root.display(),
        ))
    })?;
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to read entry in DSL {module_kind} discovery root {}: {error}",
                root.display(),
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
                    "failed to parse UTF-8 {module_kind} module stem for {}",
                    path.display(),
                ))
            })?;
        names.insert(stem.to_string());
    }
    Ok(names)
}

fn dsl_tools_root() -> Result<PathBuf, BuilderError> {
    Ok(workspace_layout()?.dsl_tools_root())
}

fn dsl_pipelines_root() -> Result<PathBuf, BuilderError> {
    Ok(workspace_layout()?.dsl_pipelines_root())
}

fn workspace_layout() -> Result<WorkspaceLayout, BuilderError> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to resolve workspace layout for subdag DSL discovery: {error}"
            ))
        })
}

fn validate_required(
    kind: &str,
    actual: &BTreeSet<String>,
    required: &BTreeSet<String>,
) -> Result<(), BuilderError> {
    let missing: Vec<&String> = required.difference(actual).collect();
    if missing.is_empty() {
        return Ok(());
    }
    let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
    Err(BuilderError::InternalInvariant(format!(
        "missing required DSL {kind} modules for workspace DAG: {}",
        names.join(", ")
    )))
}

fn validate_coverage(
    kind: &str,
    actual: &BTreeSet<String>,
    covered: &BTreeSet<String>,
) -> Result<(), BuilderError> {
    let unknown: Vec<&String> = actual.difference(covered).collect();
    if unknown.is_empty() {
        return Ok(());
    }
    let names: Vec<&str> = unknown.iter().map(|s| s.as_str()).collect();
    Err(BuilderError::InternalInvariant(format!(
        "unmapped DSL {kind} modules in workspace DAG discovery: {} (add mapping in workspace/subdags or explicit exclusion)",
        names.join(", ")
    )))
}

fn required_dsl_tool_modules() -> BTreeSet<String> {
    // Keep this list colocated with add_discovered_tool_subdags().
    // External tool crates with workspace DSL modules live here.
    let mut required: BTreeSet<String> = ["clippy", "dag_viz", "deps", "gist", "review"]
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    required.extend(
        WorkspaceBinary::all()
            .iter()
            .copied()
            .filter(|binary| binary.is_dsl_tool_module())
            .map(|binary| binary.tool_name().to_string()),
    );
    required
}

fn required_dsl_pipeline_modules() -> BTreeSet<String> {
    WorkspaceBinary::all()
        .iter()
        .copied()
        .filter(|binary| binary.is_dsl_pipeline_module())
        .map(|binary| binary.tool_name().to_string())
        .collect()
}

fn covered_dsl_tool_modules() -> BTreeSet<String> {
    let mut covered = required_dsl_tool_modules();
    covered.extend(
        intentionally_unmapped_dsl_tool_modules()
            .into_iter()
            .map(|name| name.to_string()),
    );
    covered
}

fn covered_dsl_pipeline_modules() -> BTreeSet<String> {
    let mut covered = required_dsl_pipeline_modules();
    covered.extend(
        intentionally_unmapped_dsl_pipeline_modules()
            .into_iter()
            .map(|name| name.to_string()),
    );
    covered
}

fn intentionally_unmapped_dsl_tool_modules() -> BTreeSet<&'static str> {
    let mut set = BTreeSet::new();
    set.insert("design");
    set
}

fn intentionally_unmapped_dsl_pipeline_modules() -> BTreeSet<&'static str> {
    // Pipeline modules intentionally excluded from workspace DAG composition until
    // compiled profile binding + runtime execution are wired end-to-end.
    ["reconciler", "sdlc"].into_iter().collect()
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
    if tool_names.contains("infra") {
        dag.add_node(infra::build_infra_subdag()?);
    }
    if tool_names.contains("pragma") {
        dag.add_node(pragma::build_pragma_subdag()?);
    }
    if tool_names.contains("testgen") {
        dag.add_node(testgen::build_testgen_subdag()?);
    }
    Ok(())
}

fn add_discovered_pipeline_subdags(
    dag: &mut Dag<WorkspaceOp>,
    pipeline_names: &BTreeSet<String>,
) -> Result<(), BuilderError> {
    if pipeline_names.contains("ci") {
        dag.add_node(ci::build_ci_subdag()?);
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
        assert!(node_ids.contains(&"infra"));
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
            "infra",
            "pragma",
            "review",
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
        assert!(node_ids.contains(&"infra"));
        assert!(node_ids.contains(&"pragma"));
        assert!(node_ids.contains(&"testgen"));
    }

    #[test]
    fn test_build_workspace_dag_from_discovery_is_pure() {
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
            "infra",
            "pragma",
            "review",
            "testgen",
        ]
        .into_iter()
        .map(|name| name.to_string())
        .collect();
        let pipeline_names: BTreeSet<String> = ["ci"]
            .into_iter()
            .map(|name| name.to_string())
            .collect();

        let dag = build_workspace_dag_from_discovery(&tool_names, &pipeline_names)
            .expect("pure workspace dag composition should succeed");
        let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(node_ids.contains(&"build"));
        assert!(node_ids.contains(&"ci"));
        assert!(node_ids.contains(&"languages"));
    }

    #[test]
    fn test_required_registered_tools_validation() {
        let mut tool_names = BTreeSet::new();
        tool_names.insert("makegen".to_string());
        tool_names.insert("deps".to_string());

        let required = required_dsl_tool_modules();
        let error = validate_required("tool", &tool_names, &required)
            .expect_err("missing required modules should fail");
        assert!(
            error
                .to_string()
                .contains("missing required DSL tool modules for workspace DAG"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn test_required_registered_pipelines_validation() {
        let pipeline_names = BTreeSet::new();
        let required = required_dsl_pipeline_modules();
        let error = validate_required("pipeline", &pipeline_names, &required)
            .expect_err("missing required pipeline modules should fail");
        assert!(
            error
                .to_string()
                .contains("missing required DSL pipeline modules for workspace DAG"),
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
        assert!(discovered.contains("infra"));
        assert!(discovered.contains("pragma"));
        assert!(discovered.contains("testgen"));
    }

    #[test]
    fn test_dsl_pipeline_discovery_contains_core_modules() {
        let discovered =
            discover_dsl_pipeline_names().expect("dsl pipeline discovery should succeed");
        assert!(discovered.contains("ci"));
        assert!(discovered.contains("reconciler"));
        assert!(discovered.contains("sdlc"));
    }

    #[test]
    fn test_dsl_tools_root_exists() {
        let tools_root = dsl_tools_root().expect("dsl tools root should resolve");
        assert!(
            tools_root.is_dir(),
            "dsl tools root should exist at {}",
            tools_root.display()
        );
    }

    #[test]
    fn test_dsl_pipelines_root_exists() {
        let pipelines_root = dsl_pipelines_root().expect("dsl pipelines root should resolve");
        assert!(
            pipelines_root.is_dir(),
            "dsl pipelines root should exist at {}",
            pipelines_root.display()
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
            "review",
            "testgen",
            "unknown_new_tool",
        ]
        .into_iter()
        .map(|name| name.to_string())
        .collect();

        let covered = required_dsl_tool_modules();
        let error = validate_coverage("tool", &tool_names, &covered)
            .expect_err("missing required registrations should fail");
        assert!(
            error.to_string().contains("unmapped DSL tool modules"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn test_registered_pipeline_coverage_validation_rejects_unknown() {
        let pipeline_names: BTreeSet<String> = ["ci", "unknown_new_pipeline"]
            .into_iter()
            .map(|name| name.to_string())
            .collect();

        let covered = required_dsl_pipeline_modules();
        let error = validate_coverage("pipeline", &pipeline_names, &covered)
            .expect_err("unknown pipeline registrations should fail");
        assert!(
            error.to_string().contains("unmapped DSL pipeline modules"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn test_registered_pipeline_subdag_mapping() {
        let pipeline_names: BTreeSet<String> = ["ci"]
            .into_iter()
            .map(|name| name.to_string())
            .collect();
        let mut dag = Dag::new();
        add_discovered_pipeline_subdags(&mut dag, &pipeline_names)
            .expect("registered pipeline mapping should build");

        let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
        assert!(node_ids.contains(&"ci"));
    }
}
