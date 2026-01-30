//! Graph builder for the CI tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! The CI pipeline includes a Prep stage that runs codegen to ensure
//! all generated code exists before building and testing. This is the
//! "fractal unwind" pattern - CI unwinds all DAGs before executing.
//!
//! # GitHub Actions Integration
//!
//! This module provides CI-specific workflow configuration via the
//! [`ci_workflow_config`] function. The underlying [`WorkflowConfig`]
//! type lives in `github_actions` module alongside other GitHub Actions specs.

use crate::ops::CIOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, WorkflowSignature,
    transport::cli,
    transport::github_actions::{
        checkout, rust_toolchain, ubuntu_latest,
        Integration, Permissions, WorkflowConfig,
    },
};

/// Get the declared signature for the ci workflow.
pub fn ci_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // No inputs (setup_deps has no entrypoint inputs)
        // Outputs - boundary outputs from intermediate nodes and report
        .with_output("deps_installed", "Int", Cardinality::One)
        .with_output("message", "String", Cardinality::One)
        .with_output("codegen_ran", "Bool", Cardinality::One)
        .with_output("prep_message", "String", Cardinality::One)
        .with_output("build_skipped", "Bool", Cardinality::One)
        .with_output("build_stdout", "String", Cardinality::One)
        .with_output("build_stderr", "String", Cardinality::One)
        .with_output("test_skipped", "Bool", Cardinality::One)
        .with_output("test_stdout", "String", Cardinality::One)
        .with_output("test_stderr", "String", Cardinality::One)
        .with_output("lint_skipped", "Bool", Cardinality::One)
        .with_output("lint_stdout", "String", Cardinality::One)
        .with_output("lint_stderr", "String", Cardinality::One)
        .with_output("overall_success", "Bool", Cardinality::One)
        .with_output("report", "String", Cardinality::One)
}

// ============================================================================
// CI-Specific Workflow Configuration
// ============================================================================

// WorkflowConfig is now defined in github_actions.rs and re-exported above.
// The following functions provide CI-specific configuration using that type.

/// Get the integrations used by the CI workflow.
///
/// The CI workflow uses:
/// - `actions/checkout@v4` - clone repository
/// - `dtolnay/rust-toolchain@stable` - install Rust
pub fn ci_integrations() -> Vec<Integration> {
    vec![
        checkout(),
        rust_toolchain(),
    ]
}

/// Get the complete workflow configuration for CI.
///
/// This returns a typed configuration object that can be used to:
/// - Generate workflow YAML with correct permissions
/// - Validate that runner has required tools
/// - Document which actions are used
///
/// # Example
///
/// ```ignore
/// let config = ci_workflow_config();
/// 
/// // Check permissions are minimal (contents:read only)
/// assert!(config.permissions.get(&PermissionScope::Contents) == Some(&PermissionLevel::Read));
///
/// // Verify runner has cargo
/// assert!(config.runner.has_tool("cargo"));
/// ```
pub fn ci_workflow_config() -> WorkflowConfig {
    WorkflowConfig::new(
        "CI",
        ubuntu_latest(),
        ci_integrations(),
    )
    .with_run_command("|\n          cargo run -p gunbc-codegen -- codegen\n          cargo run -p gunbc-ci -- run")
}

/// Get the required permissions for the CI workflow.
///
/// This is a convenience function that returns just the permissions,
/// computed from all integrations used by the workflow.
pub fn ci_workflow_permissions() -> Permissions {
    ci_workflow_config().permissions
}

/// Build the CI graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// SetupDeps -> Prep -> Build -> Test  -> Report
///                          \-> Lint -/
///                                     (boundary)
/// ```
///
/// The Prep stage runs codegen to ensure all generated code exists.
/// This is the "fractal unwind" pattern - CI unwinds all DAGs before executing.
#[allow(clippy::result_large_err)]
pub fn build_ci_graph() -> Result<Dag<CIOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: SetupDeps - generation 0
    let setup_deps = builder.add_root_node(Node::opaque(
        "setup_deps",
        vec![],
        vec![
            port("deps_checked", "Bool"),
            port("deps_installed", "Int"),
            port("message", "String"),
        ],
        CIOp::SetupDeps,
    ))?;

    // Node: Prep - generation 1 (codegen/unwind)
    let prep = builder.add_node_after(
        Node::opaque(
            "prep",
            vec![port("deps_checked", "Bool")],
            vec![
                port("prep_success", "Bool"),
                port("codegen_ran", "Bool"),
                port("prep_message", "String"),
            ],
            CIOp::Prep,
        ),
        &setup_deps,
    )?;

    // Node: Build - generation 2
    let build = builder.add_node_after(
        Node::opaque(
            "build",
            vec![port("prep_success", "Bool")],
            vec![
                port("build_success", "Bool"),
                port("build_skipped", "Bool"),
                port("build_stdout", "String"),
                port("build_stderr", "String"),
            ],
            CIOp::Build,
        ),
        &prep,
    )?;

    // Node: Test - generation 2
    let test = builder.add_node_after(
        Node::opaque(
            "test",
            vec![port("build_success", "Bool")],
            vec![
                port("test_success", "Bool"),
                port("test_skipped", "Bool"),
                port("test_stdout", "String"),
                port("test_stderr", "String"),
            ],
            CIOp::Test,
        ),
        &build,
    )?;

    // Node: Lint - generation 2 (parallel with test)
    // Uses .requires() to declare clippy dependency - framework handles acquisition
    let lint = builder.add_node_after(
        Node::opaque(
            "lint",
            vec![port("build_success", "Bool")],
            vec![
                port("lint_success", "Bool"),
                port("lint_skipped", "Bool"),
                port("lint_stdout", "String"),
                port("lint_stderr", "String"),
            ],
            CIOp::Lint,
        )
        .requires(&cli::CLIPPY),  // Capability-based tool acquisition
        &build,
    )?;

    // Node: Report (BOUNDARY) - generation 3
    let report = builder.add_node_after_all(
        Node::opaque(
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
        ),
        &[&test, &lint],
    )?;

    // Wire up the pipeline
    builder.add_edge(setup_deps.out("deps_checked"), prep.in_port("deps_checked"))?;
    builder.add_edge(prep.out("prep_success"), build.in_port("prep_success"))?;
    builder.add_edge(build.out("build_success"), test.in_port("build_success"))?;
    builder.add_edge(build.out("build_success"), lint.in_port("build_success"))?;
    builder.add_edge(build.out("build_success"), report.in_port("build_success"))?;
    builder.add_edge(test.out("test_success"), report.in_port("test_success"))?;
    builder.add_edge(lint.out("lint_success"), report.in_port("lint_success"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, infer_signature};
    use gunbc_ir::transport::github_actions::{PermissionLevel, PermissionScope};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_ci_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 6); // setup_deps, prep, build, test, lint, report
        assert_eq!(dag.edges.len(), 7); // setup->prep, prep->build, build->test, build->lint, build->report, test->report, lint->report
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_ci_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Report should be a boundary
        assert!(boundaries.is_boundary_node(&"report".into()));
    }

    #[test]
    fn test_graph_has_prep() {
        let dag = build_ci_graph().expect("graph should build");
        
        // Verify prep node exists
        let prep = dag.get_node(&"prep".into());
        assert!(prep.is_some(), "prep node should exist in CI graph");
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_ci_graph().expect("graph should build");

        assert_eq!(dag.nodes.len(), 6);
        // setup->prep, prep->build, build->test, build->lint, build->report, test->report, lint->report
        assert_eq!(dag.edges.len(), 7);
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_ci_graph().expect("graph should build");
        let sig = ci_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_ci_graph().expect("graph should build");
        let inferred = infer_signature(&dag);
        
        // No inputs, 15 boundary outputs (added prep outputs and build_skipped)
        assert_eq!(inferred.inputs.len(), 0);
        assert_eq!(inferred.outputs.len(), 15);
    }

    // ========================================================================
    // GitHub Actions Workflow Configuration Tests
    // ========================================================================

    #[test]
    fn test_ci_integrations() {
        let integrations = ci_integrations();
        
        // Should have checkout and rust-toolchain
        assert_eq!(integrations.len(), 2);
        assert!(integrations.iter().any(|i| i.id == "checkout"));
        assert!(integrations.iter().any(|i| i.id == "rust-toolchain"));
    }

    #[test]
    fn test_ci_workflow_config() {
        let config = ci_workflow_config();
        
        assert_eq!(config.name, "CI");
        assert_eq!(config.runner.id, "ubuntu-latest");
        assert_eq!(config.integrations.len(), 2);
    }

    #[test]
    fn test_ci_workflow_permissions() {
        let perms = ci_workflow_permissions();
        
        // Checkout requires contents:read
        assert_eq!(
            perms.get(&PermissionScope::Contents),
            Some(&PermissionLevel::Read)
        );
        
        // Should NOT have write permissions (minimal permissions)
        assert!(perms.values().all(|level| *level != PermissionLevel::Write));
    }

    #[test]
    fn test_ci_runner_has_required_tools() {
        let config = ci_workflow_config();
        
        // CI needs cargo, git (provided by runner)
        assert!(config.runner.has_tool("cargo"));
        assert!(config.runner.has_tool("git"));
        assert!(config.runner.has_tool("rustc"));
    }

    #[test]
    fn test_workflow_config_action_refs() {
        let config = ci_workflow_config();
        let refs = config.action_refs();
        
        assert!(refs.contains(&"actions/checkout@v4"));
        assert!(refs.contains(&"dtolnay/rust-toolchain@stable"));
    }

}
