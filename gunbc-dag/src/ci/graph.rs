//! Graph builder for the CI tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! # Transport Pattern (following MakegenGraphOp)
//!
//! This module follows the "every node is pure" principle:
//! - `CIGraphOp` is a union of pure CI ops, primitives, transport, and env ops
//! - I/O happens through explicit `TransportOps::Execute` nodes and the env node
//! - DryRun can intercept transport nodes and env tool acquisition
//!
//! # Pipeline Structure
//!
//! ```text
//! SetupDeps Stage:
//!   PrepareFileExists(deps.toml) -> Execute -> ParseDepsExists
//!
//! Prep Stage:
//!   PrepareCodegenExistsCheck -> Execute -> ParseCodegenExists
//!   -> (if needed) PrepareCodegenCommand -> Execute -> ParseCodegenResult
//!
//! Build Stage:
//!   PrepareBuildCommand -> Execute -> ParseBuildResult
//!
//! Test Stage (parallel with Lint):
//!   PrepareTestCommand -> Execute -> ParseTestResult
//!
//! Lint Stage:
//!   PrepareClippyLint -> ClippyLint (uses ToolHandle from env) -> ParseClippyLint
//!
//! Report:
//!   Report (pure)
//! ```

use crate::ci::env::EnvOp;
use crate::ci::ops::CIOp;
use gunbc_deps::DEFAULT_MANIFEST_FILENAME;
use gunbc_exec::{require_bool, ExecError, Executable, IntoExecResult, OutputMap};
use gunbc_ir::transport::cli::{CliToolOp, ToolHandle};
use gunbc_ir::{
    build::*,
    transport::github_actions::{
        checkout, rust_toolchain, ubuntu_latest, Integration, Permissions, WorkflowConfig,
    },
    BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::EmbeddedFileExistsOp;
use std::collections::HashMap;

// ============================================================================
// CIGraphOp - Union type following MakegenGraphOp pattern
// ============================================================================

/// The operation type for CI graphs - a union of CI ops, primitives, and transport.
///
/// This follows the MakegenGraphOp pattern:
/// - `CI(CIOp)` - domain-specific pure operations
/// - `PrepareFileExists` - embedded primitive for file existence checks (from gunbc-primitives)
/// - `Transport` - boundary for actual I/O
/// - `CliTool` - CLI tool operations (for SubDag integration)
/// - `Env` - environment node that provides tools to downstream nodes
#[derive(Debug, Clone)]
pub enum CIGraphOp {
    /// CI-specific pure operations
    CI(CIOp),
    /// Prepare file exists check (pure - path embedded, from primitives)
    PrepareFileExists(EmbeddedFileExistsOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
    /// CLI tool operations (for SubDag integration with clippy, etc.)
    CliTool(CliToolOp),
    /// Environment node that provides tools via upsert (I/O boundary)
    Env(EnvOp),
}

impl Executable for CIGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CIGraphOp::CI(op) => op.execute(inputs),
            CIGraphOp::PrepareFileExists(op) => op.execute(inputs),
            CIGraphOp::Transport(op) => op.execute(inputs),
            CIGraphOp::Env(op) => op.execute(inputs),
            CIGraphOp::CliTool(op) => {
                // Check if we should skip execution
                let skip = require_bool(&inputs, "skip")?;

                if skip {
                    // Pass through skip flag, don't run the tool
                    return OutputMap::new().bool("skip", true).ok();
                }

                // Run the tool (prefer tool handle if provided)
                let result = if let CliToolOp::Run { tool, .. } = op {
                    let port_name = format!("tool:{}", tool.id);
                    let handle_val = inputs.get(&port_name).ok_or_else(|| {
                        ExecError::new(format!("missing required tool handle input '{port_name}'"))
                    })?;
                    let handle = ToolHandle::try_from(handle_val)
                        .map_err(|e| ExecError::new(e.to_string()))?;
                    op.execute_with_handle(&handle)
                        .exec_context("CLI tool error")?
                } else {
                    op.execute().exec_context("CLI tool error")?
                };

                // Copy tool outputs and add skip=false
                let mut out = result;
                out.insert("skip".to_string(), Value::Bool(false));
                Ok(out)
            }
        }
    }
}

// ============================================================================
// Signature
// ============================================================================

/// Get the declared signature for the ci workflow.
pub fn ci_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // No inputs (all paths are hardcoded)
        // Outputs - boundary outputs from transport nodes and report
        .with_output("deps_exists", "Bool", Cardinality::ONE)
        .with_output("deps_checked", "Bool", Cardinality::ONE)
        .with_output("deps_installed", "Int", Cardinality::ONE)
        .with_output("message", "String", Cardinality::ONE)
        // Optional prep outputs from parse_codegen_exists (only when codegen already present)
        .with_output("prep_success", "Bool", Cardinality::ZERO_OR_ONE)
        .with_output("codegen_ran", "Bool", Cardinality::ZERO_OR_ONE)
        .with_output("prep_message", "String", Cardinality::ZERO_OR_ONE)
        // Final prep outputs from parse_codegen_result
        .with_output("codegen_ran", "Bool", Cardinality::ONE)
        .with_output("prep_message", "String", Cardinality::ONE)
        .with_output("build_skipped", "Bool", Cardinality::ONE)
        .with_output("build_stdout", "String", Cardinality::ONE)
        .with_output("skip_reason", "String", Cardinality::ZERO_OR_ONE)
        .with_output("test_skipped", "Bool", Cardinality::ONE)
        // Note: test_stdout is no longer a boundary output - it's wired to report node
        .with_output("lint_skipped", "Bool", Cardinality::ONE)
        .with_output("lint_stdout", "String", Cardinality::ONE)
        .with_output("overall_success", "Bool", Cardinality::ONE)
        .with_output("report", "String", Cardinality::ONE)
}

// ============================================================================
// CI-Specific Workflow Configuration
// ============================================================================

/// Get the integrations used by the CI workflow.
pub fn ci_integrations() -> Vec<Integration> {
    vec![checkout(), rust_toolchain()]
}

/// Get the complete workflow configuration for CI.
pub fn ci_workflow_config() -> WorkflowConfig {
    let codegen_cmd = gunbc_ir::CargoInvocation::standalone("codegen").command();
    let ci_cmd = gunbc_ir::CargoInvocation::composed("ci", "dag").command();
    WorkflowConfig::new("CI", ubuntu_latest(), ci_integrations()).with_run_command(format!(
        "|\n          {codegen_cmd} -- codegen\n          {ci_cmd} -- run"
    ))
}

/// Get the required permissions for the CI workflow.
pub fn ci_workflow_permissions() -> Permissions {
    ci_workflow_config().permissions
}

// ============================================================================
// Graph Builder
// ============================================================================

/// Build the CI graph using DagBuilder with explicit transport nodes.
///
/// Every I/O operation is visible as a `TransportOps::Execute` node.
/// This enables DryRun interception of all I/O.
///
/// Pipeline:
/// ```text
/// SetupDeps: PrepareFileExists -> Execute -> ParseDepsExists
/// Prep:      PrepareCodegenExists -> Execute -> ParseCodegenExists
///            -> PrepareCodegenCmd -> Execute -> ParseCodegenResult
/// Build:     PrepareBuildCommand -> Execute -> ParseBuildResult
/// Test:      PrepareTestCommand -> Execute -> ParseTestResult
/// Lint:      PrepareLintCommand -> Execute -> ParseLintResult
/// Report:    Report (pure)
/// ```
#[allow(clippy::result_large_err)]
pub fn build_ci_graph() -> Result<Dag<CIGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Environment Node: Provides tools to downstream nodes
    // ========================================================================

    // The env node is the I/O boundary for tool acquisition.
    // It upserts (check/install) each tool and emits ToolHandles.
    // In DryRun mode, this node is intercepted with mock handles.
    let env = builder.add_root_node(Node::opaque(
        "runner_env",
        vec![],
        vec![port("tool:clippy", "ToolHandle")],
        CIGraphOp::Env(EnvOp::new(vec!["clippy"])),
    ))?;

    // ========================================================================
    // SetupDeps Stage: Check if deps.toml exists
    // ========================================================================

    // PrepareFileExists("deps.toml") - pure
    let prepare_deps_exists = builder.add_root_node(Node::opaque(
        "prepare_deps_exists",
        vec![],
        vec![port("request", "TransportRequest")],
        CIGraphOp::PrepareFileExists(EmbeddedFileExistsOp::new(DEFAULT_MANIFEST_FILENAME)),
    ))?;

    // Execute - transport boundary
    let execute_deps_exists = builder.add_node_after(
        Node::opaque(
            "execute_deps_exists",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            CIGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_deps_exists,
    )?;

    // ParseDepsExists - pure
    let parse_deps_exists = builder.add_node_after(
        Node::opaque(
            "parse_deps_exists",
            vec![port("response", "TransportResponse")],
            vec![
                port("deps_exists", "Bool"),
                port("deps_checked", "Bool"),
                port("deps_installed", "Int"),
                port("message", "String"),
            ],
            CIGraphOp::CI(CIOp::ParseDepsExists),
        ),
        &execute_deps_exists,
    )?;

    // ========================================================================
    // Prep Stage: Check codegen exists, run if needed
    // ========================================================================

    // PrepareCodegenExistsCheck - pure
    let prepare_codegen_exists = builder.add_node_after(
        Node::opaque(
            "prepare_codegen_exists",
            vec![],
            vec![port("request", "TransportRequest")],
            CIGraphOp::CI(CIOp::PrepareCodegenExistsCheck),
        ),
        &parse_deps_exists,
    )?;

    // Execute - transport boundary
    let execute_codegen_exists = builder.add_node_after(
        Node::opaque(
            "execute_codegen_exists",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            CIGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_codegen_exists,
    )?;

    // ParseCodegenExists - pure (outputs codegen_needed, maybe prep_success)
    let parse_codegen_exists = builder.add_node_after(
        Node::opaque(
            "parse_codegen_exists",
            vec![port("response", "TransportResponse")],
            vec![
                port("codegen_needed", "Bool"),
                optional("prep_success", "Bool"),
                optional("codegen_ran", "Bool"),
                optional("prep_message", "String"),
            ],
            CIGraphOp::CI(CIOp::ParseCodegenExists),
        ),
        &execute_codegen_exists,
    )?;

    // PrepareCodegenCommand - pure (may skip)
    let prepare_codegen_cmd = builder.add_node_after(
        Node::opaque(
            "prepare_codegen_cmd",
            vec![port("codegen_needed", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            CIGraphOp::CI(CIOp::PrepareCodegenCommand),
        ),
        &parse_codegen_exists,
    )?;

    // Execute codegen - transport boundary (may be skipped by downstream)
    let execute_codegen = builder.add_node_after(
        Node::opaque(
            "execute_codegen",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            CIGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_codegen_cmd,
    )?;

    // ParseCodegenResult - pure
    let parse_codegen_result = builder.add_node_after(
        Node::opaque(
            "parse_codegen_result",
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
            ],
            vec![
                port("prep_success", "Bool"),
                port("codegen_ran", "Bool"),
                port("prep_message", "String"),
            ],
            CIGraphOp::CI(CIOp::ParseCodegenResult),
        ),
        &execute_codegen,
    )?;

    // ========================================================================
    // Build Stage
    // ========================================================================

    // PrepareBuildCommand - pure
    let prepare_build = builder.add_node_after(
        Node::opaque(
            "prepare_build",
            vec![port("prep_success", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            CIGraphOp::CI(CIOp::PrepareBuildCommand),
        ),
        &parse_codegen_result,
    )?;

    // Execute build - transport boundary
    let execute_build = builder.add_node_after(
        Node::opaque(
            "execute_build",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            CIGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_build,
    )?;

    // ParseBuildResult - pure
    let parse_build = builder.add_node_after(
        Node::opaque(
            "parse_build",
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                port("build_success", "Bool"),
                port("build_skipped", "Bool"),
                port("build_stdout", "String"),
                port("build_stderr", "String"),
            ],
            CIGraphOp::CI(CIOp::ParseBuildResult),
        ),
        &execute_build,
    )?;

    // ========================================================================
    // Test Stage (parallel with Lint after build)
    // ========================================================================

    // PrepareTestCommand - pure
    let prepare_test = builder.add_node_after(
        Node::opaque(
            "prepare_test",
            vec![port("build_success", "Bool")],
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            CIGraphOp::CI(CIOp::PrepareTestCommand),
        ),
        &parse_build,
    )?;

    // Execute test - transport boundary
    let execute_test = builder.add_node_after(
        Node::opaque(
            "execute_test",
            vec![
                optional("request", "TransportRequest"),
                port("skip", "Bool"),
            ],
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            CIGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_test,
    )?;

    // ParseTestResult - pure
    let parse_test = builder.add_node_after(
        Node::opaque(
            "parse_test",
            vec![
                optional("response", "TransportResponse"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                port("test_success", "Bool"),
                port("test_skipped", "Bool"),
                port("test_stdout", "String"),
                port("test_stderr", "String"),
            ],
            CIGraphOp::CI(CIOp::ParseTestResult),
        ),
        &execute_test,
    )?;

    // ========================================================================
    // Lint Stage (parallel with Test) - receives tool handle from env node
    // ========================================================================

    // PrepareClippyLint - pure gate for clippy execution
    let prepare_clippy_lint = builder.add_node_after(
        Node::opaque(
            "prepare_clippy_lint",
            vec![port("build_success", "Bool")],
            vec![port("skip", "Bool"), optional("skip_reason", "String")],
            CIGraphOp::CI(CIOp::PrepareClippyLint),
        ),
        &parse_build,
    )?;

    // ClippyLint - runs clippy with tool handle from env node
    // The tool:clippy input comes from the runner_env node via an edge
    let clippy_lint = builder.add_node_after(
        Node::opaque(
            "clippy_lint",
            vec![
                port("skip", "Bool"),
                port("tool:clippy", "ToolHandle"), // Receives handle from env node
            ],
            vec![
                optional("success", "Bool"),
                optional("stdout", "String"),
                optional("stderr", "String"),
                port("skip", "Bool"),
            ],
            CIGraphOp::CliTool(CliToolOp::run(
                &gunbc_ir::transport::cli::CLIPPY,
                &["--all-targets", "--", "-D", "warnings"],
            )),
        ),
        &prepare_clippy_lint,
    )?;

    // ParseClippyLintResult - pure parser for clippy outputs
    let parse_lint = builder.add_node_after(
        Node::opaque(
            "parse_clippy_lint",
            vec![
                optional("success", "Bool"),
                optional("stdout", "String"),
                optional("stderr", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                port("lint_success", "Bool"),
                port("lint_skipped", "Bool"),
                port("lint_stdout", "String"),
                port("lint_stderr", "String"),
            ],
            CIGraphOp::CI(CIOp::ParseClippyLintResult),
        ),
        &clippy_lint,
    )?;

    // ========================================================================
    // Report Stage
    // ========================================================================

    let report = builder.add_node_after_all(
        Node::opaque(
            "report",
            vec![
                port("build_success", "Bool"),
                port("test_success", "Bool"),
                port("lint_success", "Bool"),
                optional("build_stderr", "String"),
                optional("test_stdout", "String"),
                optional("test_stderr", "String"),
                optional("lint_stderr", "String"),
            ],
            vec![port("overall_success", "Bool"), port("report", "String")],
            CIGraphOp::CI(CIOp::Report),
        ),
        &[&parse_test, &parse_lint],
    )?;

    // ========================================================================
    // Wire up the pipeline
    // ========================================================================

    // SetupDeps stage
    builder.add_edge(
        prepare_deps_exists.out("request"),
        execute_deps_exists.in_port("request"),
    )?;
    builder.add_edge(
        execute_deps_exists.out("response"),
        parse_deps_exists.in_port("response"),
    )?;

    // Prep stage
    builder.add_edge(
        prepare_codegen_exists.out("request"),
        execute_codegen_exists.in_port("request"),
    )?;
    builder.add_edge(
        execute_codegen_exists.out("response"),
        parse_codegen_exists.in_port("response"),
    )?;
    builder.add_edge(
        parse_codegen_exists.out("codegen_needed"),
        prepare_codegen_cmd.in_port("codegen_needed"),
    )?;
    builder.add_edge(
        prepare_codegen_cmd.out("request"),
        execute_codegen.in_port("request"),
    )?;
    builder.add_edge(
        prepare_codegen_cmd.out("skip"),
        execute_codegen.in_port("skip"),
    )?;
    builder.add_edge(
        execute_codegen.out("response"),
        parse_codegen_result.in_port("response"),
    )?;
    builder.add_edge(
        execute_codegen.out("skip"),
        parse_codegen_result.in_port("skip"),
    )?;

    // Build stage
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        prepare_build.in_port("prep_success"),
    )?;
    builder.add_edge(
        prepare_build.out("request"),
        execute_build.in_port("request"),
    )?;
    builder.add_edge(prepare_build.out("skip"), execute_build.in_port("skip"))?;
    builder.add_edge(
        execute_build.out("response"),
        parse_build.in_port("response"),
    )?;
    builder.add_edge(execute_build.out("skip"), parse_build.in_port("skip"))?;
    builder.add_edge(
        prepare_build.out("skip_reason"),
        parse_build.in_port("skip_reason"),
    )?;

    // Test stage
    builder.add_edge(
        parse_build.out("build_success"),
        prepare_test.in_port("build_success"),
    )?;
    builder.add_edge(prepare_test.out("request"), execute_test.in_port("request"))?;
    builder.add_edge(prepare_test.out("skip"), execute_test.in_port("skip"))?;
    builder.add_edge(execute_test.out("response"), parse_test.in_port("response"))?;
    builder.add_edge(execute_test.out("skip"), parse_test.in_port("skip"))?;
    builder.add_edge(
        prepare_test.out("skip_reason"),
        parse_test.in_port("skip_reason"),
    )?;

    // Lint stage (parallel with test, both depend on build) - uses Clippy tool
    // Tool handle flows from runner_env -> clippy_lint
    builder.add_edge(env.out("tool:clippy"), clippy_lint.in_port("tool:clippy"))?;
    builder.add_edge(
        parse_build.out("build_success"),
        prepare_clippy_lint.in_port("build_success"),
    )?;
    builder.add_edge(prepare_clippy_lint.out("skip"), clippy_lint.in_port("skip"))?;
    // Wire clippy outputs to parse node
    builder.add_edge(clippy_lint.out("success"), parse_lint.in_port("success"))?;
    builder.add_edge(clippy_lint.out("stdout"), parse_lint.in_port("stdout"))?;
    builder.add_edge(clippy_lint.out("stderr"), parse_lint.in_port("stderr"))?;
    builder.add_edge(clippy_lint.out("skip"), parse_lint.in_port("skip"))?;
    builder.add_edge(
        prepare_clippy_lint.out("skip_reason"),
        parse_lint.in_port("skip_reason"),
    )?;

    // Report - success flags and stderr for failure details
    builder.add_edge(
        parse_build.out("build_success"),
        report.in_port("build_success"),
    )?;
    builder.add_edge(
        parse_test.out("test_success"),
        report.in_port("test_success"),
    )?;
    builder.add_edge(
        parse_lint.out("lint_success"),
        report.in_port("lint_success"),
    )?;
    builder.add_edge(
        parse_build.out("build_stderr"),
        report.in_port("build_stderr"),
    )?;
    builder.add_edge(parse_test.out("test_stdout"), report.in_port("test_stdout"))?;
    builder.add_edge(parse_test.out("test_stderr"), report.in_port("test_stderr"))?;
    builder.add_edge(parse_lint.out("lint_stderr"), report.in_port("lint_stderr"))?;

    Ok(builder.build())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::detect_boundaries;
    use gunbc_ir::transport::github_actions::{PermissionLevel, PermissionScope};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_ci_graph().expect("graph should build");
        // Should have many more nodes now with explicit transport
        assert!(
            dag.nodes.len() > 6,
            "expected more nodes with explicit transport, got {}",
            dag.nodes.len()
        );
    }

    #[test]
    fn test_graph_has_transport_nodes() {
        let dag = build_ci_graph().expect("graph should build");

        // Count transport nodes (execute_* for traditional transport, clippy_lint for CLI tool)
        let transport_nodes: Vec<_> = dag
            .nodes
            .iter()
            .filter(|n| {
                n.id.0.starts_with("execute_") || n.id.0 == "clippy_lint" || n.id.0 == "runner_env"
            })
            .collect();

        // Should have nodes for: deps_exists, codegen_exists, codegen, build, test, clippy_lint, runner_env
        assert!(
            transport_nodes.len() >= 6,
            "expected at least 6 transport/tool nodes, got {}",
            transport_nodes.len()
        );
    }

    #[test]
    fn test_graph_has_clippy_lint_node() {
        let dag = build_ci_graph().expect("graph should build");

        // Find clippy_lint node
        let clippy_lint = dag.get_node(&"clippy_lint".into());
        assert!(clippy_lint.is_some(), "clippy_lint node should exist");

        // Verify it has a tool:clippy input port (receives handle from env node)
        if let Some(node) = clippy_lint {
            let has_tool_input = node
                .inputs
                .iter()
                .any(|p| p.name.0 == "tool:clippy" && p.type_id.0 == "ToolHandle");
            assert!(
                has_tool_input,
                "clippy_lint should have tool:clippy ToolHandle input"
            );
        }
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_ci_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Report should still be a boundary
        assert!(boundaries.is_boundary_node(&"report".into()));
    }

    #[test]
    fn test_transport_nodes_are_visible() {
        let dag = build_ci_graph().expect("graph should build");

        // Find execute_build node
        let execute_build = dag.get_node(&"execute_build".into());
        assert!(execute_build.is_some(), "execute_build node should exist");

        // Verify it's a Transport op
        if let Some(node) = execute_build {
            // The node should have TransportRequest as input type
            let has_request_input = node
                .inputs
                .iter()
                .any(|p| p.type_id.0 == "TransportRequest");
            assert!(
                has_request_input,
                "execute_build should have TransportRequest input"
            );
        }
    }

    #[test]
    fn test_ci_integrations() {
        let integrations = ci_integrations();
        assert_eq!(integrations.len(), 2);
        assert!(integrations.iter().any(|i| i.id == "checkout"));
        assert!(integrations.iter().any(|i| i.id == "rust-toolchain"));
    }

    #[test]
    fn test_ci_workflow_config() {
        let config = ci_workflow_config();
        assert_eq!(config.name, "CI");
        assert_eq!(config.runner.id, "ubuntu-latest");
    }

    #[test]
    fn test_ci_workflow_permissions() {
        let perms = ci_workflow_permissions();
        assert_eq!(
            perms.get(&PermissionScope::Contents),
            Some(&PermissionLevel::Read)
        );
    }

    #[test]
    fn test_ci_runner_has_required_tools() {
        let config = ci_workflow_config();
        assert!(config.runner.has_tool("cargo"));
        assert!(config.runner.has_tool("git"));
    }
}
