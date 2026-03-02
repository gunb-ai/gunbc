//! Global flattening + projection-proof contracts (M17-M19).

use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, plan_global_workflows, project_execute_set,
    prove_non_redundancy, test_all_workflow_spec, validate_projection_equivalence,
    PlannerInputsByWorkflow, WorkflowId,
};
use gunbc_ir::NodeId;

fn temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-workflow-global-dedup-contracts-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[test]
fn global_plan_dedups_equivalent_work_across_ci_and_test_all() {
    let root = temp_root();
    let specs = vec![
        ci_workflow_spec().expect("ci spec"),
        test_all_workflow_spec().expect("test-all spec"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    let codegen = global
        .vertices
        .iter()
        .find(|vertex| vertex.work_id.unit_id == NodeId::from("codegen"))
        .expect("canonical codegen vertex");
    let refs = codegen
        .node_refs
        .iter()
        .map(|reference| reference.workflow_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(refs.len(), 2);
    assert!(refs.contains(&WorkflowId::new("ci")));
    assert!(refs.contains(&WorkflowId::new("test-all")));
}

#[test]
fn proof_and_projection_checks_hold_for_global_plan() {
    let root = temp_root();
    let specs = vec![
        ci_workflow_spec().expect("ci spec"),
        test_all_workflow_spec().expect("test-all spec"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    prove_non_redundancy(&global).expect("global plan should satisfy non-redundancy invariants");
    let projection = project_execute_set(&global);
    validate_projection_equivalence(&global, &projection)
        .expect("canonical execute projection should be drift-free");
}
