//! Deterministic workflow spec builders (WF1, WF16-WF20).

use gunbc_ir::{Dag, Edge, Node, Port};

use super::capabilities::{
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
};
use super::process_registry::{
    claim_handle_type_id, default_process_unit_registry, ProcessUnitRef, ProcessUnitRegistry,
};
use super::schema::{
    required_input_contract, required_output_contract, ReportSpec, WorkflowOp, WorkflowSpec,
    WorkflowUnit,
};

fn invoke_node(
    id: &str,
    process_ref: ProcessUnitRef,
    registry: &ProcessUnitRegistry,
) -> Result<Node<WorkflowUnit>, String> {
    let spec = registry.get(&process_ref).ok_or_else(|| {
        format!(
            "missing process unit registry entry for {}::{}",
            process_ref.process_id.0, process_ref.unit_id.0
        )
    })?;

    let mut inputs = required_input_contract();
    for claim in &spec.required_claims {
        inputs.push(Port::resource(
            claim.claim_id.as_resource_name(),
            claim_handle_type_id(&claim.claim_id),
            claim.access_mode,
        ));
    }

    Ok(Node::opaque(
        id,
        inputs,
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(process_ref)),
    ))
}

fn report_node(id: &str) -> Node<WorkflowUnit> {
    Node::opaque(
        id,
        required_input_contract(),
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::Report(ReportSpec::new(id))),
    )
}

/// Build WF1 CI workflow spec.
pub fn ci_workflow_spec() -> Result<WorkflowSpec, String> {
    ci_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF1 CI workflow spec against an explicit process registry.
pub fn ci_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();
    dag.add_node(invoke_node(
        "ci.lint_upsert",
        ProcessUnitRef::new("ci", "ci.lint_upsert"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.codegen",
        ProcessUnitRef::new("ci", "ci.codegen"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.bootstrap",
        ProcessUnitRef::new("ci", "ci.bootstrap"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.pragma",
        ProcessUnitRef::new("ci", "ci.pragma"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.testgen",
        ProcessUnitRef::new("ci", "ci.testgen"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.build_compile",
        ProcessUnitRef::new("ci", "ci.build_compile"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.test_run",
        ProcessUnitRef::new("ci", "ci.test_run"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.clippy_run",
        ProcessUnitRef::new("ci", "ci.clippy_run"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.guardrails",
        ProcessUnitRef::new("ci", "ci.guardrails"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "ci.verify",
        ProcessUnitRef::new("ci", "ci.verify"),
        registry,
    )?);
    dag.add_node(report_node("ci.report"));

    dag.add_edge(Edge::control(
        "ci.lint_upsert",
        "commit",
        "ci.codegen",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.codegen",
        "commit",
        "ci.bootstrap",
        "after",
    ));
    dag.add_edge(Edge::control("ci.codegen", "commit", "ci.pragma", "after"));
    dag.add_edge(Edge::control("ci.codegen", "commit", "ci.testgen", "after"));
    dag.add_edge(Edge::control(
        "ci.codegen",
        "commit",
        "ci.build_compile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.testgen",
        "commit",
        "ci.build_compile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.pragma",
        "commit",
        "ci.guardrails",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.testgen",
        "commit",
        "ci.guardrails",
        "after",
    ));
    dag.add_edge(Edge::control("ci.pragma", "commit", "ci.verify", "after"));
    dag.add_edge(Edge::control("ci.testgen", "commit", "ci.verify", "after"));
    dag.add_edge(Edge::control(
        "ci.bootstrap",
        "commit",
        "ci.verify",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.build_compile",
        "commit",
        "ci.test_run",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.build_compile",
        "commit",
        "ci.clippy_run",
        "after",
    ));
    dag.add_edge(Edge::control("ci.test_run", "commit", "ci.report", "after"));
    dag.add_edge(Edge::control(
        "ci.clippy_run",
        "commit",
        "ci.report",
        "after",
    ));
    dag.add_edge(Edge::control(
        "ci.guardrails",
        "commit",
        "ci.report",
        "after",
    ));
    dag.add_edge(Edge::control("ci.verify", "commit", "ci.report", "after"));

    Ok(WorkflowSpec::new("ci", dag, 1))
}

/// Build WF1 test-all workflow spec.
pub fn test_all_workflow_spec() -> Result<WorkflowSpec, String> {
    test_all_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF1 test-all workflow spec against an explicit process registry.
pub fn test_all_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();
    dag.add_node(invoke_node(
        "test_all.lint_upsert",
        ProcessUnitRef::new("test_all", "test_all.lint_upsert"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "test_all.codegen",
        ProcessUnitRef::new("test_all", "test_all.codegen"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "test_all.testgen",
        ProcessUnitRef::new("test_all", "test_all.testgen"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "test_all.build_compile",
        ProcessUnitRef::new("test_all", "test_all.build_compile"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "test_all.verify_fix",
        ProcessUnitRef::new("test_all", "test_all.verify_fix"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "test_all.cargo_test_xl",
        ProcessUnitRef::new("test_all", "test_all.cargo_test_xl"),
        registry,
    )?);
    dag.add_node(report_node("test_all.report"));

    dag.add_edge(Edge::control(
        "test_all.lint_upsert",
        "commit",
        "test_all.codegen",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.lint_upsert",
        "commit",
        "test_all.testgen",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.codegen",
        "commit",
        "test_all.build_compile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.testgen",
        "commit",
        "test_all.build_compile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.codegen",
        "commit",
        "test_all.verify_fix",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.testgen",
        "commit",
        "test_all.verify_fix",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.build_compile",
        "commit",
        "test_all.cargo_test_xl",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.verify_fix",
        "commit",
        "test_all.cargo_test_xl",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.cargo_test_xl",
        "commit",
        "test_all.report",
        "after",
    ));
    dag.add_edge(Edge::control(
        "test_all.verify_fix",
        "commit",
        "test_all.report",
        "after",
    ));

    Ok(WorkflowSpec::new("test-all", dag, 1))
}

// =============================================================================
// Gist workflow spec builders (WF16/WF17/WF18)
// =============================================================================

fn add_gist_base_nodes(
    dag: &mut Dag<WorkflowUnit>,
    registry: &ProcessUnitRegistry,
) -> Result<(), String> {
    // Universal capabilities (WF14/WF15).
    dag.add_node(invoke_node(
        "gist.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);

    // Base gist units shared across all gist modes.
    dag.add_node(invoke_node(
        "gist.branch_resolution",
        ProcessUnitRef::new("gist", "gist.branch_resolution"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.credential_resolve",
        ProcessUnitRef::new("gist", "gist.credential_resolve"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.gist_create",
        ProcessUnitRef::new("gist", "gist.gist_create"),
        registry,
    )?);
    dag.add_node(report_node("gist.report"));

    // Base control flow.
    dag.add_edge(Edge::control(
        "gist.compilation_ensure",
        "commit",
        "gist.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.codegen_ensure",
        "commit",
        "gist.branch_resolution",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.codegen_ensure",
        "commit",
        "gist.credential_resolve",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.branch_resolution",
        "commit",
        "gist.gist_create",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.credential_resolve",
        "commit",
        "gist.gist_create",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.gist_create",
        "commit",
        "gist.report",
        "after",
    ));

    Ok(())
}

/// Build WF16 gist-snapshot workflow spec.
pub fn gist_snapshot_workflow_spec() -> Result<WorkflowSpec, String> {
    gist_snapshot_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF16 gist-snapshot workflow spec against an explicit process registry.
pub fn gist_snapshot_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();
    add_gist_base_nodes(&mut dag, registry)?;

    dag.add_node(invoke_node(
        "gist.list_files",
        ProcessUnitRef::new("gist", "gist.list_files"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.read_files",
        ProcessUnitRef::new("gist", "gist.read_files"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.render_snapshot",
        ProcessUnitRef::new("gist", "gist.render_snapshot"),
        registry,
    )?);

    dag.add_edge(Edge::control(
        "gist.codegen_ensure",
        "commit",
        "gist.list_files",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.list_files",
        "commit",
        "gist.read_files",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.read_files",
        "commit",
        "gist.render_snapshot",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.render_snapshot",
        "commit",
        "gist.gist_create",
        "after",
    ));

    Ok(WorkflowSpec::new("gist-snapshot", dag, 1))
}

/// Build WF17 gist-diff workflow spec.
pub fn gist_diff_workflow_spec() -> Result<WorkflowSpec, String> {
    gist_diff_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF17 gist-diff workflow spec against an explicit process registry.
pub fn gist_diff_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();
    add_gist_base_nodes(&mut dag, registry)?;

    dag.add_node(invoke_node(
        "gist.diff",
        ProcessUnitRef::new("gist", "gist.diff"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.render_diff",
        ProcessUnitRef::new("gist", "gist.render_diff"),
        registry,
    )?);

    dag.add_edge(Edge::control(
        "gist.codegen_ensure",
        "commit",
        "gist.diff",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.diff",
        "commit",
        "gist.render_diff",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.render_diff",
        "commit",
        "gist.gist_create",
        "after",
    ));

    Ok(WorkflowSpec::new("gist-diff", dag, 1))
}

/// Build WF18 gist-recent workflow spec.
pub fn gist_recent_workflow_spec() -> Result<WorkflowSpec, String> {
    gist_recent_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF18 gist-recent workflow spec against an explicit process registry.
pub fn gist_recent_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();
    add_gist_base_nodes(&mut dag, registry)?;

    dag.add_node(invoke_node(
        "gist.rev_list",
        ProcessUnitRef::new("gist", "gist.rev_list"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.diff",
        ProcessUnitRef::new("gist", "gist.diff"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist.render_diff",
        ProcessUnitRef::new("gist", "gist.render_diff"),
        registry,
    )?);

    dag.add_edge(Edge::control(
        "gist.codegen_ensure",
        "commit",
        "gist.rev_list",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.rev_list",
        "commit",
        "gist.diff",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.diff",
        "commit",
        "gist.render_diff",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.render_diff",
        "commit",
        "gist.gist_create",
        "after",
    ));

    Ok(WorkflowSpec::new("gist-recent", dag, 1))
}

// =============================================================================
// WF19: Generator workflow spec builders (bootstrap/makegen/pragma)
// =============================================================================

/// Build WF19 bootstrap workflow spec.
///
/// DAG shape (from design pack Section 5):
/// ```text
/// compilation_ensure → codegen_ensure → workspace_scan
///     → generate_makefile   → upsert_makefile   ─┐
///     → generate_gitignore  → upsert_gitignore  ─┼→ report
/// ```
pub fn bootstrap_workflow_spec() -> Result<WorkflowSpec, String> {
    bootstrap_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF19 bootstrap workflow spec against an explicit process registry.
pub fn bootstrap_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    // Universal capabilities
    dag.add_node(invoke_node(
        "bootstrap.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "bootstrap.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);

    // Workspace scan
    dag.add_node(invoke_node(
        "bootstrap.workspace_scan",
        ProcessUnitRef::new("bootstrap", "bootstrap.workspace_scan"),
        registry,
    )?);

    // Parallel generation
    dag.add_node(invoke_node(
        "bootstrap.generate_makefile",
        ProcessUnitRef::new("bootstrap", "bootstrap.generate_makefile"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "bootstrap.generate_gitignore",
        ProcessUnitRef::new("bootstrap", "bootstrap.generate_gitignore"),
        registry,
    )?);

    // Parallel upsert
    dag.add_node(invoke_node(
        "bootstrap.upsert_makefile",
        ProcessUnitRef::new("bootstrap", "bootstrap.upsert_makefile"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "bootstrap.upsert_gitignore",
        ProcessUnitRef::new("bootstrap", "bootstrap.upsert_gitignore"),
        registry,
    )?);

    dag.add_node(report_node("bootstrap.report"));

    // Edges: compilation → codegen → scan → parallel (generate → upsert) → report
    dag.add_edge(Edge::control(
        "bootstrap.compilation_ensure",
        "commit",
        "bootstrap.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "bootstrap.codegen_ensure",
        "commit",
        "bootstrap.workspace_scan",
        "after",
    ));
    // scan → parallel generation
    dag.add_edge(Edge::control(
        "bootstrap.workspace_scan",
        "commit",
        "bootstrap.generate_makefile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "bootstrap.workspace_scan",
        "commit",
        "bootstrap.generate_gitignore",
        "after",
    ));
    // generate → upsert
    dag.add_edge(Edge::control(
        "bootstrap.generate_makefile",
        "commit",
        "bootstrap.upsert_makefile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "bootstrap.generate_gitignore",
        "commit",
        "bootstrap.upsert_gitignore",
        "after",
    ));
    // upsert → report
    dag.add_edge(Edge::control(
        "bootstrap.upsert_makefile",
        "commit",
        "bootstrap.report",
        "after",
    ));
    dag.add_edge(Edge::control(
        "bootstrap.upsert_gitignore",
        "commit",
        "bootstrap.report",
        "after",
    ));

    Ok(WorkflowSpec::new("bootstrap", dag, 1))
}

/// Build WF19 makegen workflow spec.
///
/// DAG shape (from design pack Section 6):
/// ```text
/// compilation_ensure → codegen_ensure → load_registry → render_makefile → upsert_makefile → report
/// ```
pub fn makegen_workflow_spec() -> Result<WorkflowSpec, String> {
    makegen_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF19 makegen workflow spec against an explicit process registry.
pub fn makegen_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "makegen.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "makegen.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "makegen.load_registry",
        ProcessUnitRef::new("makegen", "makegen.load_registry"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "makegen.render_makefile",
        ProcessUnitRef::new("makegen", "makegen.render_makefile"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "makegen.upsert_makefile",
        ProcessUnitRef::new("makegen", "makegen.upsert_makefile"),
        registry,
    )?);
    dag.add_node(report_node("makegen.report"));

    // Linear chain
    dag.add_edge(Edge::control(
        "makegen.compilation_ensure",
        "commit",
        "makegen.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "makegen.codegen_ensure",
        "commit",
        "makegen.load_registry",
        "after",
    ));
    dag.add_edge(Edge::control(
        "makegen.load_registry",
        "commit",
        "makegen.render_makefile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "makegen.render_makefile",
        "commit",
        "makegen.upsert_makefile",
        "after",
    ));
    dag.add_edge(Edge::control(
        "makegen.upsert_makefile",
        "commit",
        "makegen.report",
        "after",
    ));

    Ok(WorkflowSpec::new("makegen", dag, 1))
}

/// Build WF19 pragma workflow spec.
///
/// DAG shape (from design pack Section 7):
/// ```text
/// compilation_ensure → codegen_ensure →
///     render_clippy    → upsert_clippy    ─┐
///     render_allowlist → upsert_allowlist ─┼→ report
///     render_policy    → upsert_policy    ─┘
/// ```
pub fn pragma_workflow_spec() -> Result<WorkflowSpec, String> {
    pragma_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF19 pragma workflow spec against an explicit process registry.
pub fn pragma_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "pragma.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "pragma.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);

    // Three parallel render+upsert chains
    dag.add_node(invoke_node(
        "pragma.render_clippy",
        ProcessUnitRef::new("pragma", "pragma.render_clippy"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "pragma.upsert_clippy",
        ProcessUnitRef::new("pragma", "pragma.upsert_clippy"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "pragma.render_allowlist",
        ProcessUnitRef::new("pragma", "pragma.render_allowlist"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "pragma.upsert_allowlist",
        ProcessUnitRef::new("pragma", "pragma.upsert_allowlist"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "pragma.render_policy",
        ProcessUnitRef::new("pragma", "pragma.render_policy"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "pragma.upsert_policy",
        ProcessUnitRef::new("pragma", "pragma.upsert_policy"),
        registry,
    )?);
    dag.add_node(report_node("pragma.report"));

    // compilation → codegen
    dag.add_edge(Edge::control(
        "pragma.compilation_ensure",
        "commit",
        "pragma.codegen_ensure",
        "after",
    ));

    // codegen → three parallel render chains
    dag.add_edge(Edge::control(
        "pragma.codegen_ensure",
        "commit",
        "pragma.render_clippy",
        "after",
    ));
    dag.add_edge(Edge::control(
        "pragma.codegen_ensure",
        "commit",
        "pragma.render_allowlist",
        "after",
    ));
    dag.add_edge(Edge::control(
        "pragma.codegen_ensure",
        "commit",
        "pragma.render_policy",
        "after",
    ));

    // render → upsert
    dag.add_edge(Edge::control(
        "pragma.render_clippy",
        "commit",
        "pragma.upsert_clippy",
        "after",
    ));
    dag.add_edge(Edge::control(
        "pragma.render_allowlist",
        "commit",
        "pragma.upsert_allowlist",
        "after",
    ));
    dag.add_edge(Edge::control(
        "pragma.render_policy",
        "commit",
        "pragma.upsert_policy",
        "after",
    ));

    // upsert → report
    dag.add_edge(Edge::control(
        "pragma.upsert_clippy",
        "commit",
        "pragma.report",
        "after",
    ));
    dag.add_edge(Edge::control(
        "pragma.upsert_allowlist",
        "commit",
        "pragma.report",
        "after",
    ));
    dag.add_edge(Edge::control(
        "pragma.upsert_policy",
        "commit",
        "pragma.report",
        "after",
    ));

    Ok(WorkflowSpec::new("pragma", dag, 1))
}

// =============================================================================
// WF20: Remaining tool workflow spec builders (deps/dag-viz/dag-snapshot)
// =============================================================================

/// Build WF20 deps workflow spec (install + generate combined).
///
/// DAG shape (from design pack Section 8):
/// ```text
/// compilation_ensure → codegen_ensure →
///     platform_env → load_manifest → generate_scripts → execute_installs ─┐
///     load_tool_registry → render_deps_toml → write_deps_toml           ─┼→ report
/// ```
pub fn deps_workflow_spec() -> Result<WorkflowSpec, String> {
    deps_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF20 deps workflow spec against an explicit process registry.
pub fn deps_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    // Universal capabilities
    dag.add_node(invoke_node(
        "deps.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "deps.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);

    // Install graph
    dag.add_node(invoke_node(
        "deps.platform_env",
        ProcessUnitRef::new("deps", "deps.platform_env"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "deps.load_manifest",
        ProcessUnitRef::new("deps", "deps.load_manifest"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "deps.generate_scripts",
        ProcessUnitRef::new("deps", "deps.generate_scripts"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "deps.execute_installs",
        ProcessUnitRef::new("deps", "deps.execute_installs"),
        registry,
    )?);

    // Generate graph
    dag.add_node(invoke_node(
        "deps.load_tool_registry",
        ProcessUnitRef::new("deps", "deps.load_tool_registry"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "deps.render_deps_toml",
        ProcessUnitRef::new("deps", "deps.render_deps_toml"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "deps.write_deps_toml",
        ProcessUnitRef::new("deps", "deps.write_deps_toml"),
        registry,
    )?);

    dag.add_node(report_node("deps.report"));

    // Universal capability chain
    dag.add_edge(Edge::control(
        "deps.compilation_ensure",
        "commit",
        "deps.codegen_ensure",
        "after",
    ));

    // Install chain: codegen → platform_env → load_manifest → generate_scripts → execute
    dag.add_edge(Edge::control(
        "deps.codegen_ensure",
        "commit",
        "deps.platform_env",
        "after",
    ));
    dag.add_edge(Edge::control(
        "deps.platform_env",
        "commit",
        "deps.load_manifest",
        "after",
    ));
    dag.add_edge(Edge::control(
        "deps.load_manifest",
        "commit",
        "deps.generate_scripts",
        "after",
    ));
    dag.add_edge(Edge::control(
        "deps.generate_scripts",
        "commit",
        "deps.execute_installs",
        "after",
    ));

    // Generate chain: codegen → load_tool_registry → render → write
    dag.add_edge(Edge::control(
        "deps.codegen_ensure",
        "commit",
        "deps.load_tool_registry",
        "after",
    ));
    dag.add_edge(Edge::control(
        "deps.load_tool_registry",
        "commit",
        "deps.render_deps_toml",
        "after",
    ));
    dag.add_edge(Edge::control(
        "deps.render_deps_toml",
        "commit",
        "deps.write_deps_toml",
        "after",
    ));

    // Both chains → report
    dag.add_edge(Edge::control(
        "deps.execute_installs",
        "commit",
        "deps.report",
        "after",
    ));
    dag.add_edge(Edge::control(
        "deps.write_deps_toml",
        "commit",
        "deps.report",
        "after",
    ));

    Ok(WorkflowSpec::new("deps", dag, 1))
}

/// Build WF20 dag-viz workflow spec (snapshot mode by default).
///
/// DAG shape (from design pack Section 9):
/// ```text
/// compilation_ensure → codegen_ensure →
///     branch_resolution ─┐
///     serialize_dag → render_viz ─┼→ gist_upload → report
///     credential_resolve ─────────┘
/// ```
pub fn dag_viz_workflow_spec() -> Result<WorkflowSpec, String> {
    dag_viz_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF20 dag-viz workflow spec against an explicit process registry.
pub fn dag_viz_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "dag_viz.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_viz.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);

    // Shared base (same WorkIdentity as gist via canonical dedup)
    dag.add_node(invoke_node(
        "dag_viz.branch_resolution",
        ProcessUnitRef::new("dag_viz", "dag_viz.branch_resolution"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_viz.credential_resolve",
        ProcessUnitRef::new("dag_viz", "dag_viz.credential_resolve"),
        registry,
    )?);

    // Viz-specific content acquisition
    dag.add_node(invoke_node(
        "dag_viz.serialize_dag",
        ProcessUnitRef::new("dag_viz", "dag_viz.serialize_dag"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_viz.render_viz",
        ProcessUnitRef::new("dag_viz", "dag_viz.render_viz"),
        registry,
    )?);

    // Network transport (volatile)
    dag.add_node(invoke_node(
        "dag_viz.gist_upload",
        ProcessUnitRef::new("dag_viz", "dag_viz.gist_upload"),
        registry,
    )?);

    dag.add_node(report_node("dag_viz.report"));

    // Universal capability chain
    dag.add_edge(Edge::control(
        "dag_viz.compilation_ensure",
        "commit",
        "dag_viz.codegen_ensure",
        "after",
    ));

    // codegen → parallel (branch_resolution, serialize_dag, credential_resolve)
    dag.add_edge(Edge::control(
        "dag_viz.codegen_ensure",
        "commit",
        "dag_viz.branch_resolution",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_viz.codegen_ensure",
        "commit",
        "dag_viz.serialize_dag",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_viz.codegen_ensure",
        "commit",
        "dag_viz.credential_resolve",
        "after",
    ));

    // serialize_dag → render_viz
    dag.add_edge(Edge::control(
        "dag_viz.serialize_dag",
        "commit",
        "dag_viz.render_viz",
        "after",
    ));

    // All three converge → gist_upload
    dag.add_edge(Edge::control(
        "dag_viz.branch_resolution",
        "commit",
        "dag_viz.gist_upload",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_viz.render_viz",
        "commit",
        "dag_viz.gist_upload",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_viz.credential_resolve",
        "commit",
        "dag_viz.gist_upload",
        "after",
    ));

    // gist_upload → report
    dag.add_edge(Edge::control(
        "dag_viz.gist_upload",
        "commit",
        "dag_viz.report",
        "after",
    ));

    Ok(WorkflowSpec::new("dag-viz", dag, 1))
}

/// Build WF20 dag-snapshot workflow spec.
///
/// DAG shape (mirrors gist-snapshot, from design pack Section 4.3):
/// ```text
/// compilation_ensure → codegen_ensure →
///     branch_resolution ─────────────┐
///     list_files → read_files → render_snapshot ─┼→ gist_upload → report
///     credential_resolve ────────────────────────┘
/// ```
pub fn dag_snapshot_workflow_spec() -> Result<WorkflowSpec, String> {
    dag_snapshot_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build WF20 dag-snapshot workflow spec against an explicit process registry.
pub fn dag_snapshot_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "dag_snapshot.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_snapshot.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);

    // Shared base
    dag.add_node(invoke_node(
        "dag_snapshot.branch_resolution",
        ProcessUnitRef::new("dag_snapshot", "dag_snapshot.branch_resolution"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_snapshot.credential_resolve",
        ProcessUnitRef::new("dag_snapshot", "dag_snapshot.credential_resolve"),
        registry,
    )?);

    // Snapshot content acquisition
    dag.add_node(invoke_node(
        "dag_snapshot.list_files",
        ProcessUnitRef::new("dag_snapshot", "dag_snapshot.list_files"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_snapshot.read_files",
        ProcessUnitRef::new("dag_snapshot", "dag_snapshot.read_files"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "dag_snapshot.render_snapshot",
        ProcessUnitRef::new("dag_snapshot", "dag_snapshot.render_snapshot"),
        registry,
    )?);

    // Network transport (volatile)
    dag.add_node(invoke_node(
        "dag_snapshot.gist_upload",
        ProcessUnitRef::new("dag_snapshot", "dag_snapshot.gist_upload"),
        registry,
    )?);

    dag.add_node(report_node("dag_snapshot.report"));

    // Universal capability chain
    dag.add_edge(Edge::control(
        "dag_snapshot.compilation_ensure",
        "commit",
        "dag_snapshot.codegen_ensure",
        "after",
    ));

    // codegen → parallel (branch_resolution, list_files, credential_resolve)
    dag.add_edge(Edge::control(
        "dag_snapshot.codegen_ensure",
        "commit",
        "dag_snapshot.branch_resolution",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_snapshot.codegen_ensure",
        "commit",
        "dag_snapshot.list_files",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_snapshot.codegen_ensure",
        "commit",
        "dag_snapshot.credential_resolve",
        "after",
    ));

    // list_files → read_files → render_snapshot
    dag.add_edge(Edge::control(
        "dag_snapshot.list_files",
        "commit",
        "dag_snapshot.read_files",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_snapshot.read_files",
        "commit",
        "dag_snapshot.render_snapshot",
        "after",
    ));

    // All converge → gist_upload
    dag.add_edge(Edge::control(
        "dag_snapshot.branch_resolution",
        "commit",
        "dag_snapshot.gist_upload",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_snapshot.render_snapshot",
        "commit",
        "dag_snapshot.gist_upload",
        "after",
    ));
    dag.add_edge(Edge::control(
        "dag_snapshot.credential_resolve",
        "commit",
        "dag_snapshot.gist_upload",
        "after",
    ));

    // gist_upload → report
    dag.add_edge(Edge::control(
        "dag_snapshot.gist_upload",
        "commit",
        "dag_snapshot.report",
        "after",
    ));

    Ok(WorkflowSpec::new("dag-snapshot", dag, 1))
}

/// Build "gist" workflow as an alias to gist-snapshot.
pub fn gist_workflow_spec() -> Result<WorkflowSpec, String> {
    gist_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build "gist" workflow against an explicit registry.
pub fn gist_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut spec = gist_snapshot_workflow_spec_with_registry(registry)?;
    spec.id = "gist".into();
    Ok(spec)
}

/// Build "dag-viz-diff" workflow as a mode alias.
pub fn dag_viz_diff_workflow_spec() -> Result<WorkflowSpec, String> {
    dag_viz_diff_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build "dag-viz-diff" workflow against an explicit registry.
pub fn dag_viz_diff_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut spec = dag_viz_workflow_spec_with_registry(registry)?;
    spec.id = "dag-viz-diff".into();
    Ok(spec)
}

/// Build "dag-viz-recent" workflow as a mode alias.
pub fn dag_viz_recent_workflow_spec() -> Result<WorkflowSpec, String> {
    dag_viz_recent_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build "dag-viz-recent" workflow against an explicit registry.
pub fn dag_viz_recent_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut spec = dag_viz_workflow_spec_with_registry(registry)?;
    spec.id = "dag-viz-recent".into();
    Ok(spec)
}

/// Build "build-all" workflow spec.
pub fn build_all_workflow_spec() -> Result<WorkflowSpec, String> {
    build_all_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build "build-all" workflow spec against an explicit registry.
pub fn build_all_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "build_all.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "build_all.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "build_all.build",
        ProcessUnitRef::new("build_all", "build_all.build"),
        registry,
    )?);
    dag.add_node(report_node("build_all.report"));

    dag.add_edge(Edge::control(
        "build_all.compilation_ensure",
        "commit",
        "build_all.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "build_all.codegen_ensure",
        "commit",
        "build_all.build",
        "after",
    ));
    dag.add_edge(Edge::control(
        "build_all.build",
        "commit",
        "build_all.report",
        "after",
    ));

    Ok(WorkflowSpec::new("build-all", dag, 1))
}

/// Build "sdlc" workflow spec.
pub fn sdlc_workflow_spec() -> Result<WorkflowSpec, String> {
    sdlc_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build "sdlc" workflow spec against an explicit registry.
pub fn sdlc_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "sdlc.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "sdlc.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "sdlc.intake",
        ProcessUnitRef::new("sdlc", "sdlc.intake"),
        registry,
    )?);
    dag.add_node(invoke_node(
        "sdlc.worker",
        ProcessUnitRef::new("sdlc", "sdlc.worker"),
        registry,
    )?);
    dag.add_node(report_node("sdlc.report"));

    dag.add_edge(Edge::control(
        "sdlc.compilation_ensure",
        "commit",
        "sdlc.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "sdlc.codegen_ensure",
        "commit",
        "sdlc.intake",
        "after",
    ));
    dag.add_edge(Edge::control("sdlc.intake", "commit", "sdlc.worker", "after"));
    dag.add_edge(Edge::control("sdlc.worker", "commit", "sdlc.report", "after"));

    Ok(WorkflowSpec::new("sdlc", dag, 1))
}

struct ToolWorkflowDescriptor {
    canonical_name: &'static str,
    aliases: &'static [&'static str],
    build: fn() -> Result<WorkflowSpec, String>,
}

const TOOL_WORKFLOWS: &[ToolWorkflowDescriptor] = &[
    ToolWorkflowDescriptor {
        canonical_name: "gist",
        aliases: &[],
        build: gist_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "gist-snapshot",
        aliases: &["gist_snapshot"],
        build: gist_snapshot_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "gist-diff",
        aliases: &["gist_diff"],
        build: gist_diff_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "gist-recent",
        aliases: &["gist_recent"],
        build: gist_recent_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "bootstrap",
        aliases: &[],
        build: bootstrap_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "makegen",
        aliases: &[],
        build: makegen_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "pragma",
        aliases: &[],
        build: pragma_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "deps",
        aliases: &[],
        build: deps_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "dag-viz",
        aliases: &["dag_viz"],
        build: dag_viz_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "dag-viz-diff",
        aliases: &["dag_viz_diff"],
        build: dag_viz_diff_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "dag-viz-recent",
        aliases: &["dag_viz_recent"],
        build: dag_viz_recent_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "dag-snapshot",
        aliases: &["dag_snapshot"],
        build: dag_snapshot_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "build-all",
        aliases: &["build_all"],
        build: build_all_workflow_spec,
    },
    ToolWorkflowDescriptor {
        canonical_name: "sdlc",
        aliases: &[],
        build: sdlc_workflow_spec,
    },
];

/// Return canonical tool workflow names.
pub fn all_tool_workflow_names() -> Vec<&'static str> {
    TOOL_WORKFLOWS
        .iter()
        .map(|workflow| workflow.canonical_name)
        .collect()
}

/// Build a tool workflow spec by name.
pub fn tool_workflow_spec(name: &str) -> Result<WorkflowSpec, String> {
    for workflow in TOOL_WORKFLOWS {
        if workflow.canonical_name == name || workflow.aliases.contains(&name) {
            return (workflow.build)();
        }
    }
    Err(format!("unknown tool workflow: '{name}'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::schema::has_required_unit_contract;

    #[test]
    fn ci_workflow_builder_is_deterministic() {
        let a = ci_workflow_spec().expect("ci spec");
        let b = ci_workflow_spec().expect("ci spec");
        assert_eq!(a.dag.to_ascii("ci"), b.dag.to_ascii("ci"));
    }

    #[test]
    fn test_all_workflow_builder_is_deterministic() {
        let a = test_all_workflow_spec().expect("test-all spec");
        let b = test_all_workflow_spec().expect("test-all spec");
        assert_eq!(a.dag.to_ascii("test-all"), b.dag.to_ascii("test-all"));
    }

    #[test]
    fn all_ci_units_have_required_contract_ports() {
        let ci = ci_workflow_spec().expect("ci spec");
        for node in &ci.dag.nodes {
            assert!(
                has_required_unit_contract(&node.inputs, &node.outputs),
                "node '{}' missing required contract",
                node.id.0
            );
        }
    }

    #[test]
    fn gist_snapshot_workflow_builder_is_deterministic() {
        let a = gist_snapshot_workflow_spec().expect("gist-snapshot spec");
        let b = gist_snapshot_workflow_spec().expect("gist-snapshot spec");
        assert_eq!(
            a.dag.to_ascii("gist-snapshot"),
            b.dag.to_ascii("gist-snapshot")
        );
    }

    #[test]
    fn gist_diff_workflow_builder_is_deterministic() {
        let a = gist_diff_workflow_spec().expect("gist-diff spec");
        let b = gist_diff_workflow_spec().expect("gist-diff spec");
        assert_eq!(a.dag.to_ascii("gist-diff"), b.dag.to_ascii("gist-diff"));
    }

    #[test]
    fn gist_recent_workflow_builder_is_deterministic() {
        let a = gist_recent_workflow_spec().expect("gist-recent spec");
        let b = gist_recent_workflow_spec().expect("gist-recent spec");
        assert_eq!(a.dag.to_ascii("gist-recent"), b.dag.to_ascii("gist-recent"));
    }

    #[test]
    fn gist_modes_have_expected_node_counts() {
        let snapshot = gist_snapshot_workflow_spec().expect("gist-snapshot spec");
        let diff = gist_diff_workflow_spec().expect("gist-diff spec");
        let recent = gist_recent_workflow_spec().expect("gist-recent spec");
        assert_eq!(snapshot.dag.nodes.len(), 9); // base 6 + snapshot 3
        assert_eq!(diff.dag.nodes.len(), 8); // base 6 + diff 2
        assert_eq!(recent.dag.nodes.len(), 9); // base 6 + recent 3
    }

    // WF19 tests

    #[test]
    fn bootstrap_workflow_builder_is_deterministic() {
        let a = bootstrap_workflow_spec().expect("bootstrap spec");
        let b = bootstrap_workflow_spec().expect("bootstrap spec");
        assert_eq!(a.dag.to_ascii("bootstrap"), b.dag.to_ascii("bootstrap"));
    }

    #[test]
    fn bootstrap_workflow_has_expected_node_count() {
        let spec = bootstrap_workflow_spec().expect("bootstrap spec");
        // 2 universal + 1 scan + 2 generate + 2 upsert + 1 report = 8
        assert_eq!(spec.dag.nodes.len(), 8);
    }

    #[test]
    fn all_bootstrap_units_have_required_contract_ports() {
        let spec = bootstrap_workflow_spec().expect("bootstrap spec");
        for node in &spec.dag.nodes {
            assert!(
                has_required_unit_contract(&node.inputs, &node.outputs),
                "bootstrap node '{}' missing required contract",
                node.id.0
            );
        }
    }

    #[test]
    fn makegen_workflow_builder_is_deterministic() {
        let a = makegen_workflow_spec().expect("makegen spec");
        let b = makegen_workflow_spec().expect("makegen spec");
        assert_eq!(a.dag.to_ascii("makegen"), b.dag.to_ascii("makegen"));
    }

    #[test]
    fn makegen_workflow_has_expected_node_count() {
        let spec = makegen_workflow_spec().expect("makegen spec");
        // 2 universal + 1 load + 1 render + 1 upsert + 1 report = 6
        assert_eq!(spec.dag.nodes.len(), 6);
    }

    #[test]
    fn pragma_workflow_builder_is_deterministic() {
        let a = pragma_workflow_spec().expect("pragma spec");
        let b = pragma_workflow_spec().expect("pragma spec");
        assert_eq!(a.dag.to_ascii("pragma"), b.dag.to_ascii("pragma"));
    }

    #[test]
    fn pragma_workflow_has_expected_node_count() {
        let spec = pragma_workflow_spec().expect("pragma spec");
        // 2 universal + 3 render + 3 upsert + 1 report = 9
        assert_eq!(spec.dag.nodes.len(), 9);
    }

    #[test]
    fn all_pragma_units_have_required_contract_ports() {
        let spec = pragma_workflow_spec().expect("pragma spec");
        for node in &spec.dag.nodes {
            assert!(
                has_required_unit_contract(&node.inputs, &node.outputs),
                "pragma node '{}' missing required contract",
                node.id.0
            );
        }
    }

    // WF20 tests

    #[test]
    fn deps_workflow_builder_is_deterministic() {
        let a = deps_workflow_spec().expect("deps spec");
        let b = deps_workflow_spec().expect("deps spec");
        assert_eq!(a.dag.to_ascii("deps"), b.dag.to_ascii("deps"));
    }

    #[test]
    fn deps_workflow_has_expected_node_count() {
        let spec = deps_workflow_spec().expect("deps spec");
        // 2 universal + 4 install + 3 generate + 1 report = 10
        assert_eq!(spec.dag.nodes.len(), 10);
    }

    #[test]
    fn dag_viz_workflow_builder_is_deterministic() {
        let a = dag_viz_workflow_spec().expect("dag-viz spec");
        let b = dag_viz_workflow_spec().expect("dag-viz spec");
        assert_eq!(a.dag.to_ascii("dag-viz"), b.dag.to_ascii("dag-viz"));
    }

    #[test]
    fn dag_viz_workflow_has_expected_node_count() {
        let spec = dag_viz_workflow_spec().expect("dag-viz spec");
        // 2 universal + 2 base (branch, cred) + 2 viz (serialize, render) + 1 upload + 1 report = 8
        assert_eq!(spec.dag.nodes.len(), 8);
    }

    #[test]
    fn dag_snapshot_workflow_builder_is_deterministic() {
        let a = dag_snapshot_workflow_spec().expect("dag-snapshot spec");
        let b = dag_snapshot_workflow_spec().expect("dag-snapshot spec");
        assert_eq!(
            a.dag.to_ascii("dag-snapshot"),
            b.dag.to_ascii("dag-snapshot")
        );
    }

    #[test]
    fn dag_snapshot_workflow_has_expected_node_count() {
        let spec = dag_snapshot_workflow_spec().expect("dag-snapshot spec");
        // 2 universal + 2 base (branch, cred) + 3 content (list, read, render) + 1 upload + 1 report = 9
        assert_eq!(spec.dag.nodes.len(), 9);
    }

    #[test]
    fn build_all_workflow_builder_is_deterministic() {
        let a = build_all_workflow_spec().expect("build-all spec");
        let b = build_all_workflow_spec().expect("build-all spec");
        assert_eq!(a.dag.to_ascii("build-all"), b.dag.to_ascii("build-all"));
    }

    #[test]
    fn build_all_workflow_has_expected_node_count() {
        let spec = build_all_workflow_spec().expect("build-all spec");
        // 2 universal + 1 build + 1 report = 4
        assert_eq!(spec.dag.nodes.len(), 4);
    }

    #[test]
    fn sdlc_workflow_builder_is_deterministic() {
        let a = sdlc_workflow_spec().expect("sdlc spec");
        let b = sdlc_workflow_spec().expect("sdlc spec");
        assert_eq!(a.dag.to_ascii("sdlc"), b.dag.to_ascii("sdlc"));
    }

    #[test]
    fn sdlc_workflow_has_expected_node_count() {
        let spec = sdlc_workflow_spec().expect("sdlc spec");
        // 2 universal + 1 intake + 1 worker + 1 report = 5
        assert_eq!(spec.dag.nodes.len(), 5);
    }

    #[test]
    fn all_tool_workflows_build_successfully() {
        for name in all_tool_workflow_names() {
            tool_workflow_spec(name)
                .unwrap_or_else(|error| panic!("tool workflow '{name}' failed to build: {error}"));
        }
    }

    #[test]
    fn tool_workflow_spec_rejects_unknown_name() {
        assert!(tool_workflow_spec("nonexistent").is_err());
    }

    #[test]
    fn all_tool_workflow_nodes_have_required_contract() {
        for name in all_tool_workflow_names() {
            let spec = tool_workflow_spec(name).expect(name);
            for node in &spec.dag.nodes {
                assert!(
                    has_required_unit_contract(&node.inputs, &node.outputs),
                    "workflow '{}' node '{}' missing required contract",
                    name,
                    node.id.0
                );
            }
        }
    }
}
