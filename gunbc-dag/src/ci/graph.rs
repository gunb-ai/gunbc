//! Graph builder for the CI tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! # Transport Pattern (following MakegenGraphOp)
//!
//! This module follows the "every node is pure" principle:
//! - `CIGraphOp` is a union of pure CI ops, primitives, and transport
//! - I/O happens through explicit `TransportOps::Execute` nodes
//! - DryRun can intercept all transport nodes
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
//!   PrepareClippyLint -> Execute -> ParseClippyLint
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
use gunbc_deps::DEFAULT_MANIFEST_FILENAME;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::{
    add_skippable_transport_triplet, add_transport_triplet,
    build::*,
    transport::github_actions::{
        checkout, gcp_workload_identity, rust_toolchain, ubuntu_latest, Integration, Permissions,
        WorkflowConfig,
    },
    BuilderError, Cardinality, Dag, DagBuilder, Node, NodeBody, NodeId, NodeRef, Value,
    WorkflowSignature,
};
use gunbc_lib_cloud_ops::CloudEnvStatus;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{EmbeddedFileExistsOp, FsEnv};
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
/// - `Transport` - boundary for actual I/O (including clippy lint)
#[derive(Debug, Clone)]
pub enum CIGraphOp {
    /// CI-specific pure operations
    CI(CIOp),
    /// Codegen DAG operations (inlined into CI)
    Codegen(CodegenOp),
    /// Cloud env status (resource acquisition)
    CloudEnv(CloudEnvStatus),
    /// Filesystem environment (resource acquisition)
    FsEnv(FsEnv),
    /// Prepare file exists check (pure - path embedded, from primitives)
    PrepareFileExists(EmbeddedFileExistsOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for CIGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CIGraphOp::CI(op) => op.execute(inputs),
            CIGraphOp::Codegen(op) => op.execute(inputs),
            CIGraphOp::CloudEnv(op) => op.execute(inputs),
            CIGraphOp::FsEnv(op) => op.execute(inputs),
            CIGraphOp::PrepareFileExists(op) => op.execute(inputs),
            CIGraphOp::Transport(op) => op.execute(inputs),
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
        .with_output("skip_reason", "OptionalString", Cardinality::ZERO_OR_ONE)
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
    WorkflowConfig::new("CI", ubuntu_latest(), ci_integrations())
        .with_run_command(format!("|\n          {ci_cmd} -- run"))
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

    let fs_resource = resource("fs", "FilesystemHandle", AccessMode::Write);

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
        vec![fs_resource.clone()],
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
    let fs_env = codegen_nodes
        .get(&NodeId::from("fs_env"))
        .expect("codegen DAG should include fs_env")
        .clone();

    // ========================================================================
    // Bootstrap Stage
    // ========================================================================

    let bootstrap = add_skippable_transport_triplet(
        &mut builder,
        "bootstrap",
        vec![port("prep_success", "Bool")],
        vec![fs_resource.clone()],
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
        vec![fs_resource.clone()],
        vec![
            port("pragma_success", "Bool"),
            port("pragma_stderr", "String"),
        ],
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
        vec![fs_resource.clone()],
        vec![
            port("testgen_success", "Bool"),
            port("testgen_stderr", "String"),
        ],
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
        vec![
            port("prep_success", "Bool"),
            port("testgen_success", "Bool"),
        ],
        vec![fs_resource.clone()],
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
        vec![fs_resource.clone()],
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
    // Lint Stage (parallel with Test) - standard transport triplet
    // ========================================================================

    let lint = add_skippable_transport_triplet(
        &mut builder,
        "clippy_lint",
        vec![
            port("build_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![fs_resource.clone()],
        vec![
            port("lint_success", "Bool"),
            port("lint_skipped", "Bool"),
            port("lint_stdout", "String"),
            port("lint_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareClippyLint),
        CIGraphOp::CI(CIOp::ParseClippyLintResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &build.parse,
    )?;

    // ========================================================================
    // Guardrails Stage (parallel with Test/Lint after testgen)
    // ========================================================================

    let guardrail = add_skippable_transport_triplet(
        &mut builder,
        "guardrail_check",
        vec![
            port("testgen_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![fs_resource.clone()],
        vec![
            port("guardrail_success", "Bool"),
            port("guardrail_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareGuardrailCheck),
        CIGraphOp::CI(CIOp::ParseGuardrailResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &testgen.parse,
    )?;

    // ========================================================================
    // Verify Stage (after codegen, split into parallel checks)
    // ========================================================================

    let verify_makegen = add_skippable_transport_triplet(
        &mut builder,
        "verify_makegen_check",
        vec![
            port("prep_success", "Bool"),
            port("bootstrap_success", "Bool"),
            port("testgen_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![fs_resource.clone()],
        vec![
            port("verify_makegen_success", "Bool"),
            port("verify_makegen_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareVerifyMakegenCheck),
        CIGraphOp::CI(CIOp::ParseVerifyMakegenResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &bootstrap.parse,
    )?;
    let verify_bootstrap = add_skippable_transport_triplet(
        &mut builder,
        "verify_bootstrap_check",
        vec![
            port("prep_success", "Bool"),
            port("bootstrap_success", "Bool"),
            port("testgen_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![fs_resource.clone()],
        vec![
            port("verify_bootstrap_success", "Bool"),
            port("verify_bootstrap_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareVerifyBootstrapCheck),
        CIGraphOp::CI(CIOp::ParseVerifyBootstrapResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &bootstrap.parse,
    )?;
    let verify_testgen = add_skippable_transport_triplet(
        &mut builder,
        "verify_testgen_check",
        vec![
            port("prep_success", "Bool"),
            port("bootstrap_success", "Bool"),
            port("testgen_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![fs_resource.clone()],
        vec![
            port("verify_testgen_success", "Bool"),
            port("verify_testgen_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareVerifyTestgenCheck),
        CIGraphOp::CI(CIOp::ParseVerifyTestgenResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &bootstrap.parse,
    )?;
    let verify_pragma = add_skippable_transport_triplet(
        &mut builder,
        "verify_pragma_check",
        vec![
            port("prep_success", "Bool"),
            port("bootstrap_success", "Bool"),
            port("testgen_success", "Bool"),
            port("pragma_success", "Bool"),
        ],
        vec![fs_resource.clone()],
        vec![
            port("verify_pragma_success", "Bool"),
            port("verify_pragma_stderr", "String"),
        ],
        CIGraphOp::CI(CIOp::PrepareVerifyPragmaCheck),
        CIGraphOp::CI(CIOp::ParseVerifyPragmaResult),
        CIGraphOp::Transport(TransportOps::Execute),
        &bootstrap.parse,
    )?;
    let verify = builder.add_node_after_all(
        Node::opaque(
            "aggregate_verify_results",
            vec![
                port("verify_makegen_success", "Bool"),
                port("verify_makegen_stderr", "String"),
                port("verify_bootstrap_success", "Bool"),
                port("verify_bootstrap_stderr", "String"),
                port("verify_testgen_success", "Bool"),
                port("verify_testgen_stderr", "String"),
                port("verify_pragma_success", "Bool"),
                port("verify_pragma_stderr", "String"),
            ],
            vec![
                port("verify_success", "Bool"),
                port("verify_stderr", "String"),
            ],
            CIGraphOp::CI(CIOp::AggregateVerifyResults),
        ),
        &[
            &verify_makegen.parse,
            &verify_bootstrap.parse,
            &verify_testgen.parse,
            &verify_pragma.parse,
        ],
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
        &[&test.parse, &lint.parse, &guardrail.parse, &verify],
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
        lint.prepare.in_port("build_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        lint.prepare.in_port("pragma_success"),
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
        verify_makegen.prepare.in_port("prep_success"),
    )?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        verify_makegen.prepare.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        verify_makegen.prepare.in_port("testgen_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        verify_makegen.prepare.in_port("pragma_success"),
    )?;
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        verify_bootstrap.prepare.in_port("prep_success"),
    )?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        verify_bootstrap.prepare.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        verify_bootstrap.prepare.in_port("testgen_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        verify_bootstrap.prepare.in_port("pragma_success"),
    )?;
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        verify_testgen.prepare.in_port("prep_success"),
    )?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        verify_testgen.prepare.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        verify_testgen.prepare.in_port("testgen_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        verify_testgen.prepare.in_port("pragma_success"),
    )?;
    builder.add_edge(
        parse_codegen_result.out("prep_success"),
        verify_pragma.prepare.in_port("prep_success"),
    )?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        verify_pragma.prepare.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        verify_pragma.prepare.in_port("testgen_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        verify_pragma.prepare.in_port("pragma_success"),
    )?;
    builder.add_edge(
        verify_makegen.parse.out("verify_makegen_success"),
        verify.in_port("verify_makegen_success"),
    )?;
    builder.add_edge(
        verify_makegen.parse.out("verify_makegen_stderr"),
        verify.in_port("verify_makegen_stderr"),
    )?;
    builder.add_edge(
        verify_bootstrap.parse.out("verify_bootstrap_success"),
        verify.in_port("verify_bootstrap_success"),
    )?;
    builder.add_edge(
        verify_bootstrap.parse.out("verify_bootstrap_stderr"),
        verify.in_port("verify_bootstrap_stderr"),
    )?;
    builder.add_edge(
        verify_testgen.parse.out("verify_testgen_success"),
        verify.in_port("verify_testgen_success"),
    )?;
    builder.add_edge(
        verify_testgen.parse.out("verify_testgen_stderr"),
        verify.in_port("verify_testgen_stderr"),
    )?;
    builder.add_edge(
        verify_pragma.parse.out("verify_pragma_success"),
        verify.in_port("verify_pragma_success"),
    )?;
    builder.add_edge(
        verify_pragma.parse.out("verify_pragma_stderr"),
        verify.in_port("verify_pragma_stderr"),
    )?;

    // Report — success flags and stderr for failure details
    builder.add_edge(
        build.parse.out("build_success"),
        report.in_port("build_success"),
    )?;
    builder.add_edge(
        test.parse.out("test_success"),
        report.in_port("test_success"),
    )?;
    builder.add_edge(
        lint.parse.out("lint_success"),
        report.in_port("lint_success"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_success"),
        report.in_port("testgen_success"),
    )?;
    builder.add_edge(
        bootstrap.parse.out("bootstrap_success"),
        report.in_port("bootstrap_success"),
    )?;
    builder.add_edge(
        pragma.parse.out("pragma_success"),
        report.in_port("pragma_success"),
    )?;
    builder.add_edge(
        guardrail.parse.out("guardrail_success"),
        report.in_port("guardrail_success"),
    )?;
    builder.add_edge(
        build.parse.out("build_stderr"),
        report.in_port("build_stderr"),
    )?;
    builder.add_edge(
        testgen.parse.out("testgen_stderr"),
        report.in_port("testgen_stderr"),
    )?;
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
    builder.add_edge(lint.parse.out("lint_stderr"), report.in_port("lint_stderr"))?;
    builder.add_edge(
        guardrail.parse.out("guardrail_stderr"),
        report.in_port("guardrail_stderr"),
    )?;
    builder.add_edge(
        verify.out("verify_success"),
        report.in_port("verify_success"),
    )?;
    builder.add_edge(verify.out("verify_stderr"), report.in_port("verify_stderr"))?;
    builder.add_edge(
        cloud_env_status.out("status"),
        report.in_port("cloud_env_status"),
    )?;

    // Resource wiring (filesystem handle for transport nodes)
    builder.add_edge(
        fs_env.out("fs:write"),
        _deps_exists.execute.in_port("res:fs"),
    )?;
    builder.add_edge(fs_env.out("fs:write"), bootstrap.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), pragma.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), testgen.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), build.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), test.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), lint.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), guardrail.execute.in_port("res:fs"))?;
    builder.add_edge(
        fs_env.out("fs:write"),
        verify_makegen.execute.in_port("res:fs"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        verify_bootstrap.execute.in_port("res:fs"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        verify_testgen.execute.in_port("res:fs"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        verify_pragma.execute.in_port("res:fs"),
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
            CodegenGraphOp::FsEnv(op) => CIGraphOp::FsEnv(op.clone()),
            CodegenGraphOp::Transport(op) => CIGraphOp::Transport(op.clone()),
        };

        let mapped_node = Node::opaque(
            node.id.clone(),
            node.inputs.clone(),
            node.outputs.clone(),
            mapped,
        );

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
        let expected_nodes = [
            "cloud_env_status",
            "execute_deps_exists",
            "execute_codegen",
            "execute_stamp_write",
            "execute_build",
            "execute_test",
            "execute_clippy_lint",
            "execute_guardrail_check",
            "execute_verify_makegen_check",
            "execute_verify_bootstrap_check",
            "execute_verify_testgen_check",
            "execute_verify_pragma_check",
            "report",
        ];

        for node_id in expected_nodes {
            assert!(
                dag.get_node(&node_id.into()).is_some(),
                "missing node: {}",
                node_id
            );
        }
    }

    #[test]
    fn test_graph_has_transport_nodes() {
        let dag = build_ci_graph().expect("graph should build");

        for node_id in [
            "execute_deps_exists",
            "execute_codegen",
            "execute_stamp_write",
            "execute_bootstrap",
            "execute_pragma",
            "execute_testgen",
            "execute_build",
            "execute_test",
            "execute_guardrail_check",
            "execute_verify_makegen_check",
            "execute_verify_bootstrap_check",
            "execute_verify_testgen_check",
            "execute_verify_pragma_check",
        ] {
            let node = dag
                .get_node(&node_id.into())
                .unwrap_or_else(|| panic!("missing transport node: {}", node_id));
            assert!(
                matches!(node.body, NodeBody::Opaque(CIGraphOp::Transport(_))),
                "{} should be a transport node",
                node_id
            );
        }
    }

    #[test]
    fn test_graph_has_clippy_lint_triplet() {
        let dag = build_ci_graph().expect("graph should build");

        // Verify the clippy lint transport triplet exists
        for node_id in [
            "prepare_clippy_lint",
            "execute_clippy_lint",
            "parse_clippy_lint",
        ] {
            assert!(
                dag.get_node(&node_id.into()).is_some(),
                "missing clippy lint triplet node: {}",
                node_id
            );
        }

        // Verify execute node is a Transport op with TransportRequest input
        let execute_node = dag.get_node(&"execute_clippy_lint".into()).unwrap();
        assert!(
            matches!(execute_node.body, NodeBody::Opaque(CIGraphOp::Transport(_))),
            "execute_clippy_lint should be a transport node"
        );
        let has_request_input = execute_node
            .inputs
            .iter()
            .any(|p| p.type_id.0 == "TransportRequest");
        assert!(
            has_request_input,
            "execute_clippy_lint should have TransportRequest input"
        );
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
        assert_eq!(integrations.len(), 3);
        assert!(integrations.iter().any(|i| i.id == "checkout"));
        assert!(integrations.iter().any(|i| i.id == "rust-toolchain"));
        assert!(integrations.iter().any(|i| i.id == "gcp-wif"));
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
