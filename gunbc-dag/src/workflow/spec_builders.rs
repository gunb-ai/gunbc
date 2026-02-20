//! Deterministic workflow spec builders (WF1, WF14/WF15).

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

// ============================================================================
// Tool Workflow Specs (WF14/WF15+)
// ============================================================================

/// Build gist-snapshot workflow spec using universal capabilities.
///
/// This is the first tool workflow to use the planner path. The spec
/// decomposes into:
/// 1. `compilation.ensure` — shared compilation capability (WF14)
/// 2. `codegen.ensure` — shared codegen capability (WF15)
/// 3. `gist.snapshot` — gist-specific content acquisition + upload
/// 4. `gist.report` — reporting node
///
/// The compilation and codegen units share `WorkIdentity` with all other
/// workflows (ci, test-all, etc.) via the global ledger.
pub fn gist_snapshot_workflow_spec() -> Result<WorkflowSpec, String> {
    gist_snapshot_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build gist-snapshot workflow spec against an explicit process registry.
pub fn gist_snapshot_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    // Universal capability nodes (shared via WorkIdentity).
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

    // Gist-specific process unit: snapshot content acquisition + upload.
    // This node is an Aggregate placeholder — actual process decomposition
    // (list_files, read_loop, render, branch_resolution, gist_upload)
    // will be wired when WF16 is implemented.
    dag.add_node(Node::opaque(
        "gist.snapshot",
        required_input_contract(),
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::Aggregate(
            super::schema::AggregateSpec::new("gist.snapshot"),
        )),
    ));

    dag.add_node(report_node("gist.report"));

    // Edges: compilation → codegen → snapshot → report
    dag.add_edge(Edge::control(
        "gist.compilation_ensure",
        "commit",
        "gist.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.codegen_ensure",
        "commit",
        "gist.snapshot",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist.snapshot",
        "commit",
        "gist.report",
        "after",
    ));

    Ok(WorkflowSpec::new("gist-snapshot", dag, 1))
}

/// Build gist-diff workflow spec using universal capabilities.
pub fn gist_diff_workflow_spec() -> Result<WorkflowSpec, String> {
    gist_diff_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build gist-diff workflow spec against an explicit process registry.
pub fn gist_diff_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

    dag.add_node(invoke_node(
        "gist_diff.compilation_ensure",
        ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(invoke_node(
        "gist_diff.codegen_ensure",
        ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT),
        registry,
    )?);
    dag.add_node(Node::opaque(
        "gist_diff.diff",
        required_input_contract(),
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::Aggregate(
            super::schema::AggregateSpec::new("gist_diff.diff"),
        )),
    ));
    dag.add_node(report_node("gist_diff.report"));

    dag.add_edge(Edge::control(
        "gist_diff.compilation_ensure",
        "commit",
        "gist_diff.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist_diff.codegen_ensure",
        "commit",
        "gist_diff.diff",
        "after",
    ));
    dag.add_edge(Edge::control(
        "gist_diff.diff",
        "commit",
        "gist_diff.report",
        "after",
    ));

    Ok(WorkflowSpec::new("gist-diff", dag, 1))
}

/// Build bootstrap workflow spec using universal capabilities.
pub fn bootstrap_workflow_spec() -> Result<WorkflowSpec, String> {
    bootstrap_workflow_spec_with_registry(&default_process_unit_registry())
}

/// Build bootstrap workflow spec against an explicit process registry.
pub fn bootstrap_workflow_spec_with_registry(
    registry: &ProcessUnitRegistry,
) -> Result<WorkflowSpec, String> {
    let mut dag = Dag::new();

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
    dag.add_node(Node::opaque(
        "bootstrap.run",
        required_input_contract(),
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::Aggregate(
            super::schema::AggregateSpec::new("bootstrap.run"),
        )),
    ));
    dag.add_node(report_node("bootstrap.report"));

    dag.add_edge(Edge::control(
        "bootstrap.compilation_ensure",
        "commit",
        "bootstrap.codegen_ensure",
        "after",
    ));
    dag.add_edge(Edge::control(
        "bootstrap.codegen_ensure",
        "commit",
        "bootstrap.run",
        "after",
    ));
    dag.add_edge(Edge::control(
        "bootstrap.run",
        "commit",
        "bootstrap.report",
        "after",
    ));

    Ok(WorkflowSpec::new("bootstrap", dag, 1))
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

    // WF14/WF15: tool workflow spec tests

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
    fn gist_snapshot_has_compilation_and_codegen_capabilities() {
        let spec = gist_snapshot_workflow_spec().expect("gist-snapshot spec");
        assert_eq!(spec.dag.nodes.len(), 4);
        assert!(spec.dag.get_node(&"gist.compilation_ensure".into()).is_some());
        assert!(spec.dag.get_node(&"gist.codegen_ensure".into()).is_some());
        assert!(spec.dag.get_node(&"gist.snapshot".into()).is_some());
        assert!(spec.dag.get_node(&"gist.report".into()).is_some());
    }

    #[test]
    fn all_gist_snapshot_units_have_required_contract_ports() {
        let spec = gist_snapshot_workflow_spec().expect("gist-snapshot spec");
        for node in &spec.dag.nodes {
            assert!(
                has_required_unit_contract(&node.inputs, &node.outputs),
                "node '{}' missing required contract",
                node.id.0
            );
        }
    }

    #[test]
    fn gist_diff_workflow_builder_is_deterministic() {
        let a = gist_diff_workflow_spec().expect("gist-diff spec");
        let b = gist_diff_workflow_spec().expect("gist-diff spec");
        assert_eq!(a.dag.to_ascii("gist-diff"), b.dag.to_ascii("gist-diff"));
    }

    #[test]
    fn bootstrap_workflow_builder_is_deterministic() {
        let a = bootstrap_workflow_spec().expect("bootstrap spec");
        let b = bootstrap_workflow_spec().expect("bootstrap spec");
        assert_eq!(
            a.dag.to_ascii("bootstrap"),
            b.dag.to_ascii("bootstrap")
        );
    }

    #[test]
    fn bootstrap_has_compilation_and_codegen_capabilities() {
        let spec = bootstrap_workflow_spec().expect("bootstrap spec");
        assert_eq!(spec.dag.nodes.len(), 4);
        assert!(spec.dag.get_node(&"bootstrap.compilation_ensure".into()).is_some());
        assert!(spec.dag.get_node(&"bootstrap.codegen_ensure".into()).is_some());
        assert!(spec.dag.get_node(&"bootstrap.run".into()).is_some());
        assert!(spec.dag.get_node(&"bootstrap.report".into()).is_some());
    }
}
