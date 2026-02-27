//! BT5: Worker dispatch loop test — SDLC worker function.
//!
//! Validates that funcs/sdlc_worker.dag compiles with unit_test profile
//! and the dispatch_sdlc() function is structurally correct.
//!
//! The worker loop (discover→claim→dispatch→record→release) is validated
//! through compilation and structural checks. DryRun execution is covered
//! by BT3 (which runs the full pipeline that invokes the worker).
//!
//! Testing level: L3 (worker loop)
//! Profile: unit_test

#![allow(clippy::disallowed_methods)]

use gunbc_dag::dsl_builder::build_dsl_graph_with_profile;

/// Worker module compiles with unit_test profile.
#[test]
fn sdlc_worker_compiles_with_unit_test_profile() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "unit_test")
        .expect("sdlc_worker should compile with unit_test profile");
    assert!(
        !dag.nodes.is_empty(),
        "sdlc_worker should produce non-empty DAG"
    );
}

/// Worker DAG has the dispatch_sdlc entry point.
#[test]
fn sdlc_worker_has_dispatch_sdlc_node() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "unit_test")
        .expect("sdlc_worker should compile with unit_test profile");

    let has_dispatch = dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("dispatch_sdlc"));

    assert!(
        has_dispatch,
        "sdlc_worker should contain dispatch_sdlc node. Got: {:?}",
        dag.nodes.iter().map(|n| &n.id.0).collect::<Vec<_>>()
    );
}

/// Worker DAG includes claim lifecycle nodes (acquire, release).
#[test]
fn sdlc_worker_has_claim_lifecycle_nodes() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "unit_test")
        .expect("sdlc_worker should compile with unit_test profile");

    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();

    // Worker should have claim acquire and release operations
    let has_acquire = node_ids.iter().any(|id| id.contains("acquire"));
    let has_release = node_ids.iter().any(|id| id.contains("release"));

    assert!(
        has_acquire,
        "worker should have claim acquire node. Got: {node_ids:?}"
    );
    assert!(
        has_release,
        "worker should have claim release node. Got: {node_ids:?}"
    );
}

/// Worker DAG includes discover operation (issue discovery).
#[test]
fn sdlc_worker_has_discover_node() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "unit_test")
        .expect("sdlc_worker should compile with unit_test profile");

    let has_discover = dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("discover"));

    assert!(
        has_discover,
        "worker should have issue discovery node. Got: {:?}",
        dag.nodes.iter().map(|n| &n.id.0).collect::<Vec<_>>()
    );
}

/// Worker DAG has outcome ledger operations (upsert, get).
#[test]
fn sdlc_worker_has_outcome_ledger_operations() {
    let dag = build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "unit_test")
        .expect("sdlc_worker should compile with unit_test profile");

    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();

    let has_upsert = node_ids.iter().any(|id| id.contains("upsert"));
    let has_get = node_ids.iter().any(|id| id.contains("get"));

    assert!(
        has_upsert,
        "worker should have outcome upsert node. Got: {node_ids:?}"
    );
    assert!(
        has_get,
        "worker should have outcome get node (for replay-skip). Got: {node_ids:?}"
    );
}
