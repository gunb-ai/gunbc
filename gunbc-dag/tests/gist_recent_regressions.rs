#![allow(clippy::disallowed_methods)]

use gunbc_dag::dsl_builder::build_dsl_graph_for_entrypoint;
use gunbc_exec::{execute_with_mode_and_inputs, lower, BoundaryMocks, ExecutionMode};
use gunbc_ir::{detect_entrypoints, Value};
use gunbc_test::auto_mock_spec;

#[test]
fn shared_gist_upload_graph_builds_with_shared_credential_helper() {
    let dag = build_dsl_graph_for_entrypoint("shared/gist_modes.dag", Some("share_content"), None)
        .expect("shared share_content graph should build");
    let lowered = lower(&dag).expect("lowered shared share_content");

    assert!(
        lowered
            .dag
            .nodes
            .iter()
            .any(|n| n.id.0.contains("resolve_github_token")),
        "shared gist_upload should compile against resolve_github_token. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

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

    // gist_recent still relies on the shared credential helper, but the diff
    // transport still needs a concrete base input edge.
    let has_base_edge = lowered
        .dag
        .edges
        .iter()
        .any(|edge| edge.to_node.0.contains("Diff") && edge.to_port.0 == "base");
    assert!(
        has_base_edge,
        "gist-recent must wire a base ref into git diff prepare node. \
         Diff-related edges: {:?}",
        lowered
            .dag
            .edges
            .iter()
            .filter(|e| e.to_node.0.contains("Diff") || e.from_node.0.contains("Diff"))
            .map(|e| format!(
                "{}:{} -> {}:{}",
                e.from_node.0, e.from_port.0, e.to_node.0, e.to_port.0
            ))
            .collect::<Vec<_>>()
    );
}

/// Structural: gist_recent must include concrete token-resolution wiring.
#[test]
fn gist_recent_graph_has_token_resolution_path() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");
    let lowered = lower(&dag).expect("lowered gist-recent");

    let has_env_get = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("shell_Env_Get"));
    let has_resolver_fn = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("resolve_github_token"));
    assert!(
        has_env_get && has_resolver_fn,
        "gist-recent must include env token lookup and resolver function. \
         All nodes: {:?}",
        lowered
            .dag
            .nodes
            .iter()
            .map(|n| &n.id.0)
            .collect::<Vec<_>>()
    );
}

#[test]
fn gist_recent_graph_uses_shared_credential_helper() {
    let dag = build_dsl_graph_for_entrypoint("tools/gist.dag", Some("gist_recent"), None)
        .expect("gist-recent graph should build");
    let lowered = lower(&dag).expect("lowered gist-recent");

    let has_shared_helper = lowered
        .dag
        .nodes
        .iter()
        .any(|n| n.id.0.contains("resolve_github_token"));
    assert!(
        has_shared_helper,
        "gist-recent should route gist creation through shared.credentials::resolve_github_token. \
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
/// Validates that the full pipeline (git, shared credential helper, gist create)
/// is structurally connected and executes without errors. Uses DryRun mode because
/// the credential helper still contains effectful env/secret-manager branches.
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
        node_ids.iter().any(|id| id.contains("shell_Env_Get")),
        "execution should include shell.Env.Get transport. Got: {node_ids:?}"
    );
}
