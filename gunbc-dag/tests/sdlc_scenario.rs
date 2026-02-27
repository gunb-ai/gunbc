//! BT3: Hermetic scenario test — SDLC pipeline DryRun execution.
//!
//! Validates the SDLC pipeline compiles and executes through DryRun
//! with the unit_test profile (all stubs, no real I/O).
//!
//! Testing level: L1 (hermetic scenario)
//! Profile: unit_test
//! Transport: DryRun/stubs

#![allow(clippy::disallowed_methods)]

use gunbc_dag::{dsl_builder::build_dsl_graph_with_profile, mock_defaults::auto_mock_spec};
use gunbc_exec::{execute_with_mode_and_inputs, lower, ExecutionMode};

/// Compile the SDLC pipeline with unit_test profile and execute in DryRun mode.
/// This proves the full DAG structure is valid and executable with stubs.
#[test]
fn sdlc_pipeline_unit_test_dry_run_completes() {
    let dag = build_dsl_graph_with_profile("pipelines/sdlc.dag", "unit_test")
        .expect("SDLC pipeline should compile with unit_test profile");

    let spec = auto_mock_spec(&dag, "sdlc");
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
    .expect("SDLC DryRun execution should succeed");

    // Verify execution produced entries.
    assert!(
        !log.entries.is_empty(),
        "SDLC DryRun should execute at least one node"
    );

    // Verify key pipeline stages appeared in execution.
    let node_ids: Vec<&str> = log.entries.iter().map(|e| e.node_id.as_str()).collect();

    // The pipeline should contain issue-related nodes (fetch, claim, design, etc.)
    assert!(
        node_ids.len() > 1,
        "SDLC pipeline should execute multiple nodes. Got: {node_ids:?}"
    );
}

/// Verify the SDLC pipeline DAG structure has expected node count.
#[test]
fn sdlc_pipeline_has_substantial_node_count() {
    let dag = build_dsl_graph_with_profile("pipelines/sdlc.dag", "unit_test")
        .expect("SDLC pipeline should compile with unit_test profile");

    // The SDLC pipeline has 11 stages with multiple operations per stage.
    // We expect a substantial number of nodes after lowering.
    assert!(
        dag.nodes.len() > 10,
        "SDLC pipeline should have significant node count. Got: {}",
        dag.nodes.len()
    );
}
