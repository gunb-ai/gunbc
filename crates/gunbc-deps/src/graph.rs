//! Graph builder for the deps tool.

use crate::ops::DepsOp;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the deps graph.
///
/// Pipeline:
/// ```text
/// LoadManifest -> GenerateScripts -> ExecuteInstalls
///                                          ↓
///                                     (boundary)
/// ```
///
/// # Port Cardinalities
///
/// - `manifest_path`: One (optional, defaults to "deps.toml")
/// - `dep_count`: One (scalar integer)
/// - `dep_names`: ZeroOrMore (list of dependency names, may be empty)
/// - `install_script`: One (generated script)
/// - `already_installed`, `needs_install`: ZeroOrMore (lists of dep names)
/// - `platform`: One (detected platform)
/// - `executed`: One (boolean flag)
/// - `script`: One (executed script)
///
/// # Future: UpsertBuilder Pattern
///
/// Each dependency installation could be modeled as an upsert:
/// - Check: Verify if dependency is installed
/// - Create: Install the dependency if missing
/// - Resolve: Verify installation succeeded
///
/// This would allow fine-grained control over individual dependencies
/// and better dry-run testing.
pub fn build_deps_graph() -> Dag<DepsOp> {
    let mut dag = Dag::new();

    // Node: LoadManifest
    // Input: optional manifest path
    // Output: dependency metadata
    dag.add_node(Node::opaque(
        "load_manifest",
        vec![optional("manifest_path", "String")],
        vec![
            scalar("dep_count", "Int"),
            list("dep_names", "StrList"),
            scalar("manifest_path", "String"),
        ],
        DepsOp::LoadManifest,
    ));

    // Node: GenerateScripts
    // Input: manifest path
    // Output: scripts and dependency status
    dag.add_node(Node::opaque(
        "generate_scripts",
        vec![scalar("manifest_path", "String")],
        vec![
            scalar("install_script", "String"),
            list("already_installed", "StrList"),
            list("needs_install", "StrList"),
            scalar("platform", "String"),
        ],
        DepsOp::GenerateScripts,
    ));

    // Node: ExecuteInstalls (BOUNDARY - world write)
    // Input: install script
    // Output: execution results
    dag.add_node(Node::opaque(
        "execute_installs",
        vec![scalar("install_script", "String")],
        vec![scalar("executed", "Bool"), scalar("script", "String")],
        DepsOp::ExecuteInstalls,
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new(
        "load_manifest",
        "manifest_path",
        "generate_scripts",
        "manifest_path",
    ));
    dag.add_edge(Edge::new(
        "generate_scripts",
        "install_script",
        "execute_installs",
        "install_script",
    ));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_deps_graph();
        let boundaries = detect_boundaries(&dag);

        assert!(boundaries.is_boundary_node(&"execute_installs".into()));
    }

    #[test]
    fn test_graph_has_entrypoint() {
        let dag = build_deps_graph();
        let entrypoints = detect_entrypoints(&dag);

        // manifest_path is an entrypoint
        assert!(entrypoints.is_entrypoint_port(&"load_manifest".into(), &"manifest_path".into()));
    }
}
