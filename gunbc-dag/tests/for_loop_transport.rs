//! Integration tests: for-loop with service-call transport.
//!
//! Exercises the complete pipeline for the gist_snapshot for-loop:
//!   DSL compile → resolve → lower → DryRun execute
//!
//! Pinned behaviour: loop body output = transport result (file content),
//! NOT the loop element variable (file path).

#![allow(clippy::disallowed_methods)]

use gunbc_dag::{dsl_builder::{build_dsl_graph_for_entry}, mock_defaults::auto_mock_spec};
use gunbc_exec::{execute_with_mode_and_inputs, lower, ExecutionMode};
use gunbc_ir::transport::{FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;

// -------------------------------------------------------------------
// Structural: body execute node has FilesystemHandle input
// -------------------------------------------------------------------

#[test]
fn for_loop_body_dag_has_filesystem_handle_on_execute_node() {
    let dag = build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist").expect("build gist graph");
    let lowered = lower(&dag).expect("lower gist graph");

    assert!(
        !lowered.loops.is_empty(),
        "gist graph should have at least one loop"
    );

    let loop_info = &lowered.loops[0];
    let execute_node = loop_info
        .body_dag
        .nodes
        .iter()
        .find(|n| n.id.0 == "execute_body_t0")
        .expect("body_dag should have execute_body_t0 node");

    let has_fs_handle = execute_node
        .inputs
        .iter()
        .any(|p| p.type_id.0 == "FilesystemHandle");

    assert!(
        has_fs_handle,
        "execute_body_t0 should have FilesystemHandle input (added by resolve), \
         got inputs: {:?}",
        execute_node
            .inputs
            .iter()
            .map(|p| format!("{}:{}", p.name.0, p.type_id.0))
            .collect::<Vec<_>>()
    );
}

// -------------------------------------------------------------------
// Structural: body_op input is parse output (content), not element var
// -------------------------------------------------------------------

#[test]
fn for_loop_body_op_receives_parse_output_not_element_var() {
    let dag = build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist").expect("build gist graph");
    let lowered = lower(&dag).expect("lower gist graph");

    let loop_info = &lowered.loops[0];
    let body_op = loop_info
        .body_dag
        .nodes
        .iter()
        .find(|n| n.id.0 == "body_op")
        .expect("body_dag should have body_op node");

    // body_op should receive the parse output field (e.g. "content"),
    // NOT the loop element variable (e.g. "path").
    let input_names: Vec<&str> = body_op.inputs.iter().map(|p| p.name.0.as_str()).collect();

    assert!(
        !input_names.contains(&"path"),
        "body_op should NOT have 'path' (element var) as input — \
         it should receive the parse output. Got inputs: {input_names:?}"
    );
    assert!(
        input_names.contains(&"content"),
        "body_op should have 'content' (parse output) as input. \
         Got inputs: {input_names:?}"
    );
}

// -------------------------------------------------------------------
// Execution: loop body produces file content, not file path
// -------------------------------------------------------------------

#[test]
fn for_loop_fs_read_returns_file_content_not_path() {
    let dag = build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist").expect("build gist graph");

    // Auto-mock everything: transport nodes, env nodes, entrypoints.
    let spec = auto_mock_spec(&dag, "gist");
    let mut dry_run_mocks = spec.to_dry_run_mocks();

    // Override the LsFiles execute mock so the loop gets a non-empty list.
    // GenericShellParseOp with SplitLines will parse stdout into ["a.txt", "b.txt"].
    dry_run_mocks.set_value(
        "execute_transport_services_git_git_Core_LsFiles",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("a.txt\nb.txt"))),
    );

    // Override the body execute mock to return a FileResponse with sentinel content.
    // auto_mock_body_transport checks has_mock() before auto-filling, so our explicit
    // mock takes precedence.
    dry_run_mocks.set_value(
        "execute_body_t0",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok(
            "",
            "SENTINEL_CONTENT",
        ))),
    );

    // The outer Filesystem.read transport triplet is dead code (the actual call
    // happens inside the for-loop body). Intercept it so GenericFilePrepareOp
    // doesn't run without inputs.
    dry_run_mocks.set_value(
        "prepare_transport_std_resources_Filesystem_read",
        "request",
        Value::Skipped,
    );

    // Build input mocks for entrypoints from the auto-mock spec.
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

    // Use execute_with_mode_and_inputs_and_detail to get partial logs on failure.
    // But since it stops on error, let's first mock EVERYTHING to be safe.
    // Also mock the outer Filesystem.read execute/parse so they don't interfere.
    dry_run_mocks.set_value(
        "execute_transport_std_resources_Filesystem_read",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok("", ""))),
    );
    dry_run_mocks.set_value(
        "parse_transport_std_resources_Filesystem_read",
        "content",
        Value::Str("".to_string()),
    );

    let log = execute_with_mode_and_inputs(
        &dag,
        ExecutionMode::DryRun(dry_run_mocks),
        Some(&input_mocks),
    )
    .expect("dry run execution should succeed");

    // Find body_op entries in the execution log.
    // Pattern: "{loop_subdag}/unpack/body_{i}/body_op"
    let body_op_entries: Vec<_> = log
        .entries
        .iter()
        .filter(|e| e.node_id.ends_with("/body_op"))
        .collect();

    assert!(
        !body_op_entries.is_empty(),
        "expected at least one body_op entry in execution log \
         (loop should have run at least once). \
         All entries: {:?}",
        log.entries
            .iter()
            .map(|e| &e.node_id)
            .collect::<Vec<_>>()
    );

    // Each body_op should output "SENTINEL_CONTENT" (file read result),
    // NOT a path string like "a.txt".
    for entry in &body_op_entries {
        let result = entry
            .outputs
            .get("result")
            .unwrap_or_else(|| {
                panic!(
                    "body_op '{}' should have 'result' output, got: {:?}",
                    entry.node_id, entry.outputs
                )
            });

        let result_str = match result {
            Value::Str(s) => s.as_str(),
            other => panic!(
                "body_op '{}' result should be Str, got: {:?}",
                entry.node_id, other
            ),
        };

        assert_eq!(
            result_str, "SENTINEL_CONTENT",
            "body_op '{}' should output file content (from mocked FileResponse), \
             not the loop element path. Got: {result_str:?}",
            entry.node_id
        );
    }

    // Verify we got the expected number of iterations (2 files).
    assert_eq!(
        body_op_entries.len(),
        2,
        "expected 2 loop iterations (one per file), got {}",
        body_op_entries.len()
    );
}
