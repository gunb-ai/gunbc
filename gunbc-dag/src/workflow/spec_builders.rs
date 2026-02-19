//! Deterministic workflow spec builders (WF1).

use gunbc_ir::{Dag, Edge, Node, Port};

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
}
