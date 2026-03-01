//! Gist workflow planner contracts (WF16-WF18).
//!
//! These tests pin:
//! 1. Shared base units across gist modes.
//! 2. Mode-specific node shape for gist/diff/recent.
//! 3. Cross-workflow dedup for shared capability units.

#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_dag::{
    default_process_unit_registry, explain_plan, gist_diff_workflow_spec,
    gist_recent_workflow_spec, gist_workflow_spec, plan_global_workflows, plan_workflow_with_mode,
    DryRunMode, PlanAction, PlannerInputs, PlannerInputsByWorkflow,
};

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-gist-wf-contracts-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[test]
fn gist_plan_produces_execute_on_first_run() {
    let root = temp_root();
    let spec = gist_workflow_spec().expect("gist spec");
    let registry = default_process_unit_registry();
    let plan = plan_workflow_with_mode(
        &spec,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("plan");

    for node in &plan.nodes {
        assert!(
            matches!(node.action, PlanAction::Execute { .. }),
            "node '{}' should execute on first run",
            node.node_id.0
        );
    }

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn gist_recent_critical_path_includes_rev_list() {
    let root = temp_root();
    let spec = gist_recent_workflow_spec().expect("gist-recent spec");
    let registry = default_process_unit_registry();
    let plan = plan_workflow_with_mode(
        &spec,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("plan");
    let explain = explain_plan(&spec, &plan);

    assert!(
        explain.critical_path.iter().any(|n| n.0 == "gist.rev_list"),
        "expected gist.rev_list in gist-recent critical path"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn gist_diff_has_fewer_nodes_than_gist() {
    let gist = gist_workflow_spec().expect("gist");
    let diff = gist_diff_workflow_spec().expect("diff");
    assert!(
        diff.dag.nodes.len() < gist.dag.nodes.len(),
        "diff ({}) should have fewer nodes than gist ({})",
        diff.dag.nodes.len(),
        gist.dag.nodes.len()
    );
}

#[test]
fn shared_base_units_keep_identical_work_identity_across_modes() {
    let root = temp_root();
    let registry = default_process_unit_registry();
    let gist = gist_workflow_spec().expect("gist");
    let diff = gist_diff_workflow_spec().expect("diff");
    let recent = gist_recent_workflow_spec().expect("recent");

    let gist_plan = plan_workflow_with_mode(
        &gist,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("gist plan");
    let diff_plan = plan_workflow_with_mode(
        &diff,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("diff plan");
    let recent_plan = plan_workflow_with_mode(
        &recent,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("recent plan");

    for node_name in [
        "gist.compilation_ensure",
        "gist.codegen_ensure",
        "gist.branch_resolution",
        "gist.credential_resolve",
        "gist.gist_create",
    ] {
        let a = gist_plan
            .nodes
            .iter()
            .find(|n| n.node_id.0 == node_name)
            .map(|n| &n.work_id)
            .unwrap_or_else(|| panic!("gist should include {node_name}"));
        let b = diff_plan
            .nodes
            .iter()
            .find(|n| n.node_id.0 == node_name)
            .map(|n| &n.work_id)
            .unwrap_or_else(|| panic!("diff should include {node_name}"));
        let c = recent_plan
            .nodes
            .iter()
            .find(|n| n.node_id.0 == node_name)
            .map(|n| &n.work_id)
            .unwrap_or_else(|| panic!("recent should include {node_name}"));

        assert_eq!(
            a, b,
            "work identity drift for {node_name} between gist/diff"
        );
        assert_eq!(
            b, c,
            "work identity drift for {node_name} between diff/recent"
        );
    }

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn shared_compilation_key_is_identical_between_gist_and_diff() {
    let root = temp_root();
    let registry = default_process_unit_registry();
    let gist = gist_workflow_spec().expect("gist");
    let diff = gist_diff_workflow_spec().expect("diff");

    let gist_plan = plan_workflow_with_mode(
        &gist,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("gist plan");
    let diff_plan = plan_workflow_with_mode(
        &diff,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("diff plan");

    let a = gist_plan
        .nodes
        .iter()
        .find(|n| n.node_id.0 == "gist.compilation_ensure")
        .expect("gist compilation");
    let b = diff_plan
        .nodes
        .iter()
        .find(|n| n.node_id.0 == "gist.compilation_ensure")
        .expect("diff compilation");

    assert_eq!(
        a.key.digest, b.key.digest,
        "compilation key should be stable across gist modes"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn global_plan_deduplicates_shared_gist_base_units() {
    let root = temp_root();
    let registry = default_process_unit_registry();
    let gist = gist_workflow_spec().expect("gist");
    let diff = gist_diff_workflow_spec().expect("diff");

    let mut inputs = PlannerInputsByWorkflow::new();
    inputs.insert(gist.id.clone(), PlannerInputs::new());
    inputs.insert(diff.id.clone(), PlannerInputs::new());

    let specs = vec![gist, diff];
    let global = plan_global_workflows(&specs, &registry, &inputs, &root).expect("global plan");

    let compile_vertices: Vec<_> = global
        .vertices
        .iter()
        .filter(|v| {
            v.node_refs
                .iter()
                .any(|r| r.node_id.0 == "gist.compilation_ensure")
        })
        .collect();
    assert_eq!(
        compile_vertices.len(),
        1,
        "shared gist compilation node should be deduplicated"
    );
    assert_eq!(
        compile_vertices[0].node_refs.len(),
        2,
        "deduplicated gist compilation node should reference both workflows"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn explain_execute_set_matches_node_count_for_snapshot() {
    let root = temp_root();
    let spec = gist_workflow_spec().expect("gist");
    let registry = default_process_unit_registry();
    let plan = plan_workflow_with_mode(
        &spec,
        &registry,
        &PlannerInputs::new(),
        &root,
        DryRunMode::Lenient,
    )
    .expect("plan");
    let explain = explain_plan(&spec, &plan);

    assert_eq!(explain.execute_set.len(), spec.dag.nodes.len());
    assert!(explain.cache_hit_set.is_empty());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn planner_inputs_by_workflow_alias_is_btreemap() {
    let mut map = PlannerInputsByWorkflow::new();
    map.insert("gist-snapshot".into(), PlannerInputs::new());
    let _: BTreeMap<_, _> = map;
}
