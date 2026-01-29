//! Graph builder for the CI tool.

use crate::ops::CIOp;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the CI graph.
///
/// Pipeline:
/// ```text
/// SetupDeps -> Build -> Test  -> Report
///                   \-> Lint -/
///                              (boundary)
/// ```
pub fn build_ci_graph() -> Dag<CIOp> {
    let mut dag = Dag::new();

    // Node: SetupDeps
    dag.add_node(Node::opaque(
        "setup_deps",
        vec![],
        vec![
            port("deps_checked", "Bool"),
            port("deps_installed", "Int"),
            port("message", "String"),
        ],
        CIOp::SetupDeps,
    ));

    // Node: Build
    dag.add_node(Node::opaque(
        "build",
        vec![port("deps_checked", "Bool")],
        vec![
            port("build_success", "Bool"),
            port("build_stdout", "String"),
            port("build_stderr", "String"),
        ],
        CIOp::Build,
    ));

    // Node: Test
    dag.add_node(Node::opaque(
        "test",
        vec![port("build_success", "Bool")],
        vec![
            port("test_success", "Bool"),
            port("test_skipped", "Bool"),
            port("test_stdout", "String"),
            port("test_stderr", "String"),
        ],
        CIOp::Test,
    ));

    // Node: Lint
    dag.add_node(Node::opaque(
        "lint",
        vec![port("build_success", "Bool")],
        vec![
            port("lint_success", "Bool"),
            port("lint_skipped", "Bool"),
            port("lint_stdout", "String"),
            port("lint_stderr", "String"),
        ],
        CIOp::Lint,
    ));

    // Node: Report (BOUNDARY)
    dag.add_node(Node::opaque(
        "report",
        vec![
            port("build_success", "Bool"),
            port("test_success", "Bool"),
            port("lint_success", "Bool"),
        ],
        vec![
            port("overall_success", "Bool"),
            port("report", "String"),
        ],
        CIOp::Report,
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new("setup_deps", "deps_checked", "build", "deps_checked"));
    dag.add_edge(Edge::new("build", "build_success", "test", "build_success"));
    dag.add_edge(Edge::new("build", "build_success", "lint", "build_success"));
    dag.add_edge(Edge::new("build", "build_success", "report", "build_success"));
    dag.add_edge(Edge::new("test", "test_success", "report", "test_success"));
    dag.add_edge(Edge::new("lint", "lint_success", "report", "lint_success"));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::detect_boundaries;

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_ci_graph();
        let boundaries = detect_boundaries(&dag);

        // Report should be a boundary
        assert!(boundaries.is_boundary_node(&"report".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_ci_graph();

        assert_eq!(dag.nodes.len(), 5);
        // setup->build, build->test, build->lint, build->report, test->report, lint->report
        assert_eq!(dag.edges.len(), 6);
    }
}
