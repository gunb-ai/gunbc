//! Workflow admission contract tests (WF2).

use gunbc_dag::workflow::{WorkflowId, WorkflowSpec};
use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, required_input_contract,
    required_output_contract, test_all_workflow_spec, validate_workflow_admission, ClaimId,
    ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec, UnitClaim, WorkflowAdmissionError,
    WorkflowOp, WorkflowUnit,
};
use gunbc_ir::{AccessMode, Dag, Node, Port};

fn invoke_node(
    id: &str,
    process_id: &str,
    unit_id: &str,
    claims: &[(&str, AccessMode)],
) -> Node<WorkflowUnit> {
    let mut inputs = required_input_contract();
    for (claim, mode) in claims {
        inputs.push(Port::resource(*claim, "ResourceHandle", *mode));
    }
    Node::opaque(
        id,
        inputs,
        required_output_contract(),
        WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
            process_id, unit_id,
        ))),
    )
}

fn registry_for_two_nodes(
    required_a: Vec<UnitClaim>,
    required_b: Vec<UnitClaim>,
) -> ProcessUnitRegistry {
    let mut registry = ProcessUnitRegistry::new();
    registry.register(ProcessUnitSpec::new(
        ProcessUnitRef::new("wf", "a"),
        1,
        required_a,
    ));
    registry.register(ProcessUnitSpec::new(
        ProcessUnitRef::new("wf", "b"),
        1,
        required_b,
    ));
    registry
}

#[test]
fn ci_and_test_all_specs_pass_default_admission_validation() {
    let registry = default_process_unit_registry();
    let ci = ci_workflow_spec().expect("ci workflow spec");
    validate_workflow_admission(&ci, &registry).expect("ci workflow admission should pass");

    let test_all = test_all_workflow_spec().expect("test-all workflow spec");
    validate_workflow_admission(&test_all, &registry)
        .expect("test-all workflow admission should pass");
}

#[test]
fn read_read_claims_allowed() {
    let mut dag = Dag::new();
    dag.add_node(invoke_node(
        "wf.a",
        "wf",
        "a",
        &[("file:workspace", AccessMode::Read)],
    ));
    dag.add_node(invoke_node(
        "wf.b",
        "wf",
        "b",
        &[("file:workspace", AccessMode::Read)],
    ));
    let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
    let registry = registry_for_two_nodes(
        vec![UnitClaim::read("file:workspace")],
        vec![UnitClaim::read("file:workspace")],
    );

    validate_workflow_admission(&spec, &registry).expect("read/read must be allowed");
}

#[test]
fn write_write_claims_fail_with_unit_and_claim_diagnostics() {
    let mut dag = Dag::new();
    dag.add_node(invoke_node(
        "wf.a",
        "wf",
        "a",
        &[("file:workspace", AccessMode::Write)],
    ));
    dag.add_node(invoke_node(
        "wf.b",
        "wf",
        "b",
        &[("file:workspace", AccessMode::Write)],
    ));
    let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
    let registry = registry_for_two_nodes(
        vec![UnitClaim::write("file:workspace")],
        vec![UnitClaim::write("file:workspace")],
    );

    let errors =
        validate_workflow_admission(&spec, &registry).expect_err("write/write should fail");
    let conflict = errors
        .iter()
        .find_map(|error| match error {
            WorkflowAdmissionError::ConflictingClaims {
                left_node,
                right_node,
                left_claim,
                right_claim,
                ..
            } => Some((left_node, right_node, left_claim, right_claim)),
            _ => None,
        })
        .expect("conflicting claim diagnostics must be present");

    assert!(
        (conflict.0 .0 == "wf.a" && conflict.1 .0 == "wf.b")
            || (conflict.0 .0 == "wf.b" && conflict.1 .0 == "wf.a")
    );
    assert_eq!(conflict.2, &ClaimId::new("file:workspace"));
    assert_eq!(conflict.3, &ClaimId::new("file:workspace"));
}

#[test]
fn missing_required_claims_fail_closed() {
    let mut dag = Dag::new();
    dag.add_node(invoke_node("wf.a", "wf", "a", &[]));
    dag.add_node(invoke_node(
        "wf.b",
        "wf",
        "b",
        &[("file:workspace", AccessMode::Read)],
    ));
    dag.add_edge(gunbc_ir::Edge::control("wf.a", "commit", "wf.b", "after"));

    let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
    let registry = registry_for_two_nodes(
        vec![UnitClaim::write("file:workspace")],
        vec![UnitClaim::read("file:workspace")],
    );

    let errors =
        validate_workflow_admission(&spec, &registry).expect_err("missing claim should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        WorkflowAdmissionError::MissingRequiredClaims { node_id, .. } if node_id.0 == "wf.a"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        WorkflowAdmissionError::UndeclaredEffectfulIo { node_id, .. } if node_id.0 == "wf.a"
    )));
}

#[test]
fn coarse_file_claim_conflicts_with_qualified_file_claim() {
    let mut dag = Dag::new();
    dag.add_node(invoke_node(
        "wf.a",
        "wf",
        "a",
        &[("file", AccessMode::Write)],
    ));
    dag.add_node(invoke_node(
        "wf.b",
        "wf",
        "b",
        &[("file:workspace", AccessMode::Write)],
    ));
    let spec = WorkflowSpec::new(WorkflowId::new("wf"), dag, 1);
    let registry = registry_for_two_nodes(
        vec![UnitClaim::write("file")],
        vec![UnitClaim::write("file:workspace")],
    );

    let errors =
        validate_workflow_admission(&spec, &registry).expect_err("coarse vs scoped must conflict");
    assert!(errors.iter().any(|error| matches!(
        error,
        WorkflowAdmissionError::ConflictingClaims { left_claim, right_claim, .. }
            if (left_claim == &ClaimId::new("file") && right_claim == &ClaimId::new("file:workspace"))
            || (left_claim == &ClaimId::new("file:workspace") && right_claim == &ClaimId::new("file"))
    )));
}
