//! Cross-workflow capability sharing contracts (WF19/WF20).
//!
//! These tests verify:
//! 1. Universal capabilities (compilation_ensure, codegen_ensure) share
//!    canonical WorkIdentity across all tool workflows via global dedup.
//! 2. All tool workflow specs build deterministically.
//! 3. Global plan flattening and non-redundancy proofs hold when tool
//!    workflows are included alongside CI and test-all.
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    all_tool_workflow_names, bootstrap_workflow_spec, ci_workflow_spec, dag_snapshot_workflow_spec,
    dag_viz_workflow_spec, default_process_unit_registry, deps_workflow_spec,
    makegen_workflow_spec, plan_global_workflows, pragma_workflow_spec, project_execute_set,
    prove_non_redundancy, test_all_workflow_spec, tool_workflow_spec,
    validate_projection_equivalence, PlannerInputsByWorkflow,
};
use gunbc_ir::NodeId;

fn temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-workflow-tool-cap-contracts-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

// =========================================================================
// WF19/WF20: Spec construction contracts
// =========================================================================

#[test]
fn all_tool_workflow_specs_build_without_error() {
    for name in all_tool_workflow_names() {
        tool_workflow_spec(name)
            .unwrap_or_else(|error| panic!("tool workflow '{name}' failed to build: {error}"));
    }
}

#[test]
fn all_tool_workflow_specs_are_deterministic() {
    for name in all_tool_workflow_names() {
        let a = tool_workflow_spec(name).expect(name);
        let b = tool_workflow_spec(name).expect(name);
        assert_eq!(
            a.dag.to_ascii(name),
            b.dag.to_ascii(name),
            "workflow '{name}' is not deterministic"
        );
    }
}

// =========================================================================
// WF19/WF20: Universal capability dedup contracts
// =========================================================================

#[test]
fn compilation_ensure_is_deduped_across_all_tool_workflows() {
    let root = temp_root();
    let specs = vec![
        bootstrap_workflow_spec().expect("bootstrap"),
        makegen_workflow_spec().expect("makegen"),
        pragma_workflow_spec().expect("pragma"),
        deps_workflow_spec().expect("deps"),
        dag_viz_workflow_spec().expect("dag-viz"),
        dag_snapshot_workflow_spec().expect("dag-snapshot"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    // Find the canonical compilation_ensure vertex
    let compilation = global
        .vertices
        .iter()
        .find(|vertex| vertex.work_id.unit_id == NodeId::from("compilation_ensure"))
        .expect("expected canonical compilation_ensure vertex in global plan");

    // All 6 tool workflows should reference this single vertex
    let workflows: BTreeSet<_> = compilation
        .node_refs
        .iter()
        .map(|r| r.workflow_id.clone())
        .collect();
    assert_eq!(
        workflows.len(),
        6,
        "compilation_ensure should be shared across all 6 tool workflows, found: {workflows:?}"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn codegen_ensure_is_deduped_across_all_tool_workflows() {
    let root = temp_root();
    let specs = vec![
        bootstrap_workflow_spec().expect("bootstrap"),
        makegen_workflow_spec().expect("makegen"),
        pragma_workflow_spec().expect("pragma"),
        deps_workflow_spec().expect("deps"),
        dag_viz_workflow_spec().expect("dag-viz"),
        dag_snapshot_workflow_spec().expect("dag-snapshot"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    let codegen = global
        .vertices
        .iter()
        .find(|vertex| vertex.work_id.unit_id == NodeId::from("codegen_ensure"))
        .expect("expected canonical codegen_ensure vertex in global plan");

    let workflows: BTreeSet<_> = codegen
        .node_refs
        .iter()
        .map(|r| r.workflow_id.clone())
        .collect();
    assert_eq!(
        workflows.len(),
        6,
        "codegen_ensure should be shared across all 6 tool workflows, found: {workflows:?}"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn branch_resolution_is_shared_between_dag_viz_and_dag_snapshot() {
    let root = temp_root();
    let specs = vec![
        dag_viz_workflow_spec().expect("dag-viz"),
        dag_snapshot_workflow_spec().expect("dag-snapshot"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    let branch = global
        .vertices
        .iter()
        .find(|vertex| vertex.work_id.unit_id == NodeId::from("branch_resolution"))
        .expect("expected canonical branch_resolution vertex");
    assert_eq!(
        branch.node_refs.len(),
        2,
        "branch_resolution should be shared between dag-viz and dag-snapshot"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn credential_resolve_is_shared_between_dag_viz_and_dag_snapshot() {
    let root = temp_root();
    let specs = vec![
        dag_viz_workflow_spec().expect("dag-viz"),
        dag_snapshot_workflow_spec().expect("dag-snapshot"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    let cred = global
        .vertices
        .iter()
        .find(|vertex| vertex.work_id.unit_id == NodeId::from("credential_resolve"))
        .expect("expected canonical credential_resolve vertex");
    assert_eq!(
        cred.node_refs.len(),
        2,
        "credential_resolve should be shared between dag-viz and dag-snapshot"
    );

    let _ = std::fs::remove_dir_all(root);
}

// =========================================================================
// Global plan invariants with all workflows combined
// =========================================================================

#[test]
fn global_plan_with_all_workflows_satisfies_non_redundancy_proof() {
    let root = temp_root();
    let specs = vec![
        ci_workflow_spec().expect("ci"),
        test_all_workflow_spec().expect("test-all"),
        bootstrap_workflow_spec().expect("bootstrap"),
        makegen_workflow_spec().expect("makegen"),
        pragma_workflow_spec().expect("pragma"),
        deps_workflow_spec().expect("deps"),
        dag_viz_workflow_spec().expect("dag-viz"),
        dag_snapshot_workflow_spec().expect("dag-snapshot"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    prove_non_redundancy(&global).expect("global plan with all workflows should be non-redundant");

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn global_plan_with_all_workflows_satisfies_projection_equivalence() {
    let root = temp_root();
    let specs = vec![
        ci_workflow_spec().expect("ci"),
        test_all_workflow_spec().expect("test-all"),
        bootstrap_workflow_spec().expect("bootstrap"),
        makegen_workflow_spec().expect("makegen"),
        pragma_workflow_spec().expect("pragma"),
        deps_workflow_spec().expect("deps"),
        dag_viz_workflow_spec().expect("dag-viz"),
        dag_snapshot_workflow_spec().expect("dag-snapshot"),
    ];
    let registry = default_process_unit_registry();
    let global = plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
        .expect("global plan");

    let projection = project_execute_set(&global);
    validate_projection_equivalence(&global, &projection)
        .expect("execute projection should be drift-free across all workflows");

    let _ = std::fs::remove_dir_all(root);
}

// =========================================================================
// WF19: Acceptance criteria - warm no-op contracts
// =========================================================================

#[test]
fn bootstrap_workflow_contains_compilation_and_codegen_gates() {
    let spec = bootstrap_workflow_spec().expect("bootstrap spec");
    let node_ids: Vec<String> = spec.dag.nodes.iter().map(|n| n.id.0.clone()).collect();
    assert!(node_ids.contains(&"bootstrap.compilation_ensure".to_string()));
    assert!(node_ids.contains(&"bootstrap.codegen_ensure".to_string()));
}

#[test]
fn makegen_workflow_is_linear_chain() {
    let spec = makegen_workflow_spec().expect("makegen spec");
    // 6 nodes, 5 edges in a linear chain
    assert_eq!(spec.dag.nodes.len(), 6);
    assert_eq!(spec.dag.edges.len(), 5);
}

#[test]
fn pragma_workflow_has_three_parallel_upsert_chains() {
    let spec = pragma_workflow_spec().expect("pragma spec");
    let node_ids: Vec<String> = spec.dag.nodes.iter().map(|n| n.id.0.clone()).collect();
    assert!(node_ids.contains(&"pragma.upsert_clippy".to_string()));
    assert!(node_ids.contains(&"pragma.upsert_allowlist".to_string()));
    assert!(node_ids.contains(&"pragma.upsert_policy".to_string()));
}

#[test]
fn deps_workflow_has_parallel_install_and_generate_chains() {
    let spec = deps_workflow_spec().expect("deps spec");
    let node_ids: Vec<String> = spec.dag.nodes.iter().map(|n| n.id.0.clone()).collect();
    // Install chain
    assert!(node_ids.contains(&"deps.execute_installs".to_string()));
    // Generate chain
    assert!(node_ids.contains(&"deps.write_deps_toml".to_string()));
}

#[test]
fn dag_viz_workflow_contains_gist_upload() {
    let spec = dag_viz_workflow_spec().expect("dag-viz spec");
    let node_ids: Vec<String> = spec.dag.nodes.iter().map(|n| n.id.0.clone()).collect();
    assert!(node_ids.contains(&"dag_viz.gist_upload".to_string()));
    assert!(node_ids.contains(&"dag_viz.credential_resolve".to_string()));
}
