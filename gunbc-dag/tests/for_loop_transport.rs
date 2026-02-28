//! Integration tests: gist snapshot pipeline.
//!
//! Validates the gist snapshot DAG pipeline through DryRun execution.
//!
//! History: Previously tested explicit for-loop with service-call transport
//! (gist_snapshot.dag had inline `for path in files { fs.read(path) }`).
//! After consolidation into gist.dag, the loop is inside `read_text_files`
//! pattern (std.patterns). The pattern is lowered as a callable node (not
//! inlined), so no loop structure appears in the top-level graph.

#![allow(clippy::disallowed_methods)]

use gunbc_dag::{dsl_builder::build_dsl_graph_for_entry, mock_defaults::auto_mock_spec};
use gunbc_exec::{execute_with_mode_and_inputs, lower, ExecutionMode};

// -------------------------------------------------------------------
// Structural: gist graph uses read_text_files pattern (no inline loop)
// -------------------------------------------------------------------

#[test]
fn gist_graph_has_read_text_files_node() {
    let dag =
        build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist").expect("build gist graph");
    let lowered = lower(&dag).expect("lower gist graph");

    // After consolidation, gist uses read_text_files pattern.
    // The pattern is a callable node in the top-level graph.
    let has_read_text_files = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("read_text_files"));

    assert!(
        has_read_text_files,
        "gist graph should contain read_text_files pattern node. Got nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

#[test]
fn gist_graph_has_gist_callable_node() {
    let dag =
        build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist").expect("build gist graph");
    let lowered = lower(&dag).expect("lower gist graph");

    // build_snapshot_content is an extern call inside tools.gist::gist, so it
    // is no longer represented as a standalone top-level node.
    let has_gist_callable = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0 == "tools.gist::gist");

    assert!(
        has_gist_callable,
        "gist graph should contain tools.gist::gist callable node. Got nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

// -------------------------------------------------------------------
// Execution: gist DryRun completes successfully
// -------------------------------------------------------------------

#[test]
fn gist_snapshot_dry_run_completes() {
    let dag =
        build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist").expect("build gist graph");

    let spec = auto_mock_spec(&dag, "gist");
    let dry_run_mocks = spec.to_dry_run_mocks();

    let input_mocks = {
        let lowered = lower(&dag).expect("lower for entrypoint detection");
        let entrypoints = gunbc_ir::detect_entrypoints(&lowered.dag);
        let boundary = spec.to_boundary_mocks();
        let mut mocks = gunbc_exec::BoundaryMocks::new();
        for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
            if let Some(val) = boundary.get_input(&node_id.0, &port_name.0) {
                mocks.set_input(node_id.0.clone(), port_name.0.clone(), val.clone());
            }
        }
        mocks
    };

    let log = execute_with_mode_and_inputs(
        &dag,
        ExecutionMode::DryRun(dry_run_mocks),
        Some(&input_mocks),
    )
    .expect("gist DryRun execution should succeed");

    // Verify key pipeline stages appeared in execution.
    let node_ids: Vec<&str> = log.entries.iter().map(|e| e.node_id.as_str()).collect();

    assert!(
        node_ids.iter().any(|id| id.contains("LsFiles")),
        "execution should include LsFiles transport. Got: {node_ids:?}"
    );
    assert!(
        node_ids.iter().any(|id| id.contains("read_text_files")),
        "execution should include read_text_files pattern. Got: {node_ids:?}"
    );
    assert!(
        node_ids.iter().any(|id| id == &"tools.gist::gist"),
        "execution should include tools.gist::gist callable. Got: {node_ids:?}"
    );
    assert!(
        node_ids.iter().any(|id| id.contains("Gist_Create")),
        "execution should include Gist_Create transport. Got: {node_ids:?}"
    );
}
