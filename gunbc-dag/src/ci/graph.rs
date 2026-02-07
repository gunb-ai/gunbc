//! Graph builder for the CI tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! # Transport Pattern (following MakegenGraphOp)
//!
//! This module follows the "every node is pure" principle:
//! - `CIGraphOp` is a union of pure CI ops, primitives, transport, and CLI tool ops
//! - I/O happens through explicit `TransportOps::Execute` nodes and self-acquiring CLI tool nodes
//! - DryRun can intercept transport nodes and env tool acquisition
//!
//! # Pipeline Structure
//!
//! ```text
//! SetupDeps Stage:
//!   PrepareFileExists(deps.toml) -> Execute -> ParseDepsExists
//!
//! Prep Stage:
//!   (Inlined Codegen DAG) -> ParseCodegenResult
//!   -> PrepareTestgenCommand -> Execute -> ParseTestgenResult
//!
//! Build Stage:
//!   PrepareBuildCommand -> Execute -> ParseBuildResult
//!
//! Test Stage (parallel with Lint):
//!   PrepareTestCommand -> Execute -> ParseTestResult
//!
//! Lint Stage:
//!   PrepareClippyLint -> ClippyLint (self-acquiring) -> ParseClippyLint
//!
//! Guardrails Stage:
//!   PrepareGuardrailCheck -> Execute -> ParseGuardrailResult
//!
//! Report:
//!   Report (pure)
//! ```

use crate::ci::ops::CIOp;
use crate::codegen::{build_codegen_graph_with_mode, CodegenGraphOp, CodegenOp};
use crate::WorkspaceBinary;
use gunbc_lib_cloud_ops::CloudEnvStatus;
use gunbc_deps::DEFAULT_MANIFEST_FILENAME;
use gunbc_exec::{require_bool, ExecError, Executable, IntoExecResult, OutputMap};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::transport::cli::CliToolOp;
use gunbc_ir::{
    build::*,
    transport::github_actions::{
        checkout, gcp_workload_identity, rust_toolchain, ubuntu_latest, Integration, Permissions,
        WorkflowConfig,
    },
    add_skippable_transport_triplet, add_transport_triplet,
    BuilderError, Cardinality, Dag, DagBuilder, Node, NodeBody, NodeId, NodeRef, Value,
    WorkflowSignature,
};
use gunbc_lib_transport::cli::{execute_cli_tool_op, upsert_tool_with, WhichResolver};
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
/// - `Codegen(CodegenOp)` - inlined codegen DAG operations
/// - `PrepareFileExists` - embedded primitive for file existence checks (from gunbc-primitives)
/// - `Transport` - boundary for actual I/O
/// - `CliTool` - CLI tool operations (self-acquiring: check/install before run)
#[derive(Debug, Clone)]
pub enum CIGraphOp {
    /// CI-specific pure operations
    CI(CIOp),
    /// Codegen DAG operations (inlined into CI)
    Codegen(CodegenOp),
    /// Cloud env status (resource acquisition)
    CloudEnv(CloudEnvStatus),
    /// Prepare file exists check (pure - path embedded, from primitives)
    PrepareFileExists(EmbeddedFileExistsOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
    /// CLI tool operations (self-acquiring: check/install before run)
    CliTool(CliToolOp),
}

impl Executable for CIGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CIGraphOp::CI(op) => op.execute(inputs),
            CIGraphOp::Codegen(op) => op.execute(inputs),
            CIGraphOp::CloudEnv(op) => op.execute(inputs),
            CIGraphOp::PrepareFileExists(op) => op.execute(inputs),
            CIGraphOp::Transport(op) => op.execute(inputs),
            CIGraphOp::CliTool(op) => {
                // Check if we should skip execution
                let skip = require_bool(&inputs, "skip")?;

                if skip {
                    // Pass through skip flag, don't run the tool
                    return OutputMap::new().bool("skip", true).ok();
                }

                // Self-acquiring: ensure tool is installed, then run via PATH
                if let CliToolOp::Run { tool, .. } = op {
                    let _ = upsert_tool_with(tool, &WhichResolver).map_err(|e| {
                        ExecError::new(format!("Failed to acquire tool '{}': {}", tool.id, e))
                    })?;
                }
                let result = execute_cli_tool_op(op).exec_context("CLI tool error")?;

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
        // Codegen summary and stamp-write boundary outputs
        .with_output("codegen_ran", "Bool", Cardinality::ONE)
        .with_output("prep_message", "String", Cardinality::ONE)
        .with_output("response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("build_skipped", "Bool", Cardinality::ONE)
        .with_output("build_stdout", "String", Cardinality::ONE)
        .with_output("test_skipped", "Bool", Cardinality::ONE)
        // Note: test_stdout is no longer a boundary output - it's wired to report node
        .with_output("lint_skipped", "Bool", Cardinality::ONE)
        .with_output("lint_stdout", "String", Cardinality::ONE)
        .with_output("skip_reason", "String", Cardinality::ZERO_OR_ONE)
        .with_output("overall_success", "Bool", Cardinality::ONE)
        .with_output("report", "String", Cardinality::ONE)
}

// ============================================================================
// CI-Specific Workflow Configuration
// ============================================================================

/// Get the integrations used by the CI workflow.
pub fn ci_integrations() -> Vec<Integration> {
    vec![checkout(), rust_toolchain(), gcp_workload_identity()]
}

/// Get the complete workflow configuration for CI.
pub fn ci_workflow_config() -> WorkflowConfig {
    let ci_cmd = WorkspaceBinary::Ci.command();
    WorkflowConfig::new("CI", ubuntu_latest(), ci_integrations()).with_run_command(format!(
        "|\n          {ci_cmd} -- run"
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
/// Prep:      (Inlined Codegen DAG) -> ParseCodegenResult
///            -> PrepareTestgenCmd -> Execute -> ParseTestgenResult
/// Build:     PrepareBuildCommand -> Execute -> ParseBuildResult
/// Test:      PrepareTestCommand -> Execute -> ParseTestResult
/// Lint:      PrepareLintCommand -> Execute -> ParseLintResult
/// Guardrails: PrepareGuardrailCheck -> Execute -> ParseGuardrailResult
/// Report:    Report (pure)
/// ```
/// Build the CI graph with default exec mode (`ExecMode::Ensure`).
///
/// This is a convenience wrapper around [`build_ci_graph_with_mode`] that
/// avoids churn on existing callers.
pub fn build_ci_graph() -> Result<Dag<CIGraphOp>, BuilderError> {
    build_ci_graph_with_mode(ExecMode::Ensure)
}

/// Build the CI graph with the specified execution mode.
///
/// The `mode` parameter is embedded into the inlined codegen DAG, eliminating
/// the need for the `GUNBC_EXEC_MODE` environment variable.
pub fn build_ci_graph_with_mode(mode: ExecMode) -> Result<Dag<CIGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    let cloud_env_status = builder
        .add_root_node(Node::opaque(
            "cloud_env_status",
            vec![],
            vec![port("status", "String")],
            CIGraphOp::CloudEnv(CloudEnvStatus::new()),
        ))
        .expect("cloud_env_status node");

    // ========================================================================
    // SetupDeps Stage: Check if deps.toml exists
    // ========================================================================

    let _deps_exists = add_transport_triplet(
        &mut builder,
        "deps_exists",
        vec![],
        vec![
            port("deps_exists", "Bool"),
            port("deps_checked", "Bool"),
            port("deps_installed", "Int"),
            port("message", "String"),
        ],
        CIGraphOp::PrepareFileExists(EmbeddedFileExistsOp::new(DEFAULT_MANIFEST_FILENAME)),
        CIGraphOp::CI(CIOp::ParseDepsExists),
        CIGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    // ========================================================================
    // Prep Stage: Inline Codegen DAG
    // ========================================================================

    let codegen_nodes = inline_codegen_dag(&mut builder, mode)?;
    let parse_codegen_result = codegen_nodes
        .get(&NodeId::from("parse_codegen_result"))
        .expect("codegen DAG should include parse_codegen_result");

    // ========================================================================
    // Bootstrap Stage
    // ========================================================================

    let bootstrap = add_skippable_transport_triplet(
        &mut builder,
        "bootstrap",
        vec![port("prep_success", "Bool")],
        vec![
            port("bootstrap_success", "Bool"),
            port("bootstrap_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareBootstrapCommand),
        CIGraphOp::CI(CIOp::ParseBootstrapResult),
        CIGraphOp::Transport(TransportOps::Execute),
        parse_codegen_result,
    )?;

    // ========================================================================
    // Pragma Stage
    // ========================================================================

    let pragma = add_skippable_transport_triplet(
        &mut builder,
        "pragma",
        vec![port("prep_success", "Bool")],
        vec![port("pragma_success", "Bool"), port("pragma_stderr", "String")],
        CIGraphOp::CI(CIOp::PreparePragmaCommand),
        CIGraphOp::CI(CIOp::ParsePragmaResult),
        CIGraphOp::Transport(TransportOps::Execute),
        parse_codegen_result,
    )?;

    // ========================================================================
    // Testgen Stage
    // ========================================================================

    let testgen = add_skippable_transport_triplet(
        &mut builder,
        "testgen",
        vec![port("prep_success", "Bool")],
        vec![port("testgen_success", "Bool"), port("testgen_stderr", "String")],
        CIGraphOp::CI(CIOp::PrepareTestgenCommand),
        CIGraphOp::CI(CIOp::ParseTestgenResult),
        CIGraphOp::Transport(TransportOps::Execute),
        parse_codegen_result,
    )?;

    // ========================================================================
    // Build Stage
    // ========================================================================

    let build = add_skippable_transport_triplet(
        &mut builder,
        "build",
        vec![port("prep_success", "Bool"), port("testgen_success", "Bool")],
        vec![
            port("build_success", "Bool"),
            port("build_skipped", "Bool"),
            port("build_stdout", "String"),
            port("build_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareBuildCommand),
        CIGraphOp::CI(CIOp::ParseBuildResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &testgen.parse,
    )?;

    // ========================================================================
    // Test Stage (parallel with Lint after build)
    // ========================================================================

    let test = add_skippable_transport_triplet(
        &mut builder,
        "test",
        vec![port("build_success", "Bool")],
        vec![
            port("test_success", "Bool"),
            port("test_skipped", "Bool"),
            port("test_stdout", "String"),
            port("test_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareTestCommand),
        CIGraphOp::CI(CIOp::ParseTestResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &build.parse,
    )?;

    // ========================================================================
    // Lint Stage (parallel with Test) - receives tool handle from env node
    // ========================================================================

    // PrepareClippyLint - pure gate for clippy execution
    let prepare_clippy_lint = builder.add_node_after(
        Node::opaque(
            "prepare_clippy_lint",
            vec![port("build_success", "Bool"), port("pragma_success", "Bool")],
            vec![port("skip", "Bool"), optional("skip_reason", "OptionalString")],
            CIGraphOp::CI(CIOp::PrepareClippyLint),
        ),
        &build.parse,
    )?;

    // ClippyLint - self-acquiring: calls upsert_tool_with() before running
    let clippy_lint = builder.add_node_after(
        Node::opaque(
            "clippy_lint",
            vec![port("skip", "Bool")],
            vec![
                optional("success", "OptionalBool"),
                optional("stdout", "OptionalString"),
                optional("stderr", "OptionalString"),
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
                optional("success", "OptionalBool"),
                optional("stdout", "OptionalString"),
                optional("stderr", "OptionalString"),
                port("skip", "Bool"),
                optional("skip_reason", "OptionalString"),
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
    // Guardrails Stage (parallel with Test/Lint after testgen)
    // ========================================================================

    let guardrail = add_skippable_transport_triplet(
        &mut builder,
        "guardrail_check",
        vec![port("testgen_success", "Bool"), port("pragma_success", "Bool")],
        vec![port("guardrail_success", "Bool"), port("guardrail_stderr", "String")],
        CIGraphOp::CI(CIOp::PrepareGuardrailCheck),
        CIGraphOp::CI(CIOp::ParseGuardrailResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &testgen.parse,
    )?;

    // ========================================================================
    // Verify Stage (after codegen, parallel with build/test/lint)
    // ========================================================================

    let verify = add_skippable_transport_triplet(
        &mut builder,
        "verify_check",
        vec![
            port("prep_success", "Bool"),
            port("bootstrap_success", "Bool"),
            port("testgen_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![port("verify_success", "Bool"), port("verify_stderr", "String")],
        CIGraphOp::CI(CIOp::PrepareVerifyCheck),
        CIGraphOp::CI(CIOp::ParseVerifyResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &bootstrap.parse,
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
                port("testgen_success", "Bool"),
                port("bootstrap_success", "Bool"),
                port("pragma_success", "Bool"),
                port("guardrail_success", "Bool"),
                port("verify_success", "Bool"),
                optional("build_stderr", "OptionalString"),
                optional("testgen_stderr", "OptionalString"),
                optional("bootstrap_stderr", "OptionalString"),
                optional("pragma_stderr", "OptionalString"),
                optional("test_stdout", "OptionalString"),
                optional("test_stderr", "OptionalString"),
                optional("lint_stderr", "OptionalString"),
                optional("guardrail_stderr", "OptionalString"),
                optional("verify_stderr", "OptionalString"),
                optional("cloud_env_status", "OptionalString"),
            ],
            vec![port("overall_success", "Bool"), port("report", "String")],
            CIGraphOp::CI(CIOp::Report),
        ),
        &[&test.parse, &parse_lint, &guardrail.parse, &verify.parse],
    )?;

    // ========================================================================
    // Wire up the pipeline (only cross-triplet edges — internal edges are
    // handled by the transport triplet helpers)
    // ========================================================================

    // Codegen DAG edges are wired during inlining.

    // Bootstrap stage — codegen result feeds prepare
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        bootstrap.prepare.in_port("prep_success"),
    )?;

    // Pragma stage — codegen result feeds prepare
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        pragma.prepare.in_port("prep_success"),
    )?;

    // Testgen stage — codegen result feeds prepare
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        testgen.prepare.in_port("prep_success"),
    )?;

    // Build stage — codegen + testgen feed prepare
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        build.prepare.in_port("prep_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        build.prepare.in_port("testgen_success"),
    )?;

    // Test stage — build feeds prepare
    builder.add_edge(
        build.parse.out("build_success"),
        test.prepare.in_port("build_success"),
    )?;

    // Lint stage (parallel with test, both depend on build)
    builder.add_edge(
        build.parse.out("build_success"),
        prepare_clippy_lint.in_port("build_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        prepare_clippy_lint.in_port("pragma_success"),
    )?;
    builder.add_edge(prepare_clippy_lint.out("skip"), clippy_lint.in_port("skip"))?;
    builder.add_edge(clippy_lint.out("success"), parse_lint.in_port("success"))?;
    builder.add_edge(clippy_lint.out("stdout"), parse_lint.in_port("stdout"))?;
    builder.add_edge(clippy_lint.out("stderr"), parse_lint.in_port("stderr"))?;
    builder.add_edge(clippy_lint.out("skip"), parse_lint.in_port("skip"))?;
    builder.add_edge(
        prepare_clippy_lint.out("skip_reason"),
        parse_lint.in_port("skip_reason"),
    )?;

    // Guardrails stage — testgen feeds prepare
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        guardrail.prepare.in_port("testgen_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        guardrail.prepare.in_port("pragma_success"),
    )?;

    // Verify stage — codegen feeds prepare
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        verify.prepare.in_port("prep_success"),
    )?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        verify.prepare.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        verify.prepare.in_port("testgen_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        verify.prepare.in_port("pragma_success"),
    )?;

    // Report — success flags and stderr for failure details
    builder.add_edge(build.parse.out("build_success"), report.in_port("build_success"))?;
    builder.add_edge(test.parse.out("test_success"), report.in_port("test_success"))?;
    builder.add_edge(parse_lint.out("lint_success"), report.in_port("lint_success"))?;
    builder.add_edge(testgen.parse.out("testgen_success"), report.in_port("testgen_success"))?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        report.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        report.in_port("pragma_success"),
    )?;
    builder.add_edge(guardrail.parse.out("guardrail_success"), report.in_port("guardrail_success"))?;
    builder.add_edge(build.parse.out("build_stderr"), report.in_port("build_stderr"))?;
    builder.add_edge(testgen.parse.out("testgen_stderr"), report.in_port("testgen_stderr"))?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_stderr"),
        report.in_port("bootstrap_stderr"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_stderr"),
        report.in_port("pragma_stderr"),
    )?;
    builder.add_edge(test.parse.out("test_stdout"), report.in_port("test_stdout"))?;
    builder.add_edge(test.parse.out("test_stderr"), report.in_port("test_stderr"))?;
    builder.add_edge(parse_lint.out("lint_stderr"), report.in_port("lint_stderr"))?;
    builder.add_edge(guardrail.parse.out("guardrail_stderr"), report.in_port("guardrail_stderr"))?;
    builder.add_edge(verify.parse.out("verify_success"), report.in_port("verify_success"))?;
    builder.add_edge(verify.parse.out("verify_stderr"), report.in_port("verify_stderr"))?;
    builder.add_edge(
        cloud_env_status.out("status"),
        report.in_port("cloud_env_status"),
    )?;

    Ok(builder.build())
}

fn inline_codegen_dag(
    builder: &mut DagBuilder<CIGraphOp>,
    mode: ExecMode,
) -> Result<HashMap<NodeId, NodeRef<CIGraphOp>>, BuilderError> {
    let dag = build_codegen_graph_with_mode(mode)?;
    let mut incoming: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for edge in &dag.edges {
        incoming
            .entry(edge.to_node.clone())
            .or_default()
            .push(edge.from_node.clone());
    }

    let mut node_refs: HashMap<NodeId, NodeRef<CIGraphOp>> = HashMap::new();

    for node in &dag.nodes {
        let deps = incoming.get(&node.id).cloned().unwrap_or_default();

        let op = match &node.body {
            NodeBody::Opaque(op) => op,
            NodeBody::SubDag(_) => {
                unreachable!("codegen DAG should not contain sub-dags");
            }
        };

        let mapped = match op {
            CodegenGraphOp::Codegen(op) => CIGraphOp::Codegen(op.clone()),
            CodegenGraphOp::Transport(op) => CIGraphOp::Transport(op.clone()),
        };

        let mapped_node = Node::opaque(node.id.clone(), node.inputs.clone(), node.outputs.clone(), mapped);

        let node_ref = if deps.is_empty() {
            builder.add_root_node(mapped_node)?
        } else if deps.len() == 1 {
            let dep_ref = node_refs
                .get(&deps[0])
                .expect("codegen DAG dependency should be present");
            builder.add_node_after(mapped_node, dep_ref)?
        } else {
            let dep_refs: Vec<&NodeRef<CIGraphOp>> = deps
                .iter()
                .map(|id| {
                    node_refs
                        .get(id)
                        .expect("codegen DAG dependency should be present")
                })
                .collect();
            builder.add_node_after_all(mapped_node, &dep_refs)?
        };

        node_refs.insert(node.id.clone(), node_ref);
    }

    for edge in &dag.edges {
        let from_ref = node_refs
            .get(&edge.from_node)
            .expect("codegen DAG source node missing");
        let to_ref = node_refs
            .get(&edge.to_node)
            .expect("codegen DAG target node missing");
        builder.add_edge(
            from_ref.out(edge.from_port.clone()),
            to_ref.in_port(edge.to_port.clone()),
        )?;
    }

    Ok(node_refs)
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
                n.id.0.starts_with("execute_") || n.id.0 == "clippy_lint"
            })
            .collect();

        // Should have nodes for: deps_exists, codegen_exists, codegen, stamp_write, build, test,
        // clippy_lint, guardrail_check, verify_check
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

        // Verify it has a skip input (self-acquiring, no ToolHandle input)
        if let Some(node) = clippy_lint {
            let has_skip_input = node
                .inputs
                .iter()
                .any(|p| p.name.0 == "skip" && p.type_id.0 == "Bool");
            assert!(
                has_skip_input,
                "clippy_lint should have skip Bool input"
            );
            let has_tool_input = node
                .inputs
                .iter()
                .any(|p| p.name.0 == "tool:clippy");
            assert!(
                !has_tool_input,
                "clippy_lint should not have tool:clippy input (self-acquiring)"
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
