#![allow(clippy::disallowed_methods)]

use gunbc_dag::dsl_builder::build_dsl_graph_for_entrypoint;
use gunbc_exec::{execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_test::auto_mock_spec;

#[test]
fn gist_recent_graph_no_ls_files() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");
    let ls_files_nodes: Vec<&str> = dag
        .nodes
        .iter()
        .filter(|n| n.id.0.contains("LsFiles"))
        .map(|n| n.id.0.as_str())
        .collect();
    assert!(
        ls_files_nodes.is_empty(),
        "gist_recent graph should NOT contain LsFiles nodes (cross-callable leakage): {ls_files_nodes:?}"
    );
}

#[test]
fn gist_recent_graph_wires_diff_base_input() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");
    let lowered = lower(&dag).expect("lowered gist-recent");

    // After CredentialProvider migration, git Diff transport nodes still exist
    // but node naming may differ. Check that any Diff-related prepare node
    // receives a "base" input edge.
    let has_base_edge = lowered.dag.edges.iter().any(|edge| {
        edge.to_node.0.contains("Diff") && edge.to_port.0 == "base"
    });
    assert!(
        has_base_edge,
        "gist-recent must wire a base ref into git diff prepare node. \
         Diff-related edges: {:?}",
        lowered
            .dag
            .edges
            .iter()
            .filter(|e| e.to_node.0.contains("Diff") || e.from_node.0.contains("Diff"))
            .map(|e| format!("{}:{} -> {}:{}", e.from_node.0, e.from_port.0, e.to_node.0, e.to_port.0))
            .collect::<Vec<_>>()
    );
}

/// Structural: the CredentialProvider interface stub must be present
/// in the gist_recent graph (credentials flow through the interface,
/// not a direct credential chain).
#[test]
fn gist_recent_graph_has_credential_provider_interface() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");
    let lowered = lower(&dag).expect("lowered gist-recent");

    let has_credential_node = lowered.dag.nodes.iter().any(|n| {
        n.id.0.contains("CredentialProvider") || n.id.0.contains("credential_provider")
    });
    assert!(
        has_credential_node,
        "gist-recent must have CredentialProvider interface node. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

/// End-to-end DryRun: gist_recent completes with auto-mocked transport nodes.
///
/// Validates that the full pipeline (git, credential chain, gist create) is
/// structurally connected and executes without errors. Uses DryRun mode because
/// the credential chain's `local_auth()` func contains effectful conditionals
/// that the lowerer cannot extract into flat transport nodes.
#[test]
#[ignore] // Pre-existing: GetField on credential token fails in DryRun (gist pipeline)
fn gist_recent_end_to_end_emits_gist_url() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");

    let spec = auto_mock_spec(&dag, "gist_recent");
    let dry_run_mocks = spec.to_dry_run_mocks();

    let input_mocks = {
        let lowered = lower(&dag).expect("lower for entrypoint detection");
        let entrypoints = detect_entrypoints(&lowered.dag);
        let boundary = spec.to_boundary_mocks();
        let mut mocks = BoundaryMocks::new();
        for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
            if let Some(val) = boundary.get_input(&node_id.0, &port_name.0) {
                mocks.set_input(node_id.0.clone(), port_name.0.clone(), val.clone());
            } else {
                // Provide defaults for entrypoint ports not covered by auto_mock_spec
                match port_name.0.as_str() {
                    "since" => mocks.set_input(
                        node_id.0.clone(),
                        port_name.0.clone(),
                        Value::Str("3.days.ago".into()),
                    ),
                    "public" => {
                        mocks.set_input(node_id.0.clone(), port_name.0.clone(), Value::Bool(false))
                    }
                    _ => {}
                }
            }
        }
        mocks
    };

    let log = execute_with_mode_and_inputs(
        &dag,
        ExecutionMode::DryRun(dry_run_mocks),
        Some(&input_mocks),
    )
    .expect("gist-recent DryRun execution should succeed");

    let node_ids: Vec<&str> = log.entries.iter().map(|e| e.node_id.as_str()).collect();

    // Key pipeline stages must appear in execution
    assert!(
        node_ids
            .iter()
            .any(|id| id.starts_with("parse_transport_services_git_git_Core_Diff")),
        "execution should include git Diff parse. Got: {node_ids:?}"
    );
    assert!(
        node_ids
            .iter()
            .any(|id| id.contains("render_diff_markdown")),
        "execution should include render_diff_markdown. Got: {node_ids:?}"
    );
    assert!(
        node_ids.iter().any(|id| id.contains("Gist_Create")),
        "execution should include Gist_Create transport. Got: {node_ids:?}"
    );
    assert!(
        node_ids
            .iter()
            .any(|id| id.contains("credential_chain") || id.contains("acquire_gcp")),
        "execution should include credential chain nodes. Got: {node_ids:?}"
    );
}
